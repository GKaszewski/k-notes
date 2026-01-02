use async_trait::async_trait;
use k_core::ai::qdrant::QdrantAdapter as CoreQdrant;
use notes_domain::errors::{DomainError, DomainResult};
use notes_domain::ports::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct QdrantVectorAdapter {
    inner: Arc<CoreQdrant>,
}

impl QdrantVectorAdapter {
    pub fn new(inner: CoreQdrant) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    pub async fn init(&self) -> DomainResult<()> {
        self.inner
            .create_collection_if_not_exists(384)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))
    }
}

#[async_trait]
impl VectorStore for QdrantVectorAdapter {
    async fn upsert(&self, id: Uuid, vector: &[f32]) -> DomainResult<()> {
        let payload = HashMap::new();

        self.inner
            .upsert(id, vector.to_vec(), payload)
            .await
            .map_err(|e| DomainError::InfrastructureError(format!("Qdrant upsert error: {}", e)))
    }

    async fn find_similar(&self, vector: &[f32], limit: usize) -> DomainResult<Vec<(Uuid, f32)>> {
        self.inner
            .search(vector.to_vec(), limit as u64)
            .await
            .map_err(|e| DomainError::InfrastructureError(format!("Qdrant search error: {}", e)))
    }
}
