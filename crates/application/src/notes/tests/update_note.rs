use crate::{
    notes::{
        commands::{CreateNoteCommand, UpdateNoteCommand},
        create_note, update_note,
    },
    test_helpers::TestContext,
};
use uuid::Uuid;

#[tokio::test]
async fn owner_can_update_note() {
    let t = TestContext::new();
    let user_id = Uuid::new_v4();

    let note = create_note::execute(
        &t.ctx,
        CreateNoteCommand {
            user_id,
            title: None,
            content: "original".into(),
            color: None,
            is_pinned: false,
        },
    )
    .await
    .unwrap();

    let updated = update_note::execute(
        &t.ctx,
        UpdateNoteCommand {
            note_id: note.id.as_uuid(),
            user_id,
            title: None,
            content: Some("updated".into()),
            color: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(updated.content, "updated");
}

#[tokio::test]
async fn other_user_cannot_update_note() {
    let t = TestContext::new();
    let owner = Uuid::new_v4();
    let other = Uuid::new_v4();

    let note = create_note::execute(
        &t.ctx,
        CreateNoteCommand {
            user_id: owner,
            title: None,
            content: "secret".into(),
            color: None,
            is_pinned: false,
        },
    )
    .await
    .unwrap();

    let result = update_note::execute(
        &t.ctx,
        UpdateNoteCommand {
            note_id: note.id.as_uuid(),
            user_id: other,
            title: None,
            content: Some("hacked".into()),
            color: None,
        },
    )
    .await;

    assert!(matches!(
        result,
        Err(domain::errors::DomainError::Forbidden(_))
    ));
}

#[tokio::test]
async fn update_creates_version_snapshot() {
    let t = TestContext::new();
    let user_id = Uuid::new_v4();

    let note = create_note::execute(
        &t.ctx,
        CreateNoteCommand {
            user_id,
            title: None,
            content: "v1".into(),
            color: None,
            is_pinned: false,
        },
    )
    .await
    .unwrap();

    update_note::execute(
        &t.ctx,
        UpdateNoteCommand {
            note_id: note.id.as_uuid(),
            user_id,
            title: None,
            content: Some("v2".into()),
            color: None,
        },
    )
    .await
    .unwrap();

    let versions = t.ctx.repos.note.find_versions(&note.id).await.unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].content, "v1");
}
