use domain::{
    errors::{DomainError, DomainResult},
    note::entity::{MAX_TAGS_PER_NOTE, NoteId},
    tag::entity::TagId,
    user::entity::UserId,
};

use super::commands::AddTagCommand;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, cmd: AddTagCommand) -> DomainResult<()> {
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

    if !note.can_add_tag() {
        return Err(DomainError::Conflict(format!(
            "note already has the maximum of {MAX_TAGS_PER_NOTE} tags"
        )));
    }

    let tag = ctx
        .repos
        .tag
        .find_by_id(&tag_id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("tag {}", cmd.tag_id)))?;

    if tag.user_id != user_id {
        return Err(DomainError::Forbidden("tag belongs to another user".into()));
    }

    ctx.repos.tag.add_to_note(&tag_id, &note_id).await
}

#[cfg(test)]
#[path = "tests/add_tag.rs"]
mod tests;
