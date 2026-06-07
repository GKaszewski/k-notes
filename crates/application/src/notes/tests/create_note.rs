use crate::{
    notes::{commands::CreateNoteCommand, create_note},
    test_helpers::TestContext,
};
use domain::events::DomainEvent;
use uuid::Uuid;

#[tokio::test]
async fn creates_note_and_publishes_event() {
    let t = TestContext::new();
    let user_id = Uuid::new_v4();

    let note = create_note::execute(
        &t.ctx,
        CreateNoteCommand {
            user_id,
            title: Some("Hello".into()),
            content: "world".into(),
            color: None,
            is_pinned: false,
        },
    )
    .await
    .unwrap();

    assert_eq!(note.content, "world");
    assert_eq!(note.title.as_ref().unwrap().as_ref(), "Hello");

    let events = t.publisher.events.lock().unwrap();
    assert!(matches!(events[0], DomainEvent::NoteCreated { .. }));
}

#[tokio::test]
async fn creates_note_without_title() {
    let t = TestContext::new();
    let note = create_note::execute(
        &t.ctx,
        CreateNoteCommand {
            user_id: Uuid::new_v4(),
            title: None,
            content: "untitled".into(),
            color: None,
            is_pinned: false,
        },
    )
    .await
    .unwrap();

    assert!(note.title.is_none());
}
