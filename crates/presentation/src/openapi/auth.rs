use api_types::auth::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::auth::login_handler,
        crate::routes::auth::register_handler,
        crate::routes::auth::me_handler,
    ),
    components(schemas(LoginRequest, RegisterRequest, AuthResponse, UserResponse))
)]
pub struct AuthDoc;
