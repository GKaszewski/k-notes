use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use notes_domain::errors::{DomainError, DomainResult};
use notes_domain::ports::EmbeddingGenerator;
use std::sync::{Arc, Mutex};

pub struct FastEmbedAdapter {
    model: Arc<Mutex<TextEmbedding>>,
}

impl FastEmbedAdapter {
    pub fn new() -> DomainResult<Self> {
        let mut options = InitOptions::default();
        options.model_name = EmbeddingModel::AllMiniLML6V2;
        options.show_download_progress = false;

        let model = TextEmbedding::try_new(options).map_err(|e| {
            DomainError::InfrastructureError(format!("Failed to init fastembed: {}", e))
        })?;

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }
}

#[async_trait]
impl EmbeddingGenerator for FastEmbedAdapter {
    async fn generate_embedding(&self, text: &str) -> DomainResult<Vec<f32>> {
        let model = self.model.clone();
        let text = text.to_string();

        let embeddings = tokio::task::spawn_blocking(move || {
            let mut model = model.lock().map_err(|e| format!("Lock error: {}", e))?;
            model
                .embed(vec![text], None)
                .map_err(|e| format!("Embed error: {}", e))
        })
        .await
        .map_err(|e| DomainError::InfrastructureError(format!("Join error: {}", e)))?
        .map_err(|e| DomainError::InfrastructureError(e))?;

        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| DomainError::InfrastructureError("No embedding generated".to_string()))
    }
}
