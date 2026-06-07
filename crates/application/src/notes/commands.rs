use uuid::Uuid;

pub struct CreateNoteCommand {
    pub user_id: Uuid,
    pub title: Option<String>,
    pub content: String,
    pub color: Option<String>,
    pub is_pinned: bool,
}

pub struct UpdateNoteCommand {
    pub note_id: Uuid,
    pub user_id: Uuid,
    pub title: Option<String>,
    pub content: Option<String>,
    pub color: Option<String>,
}

pub struct DeleteNoteCommand {
    pub note_id: Uuid,
    pub user_id: Uuid,
}

pub struct PinNoteCommand {
    pub note_id: Uuid,
    pub user_id: Uuid,
    pub pinned: bool,
}

pub struct ArchiveNoteCommand {
    pub note_id: Uuid,
    pub user_id: Uuid,
    pub archived: bool,
}

pub struct AddTagCommand {
    pub note_id: Uuid,
    pub tag_id: Uuid,
    pub user_id: Uuid,
}

pub struct RemoveTagCommand {
    pub note_id: Uuid,
    pub tag_id: Uuid,
    pub user_id: Uuid,
}
