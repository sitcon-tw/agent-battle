//! Player domain types and display-name validation.

use serde::{Deserialize, Serialize};

use crate::domain::{DisplayNameValidationError, DomainError, PlayerId, Timestamp};

pub const MAX_DISPLAY_NAME_CHARS: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub display_name: String,
    pub token_hash: String,
    pub joined_at: Timestamp,
    pub last_seen_at: Timestamp,
}

impl Player {
    /// Creates a player with a validated, trimmed display name.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidDisplayName`] when the display name is empty, too long, or
    /// contains a control character after trimming.
    pub fn new(
        id: PlayerId,
        display_name: impl AsRef<str>,
        token_hash: impl Into<String>,
        timestamp: Timestamp,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            display_name: validate_display_name(display_name.as_ref())?,
            token_hash: token_hash.into(),
            joined_at: timestamp,
            last_seen_at: timestamp,
        })
    }
}

/// Validates and trims a player display name.
///
/// # Errors
///
/// Returns [`DomainError::InvalidDisplayName`] when the resulting name violates MVP rules.
pub fn validate_display_name(value: &str) -> Result<String, DomainError> {
    let trimmed = value.trim();
    let length = trimmed.chars().count();

    if length == 0 {
        return Err(DisplayNameValidationError::Empty.into());
    }

    if length > MAX_DISPLAY_NAME_CHARS {
        return Err(DisplayNameValidationError::TooLong {
            length,
            max: MAX_DISPLAY_NAME_CHARS,
        }
        .into());
    }

    if trimmed.chars().any(char::is_control) {
        return Err(DisplayNameValidationError::ContainsControlCharacter.into());
    }

    Ok(trimmed.to_owned())
}
