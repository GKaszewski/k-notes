use api_types::tags::{CreateTagRequest, RenameTagRequest, TagResponse};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::tags::list_tags,
        crate::routes::tags::create_tag,
        crate::routes::tags::delete_tag,
        crate::routes::tags::rename_tag,
    ),
    components(schemas(TagResponse, CreateTagRequest, RenameTagRequest))
)]
pub struct TagsDoc;
