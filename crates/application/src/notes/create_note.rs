use domain::{
    errors::DomainResult,
    events::DomainEvent,
    note::{
        entity::Note,
        value_objects::{NoteColor, NoteTitle},
    },
    user::entity::UserId,
};

use super::commands::CreateNoteCommand;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, cmd: CreateNoteCommand) -> DomainResult<Note> {
    let user_id = UserId::from_uuid(cmd.user_id);
    let title = NoteTitle::from_optional(cmd.title)?;

    let mut note = Note::new(user_id, title, cmd.content);

    if let Some(color) = cmd.color {
        note.set_color(NoteColor::new(color));
    }
    if cmd.is_pinned {
        note.set_pinned(true);
    }

    ctx.repos.note.save(&note).await?;

    if let Err(e) = ctx
        .services
        .event_publisher
        .publish(&DomainEvent::NoteCreated {
            note_id: note.id,
            user_id,
        })
        .await
    {
        tracing::warn!("failed to publish NoteCreated: {e}");
    }

    Ok(note)
}

#[cfg(test)]
#[path = "tests/create_note.rs"]
mod tests;
