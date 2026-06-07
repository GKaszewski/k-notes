use domain::{
    errors::{DomainError, DomainResult},
    note::entity::{NoteId, NoteLink},
    user::entity::UserId,
};

use super::queries::GetRelatedQuery;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, q: GetRelatedQuery) -> DomainResult<Vec<NoteLink>> {
    let note_id = NoteId::from_uuid(q.note_id);
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

    ctx.repos.link.find_for_note(&note_id).await
}
