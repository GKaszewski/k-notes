use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::error::ApiResult;
use crate::extractors::CurrentUser;
use crate::state::AppState;
use notes_domain::{Note, NoteFilter, Tag};

#[derive(Serialize, Deserialize)]
pub struct BackupData {
    pub notes: Vec<Note>,
    pub tags: Vec<Tag>,
}

/// Export user data
/// GET /api/v1/export
pub async fn export_data(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> ApiResult<Json<BackupData>> {
    let user_id = user.id;

    let notes = state
        .note_repo
        .find_by_user(user_id, NoteFilter::default())
        .await?;
    let tags = state.tag_repo.find_by_user(user_id).await?;

    Ok(Json(BackupData { notes, tags }))
}

/// Import user data
/// POST /api/v1/import
pub async fn import_data(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(payload): Json<BackupData>,
) -> ApiResult<StatusCode> {
    let user_id = user.id;

    // 1. Import standalone tags (to ensure even unused tags are restored)
    for tag in payload.tags {
        // Security check: ensure tag belongs to user
        if tag.user_id != user_id {
            // Skip tags from other users if malformed, or overwrite user_id?
            // Safer to skip or force user_id. Let's force user_id to current user to allow migrating data between accounts.
            let mut tag = tag;
            tag.user_id = user_id;
            state.tag_repo.save(&tag).await?;
        } else {
            state.tag_repo.save(&tag).await?;
        }
    }

    // 2. Import notes
    for mut note in payload.notes {
        // Security check: ensure note belongs to user
        note.user_id = user_id; // Force ownership to current user

        // Save note content
        state.note_repo.save(&note).await?;

        // 3. Re-establish tag associations
        // Note: note.tags contains the tags associated with this note
        for mut tag in note.tags {
            tag.user_id = user_id; // Force ownership

            // Ensure tag exists (upsert) - might be redundant if in payload.tags but safe
            state.tag_repo.save(&tag).await?;

            // Link tag to note
            state.tag_repo.add_to_note(tag.id, note.id).await?;
        }
    }

    Ok(StatusCode::OK)
}
