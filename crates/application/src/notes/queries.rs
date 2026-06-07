use uuid::Uuid;

use domain::note::entity::NoteFilter;

/// Query to list a user's notes.
/// Provide either `filter.tag_id` (already resolved) **or** `tag_name`
/// (the use case will resolve it). `tag_name` takes precedence.
pub struct ListNotesQuery {
    pub user_id: Uuid,
    pub filter: NoteFilter,
    /// If set, resolves the tag by name before applying the filter.
    pub tag_name: Option<String>,
}

pub struct GetNoteQuery {
    pub note_id: Uuid,
    pub user_id: Uuid,
}

pub struct SearchNotesQuery {
    pub user_id: Uuid,
    pub query: String,
}

pub struct GetVersionsQuery {
    pub note_id: Uuid,
    pub user_id: Uuid,
}

pub struct GetRelatedQuery {
    pub note_id: Uuid,
    pub user_id: Uuid,
}
