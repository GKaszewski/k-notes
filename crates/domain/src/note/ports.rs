use async_trait::async_trait;

use super::entity::{Note, NoteFilter, NoteId, NoteLink, NoteVersion};
use crate::{errors::DomainResult, user::entity::UserId};

#[async_trait]
pub trait NoteRepository: Send + Sync {
    async fn find_by_id(&self, id: &NoteId) -> DomainResult<Option<Note>>;
    async fn find_by_user(&self, user_id: &UserId, filter: NoteFilter) -> DomainResult<Vec<Note>>;
    async fn search(&self, user_id: &UserId, query: &str) -> DomainResult<Vec<Note>>;
    async fn save(&self, note: &Note) -> DomainResult<()>;
    async fn delete(&self, id: &NoteId) -> DomainResult<()>;
    async fn save_version(&self, version: &NoteVersion) -> DomainResult<()>;
    async fn find_versions(&self, note_id: &NoteId) -> DomainResult<Vec<NoteVersion>>;
}

#[async_trait]
pub trait LinkRepository: Send + Sync {
    async fn save_links(&self, links: &[NoteLink]) -> DomainResult<()>;
    async fn delete_for_source(&self, source_id: &NoteId) -> DomainResult<()>;
    async fn find_for_note(&self, note_id: &NoteId) -> DomainResult<Vec<NoteLink>>;
}
