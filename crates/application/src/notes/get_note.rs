use domain::{
    errors::{DomainError, DomainResult},
    note::entity::Note,
    user::entity::UserId,
};

use super::queries::GetNoteQuery;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, q: GetNoteQuery) -> DomainResult<Note> {
    let note_id = domain::note::entity::NoteId::from_uuid(q.note_id);
    let user_id = UserId::from_uuid(q.user_id);

    let note = ctx
        .repos
        .note
        .find_by_id(&note_id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("note {}", q.note_id)))?;

    if note.user_id != user_id {
        return Err(DomainError::Forbidden(
            "note belongs to another user".into(),
        ));
    }

    Ok(note)
}
