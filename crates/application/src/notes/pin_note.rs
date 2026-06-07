use domain::{
    errors::{DomainError, DomainResult},
    note::entity::Note,
    user::entity::UserId,
};

use super::commands::PinNoteCommand;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, cmd: PinNoteCommand) -> DomainResult<Note> {
    let note_id = domain::note::entity::NoteId::from_uuid(cmd.note_id);
    let user_id = UserId::from_uuid(cmd.user_id);

    let mut note = ctx
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

    note.set_pinned(cmd.pinned);
    ctx.repos.note.save(&note).await?;
    Ok(note)
}
