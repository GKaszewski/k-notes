//! Request and Response DTOs for notes API

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use notes_domain::{Note, Tag};

/// Request to create a new note
#[derive(Debug, Deserialize, Validate)]
pub struct CreateNoteRequest {
    #[validate(length(max = 200, message = "Title must be at most 200 characters"))]
    pub title: String,

    #[serde(default)]
    pub content: String,

    #[serde(default)]
    #[validate(length(max = 10, message = "Maximum 10 tags allowed"))]
    pub tags: Vec<String>,

    pub color: Option<String>,

    #[serde(default)]
    pub is_pinned: bool,
}

/// Request to update an existing note (all fields optional)
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateNoteRequest {
    #[validate(length(max = 200, message = "Title must be at most 200 characters"))]
    pub title: Option<String>,

    pub content: Option<String>,

    #[validate(length(max = 10, message = "Maximum 10 tags allowed"))]
    pub tags: Option<Vec<String>>,

    pub color: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_archived: Option<bool>,
}

/// Query parameters for listing notes
#[derive(Debug, Deserialize, Default)]
pub struct ListNotesQuery {
    pub pinned: Option<bool>,
    pub archived: Option<bool>,
    /// Tag name to filter by (will be looked up by route handler)
    pub tag: Option<String>,
}

/// Query parameters for search
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// Tag response DTO
#[derive(Debug, Serialize)]
pub struct TagResponse {
    pub id: Uuid,
    pub name: String,
}

impl From<Tag> for TagResponse {
    fn from(tag: Tag) -> Self {
        Self {
            id: tag.id,
            name: tag.name.into_inner(), // Convert TagName to String
        }
    }
}

/// Note response DTO
#[derive(Debug, Serialize)]
pub struct NoteResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub color: String,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<TagResponse>,
}

impl From<Note> for NoteResponse {
    fn from(note: Note) -> Self {
        Self {
            id: note.id,
            title: note.title_str().to_string(), // Convert Option<NoteTitle> to String
            content: note.content,
            color: note.color,
            is_pinned: note.is_pinned,
            is_archived: note.is_archived,
            created_at: note.created_at,
            updated_at: note.updated_at,
            tags: note.tags.into_iter().map(TagResponse::from).collect(),
        }
    }
}

/// Request to create a new tag
#[derive(Debug, Deserialize, Validate)]
pub struct CreateTagRequest {
    #[validate(length(min = 1, max = 50, message = "Tag name must be 1-50 characters"))]
    pub name: String,
}

/// Request to rename a tag
#[derive(Debug, Deserialize, Validate)]
pub struct RenameTagRequest {
    #[validate(length(min = 1, max = 50, message = "Tag name must be 1-50 characters"))]
    pub name: String,
}

/// Login request
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
}

/// Register request
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
}

/// User response DTO
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

/// Note Version response DTO
#[derive(Debug, Serialize)]
pub struct NoteVersionResponse {
    pub id: Uuid,
    pub note_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl From<notes_domain::NoteVersion> for NoteVersionResponse {
    fn from(version: notes_domain::NoteVersion) -> Self {
        Self {
            id: version.id,
            note_id: version.note_id,
            title: version.title.unwrap_or_default(), // Convert Option<String> to String
            content: version.content,
            created_at: version.created_at,
        }
    }
}

/// System configuration response
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub allow_registration: bool,
}

/// Note Link response DTO
#[derive(Debug, Serialize)]
pub struct NoteLinkResponse {
    pub source_note_id: Uuid,
    pub target_note_id: Uuid,
    pub score: f32,
    pub created_at: DateTime<Utc>,
}

impl From<notes_domain::entities::NoteLink> for NoteLinkResponse {
    fn from(link: notes_domain::entities::NoteLink) -> Self {
        Self {
            source_note_id: link.source_note_id,
            target_note_id: link.target_note_id,
            score: link.score,
            created_at: link.created_at,
        }
    }
}
