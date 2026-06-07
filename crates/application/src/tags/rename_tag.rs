use domain::{
    errors::{DomainError, DomainResult},
    tag::{entity::Tag, entity::TagId, value_objects::TagName},
    user::entity::UserId,
};

use super::commands::RenameTagCommand;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, cmd: RenameTagCommand) -> DomainResult<Tag> {
    let tag_id = TagId::from_uuid(cmd.tag_id);
    let user_id = UserId::from_uuid(cmd.user_id);
    let new_name = TagName::new(cmd.new_name)?;

    let mut tag = ctx
        .repos
        .tag
        .find_by_id(&tag_id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("tag {}", cmd.tag_id)))?;

    if tag.user_id != user_id {
        return Err(DomainError::Forbidden(
            "cannot rename another user's tag".into(),
        ));
    }

    tag.name = new_name;
    ctx.repos.tag.save(&tag).await?;
    Ok(tag)
}
