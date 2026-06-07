use serde::{Deserialize, Serialize};

/// A note in portable backup format (no IDs — uses names and content only).
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BackupNote {
    pub title: Option<String>,
    pub content: String,
    pub color: String,
    pub is_pinned: bool,
    pub is_archived: bool,
    /// Tag names associated with this note.
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BackupData {
    pub notes: Vec<BackupNote>,
}
