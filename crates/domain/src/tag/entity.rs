use std::fmt;

use uuid::Uuid;

use super::value_objects::TagName;
use crate::user::entity::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagId(Uuid);

impl TagId {
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

impl Default for TagId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: TagId,
    pub name: TagName,
    pub user_id: UserId,
}

impl Tag {
    pub fn new(name: TagName, user_id: UserId) -> Self {
        Self {
            id: TagId::new(),
            name,
            user_id,
        }
    }

    pub fn from_row(id: TagId, name: TagName, user_id: UserId) -> Self {
        Self { id, name, user_id }
    }
}
