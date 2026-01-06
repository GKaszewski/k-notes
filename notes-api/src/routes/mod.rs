//! Route definitions and module structure

pub mod auth;
pub mod config;
pub mod import_export;
pub mod notes;
pub mod tags;

use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::state::AppState;

/// Create the API v1 router
pub fn api_v1_router() -> Router<AppState> {
    let router = Router::new()
        // Auth routes
        .nest("/auth", auth::router())
        // Note routes
        .route("/notes", get(notes::list_notes).post(notes::create_note))
        .route(
            "/notes/{id}",
            get(notes::get_note)
                .patch(notes::update_note)
                .delete(notes::delete_note),
        )
        .route("/notes/{id}/versions", get(notes::list_note_versions));

    #[cfg(feature = "smart-features")]
    let router = router.route("/notes/{id}/related", get(notes::get_related_notes));

    router
        // Search route
        .route("/search", get(notes::search_notes))
        // Import/Export routes
        .route("/export", get(import_export::export_data))
        .route("/import", post(import_export::import_data))
        // Tag routes
        .route("/tags", get(tags::list_tags).post(tags::create_tag))
        .route(
            "/tags/{id}",
            delete(tags::delete_tag).patch(tags::rename_tag),
        )
        // System Config
        .route("/config", get(config::get_config))
}
