use std::sync::Arc;

use notes_domain::{
    NoteRepository, NoteService, TagRepository, TagService, UserRepository, UserService,
};

/// Application state holding all dependencies
#[derive(Clone)]
pub struct AppState {
    pub note_repo: Arc<dyn NoteRepository>,
    pub tag_repo: Arc<dyn TagRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub note_service: Arc<NoteService>,
    pub tag_service: Arc<TagService>,
    pub user_service: Arc<UserService>,
}

impl AppState {
    pub fn new(
        note_repo: Arc<dyn NoteRepository>,
        tag_repo: Arc<dyn TagRepository>,
        user_repo: Arc<dyn UserRepository>,
        note_service: Arc<NoteService>,
        tag_service: Arc<TagService>,
        user_service: Arc<UserService>,
    ) -> Self {
        Self {
            note_repo,
            tag_repo,
            user_repo,
            note_service,
            tag_service,
            user_service,
        }
    }
}
