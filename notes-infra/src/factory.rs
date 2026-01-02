use std::sync::Arc;

#[cfg(feature = "sqlite")]
use crate::{SqliteNoteRepository, SqliteTagRepository, SqliteUserRepository};
use k_core::db::DatabaseConfig;
use k_core::db::DatabasePool;
use notes_domain::{NoteRepository, TagRepository, UserRepository};

#[derive(Debug, thiserror::Error)]
pub enum FactoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] notes_domain::DomainError),
}

pub type FactoryResult<T> = Result<T, FactoryError>;

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
            let adapter = crate::embeddings::fastembed::FastEmbedAdapter::new()?;
            Ok(Arc::new(adapter))
        }
    }
}

#[cfg(feature = "smart-features")]
pub async fn build_vector_store(
    provider: &VectorProvider,
) -> FactoryResult<Arc<dyn notes_domain::ports::VectorStore>> {
    match provider {
        VectorProvider::Qdrant { url, collection } => {
            let adapter = crate::vector::qdrant::QdrantVectorAdapter::new(url, collection)?;
            adapter.create_collection_if_not_exists().await?;
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
            let broker = crate::broker::nats::NatsMessageBroker::connect(url)
                .await
                .map_err(|e| {
                    FactoryError::Infrastructure(notes_domain::DomainError::RepositoryError(
                        format!("NATS connection failed: {}", e),
                    ))
                })?;
            Ok(Some(Arc::new(broker)))
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

pub async fn build_database_pool(db_config: &DatabaseConfig) -> FactoryResult<DatabasePool> {
    if db_config.url.starts_with("sqlite:") {
        #[cfg(feature = "sqlite")]
        {
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect(&db_config.url)
                .await?;
            Ok(DatabasePool::Sqlite(pool))
        }
        #[cfg(not(feature = "sqlite"))]
        Err(FactoryError::NotImplemented(
            "SQLite feature not enabled".to_string(),
        ))
    } else if db_config.url.starts_with("postgres:") {
        #[cfg(feature = "postgres")]
        {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&db_config.url)
                .await?;
            Ok(DatabasePool::Postgres(pool))
        }
        #[cfg(not(feature = "postgres"))]
        Err(FactoryError::NotImplemented(
            "Postgres feature not enabled".to_string(),
        ))
    } else {
        Err(FactoryError::NotImplemented(format!(
            "Unsupported database URL scheme in: {}",
            db_config.url
        )))
    }
}

pub async fn build_note_repository(pool: &DatabasePool) -> FactoryResult<Arc<dyn NoteRepository>> {
    match pool {
        #[cfg(feature = "sqlite")]
        DatabasePool::Sqlite(pool) => Ok(Arc::new(SqliteNoteRepository::new(pool.clone()))),
        #[cfg(feature = "postgres")]
        DatabasePool::Postgres(_) => Err(FactoryError::NotImplemented(
            "Postgres NoteRepository".to_string(),
        )),
        #[allow(unreachable_patterns)]
        _ => Err(FactoryError::NotImplemented(
            "No database feature enabled".to_string(),
        )),
    }
}

pub async fn build_tag_repository(pool: &DatabasePool) -> FactoryResult<Arc<dyn TagRepository>> {
    match pool {
        #[cfg(feature = "sqlite")]
        DatabasePool::Sqlite(pool) => Ok(Arc::new(SqliteTagRepository::new(pool.clone()))),
        #[cfg(feature = "postgres")]
        DatabasePool::Postgres(_) => Err(FactoryError::NotImplemented(
            "Postgres TagRepository".to_string(),
        )),
        #[allow(unreachable_patterns)]
        _ => Err(FactoryError::NotImplemented(
            "No database feature enabled".to_string(),
        )),
    }
}

pub async fn build_user_repository(pool: &DatabasePool) -> FactoryResult<Arc<dyn UserRepository>> {
    match pool {
        #[cfg(feature = "sqlite")]
        DatabasePool::Sqlite(pool) => Ok(Arc::new(SqliteUserRepository::new(pool.clone()))),
        #[cfg(feature = "postgres")]
        DatabasePool::Postgres(_) => Err(FactoryError::NotImplemented(
            "Postgres UserRepository".to_string(),
        )),
        #[allow(unreachable_patterns)]
        _ => Err(FactoryError::NotImplemented(
            "No database feature enabled".to_string(),
        )),
    }
}

pub async fn build_session_store(
    pool: &DatabasePool,
) -> FactoryResult<crate::session_store::InfraSessionStore> {
    match pool {
        #[cfg(feature = "sqlite")]
        DatabasePool::Sqlite(pool) => {
            let store = tower_sessions_sqlx_store::SqliteStore::new(pool.clone());
            Ok(crate::session_store::InfraSessionStore::Sqlite(store))
        }
        #[cfg(feature = "postgres")]
        DatabasePool::Postgres(pool) => {
            let store = tower_sessions_sqlx_store::PostgresStore::new(pool.clone());
            Ok(crate::session_store::InfraSessionStore::Postgres(store))
        }
        #[allow(unreachable_patterns)]
        _ => Err(FactoryError::NotImplemented(
            "No database feature enabled".to_string(),
        )),
    }
}
