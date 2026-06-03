use crate::domain::ids::{MatchId, RoomId, MapId, TeamId, AgentId, PlayerId};
use crate::domain::slot::RoleName;
use crate::domain::team::TeamSide;
use crate::domain::player::Timestamp;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchStatus {
    Created,
    Running,
    Finished,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentControllerConfig {
    Llm { prompt: String },
    Scripted { strategy_id: String },
    Random,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchAgentConfig {
    pub agent_id: AgentId,
    pub role: RoleName,
    pub display_name: Option<String>,
    pub source_player_id: Option<PlayerId>,
    pub controller: AgentControllerConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchTeamConfig {
    pub id: TeamId,
    pub name: String,
    pub side: TeamSide,
    pub agents: Vec<MatchAgentConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchConfigSnapshot {
    pub map_id: MapId,
    pub max_turns: u32,
    pub teams: Vec<MatchTeamConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResult {
    pub winner: Option<TeamId>,
    pub scores: HashMap<TeamId, i32>,
    pub end_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchError {
    pub message: String,
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
    pub fn new(
        id: MatchId,
        room_id: RoomId,
        config_snapshot: MatchConfigSnapshot,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            room_id,
            status: MatchStatus::Created,
            config_snapshot,
            current_turn: 0,
            result: None,
            created_at,
            started_at: None,
            finished_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_creation() {
        let match_id = MatchId::new("match-1");
        let room_id = RoomId::new("room-1");
        let config_snapshot = MatchConfigSnapshot {
            map_id: MapId::new("map-1"),
            max_turns: 8,
            teams: vec![],
        };
        let created_at = 123456789;

        let m = Match::new(match_id.clone(), room_id.clone(), config_snapshot.clone(), created_at);
        assert_eq!(m.id, match_id);
        assert_eq!(m.room_id, room_id);
        assert_eq!(m.status, MatchStatus::Created);
        assert_eq!(m.config_snapshot, config_snapshot);
        assert_eq!(m.current_turn, 0);
        assert_eq!(m.result, None);
        assert_eq!(m.created_at, created_at);
    }
}
