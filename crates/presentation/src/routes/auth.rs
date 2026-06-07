use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};

use api_types::auth::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
use application::auth::{
    commands::{LoginCommand, RegisterCommand},
    login, register,
};

use crate::{
    error::{ApiError, ApiResult},
    extractors::CurrentUser,
    mapping::user_response,
    state::PresentationState,
};

pub fn router() -> Router<PresentationState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/register", post(register_handler))
        .route("/me", get(me_handler))
}

#[utoipa::path(
    post, path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, body = AuthResponse),
        (status = 403, body = api_types::errors::ErrorResponse, description = "Invalid credentials"),
    )
)]
pub async fn login_handler(
    State(state): State<PresentationState>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let user = login::execute(
        &state.ctx,
        LoginCommand {
            email: payload.email,
            password: payload.password,
        },
    )
    .await
    .map_err(ApiError::from)?;

    let token = state
        .jwt_validator
        .create_token(&user)
        .map_err(|e| ApiError::internal(format!("jwt error: {e}")))?;

    Ok(Json(AuthResponse {
        user: user_response(user),
        access_token: token,
    }))
}

#[utoipa::path(
    post, path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, body = AuthResponse),
        (status = 403, body = api_types::errors::ErrorResponse, description = "Registration disabled"),
        (status = 409, body = api_types::errors::ErrorResponse, description = "Email already exists"),
    )
)]
pub async fn register_handler(
    State(state): State<PresentationState>,
    Json(payload): Json<RegisterRequest>,
) -> ApiResult<(StatusCode, Json<AuthResponse>)> {
    if !state.ctx.config.allow_registration {
        return Err(ApiError::Forbidden("registration is disabled".into()));
    }

    let user = register::execute(
        &state.ctx,
        RegisterCommand {
            email: payload.email,
            password: payload.password,
        },
    )
    .await
    .map_err(ApiError::from)?;

    let token = state
        .jwt_validator
        .create_token(&user)
        .map_err(|e| ApiError::internal(format!("jwt error: {e}")))?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            user: user_response(user),
            access_token: token,
        }),
    ))
}

#[utoipa::path(
    get, path = "/api/v1/auth/me",
    responses(
        (status = 200, body = UserResponse),
        (status = 401, body = api_types::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn me_handler(CurrentUser(user): CurrentUser) -> Json<UserResponse> {
    Json(user_response(user))
}
