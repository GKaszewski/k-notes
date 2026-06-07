use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use api_types::tags::{CreateTagRequest, RenameTagRequest, TagResponse};
use application::tags::{
    commands::{CreateTagCommand, DeleteTagCommand, RenameTagCommand},
    create_tag, delete_tag, list_tags,
    queries::ListTagsQuery,
    rename_tag,
};

use crate::{
    error::{ApiError, ApiResult},
    extractors::CurrentUser,
    mapping::tag_response,
    state::PresentationState,
};

#[utoipa::path(
    get, path = "/api/v1/tags",
    responses(
        (status = 200, body = Vec<TagResponse>),
        (status = 401, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_tags(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
) -> ApiResult<Json<Vec<TagResponse>>> {
    let tags = list_tags::execute(
        &state.ctx,
        ListTagsQuery {
            user_id: user.id.as_uuid(),
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(Json(tags.into_iter().map(tag_response).collect()))
}

#[utoipa::path(
    post, path = "/api/v1/tags",
    request_body = CreateTagRequest,
    responses(
        (status = 201, body = TagResponse),
        (status = 401, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_tag(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Json(payload): Json<CreateTagRequest>,
) -> ApiResult<(StatusCode, Json<TagResponse>)> {
    let tag = create_tag::execute(
        &state.ctx,
        CreateTagCommand {
            user_id: user.id.as_uuid(),
            name: payload.name,
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(tag_response(tag))))
}

#[utoipa::path(
    delete, path = "/api/v1/tags/{id}",
    params(("id" = Uuid, Path, description = "Tag ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, body = api_types::errors::ErrorResponse),
        (status = 403, body = api_types::errors::ErrorResponse),
        (status = 404, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_tag(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    delete_tag::execute(
        &state.ctx,
        DeleteTagCommand {
            tag_id: id,
            user_id: user.id.as_uuid(),
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    patch, path = "/api/v1/tags/{id}",
    params(("id" = Uuid, Path, description = "Tag ID")),
    request_body = RenameTagRequest,
    responses(
        (status = 200, body = TagResponse),
        (status = 401, body = api_types::errors::ErrorResponse),
        (status = 403, body = api_types::errors::ErrorResponse),
        (status = 404, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn rename_tag(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<RenameTagRequest>,
) -> ApiResult<Json<TagResponse>> {
    let tag = rename_tag::execute(
        &state.ctx,
        RenameTagCommand {
            tag_id: id,
            user_id: user.id.as_uuid(),
            new_name: payload.name,
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(Json(tag_response(tag)))
}
