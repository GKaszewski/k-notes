//! Authentication logic
//!
//! Proxies to infra implementation if enabled.

#[cfg(feature = "auth-axum-login")]
use std::sync::Arc;

#[cfg(feature = "auth-axum-login")]
use notes_domain::UserRepository;
#[cfg(feature = "auth-axum-login")]
use notes_infra::session_store::{InfraSessionStore, SessionManagerLayer};

#[cfg(feature = "auth-axum-login")]
use crate::error::ApiError;

#[cfg(feature = "auth-axum-login")]
pub use notes_infra::auth::axum_login::{AuthManagerLayer, AuthSession, AuthUser, Credentials};

#[cfg(feature = "auth-axum-login")]
pub async fn setup_auth_layer(
    session_layer: SessionManagerLayer<InfraSessionStore>,
    user_repo: Arc<dyn UserRepository>,
) -> Result<AuthManagerLayer, ApiError> {
    notes_infra::auth::axum_login::setup_auth_layer(session_layer, user_repo)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))
}
