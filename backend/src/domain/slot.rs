//! Room agent slots.

use serde::{Deserialize, Serialize};

use crate::domain::{PlayerId, RoleName, SlotId, TeamId};

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
    #[must_use]
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
