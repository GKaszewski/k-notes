use domain::{
    errors::DomainResult,
    user::{
        entity::User,
        value_objects::{Email, Password},
    },
};

use super::commands::RegisterCommand;
use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, cmd: RegisterCommand) -> DomainResult<User> {
    let email = Email::new(&cmd.email)?;
    let password = Password::new(cmd.password)?;

    if ctx.repos.user.find_by_email(&email).await?.is_some() {
        return Err(domain::errors::DomainError::Conflict(format!(
            "user with email {} already exists",
            email
        )));
    }

    let hash = ctx.services.password_hasher.hash(&password).await?;
    let user = User::new_local(email, hash);
    ctx.repos.user.save(&user).await?;
    Ok(user)
}

#[cfg(test)]
#[path = "tests/register.rs"]
mod tests;
