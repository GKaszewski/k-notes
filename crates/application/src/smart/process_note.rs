use domain::{
    errors::DomainResult,
    note::entity::{NoteId, NoteLink},
    user::entity::UserId,
};

use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, note_id: NoteId, _user_id: UserId) -> DomainResult<()> {
    let (Some(embedder), Some(vector_store)) = (
        ctx.services.embedding.as_ref(),
        ctx.services.vector_store.as_ref(),
    ) else {
        return Ok(());
    };

    let note = ctx.repos.note.find_by_id(&note_id).await?;
    let Some(note) = note else { return Ok(()) };

    let text = match &note.title {
        Some(t) => format!("{} {}", t.as_ref(), note.content),
        None => note.content.clone(),
    };

    let embedding = embedder.generate(&text).await?;
    vector_store.upsert(&note_id, &embedding).await?;

    let limit = ctx.config.smart.neighbour_limit;
    let similar = vector_store.find_similar(&embedding, limit + 1).await?;

    let links: Vec<NoteLink> = similar
        .into_iter()
        .filter(|(id, score)| *id != note_id && *score >= ctx.config.smart.min_similarity)
        .take(limit)
        .map(|(target_id, score)| NoteLink::new(note_id, target_id, score))
        .collect();

    ctx.repos.link.delete_for_source(&note_id).await?;
    if !links.is_empty() {
        ctx.repos.link.save_links(&links).await?;
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/process_note.rs"]
mod tests;
