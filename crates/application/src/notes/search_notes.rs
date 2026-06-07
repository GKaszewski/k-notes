use domain::{errors::DomainResult, note::entity::Note, user::entity::UserId};

use super::queries::SearchNotesQuery;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, q: SearchNotesQuery) -> DomainResult<Vec<Note>> {
    let user_id = UserId::from_uuid(q.user_id);
    ctx.repos.note.search(&user_id, &q.query).await
}
