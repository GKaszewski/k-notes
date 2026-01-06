//! K-Notes API Server
//!
//! A high-performance, self-hosted note-taking API following hexagonal architecture.

use k_core::http::server::{ServerConfig, apply_standard_middleware};
use std::net::SocketAddr;
use std::{sync::Arc, time::Duration as StdDuration};
use time::Duration;
use tokio::net::TcpListener;
use tower_sessions::cookie::SameSite;
use tower_sessions::{Expiry, SessionManagerLayer};

use axum::Router;

use notes_infra::run_migrations;

mod auth;
mod config;
mod dto;
mod error;
mod extractors;
mod routes;
mod state;

use config::Config;
use state::AppState;

use crate::config::AuthMode;

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
        #[cfg(feature = "smart-features")]
        link_repo,
        note_service,
        tag_service,
        user_service,
        config.clone(),
    )
    .await?;

    // Build session store (needed for OIDC flow even in JWT mode)
    let session_store = build_session_store(&db_pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    session_store
        .migrate()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(config.secure_cookie)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));

    let server_config = ServerConfig {
        cors_origins: config.cors_allowed_origins.clone(),
        session_secret: Some(config.session_secret.clone()),
    };

    // Build the app with appropriate auth layers based on config
    let app = build_app(state, session_layer, user_repo, &config).await?;
    let app = apply_standard_middleware(app, &server_config);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("🚀 API server running at http://{}", addr);
    log_auth_info(&config);
    tracing::info!("📝 API endpoints available at /api/v1/...");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Build the application router with appropriate auth layers
#[allow(unused_variables)] // config/user_repo used conditionally based on features
async fn build_app(
    state: AppState,
    session_layer: SessionManagerLayer<notes_infra::session_store::InfraSessionStore>,
    user_repo: std::sync::Arc<dyn notes_domain::UserRepository>,
    config: &Config,
) -> anyhow::Result<Router> {
    let app = Router::new()
        .nest("/api/v1", routes::api_v1_router())
        .with_state(state);

    // When auth-axum-login feature is enabled, always apply the auth layer.
    // This is needed because:
    // 1. OIDC callback uses AuthSession for state management
    // 2. Session-based login/register routes use it
    // 3. The "JWT mode" just changes what the login endpoint returns, not the underlying session support
    #[cfg(feature = "auth-axum-login")]
    {
        let auth_layer = auth::setup_auth_layer(session_layer, user_repo).await?;
        return Ok(app.layer(auth_layer));
    }

    // When auth-axum-login is not compiled in, just use session layer for OIDC flow
    #[cfg(not(feature = "auth-axum-login"))]
    {
        let _ = user_repo; // Suppress unused warning
        Ok(app.layer(session_layer))
    }
}

/// Log authentication info based on enabled features and config
fn log_auth_info(config: &Config) {
    match config.auth_mode {
        AuthMode::Session => {
            tracing::info!("🔒 Authentication mode: Session (cookie-based)");
        }
        AuthMode::Jwt => {
            tracing::info!("🔒 Authentication mode: JWT (Bearer token)");
        }
        AuthMode::Both => {
            tracing::info!("🔒 Authentication mode: Both (JWT + Session)");
        }
    }

    #[cfg(feature = "auth-axum-login")]
    tracing::info!("  ✓ Session auth enabled (axum-login)");

    #[cfg(feature = "auth-jwt")]
    if config.jwt_secret.is_some() {
        tracing::info!("  ✓ JWT auth enabled");
    }

    #[cfg(feature = "auth-oidc")]
    if config.oidc_issuer.is_some() {
        tracing::info!("  ✓ OIDC integration enabled");
    }
}
