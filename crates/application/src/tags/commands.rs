use uuid::Uuid;

pub struct CreateTagCommand {
    pub user_id: Uuid,
    pub name: String,
}

pub struct DeleteTagCommand {
    pub tag_id: Uuid,
    pub user_id: Uuid,
}

pub struct RenameTagCommand {
    pub tag_id: Uuid,
    pub user_id: Uuid,
    pub new_name: String,
}
