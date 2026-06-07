use serde::Serialize;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ConfigResponse {
    pub allow_registration: bool,
}
