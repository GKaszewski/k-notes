use std::fmt;

use crate::errors::DomainError;

pub const MAX_NOTE_TITLE_LENGTH: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoteTitle(String);

impl NoteTitle {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let s = value.into();
        let trimmed = s.trim();
        if trimmed.len() > MAX_NOTE_TITLE_LENGTH {
            return Err(DomainError::Validation(format!(
                "note title exceeds {MAX_NOTE_TITLE_LENGTH} characters"
            )));
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns `None` for empty/whitespace input, `Some` otherwise.
    pub fn from_optional(value: Option<String>) -> Result<Option<Self>, DomainError> {
        match value {
            None => Ok(None),
            Some(s) if s.trim().is_empty() => Ok(None),
            Some(s) => Self::new(s).map(Some),
        }
    }

    pub fn into_inner(self) -> String {
        self.0
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
    type Error = DomainError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<&str> for NoteTitle {
    type Error = DomainError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

/// Background color of a note. Stored as an uppercase string (e.g. "DEFAULT", "#FF5733").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteColor(String);

impl NoteColor {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into().trim().to_uppercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Default for NoteColor {
    fn default() -> Self {
        Self::new("DEFAULT")
    }
}

impl fmt::Display for NoteColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
#[path = "tests/value_objects.rs"]
mod tests;
