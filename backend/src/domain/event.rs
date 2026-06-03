use crate::domain::ids::{EventId, MatchId, AgentId, RoomId, PlayerId, SlotId, NodeId, MapId, RoomCode, TeamId};
use crate::domain::player::{Player, Timestamp};
use crate::domain::room::{Room, RoomStatus, RoomConfig};
use crate::domain::slot::RoleName;
use crate::domain::map::{Position, ControlNode};
use crate::domain::agent::{AgentStatus, GameAgent};
use crate::domain::action::AgentDecision;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameEventType {
    #[serde(rename = "MATCH_STARTED")]
    MatchStarted,
    #[serde(rename = "TURN_STARTED")]
    TurnStarted,
    #[serde(rename = "AGENT_DECISION_RECEIVED")]
    AgentDecisionReceived,
    #[serde(rename = "INVALID_ACTION_REPLACED")]
    InvalidActionReplaced,
    #[serde(rename = "AGENT_MOVED")]
    AgentMoved,
    #[serde(rename = "MOVEMENT_CONFLICT")]
    MovementConflict,
    #[serde(rename = "AGENT_ATTACKED")]
    AgentAttacked,
    #[serde(rename = "SKILL_USED")]
    SkillUsed,
    #[serde(rename = "DAMAGE_DEALT")]
    DamageDealt,
    #[serde(rename = "AGENT_HEALED")]
    AgentHealed,
    #[serde(rename = "AGENT_DOWNED")]
    AgentDowned,
    #[serde(rename = "AGENT_RESPAWNED")]
    AgentRespawned,
    #[serde(rename = "NODE_CONTROL_CHANGED")]
    NodeControlChanged,
    #[serde(rename = "SCORE_CHANGED")]
    ScoreChanged,
    #[serde(rename = "TURN_FINISHED")]
    TurnFinished,
    #[serde(rename = "MATCH_FINISHED")]
    MatchFinished,
    #[serde(rename = "MATCH_FAILED")]
    MatchFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameEvent {
    pub id: EventId,
    pub match_id: MatchId,
    pub turn: u32,
    pub sequence: u64,
    pub event_type: GameEventType,
    pub actor_agent_id: Option<AgentId>,
    pub payload: serde_json::Value,
    pub created_at: Timestamp,
}

impl GameEvent {
    pub fn new(
        id: EventId,
        match_id: MatchId,
        turn: u32,
        sequence: u64,
        event_type: GameEventType,
        actor_agent_id: Option<AgentId>,
        payload: serde_json::Value,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            match_id,
            turn,
            sequence,
            event_type,
            actor_agent_id,
            payload,
            created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicAgentState {
    pub id: AgentId,
    pub team_id: TeamId,
    pub role: RoleName,
    pub display_name: Option<String>,
    pub hp: i32,
    pub max_hp: i32,
    pub position: Position,
    pub status: AgentStatus,
    pub marked: bool,
    pub shield: i32,
}

impl From<&GameAgent> for PublicAgentState {
    fn from(a: &GameAgent) -> Self {
        Self {
            id: a.id.clone(),
            team_id: a.team_id.clone(),
            role: a.role,
            display_name: a.display_name.clone(),
            hp: a.hp,
            max_hp: a.max_hp,
            position: a.position,
            status: a.status,
            marked: a.marked,
            shield: a.shield,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicNodeState {
    pub id: NodeId,
    pub name: String,
    pub position: Position,
    pub score_value: i32,
    pub controlled_by: Option<TeamId>,
}

impl From<&ControlNode> for PublicNodeState {
    fn from(n: &ControlNode) -> Self {
        Self {
            id: n.id.clone(),
            name: n.name.clone(),
            position: n.position,
            score_value: n.score_value,
            controlled_by: n.controlled_by.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicMapSummary {
    pub map_id: MapId,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicGameState {
    pub match_id: MatchId,
    pub turn: u32,
    pub map_id: MapId,
    pub agents: Vec<PublicAgentState>,
    pub nodes: Vec<PublicNodeState>,
    pub score: HashMap<TeamId, i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameStateSnapshot {
    pub match_id: MatchId,
    pub turn: u32,
    pub state: PublicGameState,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicPlayer {
    pub id: PlayerId,
    pub display_name: String,
    pub joined_at: Timestamp,
}

impl From<&Player> for PublicPlayer {
    fn from(p: &Player) -> Self {
        Self {
            id: p.id.clone(),
            display_name: p.display_name.clone(),
            joined_at: p.joined_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomSummary {
    pub id: RoomId,
    pub code: RoomCode,
    pub status: RoomStatus,
    pub config: RoomConfig,
    pub player_count: usize,
    pub version: u64,
}

impl From<&Room> for RoomSummary {
    fn from(r: &Room) -> Self {
        Self {
            id: r.id.clone(),
            code: r.code.clone(),
            status: r.status,
            config: r.config.clone(),
            player_count: r.players.len(),
            version: r.version,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentObservation {
    pub turn: u32,
    pub self_agent: PublicAgentState,
    pub visible_allies: Vec<PublicAgentState>,
    pub visible_enemies: Vec<PublicAgentState>,
    pub known_nodes: Vec<PublicNodeState>,
    pub score: HashMap<TeamId, i32>,
    pub map_summary: PublicMapSummary,
    pub legal_action_hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDecisionLog {
    pub match_id: MatchId,
    pub turn: u32,
    pub agent_id: AgentId,

    pub observation: AgentObservation,
    pub prompt_snapshot: Option<String>,

    pub raw_output: Option<String>,
    pub parsed_decision: Option<AgentDecision>,
    pub validated_decision: AgentDecision,

    pub was_fallback_used: bool,
    pub error: Option<String>,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomEvent {
    RoomCreated { room: RoomSummary },
    RoomUpdated { room_id: RoomId, version: u64 },
    RoomClosed { room_id: RoomId },
    RoomDeleted { room_id: RoomId },

    PlayerJoined { room_id: RoomId, player: PublicPlayer },
    PlayerLeft { room_id: RoomId, player_id: PlayerId },
    PlayerRenamed { room_id: RoomId, player_id: PlayerId, display_name: String },

    SlotClaimed { room_id: RoomId, slot_id: SlotId, player_id: PlayerId },
    SlotReleased { room_id: RoomId, slot_id: SlotId },
    SlotPromptUpdated { room_id: RoomId, slot_id: SlotId, updated_by: PlayerId },

    RoomLocked { room_id: RoomId },
    RoomUnlocked { room_id: RoomId },
    RoomStarted { room_id: RoomId, match_id: MatchId },
    RoomFinished { room_id: RoomId, match_id: MatchId, result: crate::domain::match_::MatchResult },
    RoomReset { room_id: RoomId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchLiveEvent {
    MatchCreated { room_id: RoomId, match_id: MatchId },
    MatchStarted { room_id: RoomId, match_id: MatchId },
    MatchTurnStarted { room_id: RoomId, match_id: MatchId, turn: u32 },
    MatchTurnResolved {
        room_id: RoomId,
        match_id: MatchId,
        turn: u32,
        snapshot: PublicGameState,
    },
    MatchScoreUpdated {
        room_id: RoomId,
        match_id: MatchId,
        score: HashMap<TeamId, i32>,
    },
    MatchFinished { room_id: RoomId, match_id: MatchId, result: crate::domain::match_::MatchResult },
    MatchFailed { room_id: RoomId, match_id: MatchId, error: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_event_serialization() {
        let event = GameEvent::new(
            EventId::new("evt-1"),
            MatchId::new("match-1"),
            1,
            42,
            GameEventType::AgentMoved,
            Some(AgentId::new("agent-1")),
            serde_json::json!({ "x": 5, "y": 6 }),
            1717171717,
        );

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: GameEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }
}
