use std::fmt;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::value_objects::{NoteColor, NoteTitle};
use crate::{tag::entity::Tag, user::entity::UserId};

pub const MAX_TAGS_PER_NOTE: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoteId(Uuid);

impl NoteId {
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

impl Default for NoteId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NoteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: NoteId,
    pub user_id: UserId,
    pub title: Option<NoteTitle>,
    pub content: String,
    pub color: NoteColor,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Hydrated by the repository on read. Not managed by Note itself.
    pub tags: Vec<Tag>,
}

impl Note {
    pub fn new(user_id: UserId, title: Option<NoteTitle>, content: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: NoteId::new(),
            user_id,
            title,
            content: content.into(),
            color: NoteColor::default(),
            is_pinned: false,
            is_archived: false,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }

    pub fn set_title(&mut self, title: Option<NoteTitle>) {
        self.title = title;
        self.updated_at = Utc::now();
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.updated_at = Utc::now();
    }

    pub fn set_color(&mut self, color: NoteColor) {
        self.color = color;
        self.updated_at = Utc::now();
    }

    pub fn set_pinned(&mut self, pinned: bool) {
        self.is_pinned = pinned;
        self.updated_at = Utc::now();
    }

    pub fn set_archived(&mut self, archived: bool) {
        self.is_archived = archived;
        self.updated_at = Utc::now();
    }

    pub fn can_add_tag(&self) -> bool {
        self.tags.len() < MAX_TAGS_PER_NOTE
    }
}

/// Snapshot of a note's content at a point in time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteVersion {
    pub id: Uuid,
    pub note_id: NoteId,
    pub title: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl NoteVersion {
    pub fn snapshot(note: &Note) -> Self {
        Self {
            id: Uuid::new_v4(),
            note_id: note.id,
            title: note.title.as_ref().map(|t| t.as_ref().to_string()),
            content: note.content.clone(),
            created_at: Utc::now(),
        }
    }
}

/// Semantic similarity edge between two notes, produced by smart features.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteLink {
    pub source_id: NoteId,
    pub target_id: NoteId,
    /// Cosine similarity score in [0.0, 1.0].
    pub score: f32,
    pub created_at: DateTime<Utc>,
}

impl NoteLink {
    pub fn new(source_id: NoteId, target_id: NoteId, score: f32) -> Self {
        Self {
            source_id,
            target_id,
            score,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NoteFilter {
    pub is_pinned: Option<bool>,
    pub is_archived: Option<bool>,
    pub tag_id: Option<crate::tag::entity::TagId>,
}

impl NoteFilter {
    pub fn pinned(mut self) -> Self {
        self.is_pinned = Some(true);
        self
    }

    pub fn archived(mut self) -> Self {
        self.is_archived = Some(true);
        self
    }

    pub fn not_archived(mut self) -> Self {
        self.is_archived = Some(false);
        self
    }

    pub fn with_tag(mut self, tag_id: crate::tag::entity::TagId) -> Self {
        self.tag_id = Some(tag_id);
        self
    }
}

#[cfg(test)]
#[path = "tests/entity.rs"]
mod tests;
