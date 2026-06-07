pub mod config;

use std::sync::Arc;

use application::context::{AppContext, Repositories, Services};
use auth::password::Argon2PasswordHasher;
use domain::{
    events::{EventConsumer, EventPublisher},
    smart::ports::{EmbeddingGenerator, VectorStore},
};

type OptEmbedding = Option<Arc<dyn EmbeddingGenerator>>;
type OptVectorStore = Option<Arc<dyn VectorStore>>;
use event_publisher_memory::MemoryEventBus;
use fastembed_adapter::{FastEmbedConfig, FastEmbedGenerator};
use qdrant_adapter::{QdrantConfig, QdrantVectorStore};
use sqlite::{
    db::{connect, run_migrations},
    link::SqliteLinkRepository,
    note::SqliteNoteRepository,
    tag::SqliteTagRepository,
    user::SqliteUserRepository,
};

pub use config::WiringConfig;

/// Assemble a fully wired `AppContext` from the given configuration.
///
/// Runs database migrations, connects to all configured external services,
/// and returns an `AppContext` ready to be handed to `WorkerService` or
/// the presentation layer.
pub async fn build_context(cfg: &WiringConfig) -> anyhow::Result<AppContext> {
    // ── Database ──────────────────────────────────────────────────────────────
    tracing::info!("connecting to database");
    let pool = connect(&cfg.database_url).await?;
    run_migrations(&pool).await?;
    tracing::info!("migrations applied");

    let repos = Repositories {
        note: Arc::new(SqliteNoteRepository::new(pool.clone())),
        tag: Arc::new(SqliteTagRepository::new(pool.clone())),
        user: Arc::new(SqliteUserRepository::new(pool.clone())),
        link: Arc::new(SqliteLinkRepository::new(pool.clone())),
    };

    // ── Auth ──────────────────────────────────────────────────────────────────
    let password_hasher = Arc::new(Argon2PasswordHasher);

    // ── Event bus ─────────────────────────────────────────────────────────────
    let (event_publisher, event_consumer): (Arc<dyn EventPublisher>, Arc<dyn EventConsumer>) =
        if let Some(ref url) = cfg.nats_url {
            tracing::info!("connecting to NATS at {url}");
            let (pub_, con) = nats::setup(url, cfg.jetstream_config())
                .await
                .map_err(|e| anyhow::anyhow!("nats setup failed: {e}"))?;
            tracing::info!("NATS JetStream ready");
            (Arc::new(pub_), Arc::new(con))
        } else {
            tracing::info!("no NATS_URL — using in-memory event bus");
            let bus = MemoryEventBus::new();
            (bus.publisher(), bus.consumer())
        };

    // ── Smart features ────────────────────────────────────────────────────────
    // EmbeddingGenerator: only load the fastembed model in the worker.
    // The backend only needs VectorStore (for querying related notes).
    // Loading the model in both processes wastes ~150 MB per process.
    let embedding: OptEmbedding = if cfg.enable_embeddings && cfg.qdrant_url.is_some() {
        tracing::info!("loading fastembed embedding model");
        let embedder = FastEmbedGenerator::new(FastEmbedConfig::default())
            .map_err(|e| anyhow::anyhow!("fastembed init failed: {e}"))?;
        Some(Arc::new(embedder) as Arc<dyn EmbeddingGenerator>)
    } else {
        None
    };

    let vector_store: OptVectorStore = if let Some(ref url) = cfg.qdrant_url {
        tracing::info!("connecting to qdrant at {url}");
        let qdrant = QdrantVectorStore::new(QdrantConfig {
            url: url.clone(),
            collection: cfg.qdrant_collection.clone(),
            vector_size: cfg.qdrant_vector_size,
        })
        .map_err(|e| anyhow::anyhow!("qdrant client init failed: {e}"))?;
        qdrant.init(cfg.qdrant_vector_size).await?;
        tracing::info!(collection = %cfg.qdrant_collection, "qdrant collection ready");
        Some(Arc::new(qdrant) as Arc<dyn VectorStore>)
    } else {
        tracing::info!("no QDRANT_URL — smart features disabled");
        None
    };

    Ok(AppContext {
        repos,
        services: Services {
            password_hasher,
            event_publisher,
            event_consumer,
            embedding,
            vector_store,
        },
        config: cfg.app_config(),
    })
}
