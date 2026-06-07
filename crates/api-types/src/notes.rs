use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::tags::TagResponse;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateNoteRequest {
    pub title: Option<String>,
    #[serde(default)]
    pub content: String,
    pub color: Option<String>,
    #[serde(default)]
    pub is_pinned: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Default, utoipa::IntoParams)]
pub struct ListNotesParams {
    pub pinned: Option<bool>,
    pub archived: Option<bool>,
    /// Filter by tag name.
    pub tag: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SearchParams {
    pub q: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AddTagRequest {
    pub tag_name: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct PinRequest {
    pub pinned: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ArchiveRequest {
    pub archived: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NoteResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: Option<String>,
    pub content: String,
    pub color: String,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<TagResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NoteVersionResponse {
    pub id: Uuid,
    pub note_id: Uuid,
    pub title: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NoteLinkResponse {
    pub source_id: Uuid,
    pub target_id: Uuid,
    /// Cosine similarity score in [0.0, 1.0].
    pub score: f32,
    pub created_at: DateTime<Utc>,
}
