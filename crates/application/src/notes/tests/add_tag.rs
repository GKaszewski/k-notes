use crate::{
    notes::{
        add_tag,
        commands::{AddTagCommand, CreateNoteCommand},
        create_note,
    },
    tags::{commands::CreateTagCommand, create_tag},
    test_helpers::TestContext,
};
use domain::note::entity::MAX_TAGS_PER_NOTE;
use uuid::Uuid;

#[tokio::test]
async fn adds_tag_to_note() {
    let t = TestContext::new();
    let user_id = Uuid::new_v4();

    let note = create_note::execute(
        &t.ctx,
        CreateNoteCommand {
            user_id,
            title: None,
            content: "tagged".into(),
            color: None,
            is_pinned: false,
        },
    )
    .await
    .unwrap();

    let tag = create_tag::execute(
        &t.ctx,
        CreateTagCommand {
            user_id,
            name: "rust".into(),
        },
    )
    .await
    .unwrap();

    add_tag::execute(
        &t.ctx,
        AddTagCommand {
            note_id: note.id.as_uuid(),
            tag_id: tag.id.as_uuid(),
            user_id,
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn rejects_when_tag_limit_reached() {
    let t = TestContext::new();
    let user_id = Uuid::new_v4();

    let mut note = domain::note::entity::Note::new(
        domain::user::entity::UserId::from_uuid(user_id),
        None,
        "content",
    );
    // fill tags to the limit
    for i in 0..MAX_TAGS_PER_NOTE {
        let tag = domain::tag::entity::Tag::new(
            domain::tag::value_objects::TagName::new(format!("tag-{i}")).unwrap(),
            domain::user::entity::UserId::from_uuid(user_id),
        );
        t.ctx.repos.tag.save(&tag).await.unwrap();
        note.tags.push(tag);
    }
    t.ctx.repos.note.save(&note).await.unwrap();

    let extra_tag = create_tag::execute(
        &t.ctx,
        CreateTagCommand {
            user_id,
            name: "extra".into(),
        },
    )
    .await
    .unwrap();

    let result = add_tag::execute(
        &t.ctx,
        AddTagCommand {
            note_id: note.id.as_uuid(),
            tag_id: extra_tag.id.as_uuid(),
            user_id,
        },
    )
    .await;

    assert!(matches!(
        result,
        Err(domain::errors::DomainError::Conflict(_))
    ));
}
