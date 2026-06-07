pub mod auth;
pub mod data;
pub mod notes;
pub mod tags;

use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use crate::state::PresentationState;

pub fn api_router() -> Router<PresentationState> {
    Router::new()
        .nest("/auth", auth::router())
        // Config
        .route("/config", get(data::get_config))
        // Export / Import
        .route("/export", get(data::export_data))
        .route("/import", post(data::import_data))
        // Notes
        .route("/notes", get(notes::list_notes).post(notes::create_note))
        .route(
            "/notes/{id}",
            get(notes::get_note)
                .patch(notes::update_note)
                .delete(notes::delete_note),
        )
        .route("/notes/{id}/versions", get(notes::get_versions))
        .route("/notes/{id}/related", get(notes::get_related))
        .route("/notes/{id}/pin", patch(notes::pin_note))
        .route("/notes/{id}/archive", patch(notes::archive_note))
        .route("/notes/{id}/tags", post(notes::add_tag))
        .route("/notes/{id}/tags/{tag_id}", delete(notes::remove_tag))
        .route("/search", get(notes::search_notes))
        // Tags
        .route("/tags", get(tags::list_tags).post(tags::create_tag))
        .route(
            "/tags/{id}",
            delete(tags::delete_tag).patch(tags::rename_tag),
        )
}
