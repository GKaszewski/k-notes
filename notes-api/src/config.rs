#[cfg(feature = "smart-features")]
use notes_infra::factory::{EmbeddingProvider, VectorProvider};
use std::env;

/// Server configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub session_secret: String,
    pub cors_allowed_origins: Vec<String>,
    pub allow_registration: bool,
    #[cfg(feature = "smart-features")]
    pub embedding_provider: EmbeddingProvider,
    #[cfg(feature = "smart-features")]
    pub vector_provider: VectorProvider,
    pub broker_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            database_url: "sqlite:data.db?mode=rwc".to_string(),
            session_secret: "k-notes-super-secret-key-must-be-at-least-64-bytes-long!!!!"
                .to_string(),
            cors_allowed_origins: vec!["http://localhost:5173".to_string()],
            allow_registration: true,
            #[cfg(feature = "smart-features")]
            embedding_provider: EmbeddingProvider::FastEmbed,
            #[cfg(feature = "smart-features")]
            vector_provider: VectorProvider::Qdrant {
                url: "http://localhost:6334".to_string(),
                collection: "notes".to_string(),
            },
            broker_url: "nats://localhost:4222".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        // Load .env file if it exists, ignore errors if it doesn't
        let _ = dotenvy::dotenv();

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data.db?mode=rwc".to_string());

        let session_secret = env::var("SESSION_SECRET").unwrap_or_else(|_| {
            "k-notes-super-secret-key-must-be-at-least-64-bytes-long!!!!".to_string()
        });

        let cors_origins_str = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());

        let cors_allowed_origins = cors_origins_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let allow_registration = env::var("ALLOW_REGISTRATION")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(true);

        #[cfg(feature = "smart-features")]
        let embedding_provider = match env::var("EMBEDDING_PROVIDER").unwrap_or_default().as_str() {
            // Future: "ollama" => EmbeddingProvider::Ollama(...),
            _ => EmbeddingProvider::FastEmbed,
        };

        #[cfg(feature = "smart-features")]
        let vector_provider = match env::var("VECTOR_PROVIDER").unwrap_or_default().as_str() {
            // Future: "postgres" => ...
            _ => VectorProvider::Qdrant {
                url: env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string()),
                collection: env::var("QDRANT_COLLECTION").unwrap_or_else(|_| "notes".to_string()),
            },
        };

        let broker_url =
            env::var("BROKER_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

        Self {
            host,
            port,
            database_url,
            session_secret,
            cors_allowed_origins,
            allow_registration,
            #[cfg(feature = "smart-features")]
            embedding_provider,
            #[cfg(feature = "smart-features")]
            vector_provider,
            broker_url,
        }
    }
}
