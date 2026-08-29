use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};

use api_types::auth::{
    AuthResponse, LoginRequest, LogoutRequest, RefreshRequest, RegisterRequest, UserResponse,
};
use application::auth::{
    commands::{LoginCommand, RegisterCommand},
    login, logout, refresh, register,
};
use domain::auth::RefreshSession;

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
        .route("/refresh", post(refresh_handler))
        .route("/logout", post(logout_handler))
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

    let session = RefreshSession::new(user.id, state.ctx.config.refresh_token_ttl_seconds);
    let refresh_token = session.token().to_string();
    state
        .ctx
        .repos
        .refresh_session
        .create(&session)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(AuthResponse {
        user: user_response(user),
        access_token: token,
        refresh_token,
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

    let session = RefreshSession::new(user.id, state.ctx.config.refresh_token_ttl_seconds);
    let refresh_token = session.token().to_string();
    state
        .ctx
        .repos
        .refresh_session
        .create(&session)
        .await
        .map_err(ApiError::from)?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            user: user_response(user),
            access_token: token,
            refresh_token,
        }),
    ))
}

#[utoipa::path(
    post, path = "/api/v1/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, body = AuthResponse),
        (status = 401, body = api_types::errors::ErrorResponse, description = "Invalid or expired refresh token"),
    )
)]
pub async fn refresh_handler(
    State(state): State<PresentationState>,
    Json(payload): Json<RefreshRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let result = refresh::execute(&state.ctx, &payload.refresh_token)
        .await
        .map_err(ApiError::from)?;

    let token = state
        .jwt_validator
        .create_token(&result.user)
        .map_err(|e| ApiError::internal(format!("jwt error: {e}")))?;

    Ok(Json(AuthResponse {
        user: user_response(result.user),
        access_token: token,
        refresh_token: result.refresh_token,
    }))
}

#[utoipa::path(
    post, path = "/api/v1/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 204, description = "Logged out"),
    )
)]
pub async fn logout_handler(
    State(state): State<PresentationState>,
    Json(payload): Json<LogoutRequest>,
) -> ApiResult<StatusCode> {
    logout::execute(&state.ctx, &payload.refresh_token)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
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
