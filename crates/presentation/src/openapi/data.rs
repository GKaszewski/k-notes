use api_types::{
    backup::{BackupData, BackupNote},
    config::ConfigResponse,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::data::get_config,
        crate::routes::data::export_data,
        crate::routes::data::import_data,
    ),
    components(schemas(ConfigResponse, BackupData, BackupNote))
)]
pub struct DataDoc;
