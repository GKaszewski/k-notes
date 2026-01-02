use futures_util::StreamExt;
#[cfg(feature = "smart-features")]
use notes_domain::services::SmartNoteService;
#[cfg(feature = "smart-features")]
use notes_infra::factory::{
    BrokerProvider, build_database_pool, build_embedding_generator, build_link_repository,
    build_message_broker, build_vector_store,
};

use crate::config::Config;

mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    k_core::logging::init("notes_worker");

    let config = Config::from_env();

    #[cfg(feature = "smart-features")]
    {
        // Connect to message broker via factory

        use k_core::db::DatabaseConfig;
        tracing::info!("Connecting to message broker: {}", config.broker_url);
        let broker_provider = BrokerProvider::Nats {
            url: config.broker_url.clone(),
        };
        let broker = build_message_broker(&broker_provider)
            .await?
            .expect("Message broker required for worker");

        let db_config = DatabaseConfig::new(config.database_url.clone());
        let db_pool = build_database_pool(&db_config).await?;

        // Initialize smart feature adapters
        let embedding_generator = build_embedding_generator(&config.embedding_provider).await?;
        let vector_store = build_vector_store(&config.vector_provider).await?;
        let link_repo = build_link_repository(&db_pool).await?;

        // Create the service
        let smart_service = SmartNoteService::new(embedding_generator, vector_store, link_repo);
        tracing::info!(
            "SmartNoteService initialized successfully with {:?}",
            config.embedding_provider
        );

        // Subscribe to note update events via the broker's stream API
        let mut note_stream = broker.subscribe_note_updates().await?;
        tracing::info!("Worker listening on 'notes.updated'...");

        while let Some(note) = note_stream.next().await {
            tracing::info!("Processing smart features for note: {}", note.id);
            match smart_service.process_note(&note).await {
                Ok(_) => tracing::info!("Successfully processed note {}", note.id),
                Err(e) => tracing::error!("Failed to process note {}: {}", note.id, e),
            }
        }
    }

    #[cfg(not(feature = "smart-features"))]
    {
        tracing::info!("Smart features are disabled. Worker will exit.");
    }

    Ok(())
}
