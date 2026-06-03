use crate::domain::ids::PlayerId;
use crate::domain::error::DomainError;
use serde::{Serialize, Deserialize};

pub type Timestamp = u64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Player {
    pub id: PlayerId,
    pub display_name: String,
    pub token_hash: String,
    pub joined_at: Timestamp,
    pub last_seen_at: Timestamp,
}

impl Player {
    pub fn new(id: PlayerId, display_name: String, token_hash: String, joined_at: Timestamp) -> Self {
        Self {
            id,
            display_name,
            token_hash,
            joined_at,
            last_seen_at: joined_at,
        }
    }
}

pub fn validate_display_name(name: &str) -> Result<String, DomainError> {
    let trimmed = name.trim();
    let char_count = trimmed.chars().count();
    if char_count < 1 || char_count > 20 {
        return Err(DomainError::InvalidDisplayName(format!(
            "Display name must be between 1 and 20 characters, got length {}",
            char_count
        )));
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return Err(DomainError::InvalidDisplayName(
            "Display name cannot contain control characters".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_display_name_valid() {
        assert_eq!(validate_display_name("  John Doe  ").unwrap(), "John Doe");
        assert_eq!(validate_display_name("🚀 Agent 007 🚀").unwrap(), "🚀 Agent 007 🚀");
        assert_eq!(validate_display_name("A").unwrap(), "A");
        assert_eq!(validate_display_name("12345678901234567890").unwrap(), "12345678901234567890");
    }

    #[test]
    fn test_validate_display_name_invalid_length() {
        assert!(validate_display_name("").is_err());
        assert!(validate_display_name("   ").is_err());
        assert!(validate_display_name("123456789012345678901").is_err());
    }

    #[test]
    fn test_validate_display_name_control_chars() {
        assert!(validate_display_name("John\nDoe").is_err());
        assert!(validate_display_name("John\tDoe").is_err());
        assert!(validate_display_name("John\u{0000}Doe").is_err());
    }
}
