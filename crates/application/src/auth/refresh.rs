use domain::{
    auth::RefreshSession,
    errors::{DomainError, DomainResult},
    user::entity::User,
};

use crate::context::AppContext;

pub struct RefreshResult {
    pub user: User,
    pub refresh_token: String,
}

pub async fn execute(ctx: &AppContext, old_refresh_token: &str) -> DomainResult<RefreshResult> {
    let session = ctx
        .repos
        .refresh_session
        .find_by_token(old_refresh_token)
        .await?
        .ok_or_else(|| DomainError::Unauthorized("invalid refresh token".into()))?;

    ctx.repos.refresh_session.revoke(old_refresh_token).await?;

    if session.is_expired() {
        return Err(DomainError::Unauthorized("refresh token expired".into()));
    }

    let user = ctx
        .repos
        .user
        .find_by_id(&session.user_id())
        .await?
        .ok_or_else(|| DomainError::NotFound("user no longer exists".into()))?;

    let new_session = RefreshSession::new(session.user_id(), ctx.config.refresh_token_ttl_seconds);
    let refresh_token = new_session.token().to_string();
    ctx.repos.refresh_session.create(&new_session).await?;

    Ok(RefreshResult {
        user,
        refresh_token,
    })
}
