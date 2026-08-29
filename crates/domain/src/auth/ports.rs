use async_trait::async_trait;

use super::RefreshSession;
use crate::errors::DomainResult;
use crate::user::UserId;

#[async_trait]
pub trait RefreshSessionRepository: Send + Sync {
    async fn create(&self, session: &RefreshSession) -> DomainResult<()>;
    async fn find_by_token(&self, token: &str) -> DomainResult<Option<RefreshSession>>;
    async fn revoke(&self, token: &str) -> DomainResult<()>;
    async fn revoke_all_for_user(&self, user_id: &UserId) -> DomainResult<()>;
    async fn delete_expired(&self) -> DomainResult<u64>;
}
