use api_types::{
    notes::{
        AddTagRequest, ArchiveRequest, CreateNoteRequest, NoteLinkResponse, NoteResponse,
        NoteVersionResponse, PinRequest, UpdateNoteRequest,
    },
    tags::TagResponse,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::notes::list_notes,
        crate::routes::notes::create_note,
        crate::routes::notes::get_note,
        crate::routes::notes::update_note,
        crate::routes::notes::delete_note,
        crate::routes::notes::pin_note,
        crate::routes::notes::archive_note,
        crate::routes::notes::search_notes,
        crate::routes::notes::get_versions,
        crate::routes::notes::get_related,
        crate::routes::notes::add_tag,
        crate::routes::notes::remove_tag,
    ),
    components(schemas(
        NoteResponse,
        NoteVersionResponse,
        NoteLinkResponse,
        CreateNoteRequest,
        UpdateNoteRequest,
        PinRequest,
        ArchiveRequest,
        AddTagRequest,
        TagResponse,
    ))
)]
pub struct NotesDoc;
