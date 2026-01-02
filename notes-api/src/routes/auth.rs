//! Authentication routes

use axum::{Json, extract::State, http::StatusCode};
use axum_login::AuthSession;
use validator::Validate;

use notes_domain::{Email, User};
use password_auth::generate_hash;

use crate::auth::{AuthBackend, AuthUser, Credentials};
use crate::dto::{LoginRequest, RegisterRequest};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

/// Register a new user
pub async fn register(
    State(state): State<AppState>,
    mut auth_session: AuthSession<AuthBackend>,
    Json(payload): Json<RegisterRequest>,
) -> ApiResult<StatusCode> {
    payload
        .validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    // Check if registration is allowed
    if !state.config.allow_registration {
        return Err(ApiError::Forbidden("Registration is disabled".to_string()));
    }

    // Check if user exists
    if state
        .user_repo
        .find_by_email(&payload.email)
        .await
        .map_err(ApiError::from)?
        .is_some()
    {
        return Err(ApiError::Domain(
            notes_domain::DomainError::UserAlreadyExists(payload.email.clone()),
        ));
    }

    // Hash password
    let password_hash = generate_hash(&payload.password);

    // Parse email string to Email newtype
    let email = Email::try_from(payload.email)
        .map_err(|e| ApiError::validation(format!("Invalid email: {}", e)))?;

    // Create user - for local registration, we use email as subject
    let user = User::new_local(email, &password_hash);

    state.user_repo.save(&user).await.map_err(ApiError::from)?;

    // Auto login after registration
    let user = AuthUser(user);
    auth_session
        .login(&user)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(StatusCode::CREATED)
}

/// Login user
pub async fn login(
    mut auth_session: AuthSession<AuthBackend>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<StatusCode> {
    payload
        .validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let user = auth_session
        .authenticate(Credentials {
            email: payload.email,
            password: payload.password,
        })
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .ok_or_else(|| ApiError::validation("Invalid email or password"))?; // Generic error for security

    auth_session
        .login(&user)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(StatusCode::OK)
}

/// Logout user
pub async fn logout(mut auth_session: AuthSession<AuthBackend>) -> ApiResult<StatusCode> {
    auth_session
        .logout()
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(StatusCode::OK)
}

/// Get current user
pub async fn me(
    auth_session: AuthSession<AuthBackend>,
) -> ApiResult<Json<crate::dto::UserResponse>> {
    let user =
        auth_session
            .user
            .ok_or(ApiError::Domain(notes_domain::DomainError::Unauthorized(
                "Not logged in".to_string(),
            )))?;

    Ok(Json(crate::dto::UserResponse {
        id: user.0.id,
        email: user.0.email_str().to_string(), // Convert Email to String
        created_at: user.0.created_at,
    }))
}
