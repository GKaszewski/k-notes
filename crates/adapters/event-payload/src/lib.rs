use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::{
    errors::DomainError, events::DomainEvent, note::entity::NoteId, user::entity::UserId,
};

/// Wire-format representation of a DomainEvent.
/// Uses primitive types only — no domain newtypes — so it is stable across
/// schema versions and safe to serialize to any transport (NATS, HTTP, file).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum EventPayload {
    NoteCreated { note_id: String, user_id: String },
    NoteUpdated { note_id: String, user_id: String },
    NoteDeleted { note_id: String, user_id: String },
}

impl EventPayload {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::NoteCreated { .. } => "NoteCreated",
            Self::NoteUpdated { .. } => "NoteUpdated",
            Self::NoteDeleted { .. } => "NoteDeleted",
        }
    }

    pub fn to_json(&self) -> Result<Vec<u8>, DomainError> {
        serde_json::to_vec(self)
            .map_err(|e| DomainError::Infrastructure(format!("serialize failed: {e}")))
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, DomainError> {
        serde_json::from_slice(bytes)
            .map_err(|e| DomainError::Infrastructure(format!("deserialize failed: {e}")))
    }
}

impl From<&DomainEvent> for EventPayload {
    fn from(event: &DomainEvent) -> Self {
        match event {
            DomainEvent::NoteCreated { note_id, user_id } => Self::NoteCreated {
                note_id: note_id.as_uuid().to_string(),
                user_id: user_id.as_uuid().to_string(),
            },
            DomainEvent::NoteUpdated { note_id, user_id } => Self::NoteUpdated {
                note_id: note_id.as_uuid().to_string(),
                user_id: user_id.as_uuid().to_string(),
            },
            DomainEvent::NoteDeleted { note_id, user_id } => Self::NoteDeleted {
                note_id: note_id.as_uuid().to_string(),
                user_id: user_id.as_uuid().to_string(),
            },
        }
    }
}

impl TryFrom<EventPayload> for DomainEvent {
    type Error = DomainError;

    fn try_from(payload: EventPayload) -> Result<Self, Self::Error> {
        fn parse(s: &str) -> Result<Uuid, DomainError> {
            Uuid::parse_str(s)
                .map_err(|e| DomainError::Infrastructure(format!("invalid uuid '{s}': {e}")))
        }

        match payload {
            EventPayload::NoteCreated { note_id, user_id } => Ok(DomainEvent::NoteCreated {
                note_id: NoteId::from_uuid(parse(&note_id)?),
                user_id: UserId::from_uuid(parse(&user_id)?),
            }),
            EventPayload::NoteUpdated { note_id, user_id } => Ok(DomainEvent::NoteUpdated {
                note_id: NoteId::from_uuid(parse(&note_id)?),
                user_id: UserId::from_uuid(parse(&user_id)?),
            }),
            EventPayload::NoteDeleted { note_id, user_id } => Ok(DomainEvent::NoteDeleted {
                note_id: NoteId::from_uuid(parse(&note_id)?),
                user_id: UserId::from_uuid(parse(&user_id)?),
            }),
        }
    }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
