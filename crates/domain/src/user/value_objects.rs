use std::fmt;

use crate::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(email_address::EmailAddress);

impl Email {
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let s = value.as_ref().trim().to_lowercase();
        s.parse::<email_address::EmailAddress>()
            .map(Self)
            .map_err(|e| DomainError::Validation(format!("invalid email: {e}")))
    }

    pub fn into_inner(self) -> String {
        self.0.to_string()
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Email {
    type Error = DomainError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<&str> for Email {
    type Error = DomainError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

/// Unverified plaintext password. Not stored — only used for auth operations.
#[derive(Clone, PartialEq, Eq)]
pub struct Password(String);

pub const MIN_PASSWORD_LENGTH: usize = 8;

impl Password {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let v = value.into();
        if v.len() < MIN_PASSWORD_LENGTH {
            return Err(DomainError::Validation(format!(
                "password must be at least {MIN_PASSWORD_LENGTH} characters"
            )));
        }
        Ok(Self(v))
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

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Password(***)")
    }
}

/// Stored password hash — opaque to the domain, managed by PasswordHasher port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[cfg(test)]
#[path = "tests/value_objects.rs"]
mod tests;
