use std::sync::Arc;

use crate::config::Config;
use notes_domain::{
    NoteRepository, NoteService, TagRepository, TagService, UserRepository, UserService,
};

/// Application state holding all dependencies
#[derive(Clone)]
pub struct AppState {
    pub note_repo: Arc<dyn NoteRepository>,
    pub tag_repo: Arc<dyn TagRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    #[cfg(feature = "smart-features")]
    pub link_repo: Arc<dyn notes_domain::ports::LinkRepository>,
    pub note_service: Arc<NoteService>,
    pub tag_service: Arc<TagService>,
    pub user_service: Arc<UserService>,
    #[cfg(feature = "smart-features")]
    pub nats_client: async_nats::Client,
    pub config: Config,
}

impl AppState {
    pub fn new(
        note_repo: Arc<dyn NoteRepository>,
        tag_repo: Arc<dyn TagRepository>,
        user_repo: Arc<dyn UserRepository>,
        #[cfg(feature = "smart-features")] link_repo: Arc<dyn notes_domain::ports::LinkRepository>,
        note_service: Arc<NoteService>,
        tag_service: Arc<TagService>,
        user_service: Arc<UserService>,
        #[cfg(feature = "smart-features")] nats_client: async_nats::Client,
        config: Config,
    ) -> Self {
        Self {
            note_repo,
            tag_repo,
            user_repo,
            #[cfg(feature = "smart-features")]
            link_repo,
            note_service,
            tag_service,
            user_service,
            #[cfg(feature = "smart-features")]
            nats_client,
            config,
        }
    }
}
