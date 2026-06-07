use domain::errors::DomainResult;

use super::{
    add_tag, archive_note,
    commands::{AddTagCommand, ArchiveNoteCommand, CreateNoteCommand},
    create_note,
};
use crate::context::AppContext;
use crate::tags::{commands::CreateTagCommand, create_tag};

pub struct ImportNote {
    pub title: Option<String>,
    pub content: String,
    pub color: Option<String>,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub tags: Vec<String>,
}

pub async fn execute(
    ctx: &AppContext,
    user_id: uuid::Uuid,
    notes: Vec<ImportNote>,
) -> DomainResult<()> {
    for item in notes {
        let note = create_note::execute(
            ctx,
            CreateNoteCommand {
                user_id,
                title: item.title,
                content: item.content,
                color: item.color,
                is_pinned: item.is_pinned,
            },
        )
        .await?;

        if item.is_archived {
            archive_note::execute(
                ctx,
                ArchiveNoteCommand {
                    note_id: note.id.as_uuid(),
                    user_id,
                    archived: true,
                },
            )
            .await?;
        }

        for tag_name in item.tags {
            let tag = create_tag::execute(
                ctx,
                CreateTagCommand {
                    user_id,
                    name: tag_name,
                },
            )
            .await?;

            add_tag::execute(
                ctx,
                AddTagCommand {
                    note_id: note.id.as_uuid(),
                    tag_id: tag.id.as_uuid(),
                    user_id,
                },
            )
            .await?;
        }
    }

    Ok(())
}
