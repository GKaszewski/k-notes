use async_trait::async_trait;
use k_core::ai::embeddings::FastEmbedAdapter as CoreFastEmbed;
use notes_domain::errors::{DomainError, DomainResult};
use notes_domain::ports::EmbeddingGenerator;
use std::sync::Arc;

pub struct FastEmbedAdapter {
    inner: Arc<CoreFastEmbed>,
}

impl FastEmbedAdapter {
    pub fn new(inner: CoreFastEmbed) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

#[async_trait]
impl EmbeddingGenerator for FastEmbedAdapter {
    async fn generate_embedding(&self, text: &str) -> DomainResult<Vec<f32>> {
        self.inner
            .generate_embedding_async(text)
            .await
            .map_err(|e| {
                DomainError::InfrastructureError(format!("Embedding generation failed: {}", e))
            })
    }
}
