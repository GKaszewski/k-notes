use domain::{
    errors::DomainResult, note::entity::Note, tag::value_objects::TagName, user::entity::UserId,
};

use super::queries::ListNotesQuery;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, q: ListNotesQuery) -> DomainResult<Vec<Note>> {
    let user_id = UserId::from_uuid(q.user_id);
    let mut filter = q.filter;

    if let Some(name_str) = q.tag_name {
        let name = TagName::new(name_str)?;
        match ctx.repos.tag.find_by_name(&user_id, &name).await? {
            Some(tag) => filter.tag_id = Some(tag.id),
            // Tag doesn't exist for this user — no notes can match.
            None => return Ok(vec![]),
        }
    }

    ctx.repos.note.find_by_user(&user_id, filter).await
}
