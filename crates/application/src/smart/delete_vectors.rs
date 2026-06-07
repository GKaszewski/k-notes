use domain::{errors::DomainResult, note::entity::NoteId};

use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, note_id: NoteId) -> DomainResult<()> {
    if let Some(vector_store) = ctx.services.vector_store.as_ref()
        && let Err(e) = vector_store.delete(&note_id).await
    {
        tracing::warn!("failed to delete vector for note {note_id}: {e}");
    }
    ctx.repos.link.delete_for_source(&note_id).await
}
