//! Structured agent decisions and actions.

use serde::{Deserialize, Serialize};

use crate::domain::{AgentId, NodeId, Position};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillName {
    Push,
    FocusShot,
    Heal,
    HoldPosition,
    Mark,
    Hack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDecision {
    pub intent: Option<String>,
    pub movement: MovementDecision,
    pub action: ActionDecision,
}

impl AgentDecision {
    #[must_use]
    pub fn fallback() -> Self {
        Self {
            intent: None,
            movement: MovementDecision::Stay,
            action: ActionDecision::Wait,
        }
    }
}

impl Default for AgentDecision {
    fn default() -> Self {
        Self::fallback()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementDecision {
    Stay,
    MoveTo(Position),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionDecision {
    Wait,
    Attack {
        target: AgentId,
    },
    Defend,
    UseSkill {
        skill: SkillName,
        target: SkillTarget,
    },
    CaptureOrReinforceNode {
        node_id: NodeId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillTarget {
    Agent(AgentId),
    Position(Position),
    Node(NodeId),
    None,
}
