use crate::{
    auth::{
        commands::{LoginCommand, RegisterCommand},
        login, register,
    },
    test_helpers::TestContext,
};

async fn registered_ctx() -> (TestContext, String, String) {
    let t = TestContext::new();
    let email = "user@example.com".to_string();
    let password = "password123".to_string();
    register::execute(
        &t.ctx,
        RegisterCommand {
            email: email.clone(),
            password: password.clone(),
        },
    )
    .await
    .unwrap();
    (t, email, password)
}

#[tokio::test]
async fn valid_credentials_return_user() {
    let (t, email, password) = registered_ctx().await;
    let user = login::execute(&t.ctx, LoginCommand { email, password })
        .await
        .unwrap();
    assert_eq!(user.email.as_ref(), "user@example.com");
}

#[tokio::test]
async fn wrong_password_is_rejected() {
    let (t, email, _) = registered_ctx().await;
    let result = login::execute(
        &t.ctx,
        LoginCommand {
            email,
            password: "wrongpass".into(),
        },
    )
    .await;
    assert!(matches!(
        result,
        Err(domain::errors::DomainError::Forbidden(_))
    ));
}

#[tokio::test]
async fn unknown_email_is_not_found() {
    let t = TestContext::new();
    let result = login::execute(
        &t.ctx,
        LoginCommand {
            email: "ghost@example.com".into(),
            password: "password123".into(),
        },
    )
    .await;
    assert!(matches!(
        result,
        Err(domain::errors::DomainError::NotFound(_))
    ));
}
