//! Note route handlers

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_login::AuthSession;
use uuid::Uuid;
use validator::Validate;

use axum_login::AuthUser;
use notes_domain::{CreateNoteRequest as DomainCreateNote, UpdateNoteRequest as DomainUpdateNote};

use crate::auth::AuthBackend;
use crate::dto::{CreateNoteRequest, ListNotesQuery, NoteResponse, SearchQuery, UpdateNoteRequest};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

/// List notes with optional filtering
/// GET /api/v1/notes
pub async fn list_notes(
    State(state): State<AppState>,
    auth: AuthSession<AuthBackend>,
    Query(query): Query<ListNotesQuery>,
) -> ApiResult<Json<Vec<NoteResponse>>> {
    let user = auth
        .user
        .ok_or(ApiError::Domain(notes_domain::DomainError::Unauthorized(
            "Login required".to_string(),
        )))?;
    let user_id = user.id();

    // Build the filter, looking up tag_id by name if needed
    let mut filter = notes_domain::NoteFilter::new();
    filter.is_pinned = query.pinned;
    filter.is_archived = query.archived;

    // Look up tag by name if provided
    if let Some(ref tag_name) = query.tag {
        if let Ok(Some(tag)) = state.tag_repo.find_by_name(user_id, tag_name).await {
            filter.tag_id = Some(tag.id);
        } else {
            // Tag not found, return empty results
            return Ok(Json(vec![]));
        }
    }

    let notes = state.note_service.list_notes(user_id, filter).await?;
    let response: Vec<NoteResponse> = notes.into_iter().map(NoteResponse::from).collect();

    Ok(Json(response))
}

/// Create a new note
/// POST /api/v1/notes
pub async fn create_note(
    State(state): State<AppState>,
    auth: AuthSession<AuthBackend>,
    Json(payload): Json<CreateNoteRequest>,
) -> ApiResult<(StatusCode, Json<NoteResponse>)> {
    let user = auth
        .user
        .ok_or(ApiError::Domain(notes_domain::DomainError::Unauthorized(
            "Login required".to_string(),
        )))?;
    let user_id = user.id();

    // Validate input
    payload
        .validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let domain_req = DomainCreateNote {
        user_id,
        title: payload.title,
        content: payload.content,
        tags: payload.tags,
        color: payload.color,
        is_pinned: payload.is_pinned,
    };

    let note = state.note_service.create_note(domain_req).await?;

    Ok((StatusCode::CREATED, Json(NoteResponse::from(note))))
}

/// Get a single note by ID
/// GET /api/v1/notes/:id
pub async fn get_note(
    State(state): State<AppState>,
    auth: AuthSession<AuthBackend>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<NoteResponse>> {
    let user = auth
        .user
        .ok_or(ApiError::Domain(notes_domain::DomainError::Unauthorized(
            "Login required".to_string(),
        )))?;
    let user_id = user.id();

    let note = state.note_service.get_note(id, user_id).await?;

    Ok(Json(NoteResponse::from(note)))
}

/// Update a note
/// PATCH /api/v1/notes/:id
pub async fn update_note(
    State(state): State<AppState>,
    auth: AuthSession<AuthBackend>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNoteRequest>,
) -> ApiResult<Json<NoteResponse>> {
    let user = auth
        .user
        .ok_or(ApiError::Domain(notes_domain::DomainError::Unauthorized(
            "Login required".to_string(),
        )))?;
    let user_id = user.id();

    // Validate input
    payload
        .validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let domain_req = DomainUpdateNote {
        id,
        user_id,
        title: payload.title,
        content: payload.content,
        is_pinned: payload.is_pinned,
        is_archived: payload.is_archived,
        color: payload.color,
        tags: payload.tags,
    };

    let note = state.note_service.update_note(domain_req).await?;

    Ok(Json(NoteResponse::from(note)))
}

/// Delete a note
/// DELETE /api/v1/notes/:id
pub async fn delete_note(
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

    state.note_service.delete_note(id, user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Search notes
/// GET /api/v1/search
pub async fn search_notes(
    State(state): State<AppState>,
    auth: AuthSession<AuthBackend>,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Json<Vec<NoteResponse>>> {
    let user = auth
        .user
        .ok_or(ApiError::Domain(notes_domain::DomainError::Unauthorized(
            "Login required".to_string(),
        )))?;
    let user_id = user.id();

    let notes = state.note_service.search_notes(user_id, &query.q).await?;
    let response: Vec<NoteResponse> = notes.into_iter().map(NoteResponse::from).collect();

    Ok(Json(response))
}

/// List versions of a note
/// GET /api/v1/notes/:id/versions
pub async fn list_note_versions(
    State(state): State<AppState>,
    auth: AuthSession<AuthBackend>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<crate::dto::NoteVersionResponse>>> {
    let user = auth
        .user
        .ok_or(ApiError::Domain(notes_domain::DomainError::Unauthorized(
            "Login required".to_string(),
        )))?;
    let user_id = user.id();

    let versions = state.note_service.list_note_versions(id, user_id).await?;
    let response: Vec<crate::dto::NoteVersionResponse> = versions
        .into_iter()
        .map(crate::dto::NoteVersionResponse::from)
        .collect();

    Ok(Json(response))
}
