//! Replay events and decision logs.

use serde::{Deserialize, Serialize};

use crate::domain::{AgentDecision, AgentId, AgentObservation, EventId, MatchId, Timestamp};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameEventType {
    MatchStarted,
    TurnStarted,
    AgentDecisionReceived,
    InvalidActionReplaced,
    AgentMoved,
    MovementConflict,
    AgentAttacked,
    SkillUsed,
    DamageDealt,
    AgentHealed,
    AgentDowned,
    AgentRespawned,
    NodeControlChanged,
    ScoreChanged,
    TurnFinished,
    MatchFinished,
    MatchFailed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
