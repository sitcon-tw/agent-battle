//! In-memory repository implementation for local development and tests.

use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    domain::{
        AgentDecisionLog, AgentId, GameEvent, GameStateSnapshot, Match, MatchEndReason, MatchId,
        MatchResult, MatchStatus, Player, PlayerId, Room, RoomCode, RoomId, RoomStatus, TeamId,
    },
    persistence::{
        AddPlayerInput, AdminAuditLog, AdminAuditLogRepository, AgentDecisionRepository,
        ClaimSlotInput, CreateMatchInput, CreateRoomInput, GameEventRepository,
        GameStateRepository, LockRoomInput, MatchError, MatchRepository, Persistence,
        ReleaseSlotInput, RemovePlayerInput, RenamePlayerInput, RepoError, RoomRepository,
        UpdateSlotPromptInput,
    },
};

#[derive(Debug, Default)]
pub struct MemoryPersistence {
    rooms: RwLock<HashMap<RoomId, Room>>,
    rooms_by_code: RwLock<HashMap<RoomCode, RoomId>>,
    matches: RwLock<HashMap<MatchId, Match>>,
    game_events: RwLock<HashMap<MatchId, Vec<GameEvent>>>,
    game_states: RwLock<HashMap<MatchId, Vec<GameStateSnapshot>>>,
    agent_decisions: RwLock<HashMap<MatchId, Vec<AgentDecisionLog>>>,
    admin_audit_logs: RwLock<Vec<AdminAuditLog>>,
}

impl MemoryPersistence {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Persistence for MemoryPersistence {
    fn rooms(&self) -> &dyn RoomRepository {
        self
    }

    fn matches(&self) -> &dyn MatchRepository {
        self
    }

    fn events(&self) -> &dyn GameEventRepository {
        self
    }

    fn states(&self) -> &dyn GameStateRepository {
        self
    }

    fn decisions(&self) -> &dyn AgentDecisionRepository {
        self
    }

    fn audit_logs(&self) -> &dyn AdminAuditLogRepository {
        self
    }
}

#[async_trait]
impl RoomRepository for MemoryPersistence {
    async fn create_room(&self, input: CreateRoomInput) -> Result<Room, RepoError> {
        let mut rooms = self.rooms.write().await;
        let mut rooms_by_code = self.rooms_by_code.write().await;

        if rooms.contains_key(&input.room_id) {
            return Err(RepoError::RoomAlreadyExists {
                room_id: input.room_id,
            });
        }

        if rooms_by_code.contains_key(&input.code) {
            return Err(RepoError::RoomCodeAlreadyExists { code: input.code });
        }

        let room = Room::new_default(input.room_id, input.code, input.timestamp);
        rooms_by_code.insert(room.code.clone(), room.id.clone());
        rooms.insert(room.id.clone(), room.clone());
        Ok(room)
    }

    async fn get_room(&self, room_id: &RoomId) -> Result<Option<Room>, RepoError> {
        Ok(self.rooms.read().await.get(room_id).cloned())
    }

    async fn get_room_by_code(&self, code: &RoomCode) -> Result<Option<Room>, RepoError> {
        let room_id = self.rooms_by_code.read().await.get(code).cloned();
        let Some(room_id) = room_id else {
            return Ok(None);
        };

        Ok(self.rooms.read().await.get(&room_id).cloned())
    }

    async fn list_rooms(&self) -> Result<Vec<Room>, RepoError> {
        let mut rooms = self
            .rooms
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        rooms.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
        Ok(rooms)
    }

    async fn add_player(&self, input: AddPlayerInput) -> Result<(Room, Player), RepoError> {
        let mut rooms = self.rooms.write().await;
        let room = room_mut(&mut rooms, &input.room_id)?;

        if room
            .players
            .iter()
            .any(|player| player.id == input.player_id)
        {
            return Err(RepoError::PlayerAlreadyExists {
                room_id: input.room_id,
                player_id: input.player_id,
            });
        }

        let player = Player::new(
            input.player_id,
            input.display_name,
            input.token_hash,
            input.timestamp,
        )?;
        room.players.push(player.clone());
        room.version += 1;

        Ok((room.clone(), player))
    }

    async fn rename_player(&self, input: RenamePlayerInput) -> Result<(Room, Player), RepoError> {
        let mut rooms = self.rooms.write().await;
        let room = room_mut(&mut rooms, &input.room_id)?;
        let player = room
            .players
            .iter_mut()
            .find(|player| player.id == input.player_id)
            .ok_or_else(|| RepoError::PlayerNotFound {
                room_id: input.room_id.clone(),
                player_id: input.player_id.clone(),
            })?;

        player.display_name = crate::domain::validate_display_name(&input.display_name)?;
        let player = player.clone();
        room.version += 1;

        Ok((room.clone(), player))
    }

    async fn remove_player(&self, input: RemovePlayerInput) -> Result<Room, RepoError> {
        let mut rooms = self.rooms.write().await;
        let room = room_mut(&mut rooms, &input.room_id)?;
        let original_len = room.players.len();

        room.players.retain(|player| player.id != input.player_id);
        if room.players.len() == original_len {
            return Err(RepoError::PlayerNotFound {
                room_id: input.room_id,
                player_id: input.player_id,
            });
        }

        if room.status == RoomStatus::Open {
            for slot in &mut room.slots {
                if slot.player_id.as_ref() == Some(&input.player_id) {
                    slot.player_id = None;
                    slot.prompt_draft = None;
                }
            }
        }
        room.version += 1;

        Ok(room.clone())
    }

    async fn claim_slot(&self, input: ClaimSlotInput) -> Result<Room, RepoError> {
        let mut rooms = self.rooms.write().await;
        let room = room_mut(&mut rooms, &input.room_id)?;
        ensure_player_exists(room, &input.player_id)?;

        room.claim_slot(&input.player_id, &input.slot_id)?;
        Ok(room.clone())
    }

    async fn release_slot(&self, input: ReleaseSlotInput) -> Result<Room, RepoError> {
        let mut rooms = self.rooms.write().await;
        let room = room_mut(&mut rooms, &input.room_id)?;
        ensure_player_exists(room, &input.player_id)?;

        room.release_slot(&input.player_id, &input.slot_id)?;
        Ok(room.clone())
    }

    async fn update_slot_prompt(&self, input: UpdateSlotPromptInput) -> Result<Room, RepoError> {
        let mut rooms = self.rooms.write().await;
        let room = room_mut(&mut rooms, &input.room_id)?;
        ensure_player_exists(room, &input.player_id)?;

        room.update_prompt(&input.player_id, &input.slot_id, input.prompt)?;
        Ok(room.clone())
    }

    async fn lock_room(&self, input: LockRoomInput) -> Result<Room, RepoError> {
        let mut rooms = self.rooms.write().await;
        let room = room_mut(&mut rooms, &input.room_id)?;

        room.lock_with_role_defaults()?;
        Ok(room.clone())
    }

    async fn unlock_room(&self, room_id: &RoomId) -> Result<Room, RepoError> {
        let mut rooms = self.rooms.write().await;
        let room = room_mut(&mut rooms, room_id)?;

        room.unlock()?;
        Ok(room.clone())
    }

    async fn attach_match(&self, room_id: &RoomId, match_id: &MatchId) -> Result<Room, RepoError> {
        let mut rooms = self.rooms.write().await;
        let room = room_mut(&mut rooms, room_id)?;

        if let Some(existing_match_id) = &room.match_id {
            return Err(RepoError::RoomMatchAlreadyExists {
                room_id: room_id.clone(),
                match_id: existing_match_id.clone(),
            });
        }

        room.mark_running(match_id.clone())?;
        Ok(room.clone())
    }

    async fn set_status(&self, room_id: &RoomId, status: RoomStatus) -> Result<Room, RepoError> {
        let mut rooms = self.rooms.write().await;
        let room = room_mut(&mut rooms, room_id)?;

        room.status = status;
        room.version += 1;
        Ok(room.clone())
    }
}

#[async_trait]
impl MatchRepository for MemoryPersistence {
    async fn create_match(&self, input: CreateMatchInput) -> Result<Match, RepoError> {
        let mut matches = self.matches.write().await;
        if matches.contains_key(&input.match_id) {
            return Err(RepoError::MatchAlreadyExists {
                match_id: input.match_id,
            });
        }

        let game_match = Match::new(
            input.match_id,
            input.room_id,
            input.config_snapshot,
            input.timestamp,
        );
        matches.insert(game_match.id.clone(), game_match.clone());
        Ok(game_match)
    }

    async fn get_match(&self, match_id: &MatchId) -> Result<Option<Match>, RepoError> {
        Ok(self.matches.read().await.get(match_id).cloned())
    }

    async fn get_match_by_room_id(&self, room_id: &RoomId) -> Result<Option<Match>, RepoError> {
        Ok(self
            .matches
            .read()
            .await
            .values()
            .find(|game_match| &game_match.room_id == room_id)
            .cloned())
    }

    async fn set_running(&self, match_id: &MatchId) -> Result<Match, RepoError> {
        let mut matches = self.matches.write().await;
        let game_match = match_mut(&mut matches, match_id)?;

        game_match.status = MatchStatus::Running;
        game_match.started_at = Some(game_match.created_at);
        Ok(game_match.clone())
    }

    async fn set_finished(
        &self,
        match_id: &MatchId,
        result: MatchResult,
    ) -> Result<Match, RepoError> {
        let mut matches = self.matches.write().await;
        let game_match = match_mut(&mut matches, match_id)?;

        game_match.status = MatchStatus::Finished;
        game_match.result = Some(result);
        game_match.finished_at = Some(game_match.started_at.unwrap_or(game_match.created_at));
        Ok(game_match.clone())
    }

    async fn set_failed(&self, match_id: &MatchId, error: MatchError) -> Result<Match, RepoError> {
        let mut matches = self.matches.write().await;
        let game_match = match_mut(&mut matches, match_id)?;

        game_match.status = MatchStatus::Failed;
        game_match.result = Some(MatchResult {
            winner: None,
            score: HashMap::<TeamId, i32>::new(),
            reason: MatchEndReason::Failed,
        });
        game_match.finished_at = Some(game_match.started_at.unwrap_or(game_match.created_at));
        tracing::debug!(match_id = %match_id, error = %error.message, "match marked failed");
        Ok(game_match.clone())
    }

    async fn update_current_turn(&self, match_id: &MatchId, turn: u32) -> Result<Match, RepoError> {
        let mut matches = self.matches.write().await;
        let game_match = match_mut(&mut matches, match_id)?;

        game_match.current_turn = turn;
        Ok(game_match.clone())
    }
}

#[async_trait]
impl GameEventRepository for MemoryPersistence {
    async fn append_event(&self, event: GameEvent) -> Result<(), RepoError> {
        self.game_events
            .write()
            .await
            .entry(event.match_id.clone())
            .or_default()
            .push(event);
        Ok(())
    }

    async fn append_events(&self, events: Vec<GameEvent>) -> Result<(), RepoError> {
        let mut game_events = self.game_events.write().await;
        for event in events {
            game_events
                .entry(event.match_id.clone())
                .or_default()
                .push(event);
        }
        Ok(())
    }

    async fn list_events_by_match(&self, match_id: &MatchId) -> Result<Vec<GameEvent>, RepoError> {
        Ok(self
            .game_events
            .read()
            .await
            .get(match_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn list_events_by_turn(
        &self,
        match_id: &MatchId,
        turn: u32,
    ) -> Result<Vec<GameEvent>, RepoError> {
        Ok(self
            .game_events
            .read()
            .await
            .get(match_id)
            .map(|events| {
                events
                    .iter()
                    .filter(|event| event.turn == turn)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }
}

#[async_trait]
impl GameStateRepository for MemoryPersistence {
    async fn save_snapshot(&self, snapshot: GameStateSnapshot) -> Result<(), RepoError> {
        let mut game_states = self.game_states.write().await;
        let snapshots = game_states.entry(snapshot.match_id.clone()).or_default();

        if let Some(existing) = snapshots
            .iter_mut()
            .find(|existing| existing.turn == snapshot.turn)
        {
            *existing = snapshot;
        } else {
            snapshots.push(snapshot);
            snapshots.sort_by_key(|snapshot| snapshot.turn);
        }

        Ok(())
    }

    async fn get_snapshot(
        &self,
        match_id: &MatchId,
        turn: u32,
    ) -> Result<Option<GameStateSnapshot>, RepoError> {
        Ok(self
            .game_states
            .read()
            .await
            .get(match_id)
            .and_then(|snapshots| snapshots.iter().find(|snapshot| snapshot.turn == turn))
            .cloned())
    }

    async fn list_snapshots(
        &self,
        match_id: &MatchId,
    ) -> Result<Vec<GameStateSnapshot>, RepoError> {
        Ok(self
            .game_states
            .read()
            .await
            .get(match_id)
            .cloned()
            .unwrap_or_default())
    }
}

#[async_trait]
impl AgentDecisionRepository for MemoryPersistence {
    async fn save_decision(&self, decision: AgentDecisionLog) -> Result<(), RepoError> {
        self.agent_decisions
            .write()
            .await
            .entry(decision.match_id.clone())
            .or_default()
            .push(decision);
        Ok(())
    }

    async fn list_decisions_by_match(
        &self,
        match_id: &MatchId,
    ) -> Result<Vec<AgentDecisionLog>, RepoError> {
        Ok(self
            .agent_decisions
            .read()
            .await
            .get(match_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn list_decisions_by_agent(
        &self,
        match_id: &MatchId,
        agent_id: &AgentId,
    ) -> Result<Vec<AgentDecisionLog>, RepoError> {
        Ok(self
            .agent_decisions
            .read()
            .await
            .get(match_id)
            .map(|decisions| {
                decisions
                    .iter()
                    .filter(|decision| &decision.agent_id == agent_id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }
}

#[async_trait]
impl AdminAuditLogRepository for MemoryPersistence {
    async fn append_audit_log(&self, log: AdminAuditLog) -> Result<(), RepoError> {
        self.admin_audit_logs.write().await.push(log);
        Ok(())
    }

    async fn list_audit_logs(&self) -> Result<Vec<AdminAuditLog>, RepoError> {
        Ok(self.admin_audit_logs.read().await.clone())
    }
}

fn room_mut<'a>(
    rooms: &'a mut HashMap<RoomId, Room>,
    room_id: &RoomId,
) -> Result<&'a mut Room, RepoError> {
    rooms
        .get_mut(room_id)
        .ok_or_else(|| RepoError::RoomNotFound {
            room_id: room_id.clone(),
        })
}

fn match_mut<'a>(
    matches: &'a mut HashMap<MatchId, Match>,
    match_id: &MatchId,
) -> Result<&'a mut Match, RepoError> {
    matches
        .get_mut(match_id)
        .ok_or_else(|| RepoError::MatchNotFound {
            match_id: match_id.clone(),
        })
}

fn ensure_player_exists(room: &Room, player_id: &PlayerId) -> Result<(), RepoError> {
    if room.players.iter().any(|player| &player.id == player_id) {
        Ok(())
    } else {
        Err(RepoError::PlayerNotFound {
            room_id: room.id.clone(),
            player_id: player_id.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::domain::{
        AgentDecision, AgentObservation, EventId, GameEventType, MapId, MatchConfigSnapshot,
        PublicAgentState, PublicGameState, PublicMapSummary, PublicNodeState, RoleName,
    };

    fn room_id() -> RoomId {
        RoomId::new("room_1")
    }

    fn room_code() -> RoomCode {
        RoomCode::new("ABC123")
    }

    fn match_id() -> MatchId {
        MatchId::new("match_1")
    }

    async fn create_room(store: &MemoryPersistence) -> Room {
        store
            .rooms()
            .create_room(CreateRoomInput {
                room_id: room_id(),
                code: room_code(),
                timestamp: 100,
            })
            .await
            .expect("room creation succeeds")
    }

    async fn add_player(
        store: &MemoryPersistence,
        player_id: PlayerId,
        display_name: &str,
    ) -> Player {
        store
            .rooms()
            .add_player(AddPlayerInput {
                room_id: room_id(),
                player_id,
                display_name: display_name.to_owned(),
                token_hash: "token_hash".to_owned(),
                timestamp: 101,
            })
            .await
            .expect("player add succeeds")
            .1
    }

    #[tokio::test]
    async fn creates_and_reads_rooms_by_id_code_and_list() {
        let store = MemoryPersistence::new();
        let created = create_room(&store).await;

        assert_eq!(
            store
                .rooms()
                .get_room(&created.id)
                .await
                .expect("read succeeds"),
            Some(created.clone())
        );
        assert_eq!(
            store
                .rooms()
                .get_room_by_code(&created.code)
                .await
                .expect("read by code succeeds"),
            Some(created.clone())
        );
        assert_eq!(
            store.rooms().list_rooms().await.expect("list succeeds"),
            vec![created]
        );
    }

    #[tokio::test]
    async fn adds_and_removes_players() {
        let store = MemoryPersistence::new();
        create_room(&store).await;
        let player = add_player(&store, PlayerId::new("player_1"), " Ada ").await;

        assert_eq!(player.display_name, "Ada");

        let room = store
            .rooms()
            .remove_player(RemovePlayerInput {
                room_id: room_id(),
                player_id: player.id,
            })
            .await
            .expect("remove succeeds");

        assert!(room.players.is_empty());
    }

    #[tokio::test]
    async fn claims_slots_atomically_and_rejects_occupied_or_double_claims() {
        let store = MemoryPersistence::new();
        let room = create_room(&store).await;
        let first_slot = room.slots[0].id.clone();
        let second_slot = room.slots[1].id.clone();
        let first_player = add_player(&store, PlayerId::new("player_1"), "Ada").await;
        let second_player = add_player(&store, PlayerId::new("player_2"), "Grace").await;

        let room = store
            .rooms()
            .claim_slot(ClaimSlotInput {
                room_id: room_id(),
                player_id: first_player.id.clone(),
                slot_id: first_slot.clone(),
            })
            .await
            .expect("claim succeeds");
        assert_eq!(room.slots[0].player_id.as_ref(), Some(&first_player.id));

        assert!(matches!(
            store
                .rooms()
                .claim_slot(ClaimSlotInput {
                    room_id: room_id(),
                    player_id: second_player.id.clone(),
                    slot_id: first_slot,
                })
                .await,
            Err(RepoError::Domain(
                crate::domain::DomainError::SlotAlreadyClaimed { .. }
            ))
        ));

        assert!(matches!(
            store
                .rooms()
                .claim_slot(ClaimSlotInput {
                    room_id: room_id(),
                    player_id: first_player.id,
                    slot_id: second_slot,
                })
                .await,
            Err(RepoError::Domain(
                crate::domain::DomainError::PlayerAlreadyClaimedSlot { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn updates_prompt_locks_room_and_attaches_match() {
        let store = MemoryPersistence::new();
        let room = create_room(&store).await;
        let slot_id = room.slots[0].id.clone();
        let player = add_player(&store, PlayerId::new("player_1"), "Ada").await;

        store
            .rooms()
            .claim_slot(ClaimSlotInput {
                room_id: room_id(),
                player_id: player.id.clone(),
                slot_id: slot_id.clone(),
            })
            .await
            .expect("claim succeeds");

        let room = store
            .rooms()
            .update_slot_prompt(UpdateSlotPromptInput {
                room_id: room_id(),
                player_id: player.id,
                slot_id,
                prompt: "Hold the center".to_owned(),
            })
            .await
            .expect("prompt update succeeds");
        assert_eq!(
            room.slots[0].prompt_draft.as_deref(),
            Some("Hold the center")
        );

        let room = store
            .rooms()
            .lock_room(LockRoomInput { room_id: room_id() })
            .await
            .expect("lock succeeds");
        assert_eq!(room.status, RoomStatus::Locked);
        assert!(room.slots.iter().all(|slot| slot.locked_prompt.is_some()));

        let room = store
            .rooms()
            .attach_match(&room_id(), &match_id())
            .await
            .expect("attach succeeds");
        assert_eq!(room.status, RoomStatus::Running);
        assert_eq!(room.match_id, Some(match_id()));
    }

    #[tokio::test]
    async fn saves_and_lists_events_snapshots_and_decisions() {
        let store = MemoryPersistence::new();
        let first_event = game_event(1, 1);
        let second_event = game_event(2, 2);
        store
            .events()
            .append_events(vec![first_event.clone(), second_event.clone()])
            .await
            .expect("events save succeeds");

        assert_eq!(
            store
                .events()
                .list_events_by_match(&match_id())
                .await
                .expect("events list succeeds"),
            vec![first_event.clone(), second_event]
        );
        assert_eq!(
            store
                .events()
                .list_events_by_turn(&match_id(), 1)
                .await
                .expect("events list by turn succeeds"),
            vec![first_event]
        );

        let snapshot = game_state_snapshot(3);
        store
            .states()
            .save_snapshot(snapshot.clone())
            .await
            .expect("snapshot save succeeds");
        assert_eq!(
            store
                .states()
                .get_snapshot(&match_id(), 3)
                .await
                .expect("snapshot get succeeds"),
            Some(snapshot.clone())
        );
        assert_eq!(
            store
                .states()
                .list_snapshots(&match_id())
                .await
                .expect("snapshot list succeeds"),
            vec![snapshot]
        );

        let first_decision = decision_log("agent_1");
        let second_decision = decision_log("agent_2");
        store
            .decisions()
            .save_decision(first_decision.clone())
            .await
            .expect("decision save succeeds");
        store
            .decisions()
            .save_decision(second_decision.clone())
            .await
            .expect("decision save succeeds");
        assert_eq!(
            store
                .decisions()
                .list_decisions_by_match(&match_id())
                .await
                .expect("decision list succeeds"),
            vec![first_decision.clone(), second_decision]
        );
        assert_eq!(
            store
                .decisions()
                .list_decisions_by_agent(&match_id(), &AgentId::new("agent_1"))
                .await
                .expect("decision list by agent succeeds"),
            vec![first_decision]
        );
    }

    #[tokio::test]
    async fn updates_match_lifecycle() {
        let store = MemoryPersistence::new();
        let game_match = store
            .matches()
            .create_match(CreateMatchInput {
                match_id: match_id(),
                room_id: room_id(),
                config_snapshot: MatchConfigSnapshot {
                    map_id: MapId::new("default-15x9"),
                    max_turns: 8,
                    teams: Vec::new(),
                },
                timestamp: 200,
            })
            .await
            .expect("match create succeeds");
        assert_eq!(game_match.status, MatchStatus::Created);

        let game_match = store
            .matches()
            .set_running(&match_id())
            .await
            .expect("running update succeeds");
        assert_eq!(game_match.status, MatchStatus::Running);
        assert_eq!(
            store
                .matches()
                .get_match_by_room_id(&room_id())
                .await
                .expect("match by room read succeeds"),
            Some(game_match.clone())
        );

        let game_match = store
            .matches()
            .update_current_turn(&match_id(), 4)
            .await
            .expect("turn update succeeds");
        assert_eq!(game_match.current_turn, 4);

        let result = MatchResult {
            winner: None,
            score: HashMap::new(),
            reason: MatchEndReason::MaxTurnsReached,
        };
        let game_match = store
            .matches()
            .set_finished(&match_id(), result.clone())
            .await
            .expect("finish succeeds");
        assert_eq!(game_match.status, MatchStatus::Finished);
        assert_eq!(game_match.result, Some(result));
    }

    fn game_event(sequence: u64, turn: u32) -> GameEvent {
        GameEvent {
            id: EventId::new(format!("event_{sequence}")),
            match_id: match_id(),
            turn,
            sequence,
            event_type: GameEventType::TurnStarted,
            actor_agent_id: None,
            payload: serde_json::json!({ "turn": turn }),
            created_at: 300 + sequence,
        }
    }

    fn game_state_snapshot(turn: u32) -> GameStateSnapshot {
        GameStateSnapshot {
            match_id: match_id(),
            turn,
            state: PublicGameState {
                match_id: match_id(),
                turn,
                agents: Vec::new(),
                nodes: Vec::new(),
                score: HashMap::new(),
            },
            created_at: 400,
        }
    }

    fn decision_log(agent_id: &str) -> AgentDecisionLog {
        AgentDecisionLog {
            match_id: match_id(),
            turn: 1,
            agent_id: AgentId::new(agent_id),
            observation: AgentObservation {
                turn: 1,
                self_agent: public_agent(agent_id),
                visible_allies: Vec::new(),
                visible_enemies: Vec::new(),
                known_nodes: Vec::<PublicNodeState>::new(),
                score: HashMap::new(),
                map_summary: PublicMapSummary {
                    map_id: MapId::new("default-15x9"),
                    width: 15,
                    height: 9,
                },
                legal_action_hints: Vec::new(),
            },
            prompt_snapshot: Some("act carefully".to_owned()),
            raw_output: Some("wait".to_owned()),
            parsed_decision: Some(AgentDecision::fallback()),
            validated_decision: AgentDecision::fallback(),
            was_fallback_used: false,
            error: None,
            latency_ms: Some(12),
        }
    }

    fn public_agent(agent_id: &str) -> PublicAgentState {
        PublicAgentState {
            id: AgentId::new(agent_id),
            team_id: TeamId::new("team_a"),
            role: RoleName::Vanguard,
            display_name: Some("Ada".to_owned()),
            hp: 14,
            max_hp: 14,
            position: crate::domain::Position::new(0, 0),
            status: crate::domain::AgentStatus::Active,
        }
    }
}
