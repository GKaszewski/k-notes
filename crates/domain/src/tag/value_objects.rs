use std::fmt;

use crate::errors::DomainError;

pub const MAX_TAG_NAME_LENGTH: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagName(String);

impl TagName {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let s = value.into().trim().to_lowercase();
        if s.is_empty() {
            return Err(DomainError::Validation("tag name cannot be empty".into()));
        }
        if s.len() > MAX_TAG_NAME_LENGTH {
            return Err(DomainError::Validation(format!(
                "tag name exceeds {MAX_TAG_NAME_LENGTH} characters"
            )));
        }
        Ok(Self(s))
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
    type Error = DomainError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<&str> for TagName {
    type Error = DomainError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

#[cfg(test)]
#[path = "tests/value_objects.rs"]
mod tests;
