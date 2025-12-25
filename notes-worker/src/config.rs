#[cfg(feature = "smart-features")]
use notes_infra::factory::{EmbeddingProvider, VectorProvider};

#[derive(Debug, Clone)]
pub struct Config {
    pub broker_url: String,
    pub database_url: String,
    #[cfg(feature = "smart-features")]
    pub embedding_provider: EmbeddingProvider,
    #[cfg(feature = "smart-features")]
    pub vector_provider: VectorProvider,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            broker_url: "nats://localhost:4222".to_string(),
            database_url: "sqlite::memory:".to_string(),
            #[cfg(feature = "smart-features")]
            embedding_provider: EmbeddingProvider::FastEmbed,
            #[cfg(feature = "smart-features")]
            vector_provider: VectorProvider::Qdrant {
                url: "http://localhost:6334".to_string(),
                collection: "notes".to_string(),
            },
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        #[cfg(feature = "smart-features")]
        let embedding_provider = match std::env::var("EMBEDDING_PROVIDER")
            .unwrap_or_default()
            .as_str()
        {
            _ => EmbeddingProvider::FastEmbed,
        };

        #[cfg(feature = "smart-features")]
        let vector_provider = match std::env::var("VECTOR_PROVIDER")
            .unwrap_or_default()
            .as_str()
        {
            _ => VectorProvider::Qdrant {
                url: std::env::var("QDRANT_URL")
                    .unwrap_or_else(|_| "http://localhost:6334".to_string()),
                collection: std::env::var("QDRANT_COLLECTION")
                    .unwrap_or_else(|_| "notes".to_string()),
            },
        };

        Self {
            broker_url: std::env::var("BROKER_URL").unwrap_or("nats://localhost:4222".to_string()),
            database_url: std::env::var("DATABASE_URL").unwrap_or("sqlite::memory:".to_string()),
            #[cfg(feature = "smart-features")]
            embedding_provider,
            #[cfg(feature = "smart-features")]
            vector_provider,
        }
    }
}
