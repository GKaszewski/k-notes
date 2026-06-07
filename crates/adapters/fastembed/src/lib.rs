use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};

use domain::{
    errors::{DomainError, DomainResult},
    smart::ports::EmbeddingGenerator,
};

pub struct FastEmbedConfig {
    pub model: EmbeddingModel,
    /// Directory used to cache downloaded model files.
    /// Defaults to the system cache directory when `None`.
    pub cache_dir: Option<PathBuf>,
    pub show_download_progress: bool,
}

impl Default for FastEmbedConfig {
    fn default() -> Self {
        Self {
            model: EmbeddingModel::AllMiniLML6V2,
            cache_dir: None,
            show_download_progress: false,
        }
    }
}

impl FastEmbedConfig {
    pub fn with_model(model: EmbeddingModel) -> Self {
        Self {
            model,
            ..Default::default()
        }
    }
}

pub struct FastEmbedGenerator {
    model: Arc<Mutex<TextEmbedding>>,
}

impl FastEmbedGenerator {
    /// Initialise the model. Downloads and caches model files on first call.
    pub fn new(config: FastEmbedConfig) -> Result<Self, DomainError> {
        let mut opts = TextInitOptions::new(config.model)
            .with_show_download_progress(config.show_download_progress);

        if let Some(dir) = config.cache_dir {
            opts = opts.with_cache_dir(dir);
        }

        let model = TextEmbedding::try_new(opts)
            .map_err(|e| DomainError::Infrastructure(format!("fastembed init failed: {e}")))?;

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }
}

#[async_trait]
impl EmbeddingGenerator for FastEmbedGenerator {
    async fn generate(&self, text: &str) -> DomainResult<Vec<f32>> {
        let model = Arc::clone(&self.model);
        let text = text.to_owned();

        tokio::task::spawn_blocking(move || {
            let mut guard = model
                .lock()
                .map_err(|_| DomainError::Infrastructure("model mutex poisoned".into()))?;
            guard
                .embed(vec![text.as_str()], None)
                .map_err(|e| DomainError::Infrastructure(format!("embedding failed: {e}")))?
                .into_iter()
                .next()
                .ok_or_else(|| DomainError::Infrastructure("no embedding returned".into()))
        })
        .await
        .map_err(|e| DomainError::Infrastructure(format!("spawn_blocking panicked: {e}")))?
    }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
