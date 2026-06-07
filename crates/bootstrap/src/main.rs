use anyhow::Context as _;
use auth::{config::JwtConfig, jwt::JwtValidator};
use presentation::{PresentationState, apply_middleware, router};
use wiring::{WiringConfig, build_context};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing("bootstrap");

    let wiring_cfg = WiringConfig::from_env()?;
    let ctx = build_context(&wiring_cfg).await?;

    let jwt_validator = jwt_validator_from_env()?;
    let state = PresentationState::new(ctx, jwt_validator);

    let cors_origins = std::env::var("CORS_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // SPA_DIR — path to the built frontend dist directory.
    // Defaults to k-notes-frontend/dist relative to the working directory.
    // Set to "" to disable SPA serving (API-only mode).
    let spa_dir = std::env::var("SPA_DIR").unwrap_or_else(|_| "k-notes-frontend/dist".into());
    let spa_dir = if spa_dir.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(spa_dir))
    };

    let app = apply_middleware(router(state, spa_dir), cors_origins);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(3000);

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn jwt_validator_from_env() -> anyhow::Result<JwtValidator> {
    let secret = std::env::var("JWT_SECRET").context("JWT_SECRET must be set")?;
    let expiry_hours = std::env::var("JWT_EXPIRY_HOURS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(24);

    Ok(JwtValidator::new(JwtConfig {
        secret,
        issuer: std::env::var("JWT_ISSUER").ok(),
        audience: std::env::var("JWT_AUDIENCE").ok(),
        expiry_hours,
    }))
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
    tracing::info!("shutdown signal received");
}

fn init_tracing(service: &str) {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{service}=info,tower_http=info").into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
