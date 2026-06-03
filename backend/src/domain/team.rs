//! Team and role domain types.

use serde::{Deserialize, Serialize};

use crate::domain::TeamId;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamSide {
    A,
    B,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoleName {
    Vanguard,
    Striker,
    Medic,
    Guardian,
    Scout,
    Engineer,
}

impl RoleName {
    pub const ALL: [Self; 6] = [
        Self::Vanguard,
        Self::Striker,
        Self::Medic,
        Self::Guardian,
        Self::Scout,
        Self::Engineer,
    ];

    #[must_use]
    pub const fn stats(self) -> RoleStats {
        match self {
            Self::Vanguard => RoleStats {
                hp: 14,
                movement: 2,
                attack_range: 1,
                attack_damage: 2,
            },
            Self::Striker => RoleStats {
                hp: 9,
                movement: 3,
                attack_range: 3,
                attack_damage: 3,
            },
            Self::Medic => RoleStats {
                hp: 8,
                movement: 3,
                attack_range: 2,
                attack_damage: 1,
            },
            Self::Guardian => RoleStats {
                hp: 12,
                movement: 2,
                attack_range: 2,
                attack_damage: 2,
            },
            Self::Scout => RoleStats {
                hp: 8,
                movement: 4,
                attack_range: 2,
                attack_damage: 1,
            },
            Self::Engineer => RoleStats {
                hp: 10,
                movement: 2,
                attack_range: 3,
                attack_damage: 2,
            },
        }
    }

    #[must_use]
    pub const fn as_slug(self) -> &'static str {
        match self {
            Self::Vanguard => "vanguard",
            Self::Striker => "striker",
            Self::Medic => "medic",
            Self::Guardian => "guardian",
            Self::Scout => "scout",
            Self::Engineer => "engineer",
        }
    }

    #[must_use]
    pub const fn default_prompt(self) -> &'static str {
        match self {
            Self::Vanguard => "Protect allies and contest the nearest node.",
            Self::Striker => "Find safe angles and attack vulnerable enemies.",
            Self::Medic => "Stay near allies and prioritize keeping them active.",
            Self::Guardian => "Hold important ground and defend nearby teammates.",
            Self::Scout => "Scout ahead, reveal threats, and pressure nodes.",
            Self::Engineer => "Support node control and reinforce tactical positions.",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleStats {
    pub hp: i32,
    pub movement: u32,
    pub attack_range: u32,
    pub attack_damage: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub side: TeamSide,
}

impl Team {
    #[must_use]
    pub fn new(id: TeamId, name: impl Into<String>, side: TeamSide) -> Self {
        Self {
            id,
            name: name.into(),
            side,
        }
    }
}
