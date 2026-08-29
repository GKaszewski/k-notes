use domain::errors::DomainResult;

use crate::context::AppContext;

pub async fn execute(ctx: &AppContext, refresh_token: &str) -> DomainResult<()> {
    ctx.repos.refresh_session.revoke(refresh_token).await
}
