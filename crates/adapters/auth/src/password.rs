use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};
use async_trait::async_trait;

use domain::{
    errors::{DomainError, DomainResult},
    user::{
        ports::PasswordHasher,
        value_objects::{Password, PasswordHash as DomainPasswordHash},
    },
};

pub struct Argon2PasswordHasher;

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(&self, password: &Password) -> DomainResult<DomainPasswordHash> {
        let password_str = password.as_ref().to_owned();
        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            let hash = Argon2::default()
                .hash_password(password_str.as_bytes(), &salt)
                .map_err(|e| DomainError::Infrastructure(format!("hash failed: {e}")))?;
            Ok(DomainPasswordHash::new(hash.to_string()))
        })
        .await
        .map_err(|e| DomainError::Infrastructure(format!("task panicked: {e}")))?
    }

    async fn verify(&self, password: &Password, hash: &DomainPasswordHash) -> DomainResult<bool> {
        let password_str = password.as_ref().to_owned();
        let hash_str = hash.as_str().to_owned();
        tokio::task::spawn_blocking(move || {
            let parsed = PasswordHash::new(&hash_str)
                .map_err(|e| DomainError::Infrastructure(format!("invalid hash: {e}")))?;
            Ok(Argon2::default()
                .verify_password(password_str.as_bytes(), &parsed)
                .is_ok())
        })
        .await
        .map_err(|e| DomainError::Infrastructure(format!("task panicked: {e}")))?
    }
}

#[cfg(test)]
#[path = "tests/password.rs"]
mod tests;
