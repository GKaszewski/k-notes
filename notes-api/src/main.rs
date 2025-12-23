//! K-Notes API Server
//!
//! A high-performance, self-hosted note-taking API following hexagonal architecture.

use std::sync::Arc;
use time::Duration;

use axum::Router;
use axum_login::AuthManagerLayerBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use notes_infra::{
    DatabaseConfig, SqliteNoteRepository, SqliteTagRepository, SqliteUserRepository, create_pool,
    run_migrations,
};

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
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "notes_api=debug,tower_http=debug,axum_login=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env();

    // Setup database
    tracing::info!("Connecting to database: {}", config.database_url);
    let db_config = DatabaseConfig::new(&config.database_url);
    let pool = create_pool(&db_config).await?;

    // Run migrations
    tracing::info!("Running database migrations...");
    run_migrations(&pool).await?;

    // Create a default user for development (optional now that we have registration)
    create_dev_user(&pool).await?;

    // Create repositories
    let note_repo = Arc::new(SqliteNoteRepository::new(pool.clone()));
    let tag_repo = Arc::new(SqliteTagRepository::new(pool.clone()));
    let user_repo = Arc::new(SqliteUserRepository::new(pool.clone()));

    // Create application state
    let state = AppState::new(note_repo, tag_repo, user_repo.clone());

    // Auth backend
    let backend = AuthBackend::new(user_repo);

    // Session layer
    let session_store = SqliteStore::new(pool.clone());
    session_store.migrate().await?;

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_expiry(Expiry::OnInactivity(Duration::seconds(60 * 60 * 24 * 7))); // 7 days

    // Auth layer
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    // Parse CORS origins
    let mut cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::CONTENT_TYPE,
        ])
        .allow_credentials(true);

    // Add allowed origins
    let mut allowed_origins = Vec::new();
    for origin in &config.cors_allowed_origins {
        tracing::debug!("Allowing CORS origin: {}", origin);
        if let Ok(value) = origin.parse::<axum::http::HeaderValue>() {
            allowed_origins.push(value);
        } else {
            tracing::warn!("Invalid CORS origin: {}", origin);
        }
    }

    if !allowed_origins.is_empty() {
        cors = cors.allow_origin(allowed_origins);
    }

    // Build the application
    let app = Router::new()
        .nest("/api/v1", routes::api_v1_router())
        .layer(auth_layer)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start the server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚀 K-Notes API server running at http://{}", addr);
    tracing::info!("🔒 Authentication enabled (axum-login)");
    tracing::info!("📝 API endpoints available at /api/v1/...");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Create a development user for testing
/// In production, users will be created via OIDC authentication
async fn create_dev_user(pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
    use notes_domain::{User, UserRepository};
    use notes_infra::SqliteUserRepository;
    use password_auth::generate_hash;
    use uuid::Uuid;

    let user_repo = SqliteUserRepository::new(pool.clone());

    // Check if dev user exists
    let dev_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    if user_repo.find_by_id(dev_user_id).await?.is_none() {
        // Create dev user with fixed ID and password 'password'
        let hash = generate_hash("password");
        let user = User::with_id(
            dev_user_id,
            "dev|local",
            "dev@localhost",
            Some(hash),
            chrono::Utc::now(),
        );
        user_repo.save(&user).await?;
        tracing::info!("Created development user: dev@localhost / password");
    }

    Ok(())
}
