use crate::domain::ids::TeamId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamSide {
    A,
    B,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub side: TeamSide,
}

impl Team {
    pub fn new(id: TeamId, name: String, side: TeamSide) -> Self {
        Self { id, name, side }
    }
}
