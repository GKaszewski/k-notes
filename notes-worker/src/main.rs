use futures_util::StreamExt;
#[cfg(feature = "smart-features")]
use notes_domain::services::SmartNoteService;
#[cfg(feature = "smart-features")]
use notes_infra::{
    DatabaseConfig,
    factory::{
        build_database_pool, build_embedding_generator, build_link_repository, build_vector_store,
    },
};

use crate::config::Config;

mod config;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "notes_worker=info,notes_infra=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let nats_client = async_nats::connect(&config.broker_url).await?;

    #[cfg(feature = "smart-features")]
    {
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

        // Subscribe to note update events
        let mut subscriber = nats_client.subscribe("notes.updated").await?;
        tracing::info!("Worker listening on 'notes.updated'...");

        while let Some(msg) = subscriber.next().await {
            // Parse message payload (assuming the payload IS the Note JSON)
            let note_result: Result<notes_domain::Note, _> = serde_json::from_slice(&msg.payload);

            match note_result {
                Ok(note) => {
                    tracing::info!("Processing smart features for note: {}", note.id);
                    match smart_service.process_note(&note).await {
                        Ok(_) => tracing::info!("Successfully processed note {}", note.id),
                        Err(e) => tracing::error!("Failed to process note {}: {}", note.id, e),
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to deserialize note from message: {}", e);
                }
            }
        }
    }

    #[cfg(not(feature = "smart-features"))]
    {
        tracing::info!("Smart features are disabled. Worker will exit.");
    }

    Ok(())
}
