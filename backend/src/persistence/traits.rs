//! Repository traits and persistence-layer input types.

use async_trait::async_trait;
use thiserror::Error;

use crate::domain::{
    AgentDecisionLog, AgentId, DomainError, GameEvent, GameStateSnapshot, Match,
    MatchConfigSnapshot, MatchId, MatchResult, Player, PlayerId, Room, RoomCode, RoomId,
    RoomStatus, SlotId, Timestamp,
};

pub trait Persistence: Send + Sync {
    fn rooms(&self) -> &dyn RoomRepository;
    fn matches(&self) -> &dyn MatchRepository;
    fn events(&self) -> &dyn GameEventRepository;
    fn states(&self) -> &dyn GameStateRepository;
    fn decisions(&self) -> &dyn AgentDecisionRepository;
    fn audit_logs(&self) -> &dyn AdminAuditLogRepository;
}

#[async_trait]
pub trait RoomRepository: Send + Sync {
    async fn create_room(&self, input: CreateRoomInput) -> Result<Room, RepoError>;
    async fn get_room(&self, room_id: &RoomId) -> Result<Option<Room>, RepoError>;
    async fn get_room_by_code(&self, code: &RoomCode) -> Result<Option<Room>, RepoError>;
    async fn list_rooms(&self) -> Result<Vec<Room>, RepoError>;

    async fn add_player(&self, input: AddPlayerInput) -> Result<(Room, Player), RepoError>;
    async fn rename_player(&self, input: RenamePlayerInput) -> Result<(Room, Player), RepoError>;
    async fn remove_player(&self, input: RemovePlayerInput) -> Result<Room, RepoError>;

    async fn claim_slot(&self, input: ClaimSlotInput) -> Result<Room, RepoError>;
    async fn release_slot(&self, input: ReleaseSlotInput) -> Result<Room, RepoError>;
    async fn update_slot_prompt(&self, input: UpdateSlotPromptInput) -> Result<Room, RepoError>;

    async fn lock_room(&self, input: LockRoomInput) -> Result<Room, RepoError>;
    async fn unlock_room(&self, room_id: &RoomId) -> Result<Room, RepoError>;

    async fn attach_match(&self, room_id: &RoomId, match_id: &MatchId) -> Result<Room, RepoError>;
    async fn set_status(&self, room_id: &RoomId, status: RoomStatus) -> Result<Room, RepoError>;
}

#[async_trait]
pub trait MatchRepository: Send + Sync {
    async fn create_match(&self, input: CreateMatchInput) -> Result<Match, RepoError>;
    async fn get_match(&self, match_id: &MatchId) -> Result<Option<Match>, RepoError>;
    async fn get_match_by_room_id(&self, room_id: &RoomId) -> Result<Option<Match>, RepoError>;

    async fn set_running(&self, match_id: &MatchId) -> Result<Match, RepoError>;
    async fn set_finished(
        &self,
        match_id: &MatchId,
        result: MatchResult,
    ) -> Result<Match, RepoError>;
    async fn set_failed(&self, match_id: &MatchId, error: MatchError) -> Result<Match, RepoError>;

    async fn update_current_turn(&self, match_id: &MatchId, turn: u32) -> Result<Match, RepoError>;
}

#[async_trait]
pub trait GameEventRepository: Send + Sync {
    async fn append_event(&self, event: GameEvent) -> Result<(), RepoError>;
    async fn append_events(&self, events: Vec<GameEvent>) -> Result<(), RepoError>;

    async fn list_events_by_match(&self, match_id: &MatchId) -> Result<Vec<GameEvent>, RepoError>;
    async fn list_events_by_turn(
        &self,
        match_id: &MatchId,
        turn: u32,
    ) -> Result<Vec<GameEvent>, RepoError>;
}

#[async_trait]
pub trait GameStateRepository: Send + Sync {
    async fn save_snapshot(&self, snapshot: GameStateSnapshot) -> Result<(), RepoError>;

    async fn get_snapshot(
        &self,
        match_id: &MatchId,
        turn: u32,
    ) -> Result<Option<GameStateSnapshot>, RepoError>;

    async fn list_snapshots(&self, match_id: &MatchId)
    -> Result<Vec<GameStateSnapshot>, RepoError>;
}

#[async_trait]
pub trait AgentDecisionRepository: Send + Sync {
    async fn save_decision(&self, decision: AgentDecisionLog) -> Result<(), RepoError>;

    async fn list_decisions_by_match(
        &self,
        match_id: &MatchId,
    ) -> Result<Vec<AgentDecisionLog>, RepoError>;

    async fn list_decisions_by_agent(
        &self,
        match_id: &MatchId,
        agent_id: &AgentId,
    ) -> Result<Vec<AgentDecisionLog>, RepoError>;
}

#[async_trait]
pub trait AdminAuditLogRepository: Send + Sync {
    async fn append_audit_log(&self, log: AdminAuditLog) -> Result<(), RepoError>;
    async fn list_audit_logs(&self) -> Result<Vec<AdminAuditLog>, RepoError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRoomInput {
    pub room_id: RoomId,
    pub code: RoomCode,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddPlayerInput {
    pub room_id: RoomId,
    pub player_id: PlayerId,
    pub display_name: String,
    pub token_hash: String,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovePlayerInput {
    pub room_id: RoomId,
    pub player_id: PlayerId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamePlayerInput {
    pub room_id: RoomId,
    pub player_id: PlayerId,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimSlotInput {
    pub room_id: RoomId,
    pub player_id: PlayerId,
    pub slot_id: SlotId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSlotInput {
    pub room_id: RoomId,
    pub player_id: PlayerId,
    pub slot_id: SlotId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSlotPromptInput {
    pub room_id: RoomId,
    pub player_id: PlayerId,
    pub slot_id: SlotId,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockRoomInput {
    pub room_id: RoomId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateMatchInput {
    pub match_id: MatchId,
    pub room_id: RoomId,
    pub config_snapshot: MatchConfigSnapshot,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchError {
    pub message: String,
}

impl MatchError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAuditLog {
    pub id: String,
    pub action: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepoError {
    #[error("room {room_id} was not found")]
    RoomNotFound { room_id: RoomId },

    #[error("room {room_id} already exists")]
    RoomAlreadyExists { room_id: RoomId },

    #[error("room code {code} already exists")]
    RoomCodeAlreadyExists { code: RoomCode },

    #[error("player {player_id} already exists in room {room_id}")]
    PlayerAlreadyExists {
        room_id: RoomId,
        player_id: PlayerId,
    },

    #[error("player {player_id} was not found in room {room_id}")]
    PlayerNotFound {
        room_id: RoomId,
        player_id: PlayerId,
    },

    #[error("match {match_id} was not found")]
    MatchNotFound { match_id: MatchId },

    #[error("match {match_id} already exists")]
    MatchAlreadyExists { match_id: MatchId },

    #[error("room {room_id} already has match {match_id}")]
    RoomMatchAlreadyExists { room_id: RoomId, match_id: MatchId },

    #[error(transparent)]
    Domain(#[from] DomainError),
}
