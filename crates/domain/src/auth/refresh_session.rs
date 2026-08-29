use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::user::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RefreshSessionId(Uuid);

impl RefreshSessionId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct RefreshSession {
    id: RefreshSessionId,
    user_id: UserId,
    token: String,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl RefreshSession {
    pub fn new(user_id: UserId, ttl_seconds: i64) -> Self {
        let now = Utc::now();
        Self {
            id: RefreshSessionId::generate(),
            user_id,
            token: Uuid::new_v4().to_string(),
            expires_at: now + Duration::seconds(ttl_seconds),
            created_at: now,
        }
    }

    pub fn from_persistence(
        id: RefreshSessionId,
        user_id: UserId,
        token: String,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            token,
            expires_at,
            created_at,
        }
    }

    pub fn id(&self) -> RefreshSessionId {
        self.id
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
