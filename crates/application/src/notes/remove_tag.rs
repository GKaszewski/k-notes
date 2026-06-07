use domain::{
    errors::{DomainError, DomainResult},
    note::entity::NoteId,
    tag::entity::TagId,
    user::entity::UserId,
};

use super::commands::RemoveTagCommand;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, cmd: RemoveTagCommand) -> DomainResult<()> {
    let note_id = NoteId::from_uuid(cmd.note_id);
    let tag_id = TagId::from_uuid(cmd.tag_id);
    let user_id = UserId::from_uuid(cmd.user_id);

    let note = ctx
        .repos
        .note
        .find_by_id(&note_id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("note {}", cmd.note_id)))?;

    if note.user_id != user_id {
        return Err(DomainError::Forbidden(
            "cannot modify another user's note".into(),
        ));
    }

    ctx.repos.tag.remove_from_note(&tag_id, &note_id).await
}
