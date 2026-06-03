use crate::domain::ids::{AgentId, TeamId};
use crate::domain::slot::RoleName;
use crate::domain::map::Position;
use crate::domain::action::SkillName;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Downed { turns_remaining_before_respawn: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleStats {
    pub hp: i32,
    pub movement: u32,
    pub attack_range: u32,
    pub attack_damage: i32,
}

impl RoleStats {
    pub fn for_role(role: RoleName) -> Self {
        match role {
            RoleName::Vanguard => RoleStats { hp: 14, movement: 2, attack_range: 1, attack_damage: 2 },
            RoleName::Striker => RoleStats { hp: 9, movement: 3, attack_range: 3, attack_damage: 3 },
            RoleName::Medic => RoleStats { hp: 8, movement: 3, attack_range: 2, attack_damage: 1 },
            RoleName::Guardian => RoleStats { hp: 12, movement: 2, attack_range: 2, attack_damage: 2 },
            RoleName::Scout => RoleStats { hp: 8, movement: 4, attack_range: 2, attack_damage: 1 },
            RoleName::Engineer => RoleStats { hp: 10, movement: 2, attack_range: 3, attack_damage: 2 },
        }
    }
}

impl RoleName {
    pub fn stats(self) -> RoleStats {
        RoleStats::for_role(self)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_stats_mapping() {
        let stats = RoleName::Vanguard.stats();
        assert_eq!(stats.hp, 14);
        assert_eq!(stats.movement, 2);
        assert_eq!(stats.attack_range, 1);
        assert_eq!(stats.attack_damage, 2);

        let stats_scout = RoleName::Scout.stats();
        assert_eq!(stats_scout.hp, 8);
        assert_eq!(stats_scout.movement, 4);
    }

    #[test]
    fn test_game_agent_new() {
        let agent = GameAgent::new(
            AgentId::new("agent-1"),
            TeamId::new("team-A"),
            RoleName::Medic,
            Some("Doc".to_string()),
            Position::new(1, 2),
        );

        assert_eq!(agent.id, AgentId::new("agent-1"));
        assert_eq!(agent.team_id, TeamId::new("team-A"));
        assert_eq!(agent.role, RoleName::Medic);
        assert_eq!(agent.display_name, Some("Doc".to_string()));
        assert_eq!(agent.hp, 8);
        assert_eq!(agent.max_hp, 8);
        assert_eq!(agent.position, Position::new(1, 2));
        assert_eq!(agent.status, AgentStatus::Active);
        assert!(!agent.marked);
        assert_eq!(agent.shield, 0);
        assert!(agent.cooldowns.is_empty());
    }
}
