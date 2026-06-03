use crate::domain::ids::{AgentId, NodeId};
use crate::domain::map::Position;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillName {
    Push,
    FocusShot,
    Heal,
    HoldPosition,
    Mark,
    Hack,
    Guard,
    Reposition,
    Revive,
    ShieldPatch,
    Barrier,
    Scan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillTarget {
    Position(Position),
    Agent(AgentId),
    Node(NodeId),
    SelfTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementDecision {
    Stay,
    MoveTo(Position),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionDecision {
    Wait,
    Attack { target: AgentId },
    Defend,
    UseSkill { skill: SkillName, target: SkillTarget },
    CaptureOrReinforceNode { node_id: NodeId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDecision {
    pub intent: Option<String>,
    pub movement: MovementDecision,
    pub action: ActionDecision,
}

impl AgentDecision {
    pub fn new(intent: Option<String>, movement: MovementDecision, action: ActionDecision) -> Self {
        Self { intent, movement, action }
    }

    pub fn fallback() -> Self {
        Self {
            intent: Some("Fallback action due to invalid decision".to_string()),
            movement: MovementDecision::Stay,
            action: ActionDecision::Wait,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_decision() {
        let dec = AgentDecision::fallback();
        assert_eq!(dec.movement, MovementDecision::Stay);
        assert_eq!(dec.action, ActionDecision::Wait);
    }
}
