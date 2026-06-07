use std::sync::Arc;

use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
};
use uuid::Uuid;

use api_types::errors::ErrorResponse;
use domain::user::entity::UserId;

use crate::state::PresentationState;

/// Extracts the authenticated user from `Authorization: Bearer <jwt>`.
/// Returns `401 Unauthorized` if the header is absent, malformed, or the token is invalid.
pub struct CurrentUser(pub domain::user::entity::User);

impl FromRequestParts<PresentationState> for CurrentUser {
    type Rejection = (StatusCode, Json<ErrorResponse>);

    fn from_request_parts(
        parts: &mut Parts,
        state: &PresentationState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let validator = state.jwt_validator.clone();
        let user_repo = Arc::clone(&state.ctx.repos.user);
        let auth_header = parts.headers.get(header::AUTHORIZATION).cloned();

        async move {
            let header_val = auth_header.ok_or_else(unauthorized)?;
            let s = header_val.to_str().map_err(|_| unauthorized())?;
            let token = s.strip_prefix("Bearer ").ok_or_else(unauthorized)?;

            let claims = validator
                .validate_token(token.trim())
                .map_err(|_| unauthorized())?;
            let uuid = Uuid::parse_str(&claims.sub).map_err(|_| unauthorized())?;

            let user = user_repo
                .find_by_id(&UserId::from_uuid(uuid))
                .await
                .map_err(|_| unauthorized())?
                .ok_or_else(unauthorized)?;

            Ok(CurrentUser(user))
        }
    }
}

fn unauthorized() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse::new(
            "UNAUTHORIZED",
            "authentication required",
        )),
    )
}
