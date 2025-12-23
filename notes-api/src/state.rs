//! Application state for dependency injection

use std::sync::Arc;

use notes_domain::{NoteRepository, TagRepository, UserRepository};

/// Application state holding all dependencies
#[derive(Clone)]
pub struct AppState {
    pub note_repo: Arc<dyn NoteRepository>,
    pub tag_repo: Arc<dyn TagRepository>,
    pub user_repo: Arc<dyn UserRepository>,
}

impl AppState {
    pub fn new(
        note_repo: Arc<dyn NoteRepository>,
        tag_repo: Arc<dyn TagRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            note_repo,
            tag_repo,
            user_repo,
        }
    }
}
