use std::sync::Arc;

use application::context::AppContext;
use auth::jwt::JwtValidator;

#[derive(Clone)]
pub struct PresentationState {
    pub ctx: AppContext,
    pub jwt_validator: Arc<JwtValidator>,
}

impl PresentationState {
    pub fn new(ctx: AppContext, jwt_validator: JwtValidator) -> Self {
        Self {
            ctx,
            jwt_validator: Arc::new(jwt_validator),
        }
    }
}
