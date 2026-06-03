use crate::domain::ids::{SlotId, TeamId, PlayerId};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoleName {
    Vanguard,
    Striker,
    Medic,
    Guardian,
    Scout,
    Engineer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSlot {
    pub id: SlotId,
    pub team_id: TeamId,
    pub role: RoleName,
    pub player_id: Option<PlayerId>,
    pub prompt_draft: Option<String>,
    pub locked_prompt: Option<String>,
}

impl AgentSlot {
    pub fn new(id: SlotId, team_id: TeamId, role: RoleName) -> Self {
        Self {
            id,
            team_id,
            role,
            player_id: None,
            prompt_draft: None,
            locked_prompt: None,
        }
    }
}
