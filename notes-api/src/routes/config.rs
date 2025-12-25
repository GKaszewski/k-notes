//! Configuration routes

use axum::{Json, extract::State};

use crate::dto::ConfigResponse;
use crate::error::ApiResult;
use crate::state::AppState;

/// Get system configuration
pub async fn get_config(State(state): State<AppState>) -> ApiResult<Json<ConfigResponse>> {
    Ok(Json(ConfigResponse {
        allow_registration: state.config.allow_registration,
    }))
}
