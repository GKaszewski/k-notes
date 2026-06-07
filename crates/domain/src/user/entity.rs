use std::fmt;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::value_objects::{Email, PasswordHash};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    /// OIDC subject claim; equals email string for local-auth users.
    pub subject: String,
    pub email: Email,
    pub password_hash: Option<PasswordHash>,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new_oidc(subject: impl Into<String>, email: Email) -> Self {
        Self {
            id: UserId::new(),
            subject: subject.into(),
            email,
            password_hash: None,
            created_at: Utc::now(),
        }
    }

    pub fn new_local(email: Email, password_hash: PasswordHash) -> Self {
        let subject = email.as_ref().to_string();
        Self {
            id: UserId::new(),
            subject,
            email,
            password_hash: Some(password_hash),
            created_at: Utc::now(),
        }
    }

    /// Reconstruct from storage. Does not validate business rules.
    pub fn from_row(
        id: UserId,
        subject: String,
        email: Email,
        password_hash: Option<PasswordHash>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            subject,
            email,
            password_hash,
            created_at,
        }
    }
}
