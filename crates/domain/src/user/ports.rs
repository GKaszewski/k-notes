use async_trait::async_trait;

use super::{
    entity::{User, UserId},
    value_objects::{Email, Password, PasswordHash},
};
use crate::errors::DomainResult;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &UserId) -> DomainResult<Option<User>>;
    async fn find_by_subject(&self, subject: &str) -> DomainResult<Option<User>>;
    async fn find_by_email(&self, email: &Email) -> DomainResult<Option<User>>;
    async fn save(&self, user: &User) -> DomainResult<()>;
    async fn delete(&self, id: &UserId) -> DomainResult<()>;
}

#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, password: &Password) -> DomainResult<PasswordHash>;
    async fn verify(&self, password: &Password, hash: &PasswordHash) -> DomainResult<bool>;
}
