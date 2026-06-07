use async_trait::async_trait;

use super::{
    entity::{Tag, TagId},
    value_objects::TagName,
};
use crate::{errors::DomainResult, note::entity::NoteId, user::entity::UserId};

#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn find_by_id(&self, id: &TagId) -> DomainResult<Option<Tag>>;
    async fn find_by_user(&self, user_id: &UserId) -> DomainResult<Vec<Tag>>;
    async fn find_by_name(&self, user_id: &UserId, name: &TagName) -> DomainResult<Option<Tag>>;
    async fn find_by_note(&self, note_id: &NoteId) -> DomainResult<Vec<Tag>>;
    async fn save(&self, tag: &Tag) -> DomainResult<()>;
    async fn delete(&self, id: &TagId) -> DomainResult<()>;
    async fn add_to_note(&self, tag_id: &TagId, note_id: &NoteId) -> DomainResult<()>;
    async fn remove_from_note(&self, tag_id: &TagId, note_id: &NoteId) -> DomainResult<()>;
}
