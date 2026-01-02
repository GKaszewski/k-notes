use std::sync::Arc;

#[cfg(feature = "sqlite")]
use crate::{SqliteNoteRepository, SqliteTagRepository, SqliteUserRepository};
use k_core::db::DatabasePool;
use k_core::session::store::InfraSessionStore;
use notes_domain::{NoteRepository, TagRepository, UserRepository};

#[cfg(feature = "smart-features")]
use crate::embeddings::fastembed::FastEmbedAdapter;
#[cfg(feature = "smart-features")]
use crate::vector::qdrant::QdrantVectorAdapter;
#[cfg(feature = "smart-features")]
use k_core::ai::{
    embeddings::FastEmbedAdapter as CoreFastEmbed, qdrant::QdrantAdapter as CoreQdrant,
};
#[cfg(feature = "smart-features")]
use k_core::broker::nats::NatsBroker;

#[derive(Debug, thiserror::Error)]
pub enum FactoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] notes_domain::DomainError),
}

pub type FactoryResult<T> = anyhow::Result<T>;

#[cfg(feature = "smart-features")]
#[derive(Debug, Clone)]
pub enum EmbeddingProvider {
    FastEmbed,
    // Ollama(String), // Url
    // OpenAI(String), // ApiKey
}

#[cfg(feature = "smart-features")]
#[derive(Debug, Clone)]
pub enum VectorProvider {
    Qdrant { url: String, collection: String },
    // InMemory,
}

#[cfg(feature = "smart-features")]
pub async fn build_embedding_generator(
    provider: &EmbeddingProvider,
) -> FactoryResult<Arc<dyn notes_domain::ports::EmbeddingGenerator>> {
    match provider {
        EmbeddingProvider::FastEmbed => {
            let core_embed = CoreFastEmbed::new()?;
            Ok(Arc::new(FastEmbedAdapter::new(core_embed)))
        }
    }
}

#[cfg(feature = "smart-features")]
pub async fn build_vector_store(
    provider: &VectorProvider,
) -> FactoryResult<Arc<dyn notes_domain::ports::VectorStore>> {
    match provider {
        VectorProvider::Qdrant { url, collection } => {
            let core_qdrant = CoreQdrant::new(url, collection)?;
            let adapter = QdrantVectorAdapter::new(core_qdrant);
            adapter.init().await.map_err(|e| anyhow::anyhow!(e))?;
            Ok(Arc::new(adapter))
        }
    }
}

/// Configuration for message broker providers.
#[derive(Debug, Clone)]
pub enum BrokerProvider {
    /// NATS message broker (requires `broker-nats` feature).
    #[cfg(feature = "broker-nats")]
    Nats { url: String },
    /// No message broker (messaging disabled).
    None,
}

/// Build a message broker based on the provider configuration.
/// Returns `None` if `BrokerProvider::None` is specified.
pub async fn build_message_broker(
    provider: &BrokerProvider,
) -> FactoryResult<Option<Arc<dyn notes_domain::MessageBroker>>> {
    match provider {
        #[cfg(feature = "broker-nats")]
        BrokerProvider::Nats { url } => {
            let core_broker = NatsBroker::connect(url).await?;
            let adapter = crate::broker::nats::NatsMessageBroker::new(core_broker);
            Ok(Some(Arc::new(adapter)))
        }
        BrokerProvider::None => Ok(None),
    }
}

#[cfg(feature = "sqlite")]
pub async fn build_link_repository(
    pool: &DatabasePool,
) -> FactoryResult<Arc<dyn notes_domain::ports::LinkRepository>> {
    match pool {
        DatabasePool::Sqlite(pool) => Ok(Arc::new(
            crate::link_repository::SqliteLinkRepository::new(pool.clone()),
        )),
    }
}

pub async fn build_note_repository(pool: &DatabasePool) -> FactoryResult<Arc<dyn NoteRepository>> {
    match pool {
        #[cfg(feature = "sqlite")]
        DatabasePool::Sqlite(pool) => Ok(Arc::new(SqliteNoteRepository::new(pool.clone()))),
        #[cfg(feature = "postgres")]
        DatabasePool::Postgres(_) => anyhow::bail!("Postgres NoteRepository not implemented"),
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("No database feature enabled"),
    }
}

pub async fn build_tag_repository(pool: &DatabasePool) -> FactoryResult<Arc<dyn TagRepository>> {
    match pool {
        #[cfg(feature = "sqlite")]
        DatabasePool::Sqlite(pool) => Ok(Arc::new(SqliteTagRepository::new(pool.clone()))),
        #[cfg(feature = "postgres")]
        DatabasePool::Postgres(_) => anyhow::bail!("Postgres TagRepository not implemented"),
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("No database feature enabled"),
    }
}

pub async fn build_user_repository(pool: &DatabasePool) -> FactoryResult<Arc<dyn UserRepository>> {
    match pool {
        #[cfg(feature = "sqlite")]
        DatabasePool::Sqlite(pool) => Ok(Arc::new(SqliteUserRepository::new(pool.clone()))),
        #[cfg(feature = "postgres")]
        DatabasePool::Postgres(_) => anyhow::bail!("Postgres UserRepository not implemented"),
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("No database feature enabled"),
    }
}

pub async fn build_session_store(pool: &DatabasePool) -> Result<InfraSessionStore, sqlx::Error> {
    Ok(match pool {
        #[cfg(feature = "sqlite")]
        DatabasePool::Sqlite(p) => {
            InfraSessionStore::Sqlite(tower_sessions_sqlx_store::SqliteStore::new(p.clone()))
        }
        #[cfg(feature = "postgres")]
        DatabasePool::Postgres(p) => {
            InfraSessionStore::Postgres(tower_sessions_sqlx_store::PostgresStore::new(p.clone()))
        }
    })
}
