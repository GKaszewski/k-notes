use domain::{
    errors::DomainResult,
    tag::{entity::Tag, value_objects::TagName},
    user::entity::UserId,
};

use super::commands::CreateTagCommand;
use crate::context::AppContext;

/// Returns an existing tag with the same name if one exists, otherwise creates a new one.
pub async fn execute(ctx: &AppContext, cmd: CreateTagCommand) -> DomainResult<Tag> {
    let user_id = UserId::from_uuid(cmd.user_id);
    let name = TagName::new(cmd.name)?;

    if let Some(existing) = ctx.repos.tag.find_by_name(&user_id, &name).await? {
        return Ok(existing);
    }

    let tag = Tag::new(name, user_id);
    ctx.repos.tag.save(&tag).await?;
    Ok(tag)
}

#[cfg(test)]
#[path = "tests/create_tag.rs"]
mod tests;
