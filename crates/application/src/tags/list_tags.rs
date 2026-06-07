use domain::{errors::DomainResult, tag::entity::Tag, user::entity::UserId};

use super::queries::ListTagsQuery;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, q: ListTagsQuery) -> DomainResult<Vec<Tag>> {
    let user_id = UserId::from_uuid(q.user_id);
    ctx.repos.tag.find_by_user(&user_id).await
}
