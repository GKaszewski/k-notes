//! Configuration routes

use axum::{Json, extract::State};

use crate::dto::ConfigResponse;
use crate::error::ApiResult;
use crate::state::AppState;

/// Get system configuration
pub async fn get_config(State(state): State<AppState>) -> ApiResult<Json<ConfigResponse>> {
    Ok(Json(ConfigResponse {
        allow_registration: state.config.allow_registration,
        auth_mode: state.config.auth_mode,
        #[cfg(feature = "auth-oidc")]
        oidc_enabled: state.oidc_service.is_some(),
        #[cfg(not(feature = "auth-oidc"))]
        oidc_enabled: false,
        password_login_enabled: cfg!(feature = "auth-axum-login"),
    }))
}
