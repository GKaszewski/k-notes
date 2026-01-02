//! Value Objects for K-Notes Domain
//!
//! Newtypes that encapsulate validation logic, following the "parse, don't validate" pattern.
//! These types can only be constructed if the input is valid, providing compile-time guarantees.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use thiserror::Error;

// ============================================================================
// Validation Error
// ============================================================================

/// Errors that occur when parsing/validating value objects
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("Password must be at least {min} characters, got {actual}")]
    PasswordTooShort { min: usize, actual: usize },

    #[error("Tag name must be 1-{max} characters, got {actual}")]
    InvalidTagNameLength { max: usize, actual: usize },

    #[error("Tag name cannot be empty")]
    EmptyTagName,

    #[error("Note title cannot exceed {max} characters, got {actual}")]
    TitleTooLong { max: usize, actual: usize },
}

// ============================================================================
// Email
// ============================================================================

/// A validated email address.
///
/// Simple validation: must contain exactly one `@` with non-empty parts on both sides.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    /// Minimum validation: contains @ with non-empty local and domain parts
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim().to_lowercase();

        // Basic email validation
        let parts: Vec<&str> = trimmed.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(ValidationError::InvalidEmail(value));
        }

        // Domain must contain at least one dot
        if !parts[1].contains('.') {
            return Err(ValidationError::InvalidEmail(value));
        }

        Ok(Self(trimmed))
    }

    /// Get the inner value
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Email {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Email {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Serialize for Email {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

// ============================================================================
// Password
// ============================================================================

/// A validated password input (NOT the hash).
///
/// Enforces minimum length of 6 characters.
#[derive(Clone, PartialEq, Eq)]
pub struct Password(String);

/// Minimum password length
pub const MIN_PASSWORD_LENGTH: usize = 6;

impl Password {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();

        if value.len() < MIN_PASSWORD_LENGTH {
            return Err(ValidationError::PasswordTooShort {
                min: MIN_PASSWORD_LENGTH,
                actual: value.len(),
            });
        }

        Ok(Self(value))
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// Intentionally hide password content in Debug
impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Password(***)")
    }
}

impl TryFrom<String> for Password {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Password {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for Password {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

// Note: Password should NOT implement Serialize to prevent accidental exposure

// ============================================================================
// TagName
// ============================================================================

/// A validated tag name.
///
/// Enforces: 1-50 characters, trimmed and lowercase.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagName(String);

/// Maximum tag name length
pub const MAX_TAG_NAME_LENGTH: usize = 50;

impl TagName {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim().to_lowercase();

        if trimmed.is_empty() {
            return Err(ValidationError::EmptyTagName);
        }

        if trimmed.len() > MAX_TAG_NAME_LENGTH {
            return Err(ValidationError::InvalidTagNameLength {
                max: MAX_TAG_NAME_LENGTH,
                actual: trimmed.len(),
            });
        }

        Ok(Self(trimmed))
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for TagName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TagName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for TagName {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for TagName {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Serialize for TagName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for TagName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

// ============================================================================
// NoteTitle
// ============================================================================

/// A validated note title.
///
/// Enforces: maximum 200 characters when present. Trimmed but preserves case.
/// Note: This is for the inner value; the title on a Note is Option<NoteTitle>.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoteTitle(String);

/// Maximum note title length
pub const MAX_NOTE_TITLE_LENGTH: usize = 200;

impl NoteTitle {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let trimmed = value.trim().to_string();

        if trimmed.len() > MAX_NOTE_TITLE_LENGTH {
            return Err(ValidationError::TitleTooLong {
                max: MAX_NOTE_TITLE_LENGTH,
                actual: trimmed.len(),
            });
        }

        // Allow empty strings - this becomes None at the Note level
        Ok(Self(trimmed))
    }

    /// Create from optional string, returning None for empty/whitespace
    pub fn from_optional(value: Option<String>) -> Result<Option<Self>, ValidationError> {
        match value {
            None => Ok(None),
            Some(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    Ok(None)
                } else {
                    Self::new(trimmed).map(Some)
                }
            }
        }
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    /// Check if the title is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl AsRef<str> for NoteTitle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NoteTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for NoteTitle {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for NoteTitle {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Serialize for NoteTitle {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for NoteTitle {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod email_tests {
        use super::*;

        #[test]
        fn test_valid_email() {
            assert!(Email::new("user@example.com").is_ok());
            assert!(Email::new("USER@EXAMPLE.COM").is_ok()); // Should lowercase
            assert!(Email::new("  user@example.com  ").is_ok()); // Should trim
        }

        #[test]
        fn test_email_normalizes() {
            let email = Email::new("  USER@EXAMPLE.COM  ").unwrap();
            assert_eq!(email.as_ref(), "user@example.com");
        }

        #[test]
        fn test_invalid_email_no_at() {
            assert!(Email::new("userexample.com").is_err());
        }

        #[test]
        fn test_invalid_email_no_domain() {
            assert!(Email::new("user@").is_err());
        }

        #[test]
        fn test_invalid_email_no_local() {
            assert!(Email::new("@example.com").is_err());
        }

        #[test]
        fn test_invalid_email_no_dot_in_domain() {
            assert!(Email::new("user@localhost").is_err());
        }
    }

    mod password_tests {
        use super::*;

        #[test]
        fn test_valid_password() {
            assert!(Password::new("secret123").is_ok());
            assert!(Password::new("123456").is_ok()); // Exactly 6 chars
        }

        #[test]
        fn test_password_too_short() {
            assert!(Password::new("12345").is_err()); // 5 chars
            assert!(Password::new("").is_err());
        }

        #[test]
        fn test_password_debug_hides_content() {
            let password = Password::new("supersecret").unwrap();
            let debug = format!("{:?}", password);
            assert!(!debug.contains("supersecret"));
            assert!(debug.contains("***"));
        }
    }

    mod tag_name_tests {
        use super::*;

        #[test]
        fn test_valid_tag_name() {
            assert!(TagName::new("work").is_ok());
            assert!(TagName::new("  WORK  ").is_ok()); // Should trim and lowercase
        }

        #[test]
        fn test_tag_name_normalizes() {
            let tag = TagName::new("  Important  ").unwrap();
            assert_eq!(tag.as_ref(), "important");
        }

        #[test]
        fn test_empty_tag_name_fails() {
            assert!(TagName::new("").is_err());
            assert!(TagName::new("   ").is_err());
        }

        #[test]
        fn test_tag_name_max_length() {
            let long_name = "a".repeat(MAX_TAG_NAME_LENGTH);
            assert!(TagName::new(&long_name).is_ok());

            let too_long = "a".repeat(MAX_TAG_NAME_LENGTH + 1);
            assert!(TagName::new(&too_long).is_err());
        }
    }

    mod note_title_tests {
        use super::*;

        #[test]
        fn test_valid_title() {
            assert!(NoteTitle::new("My Note").is_ok());
            assert!(NoteTitle::new("").is_ok()); // Empty is valid for NoteTitle
        }

        #[test]
        fn test_title_trims() {
            let title = NoteTitle::new("  My Note  ").unwrap();
            assert_eq!(title.as_ref(), "My Note");
        }

        #[test]
        fn test_title_max_length() {
            let long_title = "a".repeat(MAX_NOTE_TITLE_LENGTH);
            assert!(NoteTitle::new(&long_title).is_ok());

            let too_long = "a".repeat(MAX_NOTE_TITLE_LENGTH + 1);
            assert!(NoteTitle::new(&too_long).is_err());
        }

        #[test]
        fn test_from_optional_none() {
            assert_eq!(NoteTitle::from_optional(None).unwrap(), None);
        }

        #[test]
        fn test_from_optional_empty() {
            assert_eq!(NoteTitle::from_optional(Some("".into())).unwrap(), None);
            assert_eq!(NoteTitle::from_optional(Some("   ".into())).unwrap(), None);
        }

        #[test]
        fn test_from_optional_valid() {
            let result = NoteTitle::from_optional(Some("My Note".into())).unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap().as_ref(), "My Note");
        }
    }
}
