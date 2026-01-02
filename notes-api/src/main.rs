//! K-Notes API Server
//!
//! A high-performance, self-hosted note-taking API following hexagonal architecture.

use k_core::{
    db::DatabasePool,
    http::server::{ServerConfig, apply_standard_middleware},
};
use std::{sync::Arc, time::Duration as StdDuration};
use time::Duration;

use axum::Router;
use axum_login::AuthManagerLayerBuilder;

use tower_sessions::{Expiry, SessionManagerLayer};

use notes_infra::run_migrations;

mod auth;
mod config;
mod dto;
mod error;
mod routes;
mod state;

use auth::AuthBackend;
use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    k_core::logging::init("notes_api");

    // Load configuration
    let config = Config::from_env();

    // Setup database
    tracing::info!("Connecting to database: {}", config.database_url);
    let db_config = k_core::db::DatabaseConfig {
        url: config.database_url.clone(),
        max_connections: 5,
        min_connections: 1,
        acquire_timeout: StdDuration::from_secs(30),
    };

    let db_pool = k_core::db::connect(&db_config).await?;

    run_migrations(&db_pool).await?;

    #[cfg(feature = "smart-features")]
    use notes_infra::factory::build_link_repository;
    use notes_infra::factory::{
        build_note_repository, build_session_store, build_tag_repository, build_user_repository,
    };

    // Create a default user for development
    create_dev_user(&db_pool).await.ok();

    // Create repositories via factory
    let note_repo = build_note_repository(&db_pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let tag_repo = build_tag_repository(&db_pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let user_repo = build_user_repository(&db_pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    #[cfg(feature = "smart-features")]
    let link_repo = build_link_repository(&db_pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Connect to message broker via factory
    #[cfg(feature = "smart-features")]
    let message_broker = {
        use notes_infra::factory::{BrokerProvider, build_message_broker};
        tracing::info!("Connecting to message broker: {}", config.broker_url);
        let provider = BrokerProvider::Nats {
            url: config.broker_url.clone(),
        };
        build_message_broker(&provider)
            .await
            .map_err(|e| anyhow::anyhow!("Broker connection failed: {}", e))?
    };

    // Create services
    use notes_domain::{NoteService, TagService, UserService};

    // Build NoteService with optional MessageBroker
    #[cfg(feature = "smart-features")]
    let note_service = match message_broker {
        Some(broker) => Arc::new(
            NoteService::new(note_repo.clone(), tag_repo.clone()).with_message_broker(broker),
        ),
        None => Arc::new(NoteService::new(note_repo.clone(), tag_repo.clone())),
    };
    #[cfg(not(feature = "smart-features"))]
    let note_service = Arc::new(NoteService::new(note_repo.clone(), tag_repo.clone()));

    let tag_service = Arc::new(TagService::new(tag_repo.clone()));
    let user_service = Arc::new(UserService::new(user_repo.clone()));

    // Create application state
    let state = AppState::new(
        note_repo,
        tag_repo,
        user_repo.clone(),
        #[cfg(feature = "smart-features")]
        link_repo,
        note_service,
        tag_service,
        user_service,
        config.clone(),
    );

    // Auth backend
    let backend = AuthBackend::new(user_repo); // no idea what now with this

    // Session layer
    // Use the factory to build the session store, agnostic of the underlying DB
    let session_store = build_session_store(&db_pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    session_store
        .migrate()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in prod
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));

    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let server_config = ServerConfig {
        cors_origins: config.cors_allowed_origins.clone(),
        session_secret: Some(config.session_secret.clone()),
    };

    let app = Router::new()
        .nest("/api/v1", routes::api_v1_router())
        .layer(auth_layer)
        .with_state(state);

    let app = apply_standard_middleware(app, &server_config);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚀 K-Notes API server running at http://{}", addr);
    tracing::info!("🔒 Authentication enabled (axum-login)");
    tracing::info!("📝 API endpoints available at /api/v1/...");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_dev_user(pool: &DatabasePool) -> anyhow::Result<()> {
    use notes_domain::{Email, User};
    use notes_infra::factory::build_user_repository;
    use password_auth::generate_hash;
    use uuid::Uuid;

    let user_repo = build_user_repository(pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Check if dev user exists
    let dev_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    if user_repo.find_by_id(dev_user_id).await?.is_none() {
        let hash = generate_hash("password");
        let dev_email = Email::try_from("dev@localhost.com")
            .map_err(|e| anyhow::anyhow!("Invalid dev email: {}", e))?;
        let user = User::with_id(
            dev_user_id,
            "dev|local",
            dev_email,
            Some(hash),
            chrono::Utc::now(),
        );
        user_repo.save(&user).await?;
        tracing::info!("Created development user: dev@localhost.com / password");
    }

    Ok(())
}
