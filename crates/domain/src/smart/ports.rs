use async_trait::async_trait;

use crate::{errors::DomainResult, note::entity::NoteId};

#[async_trait]
pub trait EmbeddingGenerator: Send + Sync {
    async fn generate(&self, text: &str) -> DomainResult<Vec<f32>>;
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn upsert(&self, id: &NoteId, vector: &[f32]) -> DomainResult<()>;
    async fn find_similar(&self, vector: &[f32], limit: usize) -> DomainResult<Vec<(NoteId, f32)>>;
    async fn delete(&self, id: &NoteId) -> DomainResult<()>;
}
