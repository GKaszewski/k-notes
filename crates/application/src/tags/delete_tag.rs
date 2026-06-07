use domain::{
    errors::{DomainError, DomainResult},
    tag::entity::TagId,
    user::entity::UserId,
};

use super::commands::DeleteTagCommand;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, cmd: DeleteTagCommand) -> DomainResult<()> {
    let tag_id = TagId::from_uuid(cmd.tag_id);
    let user_id = UserId::from_uuid(cmd.user_id);

    let tag = ctx
        .repos
        .tag
        .find_by_id(&tag_id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("tag {}", cmd.tag_id)))?;

    if tag.user_id != user_id {
        return Err(DomainError::Forbidden(
            "cannot delete another user's tag".into(),
        ));
    }

    ctx.repos.tag.delete(&tag_id).await
}
