use crate::domain::ids::{RoomId, RoomCode, MatchId, MapId};
use crate::domain::player::{Player, Timestamp};
use crate::domain::team::Team;
use crate::domain::slot::AgentSlot;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoomStatus {
    Open,
    Locked,
    Running,
    Finished,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameMode {
    PromptOpsArena,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomConfig {
    pub mode: GameMode,
    pub map_id: MapId,
    pub max_turns: u32,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            mode: GameMode::PromptOpsArena,
            map_id: MapId::new("default-15x9"),
            max_turns: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub code: RoomCode,
    pub status: RoomStatus,
    pub config: RoomConfig,

    pub players: Vec<Player>,
    pub teams: Vec<Team>,
    pub slots: Vec<AgentSlot>,

    pub match_id: Option<MatchId>,

    pub version: u64,

    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Room {
    pub fn new(id: RoomId, code: RoomCode, config: RoomConfig, created_at: Timestamp) -> Self {
        Self {
            id,
            code,
            status: RoomStatus::Open,
            config,
            players: Vec::new(),
            teams: Vec::new(),
            slots: Vec::new(),
            match_id: None,
            version: 1,
            created_at,
            updated_at: created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_new() {
        let room_id = RoomId::new("room-1");
        let room_code = RoomCode::new("ABCD");
        let config = RoomConfig::default();
        let timestamp = 1717171717;

        let room = Room::new(room_id.clone(), room_code.clone(), config.clone(), timestamp);

        assert_eq!(room.id, room_id);
        assert_eq!(room.code, room_code);
        assert_eq!(room.status, RoomStatus::Open);
        assert_eq!(room.config, config);
        assert!(room.players.is_empty());
        assert!(room.teams.is_empty());
        assert!(room.slots.is_empty());
        assert_eq!(room.match_id, None);
        assert_eq!(room.version, 1);
        assert_eq!(room.created_at, timestamp);
        assert_eq!(room.updated_at, timestamp);
    }
}
