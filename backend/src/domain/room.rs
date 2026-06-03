//! Room aggregate and room-level domain behavior.

use serde::{Deserialize, Serialize};

use crate::domain::{
    AgentSlot, DomainError, MapId, MatchId, Player, PlayerId, RoleName, RoomCode, RoomId, SlotId,
    Team, TeamId, TeamSide, Timestamp,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoomStatus {
    Open,
    Locked,
    Running,
    Finished,
    Closed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameMode {
    PromptOpsArena,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomConfig {
    pub mode: GameMode,
    pub map_id: MapId,
    pub max_turns: u32,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            mode: GameMode::PromptOpsArena,
            map_id: MapId::new("default-15x9"),
            max_turns: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub code: RoomCode,
    pub status: RoomStatus,
    pub config: RoomConfig,
    pub players: Vec<Player>,
    pub teams: Vec<Team>,
    pub slots: Vec<AgentSlot>,
    pub match_id: Option<MatchId>,
    pub version: u64,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Room {
    #[must_use]
    pub fn new_default(id: RoomId, code: RoomCode, timestamp: Timestamp) -> Self {
        let teams = vec![
            Team::new(TeamId::new("team_a"), "Team A", TeamSide::A),
            Team::new(TeamId::new("team_b"), "Team B", TeamSide::B),
        ];
        let mut slots = Vec::with_capacity(teams.len() * RoleName::ALL.len());

        for team in &teams {
            let side_slug = match team.side {
                TeamSide::A => "a",
                TeamSide::B => "b",
            };

            for role in RoleName::ALL {
                slots.push(AgentSlot::new(
                    SlotId::new(format!("slot_{side_slug}_{}", role.as_slug())),
                    team.id.clone(),
                    role,
                ));
            }
        }

        Self {
            id,
            code,
            status: RoomStatus::Open,
            config: RoomConfig::default(),
            players: Vec::new(),
            teams,
            slots,
            match_id: None,
            version: 1,
            created_at: timestamp,
            updated_at: timestamp,
        }
    }

    /// Claims an empty slot for a player.
    ///
    /// # Errors
    ///
    /// Returns a domain error when the room is not open, the slot is missing or occupied, or the
    /// player already owns another slot.
    pub fn claim_slot(
        &mut self,
        player_id: &PlayerId,
        slot_id: &SlotId,
    ) -> Result<(), DomainError> {
        self.require_status(RoomStatus::Open, "claim slot")?;

        if let Some(existing) = self
            .slots
            .iter()
            .find(|slot| slot.player_id.as_ref() == Some(player_id))
        {
            return Err(DomainError::PlayerAlreadyClaimedSlot {
                player_id: player_id.clone(),
                slot_id: existing.id.clone(),
            });
        }

        let slot = self.slot_mut(slot_id)?;
        if slot.player_id.is_some() {
            return Err(DomainError::SlotAlreadyClaimed {
                slot_id: slot_id.clone(),
            });
        }

        slot.player_id = Some(player_id.clone());
        self.bump_version();
        Ok(())
    }

    /// Releases a slot if the player owns it.
    ///
    /// # Errors
    ///
    /// Returns a domain error when the room is not open, the slot is missing, or ownership does not
    /// match.
    pub fn release_slot(
        &mut self,
        player_id: &PlayerId,
        slot_id: &SlotId,
    ) -> Result<(), DomainError> {
        self.require_status(RoomStatus::Open, "release slot")?;
        let slot = self.slot_mut(slot_id)?;

        if slot.player_id.as_ref() != Some(player_id) {
            return Err(DomainError::SlotNotOwnedByPlayer {
                slot_id: slot_id.clone(),
                player_id: player_id.clone(),
            });
        }

        slot.player_id = None;
        slot.prompt_draft = None;
        self.bump_version();
        Ok(())
    }

    /// Updates a prompt draft for a slot owner.
    ///
    /// # Errors
    ///
    /// Returns a domain error when the room is not open, the slot is missing, or ownership does not
    /// match.
    pub fn update_prompt(
        &mut self,
        player_id: &PlayerId,
        slot_id: &SlotId,
        prompt: impl Into<String>,
    ) -> Result<(), DomainError> {
        self.require_status(RoomStatus::Open, "update prompt")?;
        let slot = self.slot_mut(slot_id)?;

        if slot.player_id.as_ref() != Some(player_id) {
            return Err(DomainError::SlotNotOwnedByPlayer {
                slot_id: slot_id.clone(),
                player_id: player_id.clone(),
            });
        }

        slot.prompt_draft = Some(prompt.into());
        self.bump_version();
        Ok(())
    }

    /// Locks room input and snapshots prompt drafts.
    ///
    /// # Errors
    ///
    /// Returns a domain error when the room is not open.
    pub fn lock(&mut self, fallback_prompt: &str) -> Result<(), DomainError> {
        self.require_transition(RoomStatus::Locked)?;

        for slot in &mut self.slots {
            let prompt = slot
                .prompt_draft
                .clone()
                .unwrap_or_else(|| fallback_prompt.to_owned());
            slot.locked_prompt = Some(prompt);
        }

        self.status = RoomStatus::Locked;
        self.bump_version();
        Ok(())
    }

    /// Locks room input using each role's default prompt for empty prompt drafts.
    ///
    /// # Errors
    ///
    /// Returns a domain error when the room is not open.
    pub fn lock_with_role_defaults(&mut self) -> Result<(), DomainError> {
        self.require_transition(RoomStatus::Locked)?;

        for slot in &mut self.slots {
            let prompt = slot
                .prompt_draft
                .clone()
                .unwrap_or_else(|| slot.role.default_prompt().to_owned());
            slot.locked_prompt = Some(prompt);
        }

        self.status = RoomStatus::Locked;
        self.bump_version();
        Ok(())
    }

    /// Unlocks a locked room.
    ///
    /// # Errors
    ///
    /// Returns a domain error when the room is not locked.
    pub fn unlock(&mut self) -> Result<(), DomainError> {
        self.require_transition(RoomStatus::Open)?;
        self.status = RoomStatus::Open;
        self.bump_version();
        Ok(())
    }

    /// Marks a locked room as running and attaches its child match ID.
    ///
    /// # Errors
    ///
    /// Returns a domain error when the room is not locked.
    pub fn mark_running(&mut self, match_id: MatchId) -> Result<(), DomainError> {
        self.require_transition(RoomStatus::Running)?;
        self.status = RoomStatus::Running;
        self.match_id = Some(match_id);
        self.bump_version();
        Ok(())
    }

    /// Marks a running room as finished.
    ///
    /// # Errors
    ///
    /// Returns a domain error when the room is not running.
    pub fn mark_finished(&mut self) -> Result<(), DomainError> {
        self.require_transition(RoomStatus::Finished)?;
        self.status = RoomStatus::Finished;
        self.bump_version();
        Ok(())
    }

    pub fn close(&mut self) {
        self.status = RoomStatus::Closed;
        self.bump_version();
    }

    fn slot_mut(&mut self, slot_id: &SlotId) -> Result<&mut AgentSlot, DomainError> {
        self.slots
            .iter_mut()
            .find(|slot| &slot.id == slot_id)
            .ok_or_else(|| DomainError::SlotNotFound {
                slot_id: slot_id.clone(),
            })
    }

    fn require_status(&self, status: RoomStatus, action: &'static str) -> Result<(), DomainError> {
        if self.status == status {
            Ok(())
        } else {
            Err(DomainError::RoomStatusDoesNotAllowAction {
                status: self.status,
                action,
            })
        }
    }

    fn require_transition(&self, to: RoomStatus) -> Result<(), DomainError> {
        let valid = matches!(
            (self.status, to),
            (RoomStatus::Open, RoomStatus::Locked)
                | (RoomStatus::Locked, RoomStatus::Open)
                | (RoomStatus::Locked, RoomStatus::Running)
                | (RoomStatus::Running, RoomStatus::Finished)
        );

        if valid {
            Ok(())
        } else {
            Err(DomainError::InvalidRoomStatusTransition {
                from: self.status,
                to,
            })
        }
    }

    fn bump_version(&mut self) {
        self.version += 1;
    }
}
