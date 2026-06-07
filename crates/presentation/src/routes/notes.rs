use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use api_types::notes::{
    AddTagRequest, ArchiveRequest, CreateNoteRequest, ListNotesParams, NoteLinkResponse,
    NoteResponse, NoteVersionResponse, PinRequest, SearchParams, UpdateNoteRequest,
};
use application::notes::{
    add_tag as uc_add_tag, archive_note as uc_archive_note,
    commands::{
        AddTagCommand, ArchiveNoteCommand, CreateNoteCommand, DeleteNoteCommand, PinNoteCommand,
        RemoveTagCommand, UpdateNoteCommand,
    },
    create_note as uc_create_note, delete_note as uc_delete_note, get_note as uc_get_note,
    get_related as uc_get_related, get_versions as uc_get_versions, list_notes as uc_list_notes,
    pin_note as uc_pin_note,
    queries::{GetNoteQuery, GetRelatedQuery, GetVersionsQuery, ListNotesQuery, SearchNotesQuery},
    remove_tag as uc_remove_tag, search_notes as uc_search_notes, update_note as uc_update_note,
};
use domain::note::entity::NoteFilter;

use crate::{
    error::{ApiError, ApiResult},
    extractors::CurrentUser,
    mapping::{note_link_response, note_response, note_version_response},
    state::PresentationState,
};

#[utoipa::path(
    get, path = "/api/v1/notes",
    params(ListNotesParams),
    responses(
        (status = 200, body = Vec<NoteResponse>),
        (status = 401, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_notes(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Query(params): Query<ListNotesParams>,
) -> ApiResult<Json<Vec<NoteResponse>>> {
    let user_id = user.id.as_uuid();

    let filter = NoteFilter {
        is_pinned: params.pinned,
        is_archived: params.archived,
        ..Default::default()
    };

    let notes = uc_list_notes::execute(
        &state.ctx,
        ListNotesQuery {
            user_id,
            filter,
            tag_name: params.tag,
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(Json(notes.into_iter().map(note_response).collect()))
}

#[utoipa::path(
    post, path = "/api/v1/notes",
    request_body = CreateNoteRequest,
    responses(
        (status = 201, body = NoteResponse),
        (status = 401, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_note(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Json(payload): Json<CreateNoteRequest>,
) -> ApiResult<(StatusCode, Json<NoteResponse>)> {
    let note = uc_create_note::execute(
        &state.ctx,
        CreateNoteCommand {
            user_id: user.id.as_uuid(),
            title: payload.title,
            content: payload.content,
            color: payload.color,
            is_pinned: payload.is_pinned,
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(note_response(note))))
}

#[utoipa::path(
    get, path = "/api/v1/notes/{id}",
    params(("id" = Uuid, Path, description = "Note ID")),
    responses(
        (status = 200, body = NoteResponse),
        (status = 401, body = api_types::errors::ErrorResponse),
        (status = 403, body = api_types::errors::ErrorResponse),
        (status = 404, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_note(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<NoteResponse>> {
    let note = uc_get_note::execute(
        &state.ctx,
        GetNoteQuery {
            note_id: id,
            user_id: user.id.as_uuid(),
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(Json(note_response(note)))
}

#[utoipa::path(
    patch, path = "/api/v1/notes/{id}",
    params(("id" = Uuid, Path, description = "Note ID")),
    request_body = UpdateNoteRequest,
    responses(
        (status = 200, body = NoteResponse),
        (status = 401, body = api_types::errors::ErrorResponse),
        (status = 403, body = api_types::errors::ErrorResponse),
        (status = 404, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_note(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNoteRequest>,
) -> ApiResult<Json<NoteResponse>> {
    let note = uc_update_note::execute(
        &state.ctx,
        UpdateNoteCommand {
            note_id: id,
            user_id: user.id.as_uuid(),
            title: payload.title,
            content: payload.content,
            color: payload.color,
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(Json(note_response(note)))
}

#[utoipa::path(
    delete, path = "/api/v1/notes/{id}",
    params(("id" = Uuid, Path, description = "Note ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, body = api_types::errors::ErrorResponse),
        (status = 403, body = api_types::errors::ErrorResponse),
        (status = 404, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_note(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    uc_delete_note::execute(
        &state.ctx,
        DeleteNoteCommand {
            note_id: id,
            user_id: user.id.as_uuid(),
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    patch, path = "/api/v1/notes/{id}/pin",
    params(("id" = Uuid, Path, description = "Note ID")),
    request_body = PinRequest,
    responses(
        (status = 200, body = NoteResponse),
        (status = 401, body = api_types::errors::ErrorResponse),
        (status = 403, body = api_types::errors::ErrorResponse),
        (status = 404, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn pin_note(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<PinRequest>,
) -> ApiResult<Json<NoteResponse>> {
    let note = uc_pin_note::execute(
        &state.ctx,
        PinNoteCommand {
            note_id: id,
            user_id: user.id.as_uuid(),
            pinned: payload.pinned,
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(Json(note_response(note)))
}

#[utoipa::path(
    patch, path = "/api/v1/notes/{id}/archive",
    params(("id" = Uuid, Path, description = "Note ID")),
    request_body = ArchiveRequest,
    responses(
        (status = 200, body = NoteResponse),
        (status = 401, body = api_types::errors::ErrorResponse),
        (status = 403, body = api_types::errors::ErrorResponse),
        (status = 404, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn archive_note(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<ArchiveRequest>,
) -> ApiResult<Json<NoteResponse>> {
    let note = uc_archive_note::execute(
        &state.ctx,
        ArchiveNoteCommand {
            note_id: id,
            user_id: user.id.as_uuid(),
            archived: payload.archived,
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(Json(note_response(note)))
}

#[utoipa::path(
    get, path = "/api/v1/search",
    params(SearchParams),
    responses(
        (status = 200, body = Vec<NoteResponse>),
        (status = 401, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn search_notes(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Query(params): Query<SearchParams>,
) -> ApiResult<Json<Vec<NoteResponse>>> {
    let notes = uc_search_notes::execute(
        &state.ctx,
        SearchNotesQuery {
            user_id: user.id.as_uuid(),
            query: params.q,
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(Json(notes.into_iter().map(note_response).collect()))
}

#[utoipa::path(
    get, path = "/api/v1/notes/{id}/versions",
    params(("id" = Uuid, Path, description = "Note ID")),
    responses(
        (status = 200, body = Vec<NoteVersionResponse>),
        (status = 401, body = api_types::errors::ErrorResponse),
        (status = 403, body = api_types::errors::ErrorResponse),
        (status = 404, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_versions(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<NoteVersionResponse>>> {
    let versions = uc_get_versions::execute(
        &state.ctx,
        GetVersionsQuery {
            note_id: id,
            user_id: user.id.as_uuid(),
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(Json(
        versions.into_iter().map(note_version_response).collect(),
    ))
}

#[utoipa::path(
    get, path = "/api/v1/notes/{id}/related",
    params(("id" = Uuid, Path, description = "Note ID")),
    responses(
        (status = 200, body = Vec<NoteLinkResponse>),
        (status = 401, body = api_types::errors::ErrorResponse),
        (status = 403, body = api_types::errors::ErrorResponse),
        (status = 404, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_related(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<NoteLinkResponse>>> {
    let links = uc_get_related::execute(
        &state.ctx,
        GetRelatedQuery {
            note_id: id,
            user_id: user.id.as_uuid(),
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(Json(links.into_iter().map(note_link_response).collect()))
}

#[utoipa::path(
    post, path = "/api/v1/notes/{id}/tags",
    params(("id" = Uuid, Path, description = "Note ID")),
    request_body = AddTagRequest,
    responses(
        (status = 204, description = "Tag added"),
        (status = 401, body = api_types::errors::ErrorResponse),
        (status = 403, body = api_types::errors::ErrorResponse),
        (status = 404, body = api_types::errors::ErrorResponse),
        (status = 409, body = api_types::errors::ErrorResponse, description = "Tag limit reached"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_tag(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Path(note_id): Path<Uuid>,
    Json(payload): Json<AddTagRequest>,
) -> ApiResult<StatusCode> {
    use application::tags::{commands::CreateTagCommand, create_tag};

    let tag = create_tag::execute(
        &state.ctx,
        CreateTagCommand {
            user_id: user.id.as_uuid(),
            name: payload.tag_name,
        },
    )
    .await
    .map_err(ApiError::from)?;

    uc_add_tag::execute(
        &state.ctx,
        AddTagCommand {
            note_id,
            tag_id: tag.id.as_uuid(),
            user_id: user.id.as_uuid(),
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete, path = "/api/v1/notes/{id}/tags/{tag_id}",
    params(
        ("id" = Uuid, Path, description = "Note ID"),
        ("tag_id" = Uuid, Path, description = "Tag ID"),
    ),
    responses(
        (status = 204, description = "Tag removed"),
        (status = 401, body = api_types::errors::ErrorResponse),
        (status = 403, body = api_types::errors::ErrorResponse),
        (status = 404, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn remove_tag(
    State(state): State<PresentationState>,
    CurrentUser(user): CurrentUser,
    Path((note_id, tag_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    uc_remove_tag::execute(
        &state.ctx,
        RemoveTagCommand {
            note_id,
            tag_id,
            user_id: user.id.as_uuid(),
        },
    )
    .await
    .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
