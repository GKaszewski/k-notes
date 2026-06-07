use domain::{
    errors::{DomainError, DomainResult},
    events::DomainEvent,
    note::{
        entity::{Note, NoteVersion},
        value_objects::{NoteColor, NoteTitle},
    },
    user::entity::UserId,
};

use super::commands::UpdateNoteCommand;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, cmd: UpdateNoteCommand) -> DomainResult<Note> {
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

    let version = NoteVersion::snapshot(&note);
    ctx.repos.note.save_version(&version).await?;

    if let Some(title) = cmd.title {
        note.set_title(NoteTitle::from_optional(Some(title))?);
    }
    if let Some(content) = cmd.content {
        note.set_content(content);
    }
    if let Some(color) = cmd.color {
        note.set_color(NoteColor::new(color));
    }

    ctx.repos.note.save(&note).await?;

    if let Err(e) = ctx
        .services
        .event_publisher
        .publish(&DomainEvent::NoteUpdated {
            note_id: note.id,
            user_id,
        })
        .await
    {
        tracing::warn!("failed to publish NoteUpdated: {e}");
    }

    Ok(note)
}

#[cfg(test)]
#[path = "tests/update_note.rs"]
mod tests;
