pub mod error;
pub mod extractors;
pub mod mapping;
pub mod openapi;
pub mod routes;
pub mod state;

use std::path::PathBuf;

use axum::Router;
use axum::http::{HeaderValue, Method, header};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

pub use state::PresentationState;

/// Build the Axum router with all API routes and OpenAPI docs.
///
/// `spa_dir` — when `Some`, the built frontend is served at `/` as a fallback
/// so client-side routing works. API routes and docs take priority.
/// Set `SPA_DIR` env var in bootstrap to configure at runtime.
pub fn router(state: PresentationState, spa_dir: Option<PathBuf>) -> Router {
    let mut app = Router::new()
        .nest("/api/v1", routes::api_router())
        .with_state(state);

    // OpenAPI docs at /docs and /scalar
    app = openapi::serve(app);

    // Serve the SPA at root — must be last so API routes take priority.
    if let Some(dir) = spa_dir {
        let index = dir.join("index.html");
        tracing::info!("serving SPA from {}", dir.display());
        app = app.fallback_service(ServeDir::new(dir).fallback(ServeFile::new(index)));
    }

    app
}

/// Apply CORS and request tracing middleware.
pub fn apply_middleware(app: Router, cors_origins: Vec<String>) -> Router {
    let mut cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
        .allow_credentials(true);

    let origins: Vec<HeaderValue> = cors_origins.iter().filter_map(|o| o.parse().ok()).collect();

    if !origins.is_empty() {
        cors = cors.allow_origin(origins);
    }

    app.layer(TraceLayer::new_for_http()).layer(cors)
}
