use crate::{
    notes::{
        commands::{CreateNoteCommand, DeleteNoteCommand},
        create_note, delete_note,
    },
    test_helpers::TestContext,
};
use uuid::Uuid;

#[tokio::test]
async fn owner_can_delete_note() {
    let t = TestContext::new();
    let user_id = Uuid::new_v4();

    let note = create_note::execute(
        &t.ctx,
        CreateNoteCommand {
            user_id,
            title: None,
            content: "bye".into(),
            color: None,
            is_pinned: false,
        },
    )
    .await
    .unwrap();

    delete_note::execute(
        &t.ctx,
        DeleteNoteCommand {
            note_id: note.id.as_uuid(),
            user_id,
        },
    )
    .await
    .unwrap();

    let found = t.ctx.repos.note.find_by_id(&note.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn other_user_cannot_delete_note() {
    let t = TestContext::new();
    let owner = Uuid::new_v4();

    let note = create_note::execute(
        &t.ctx,
        CreateNoteCommand {
            user_id: owner,
            title: None,
            content: "mine".into(),
            color: None,
            is_pinned: false,
        },
    )
    .await
    .unwrap();

    let result = delete_note::execute(
        &t.ctx,
        DeleteNoteCommand {
            note_id: note.id.as_uuid(),
            user_id: Uuid::new_v4(),
        },
    )
    .await;

    assert!(matches!(
        result,
        Err(domain::errors::DomainError::Forbidden(_))
    ));
}
