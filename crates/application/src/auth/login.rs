use domain::{
    errors::{DomainError, DomainResult},
    user::{
        entity::User,
        value_objects::{Email, Password},
    },
};

use super::commands::LoginCommand;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, cmd: LoginCommand) -> DomainResult<User> {
    let email = Email::new(&cmd.email)?;
    let password = Password::new(cmd.password)?;

    let user = ctx
        .repos
        .user
        .find_by_email(&email)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("user {email}")))?;

    let hash = user
        .password_hash
        .as_ref()
        .ok_or_else(|| DomainError::Forbidden("account uses external authentication".into()))?;

    if !ctx.services.password_hasher.verify(&password, hash).await? {
        return Err(DomainError::Forbidden("invalid credentials".into()));
    }

    Ok(user)
}

#[cfg(test)]
#[path = "tests/login.rs"]
mod tests;
