//! Runtime agent and game-state domain types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{AgentId, GameMap, Position, RoleName, SkillName, TeamId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    pub match_id: crate::domain::MatchId,
    pub turn: u32,
    pub map: GameMap,
    pub teams: Vec<GameTeam>,
    pub agents: Vec<GameAgent>,
    pub nodes: Vec<crate::domain::ControlNode>,
    pub score: HashMap<TeamId, i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTeam {
    pub id: TeamId,
    pub name: String,
    pub side: crate::domain::TeamSide,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameAgent {
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
    pub cooldowns: HashMap<SkillName, u32>,
}

impl GameAgent {
    #[must_use]
    pub fn new(
        id: AgentId,
        team_id: TeamId,
        role: RoleName,
        display_name: Option<String>,
        position: Position,
    ) -> Self {
        let stats = role.stats();
        Self {
            id,
            team_id,
            role,
            display_name,
            hp: stats.hp,
            max_hp: stats.hp,
            position,
            status: AgentStatus::Active,
            marked: false,
            shield: 0,
            cooldowns: HashMap::new(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Downed { turns_remaining_before_respawn: u32 },
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
pub struct PublicAgentState {
    pub id: AgentId,
    pub team_id: TeamId,
    pub role: RoleName,
    pub display_name: Option<String>,
    pub hp: i32,
    pub max_hp: i32,
    pub position: Position,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicNodeState {
    pub id: crate::domain::NodeId,
    pub name: String,
    pub position: Position,
    pub score_per_turn: i32,
    pub controlled_by: Option<TeamId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicMapSummary {
    pub map_id: crate::domain::MapId,
    pub width: usize,
    pub height: usize,
}
