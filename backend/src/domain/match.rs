//! Match configuration and lifecycle domain types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{AgentId, MapId, MatchId, RoleName, RoomId, TeamId, TeamSide, Timestamp};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchStatus {
    Created,
    Running,
    Finished,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Match {
    pub id: MatchId,
    pub room_id: RoomId,
    pub status: MatchStatus,
    pub config_snapshot: MatchConfigSnapshot,
    pub current_turn: u32,
    pub result: Option<MatchResult>,
    pub created_at: Timestamp,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
}

impl Match {
    #[must_use]
    pub fn new(
        id: MatchId,
        room_id: RoomId,
        config_snapshot: MatchConfigSnapshot,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            id,
            room_id,
            status: MatchStatus::Created,
            config_snapshot,
            current_turn: 0,
            result: None,
            created_at: timestamp,
            started_at: None,
            finished_at: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchConfigSnapshot {
    pub map_id: MapId,
    pub max_turns: u32,
    pub teams: Vec<MatchTeamConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchTeamConfig {
    pub id: TeamId,
    pub name: String,
    pub side: TeamSide,
    pub agents: Vec<MatchAgentConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchAgentConfig {
    pub agent_id: AgentId,
    pub role: RoleName,
    pub display_name: Option<String>,
    pub source_player_id: Option<crate::domain::PlayerId>,
    pub controller: AgentControllerConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentControllerConfig {
    Llm { prompt: String },
    Scripted { strategy_id: String },
    Random,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResult {
    pub winner: Option<TeamId>,
    pub score: HashMap<TeamId, i32>,
    pub reason: MatchEndReason,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchEndReason {
    MaxTurnsReached,
    Cancelled,
    Failed,
}
