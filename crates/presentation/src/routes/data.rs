use axum::{Json, extract::State, http::StatusCode};

use api_types::{
    backup::{BackupData, BackupNote},
    config::ConfigResponse,
};
use application::notes::{export_notes, import_notes};

use crate::{
    error::{ApiError, ApiResult},
    extractors::CurrentUser,
    state::PresentationState,
};

#[utoipa::path(
    get, path = "/api/v1/config",
    responses((status = 200, body = ConfigResponse))
)]
pub async fn get_config(State(state): State<PresentationState>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        allow_registration: state.ctx.config.allow_registration,
    })
}

#[utoipa::path(
    get, path = "/api/v1/export",
    responses(
        (status = 200, body = BackupData),
        (status = 401, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn export_data(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
) -> ApiResult<Json<BackupData>> {
    let notes = export_notes::execute(&state.ctx, user.id.as_uuid())
        .await
        .map_err(ApiError::from)?;

    Ok(Json(BackupData {
        notes: notes
            .into_iter()
            .map(|n| BackupNote {
                title: n.title,
                content: n.content,
                color: n.color,
                is_pinned: n.is_pinned,
                is_archived: n.is_archived,
                tags: n.tags,
            })
            .collect(),
    }))
}

#[utoipa::path(
    post, path = "/api/v1/import",
    request_body = BackupData,
    responses(
        (status = 200, description = "Imported successfully"),
        (status = 401, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn import_data(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Json(payload): Json<BackupData>,
) -> ApiResult<StatusCode> {
    let notes = payload
        .notes
        .into_iter()
        .map(|n| import_notes::ImportNote {
            title: n.title,
            content: n.content,
            color: Some(n.color),
            is_pinned: n.is_pinned,
            is_archived: n.is_archived,
            tags: n.tags,
        })
        .collect();

    import_notes::execute(&state.ctx, user.id.as_uuid(), notes)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::OK)
}
