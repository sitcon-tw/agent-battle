use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("Invalid display name: {0}")]
    InvalidDisplayName(String),

    #[error("Room error: {0}")]
    RoomError(String),

    #[error("Match error: {0}")]
    MatchError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Map error: {0}")]
    MapError(String),
}
