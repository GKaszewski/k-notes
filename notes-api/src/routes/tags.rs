//! Tag route handlers

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_login::{AuthSession, AuthUser};
use uuid::Uuid;
use validator::Validate;

use notes_domain::TagService;

use crate::auth::AuthBackend;
use crate::dto::{CreateTagRequest, RenameTagRequest, TagResponse};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

/// List all tags for the user
/// GET /api/v1/tags
pub async fn list_tags(
    State(state): State<AppState>,
    auth: AuthSession<AuthBackend>,
) -> ApiResult<Json<Vec<TagResponse>>> {
    let user = auth
        .user
        .ok_or(ApiError::Domain(notes_domain::DomainError::Unauthorized(
            "Login required".to_string(),
        )))?;
    let user_id = user.id();

    let service = TagService::new(state.tag_repo);

    let tags = service.list_tags(user_id).await?;
    let response: Vec<TagResponse> = tags.into_iter().map(TagResponse::from).collect();

    Ok(Json(response))
}

/// Create a new tag
/// POST /api/v1/tags
pub async fn create_tag(
    State(state): State<AppState>,
    auth: AuthSession<AuthBackend>,
    Json(payload): Json<CreateTagRequest>,
) -> ApiResult<(StatusCode, Json<TagResponse>)> {
    let user = auth
        .user
        .ok_or(ApiError::Domain(notes_domain::DomainError::Unauthorized(
            "Login required".to_string(),
        )))?;
    let user_id = user.id();

    payload
        .validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let service = TagService::new(state.tag_repo);

    let tag = service.create_tag(user_id, &payload.name).await?;

    Ok((StatusCode::CREATED, Json(TagResponse::from(tag))))
}

/// Rename a tag
/// PATCH /api/v1/tags/:id
pub async fn rename_tag(
    State(state): State<AppState>,
    auth: AuthSession<AuthBackend>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RenameTagRequest>,
) -> ApiResult<Json<TagResponse>> {
    let user = auth
        .user
        .ok_or(ApiError::Domain(notes_domain::DomainError::Unauthorized(
            "Login required".to_string(),
        )))?;
    let user_id = user.id();

    payload
        .validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let service = TagService::new(state.tag_repo);

    let tag = service.rename_tag(id, user_id, &payload.name).await?;

    Ok(Json(TagResponse::from(tag)))
}

/// Delete a tag
/// DELETE /api/v1/tags/:id
pub async fn delete_tag(
    State(state): State<AppState>,
    auth: AuthSession<AuthBackend>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let user = auth
        .user
        .ok_or(ApiError::Domain(notes_domain::DomainError::Unauthorized(
            "Login required".to_string(),
        )))?;
    let user_id = user.id();

    let service = TagService::new(state.tag_repo);

    service.delete_tag(id, user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
