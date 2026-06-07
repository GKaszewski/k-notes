use crate::{
    tags::{commands::CreateTagCommand, create_tag},
    test_helpers::TestContext,
};
use uuid::Uuid;

#[tokio::test]
async fn creates_new_tag() {
    let t = TestContext::new();
    let tag = create_tag::execute(
        &t.ctx,
        CreateTagCommand {
            user_id: Uuid::new_v4(),
            name: "work".into(),
        },
    )
    .await
    .unwrap();

    assert_eq!(tag.name.as_ref(), "work");
}

#[tokio::test]
async fn returns_existing_tag_with_same_name() {
    let t = TestContext::new();
    let user_id = Uuid::new_v4();

    let first = create_tag::execute(
        &t.ctx,
        CreateTagCommand {
            user_id,
            name: "rust".into(),
        },
    )
    .await
    .unwrap();

    let second = create_tag::execute(
        &t.ctx,
        CreateTagCommand {
            user_id,
            name: "rust".into(),
        },
    )
    .await
    .unwrap();

    assert_eq!(first.id, second.id);
}

#[tokio::test]
async fn different_users_can_have_same_tag_name() {
    let t = TestContext::new();

    let a = create_tag::execute(
        &t.ctx,
        CreateTagCommand {
            user_id: Uuid::new_v4(),
            name: "shared".into(),
        },
    )
    .await
    .unwrap();

    let b = create_tag::execute(
        &t.ctx,
        CreateTagCommand {
            user_id: Uuid::new_v4(),
            name: "shared".into(),
        },
    )
    .await
    .unwrap();

    assert_ne!(a.id, b.id);
}
