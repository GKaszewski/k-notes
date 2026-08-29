use std::sync::Arc;

use domain::{
    auth::ports::RefreshSessionRepository,
    events::{EventConsumer, EventPublisher},
    note::ports::{LinkRepository, NoteRepository},
    smart::ports::{EmbeddingGenerator, VectorStore},
    tag::ports::TagRepository,
    user::ports::{PasswordHasher, UserRepository},
};

use crate::config::AppConfig;

#[derive(Clone)]
pub struct Repositories {
    pub note: Arc<dyn NoteRepository>,
    pub tag: Arc<dyn TagRepository>,
    pub user: Arc<dyn UserRepository>,
    pub link: Arc<dyn LinkRepository>,
    pub refresh_session: Arc<dyn RefreshSessionRepository>,
}

#[derive(Clone)]
pub struct Services {
    pub password_hasher: Arc<dyn PasswordHasher>,
    pub event_publisher: Arc<dyn EventPublisher>,
    /// None when smart features are not configured.
    pub embedding: Option<Arc<dyn EmbeddingGenerator>>,
    /// None when smart features are not configured.
    pub vector_store: Option<Arc<dyn VectorStore>>,
    pub event_consumer: Arc<dyn EventConsumer>,
}

#[derive(Clone)]
pub struct AppContext {
    pub repos: Repositories,
    pub services: Services,
    pub config: AppConfig,
}
