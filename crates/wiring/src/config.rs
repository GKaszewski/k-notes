use std::time::Duration;

use application::config::{AppConfig, SmartConfig};
use nats::JetStreamConfig;

/// Full wiring configuration, sourced from environment variables.
///
/// Call `WiringConfig::from_env()` at startup. All fields have documented
/// env var names and defaults.
#[derive(Debug, Clone)]
pub struct WiringConfig {
    /// `DATABASE_URL` — SQLite file path, e.g. `sqlite://data.db`
    pub database_url: String,

    /// `NATS_URL` — if set, NATS JetStream is used for events.
    /// If absent, an in-memory bus is used (suitable for single-process dev).
    pub nats_url: Option<String>,

    /// `QDRANT_URL` — if set, smart features (embeddings + semantic links) are
    /// enabled. If absent, `AppContext::services.embedding` and `vector_store`
    /// are `None`.
    pub qdrant_url: Option<String>,

    /// `QDRANT_COLLECTION` — collection name. Default: `"notes"`.
    pub qdrant_collection: String,

    /// `QDRANT_VECTOR_SIZE` — must match the embedding model's output dimension.
    /// Default: `384` (AllMiniLML6V2).
    pub qdrant_vector_size: u64,

    /// `BASE_URL` — public base URL, e.g. `http://localhost:3000`.
    pub base_url: String,

    /// `SMART_NEIGHBOUR_LIMIT` — max similar notes to link per note. Default: `10`.
    pub smart_neighbour_limit: usize,

    /// `SMART_MIN_SIMILARITY` — cosine similarity threshold for links. Default: `0.7`.
    pub smart_min_similarity: f32,

    /// `ALLOW_REGISTRATION` — set to `false` to disable the register endpoint.
    /// Default: `true`.
    pub allow_registration: bool,

    /// `NATS_STREAM_NAME` — JetStream stream name. Default: `"KNOTES"`.
    pub nats_stream_name: String,

    /// `NATS_CONSUMER_NAME` — durable consumer name. Default: `"knotes-worker"`.
    pub nats_consumer_name: String,

    /// `NATS_MAX_DELIVER` — max delivery attempts before a message is dead.
    /// Default: `5`.
    pub nats_max_deliver: i64,

    /// `ENABLE_EMBEDDINGS` — load the fastembed model and generate embeddings.
    /// Should only be `true` in the worker process. The backend only needs
    /// `VectorStore` (for querying related notes), not `EmbeddingGenerator`.
    /// Default: `false`.
    pub enable_embeddings: bool,
}

impl WiringConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: require_env("DATABASE_URL")?,
            nats_url: optional_env("NATS_URL"),
            qdrant_url: optional_env("QDRANT_URL"),
            qdrant_collection: optional_env("QDRANT_COLLECTION").unwrap_or_else(|| "notes".into()),
            qdrant_vector_size: parse_env("QDRANT_VECTOR_SIZE", 384)?,
            base_url: optional_env("BASE_URL").unwrap_or_else(|| "http://localhost:3000".into()),
            smart_neighbour_limit: parse_env("SMART_NEIGHBOUR_LIMIT", 10)?,
            smart_min_similarity: parse_env("SMART_MIN_SIMILARITY", 0.7f32)?,
            nats_stream_name: optional_env("NATS_STREAM_NAME").unwrap_or_else(|| "KNOTES".into()),
            nats_consumer_name: optional_env("NATS_CONSUMER_NAME")
                .unwrap_or_else(|| "knotes-worker".into()),
            nats_max_deliver: parse_env("NATS_MAX_DELIVER", 5i64)?,
            allow_registration: optional_env("ALLOW_REGISTRATION")
                .map(|s| s != "false" && s != "0")
                .unwrap_or(true),
            enable_embeddings: optional_env("ENABLE_EMBEDDINGS")
                .map(|s| s == "true" || s == "1")
                .unwrap_or(false),
        })
    }

    pub(crate) fn app_config(&self) -> AppConfig {
        AppConfig {
            base_url: self.base_url.clone(),
            smart: SmartConfig {
                neighbour_limit: self.smart_neighbour_limit,
                min_similarity: self.smart_min_similarity,
            },
            allow_registration: self.allow_registration,
        }
    }

    pub(crate) fn jetstream_config(&self) -> JetStreamConfig {
        JetStreamConfig {
            stream_name: self.nats_stream_name.clone(),
            consumer_name: self.nats_consumer_name.clone(),
            max_deliver: self.nats_max_deliver,
            ack_wait: Duration::from_secs(30),
        }
    }
}

fn require_env(key: &str) -> anyhow::Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("{key} must be set"))
}

fn optional_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

fn parse_env<T: std::str::FromStr + ToString>(key: &str, default: T) -> anyhow::Result<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match std::env::var(key) {
        Ok(val) => val
            .parse::<T>()
            .map_err(|e| anyhow::anyhow!("invalid {key}={val}: {e}")),
        Err(_) => Ok(default),
    }
}
