use crate::{
    auth::{commands::RegisterCommand, register},
    test_helpers::TestContext,
};

#[tokio::test]
async fn registers_new_user() {
    let t = TestContext::new();
    let user = register::execute(
        &t.ctx,
        RegisterCommand {
            email: "user@example.com".into(),
            password: "password123".into(),
        },
    )
    .await
    .unwrap();

    assert_eq!(user.email.as_ref(), "user@example.com");
    assert!(user.password_hash.is_some());
}

#[tokio::test]
async fn rejects_duplicate_email() {
    let t = TestContext::new();
    let cmd = || RegisterCommand {
        email: "dup@example.com".into(),
        password: "password123".into(),
    };

    register::execute(&t.ctx, cmd()).await.unwrap();
    let result = register::execute(&t.ctx, cmd()).await;

    assert!(matches!(
        result,
        Err(domain::errors::DomainError::Conflict(_))
    ));
}

#[tokio::test]
async fn rejects_invalid_email() {
    let t = TestContext::new();
    let result = register::execute(
        &t.ctx,
        RegisterCommand {
            email: "not-an-email".into(),
            password: "password123".into(),
        },
    )
    .await;

    assert!(matches!(
        result,
        Err(domain::errors::DomainError::Validation(_))
    ));
}
