use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTagRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RenameTagRequest {
    pub name: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TagResponse {
    pub id: Uuid,
    pub name: String,
}
