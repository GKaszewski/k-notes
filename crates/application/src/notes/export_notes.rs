use domain::{errors::DomainResult, note::entity::NoteFilter, user::entity::UserId};

use crate::context::AppContext;

pub struct ExportedNote {
    pub title: Option<String>,
    pub content: String,
    pub color: String,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub tags: Vec<String>,
}

pub async fn execute(ctx: &AppContext, user_id: uuid::Uuid) -> DomainResult<Vec<ExportedNote>> {
    let uid = UserId::from_uuid(user_id);
    let notes = ctx
        .repos
        .note
        .find_by_user(&uid, NoteFilter::default())
        .await?;

    Ok(notes
        .into_iter()
        .map(|n| ExportedNote {
            title: n.title.map(|t| t.into_inner()),
            content: n.content,
            color: n.color.into_inner(),
            is_pinned: n.is_pinned,
            is_archived: n.is_archived,
            tags: n.tags.into_iter().map(|t| t.name.into_inner()).collect(),
        })
        .collect())
}
