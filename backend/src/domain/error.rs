//! Domain-level error types.

use thiserror::Error;

use crate::domain::{PlayerId, RoomStatus, SlotId, TeamSide};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("invalid display name: {0}")]
    InvalidDisplayName(#[from] DisplayNameValidationError),

    #[error("invalid map: {0}")]
    InvalidMap(#[from] MapValidationError),

    #[error("room status {status:?} does not allow {action}")]
    RoomStatusDoesNotAllowAction {
        status: RoomStatus,
        action: &'static str,
    },

    #[error("invalid room status transition from {from:?} to {to:?}")]
    InvalidRoomStatusTransition { from: RoomStatus, to: RoomStatus },

    #[error("slot {slot_id} was not found")]
    SlotNotFound { slot_id: SlotId },

    #[error("slot {slot_id} is already claimed")]
    SlotAlreadyClaimed { slot_id: SlotId },

    #[error("player {player_id} has already claimed slot {slot_id}")]
    PlayerAlreadyClaimedSlot {
        player_id: PlayerId,
        slot_id: SlotId,
    },

    #[error("slot {slot_id} is not owned by player {player_id}")]
    SlotNotOwnedByPlayer {
        slot_id: SlotId,
        player_id: PlayerId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DisplayNameValidationError {
    #[error("display name is empty")]
    Empty,

    #[error("display name is {length} characters long, maximum is {max}")]
    TooLong { length: usize, max: usize },

    #[error("display name contains a control character")]
    ContainsControlCharacter,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MapValidationError {
    #[error("map height is {actual}, expected {expected}")]
    WrongHeight { actual: usize, expected: usize },

    #[error("map width is {actual}, expected {expected}")]
    WrongWidth { actual: usize, expected: usize },

    #[error("map row {row} width is {actual}, expected {expected}")]
    UnequalRowWidth {
        row: usize,
        actual: usize,
        expected: usize,
    },

    #[error("invalid map tile {tile:?} at ({x}, {y})")]
    InvalidTile { tile: char, x: i32, y: i32 },

    #[error("map has {actual} control nodes, expected {expected}")]
    WrongControlNodeCount { actual: usize, expected: usize },

    #[error("map is missing spawn locations for team {side:?}")]
    MissingSpawn { side: TeamSide },
}
