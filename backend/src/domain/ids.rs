use std::fmt;
use serde::{Serialize, Deserialize};

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(val: impl Into<String>) -> Self {
                Self(val.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<String> for $name {
            fn from(val: String) -> Self {
                Self(val)
            }
        }

        impl From<&str> for $name {
            fn from(val: &str) -> Self {
                Self(val.to_string())
            }
        }
    };
}

define_id!(RoomId);
define_id!(PlayerId);
define_id!(TeamId);
define_id!(SlotId);
define_id!(MatchId);
define_id!(MapId);
define_id!(AgentId);
define_id!(NodeId);
define_id!(EventId);
define_id!(CommandId);
define_id!(RoomCode);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ids_basic() {
        let room_id = RoomId::new("room-123");
        assert_eq!(room_id.as_str(), "room-123");
        assert_eq!(room_id.to_string(), "room-123");
        
        let room_id_from = RoomId::from("room-123");
        assert_eq!(room_id, room_id_from);

        let serialized = serde_json::to_string(&room_id).unwrap();
        assert_eq!(serialized, "\"room-123\"");

        let deserialized: RoomId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(room_id, deserialized);
    }
}
