use api_types::auth::{
    AuthResponse, LoginRequest, LogoutRequest, RefreshRequest, RegisterRequest, UserResponse,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::auth::login_handler,
        crate::routes::auth::register_handler,
        crate::routes::auth::refresh_handler,
        crate::routes::auth::logout_handler,
        crate::routes::auth::me_handler,
    ),
    components(schemas(
        LoginRequest,
        RegisterRequest,
        RefreshRequest,
        LogoutRequest,
        AuthResponse,
        UserResponse
    ))
)]
pub struct AuthDoc;
