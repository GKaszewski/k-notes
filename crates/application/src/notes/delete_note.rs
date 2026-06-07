use domain::{
    errors::{DomainError, DomainResult},
    events::DomainEvent,
    note::entity::NoteId,
    user::entity::UserId,
};

use super::commands::DeleteNoteCommand;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, cmd: DeleteNoteCommand) -> DomainResult<()> {
    let note_id = NoteId::from_uuid(cmd.note_id);
    let user_id = UserId::from_uuid(cmd.user_id);

    let note = ctx
        .repos
        .note
        .find_by_id(&note_id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("note {}", cmd.note_id)))?;

    if note.user_id != user_id {
        return Err(DomainError::Forbidden(
            "cannot delete another user's note".into(),
        ));
    }

    ctx.repos.note.delete(&note_id).await?;

    if let Err(e) = ctx
        .services
        .event_publisher
        .publish(&DomainEvent::NoteDeleted { note_id, user_id })
        .await
    {
        tracing::warn!("failed to publish NoteDeleted: {e}");
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/delete_note.rs"]
mod tests;
