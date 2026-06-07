use sqlx::SqlitePool;

use domain::user::{
    entity::{User, UserId},
    ports::UserRepository,
    value_objects::{Email, PasswordHash},
};

use crate::{db::run_migrations, user::SqliteUserRepository};

async fn pool() -> SqlitePool {
    let p = SqlitePool::connect("sqlite::memory:").await.unwrap();
    run_migrations(&p).await.unwrap();
    p
}

#[tokio::test]
async fn save_and_find_by_id() {
    let repo = SqliteUserRepository::new(pool().await);
    let user = User::new_oidc("oidc|123", Email::new("a@example.com").unwrap());
    repo.save(&user).await.unwrap();

    let found = repo.find_by_id(&user.id).await.unwrap().unwrap();
    assert_eq!(found.subject, "oidc|123");
    assert_eq!(found.email.as_ref(), "a@example.com");
    assert!(found.password_hash.is_none());
}

#[tokio::test]
async fn save_local_user_with_password_hash() {
    let repo = SqliteUserRepository::new(pool().await);
    let user = User::new_local(
        Email::new("local@example.com").unwrap(),
        PasswordHash::new("argon2hash"),
    );
    repo.save(&user).await.unwrap();

    let found = repo.find_by_id(&user.id).await.unwrap().unwrap();
    assert_eq!(found.password_hash.unwrap().as_str(), "argon2hash");
}

#[tokio::test]
async fn find_by_subject() {
    let repo = SqliteUserRepository::new(pool().await);
    let user = User::new_oidc("google|456", Email::new("g@example.com").unwrap());
    repo.save(&user).await.unwrap();

    let found = repo.find_by_subject("google|456").await.unwrap().unwrap();
    assert_eq!(found.id, user.id);
}

#[tokio::test]
async fn find_by_email() {
    let repo = SqliteUserRepository::new(pool().await);
    let email = Email::new("find@example.com").unwrap();
    let user = User::new_oidc("sub", email.clone());
    repo.save(&user).await.unwrap();

    let found = repo.find_by_email(&email).await.unwrap().unwrap();
    assert_eq!(found.id, user.id);
}

#[tokio::test]
async fn delete_removes_user() {
    let repo = SqliteUserRepository::new(pool().await);
    let user = User::new_oidc("del|1", Email::new("del@example.com").unwrap());
    repo.save(&user).await.unwrap();
    repo.delete(&user.id).await.unwrap();

    assert!(repo.find_by_id(&user.id).await.unwrap().is_none());
}

#[tokio::test]
async fn upsert_updates_existing_user() {
    let repo = SqliteUserRepository::new(pool().await);
    let mut user = User::new_oidc("sub", Email::new("u@example.com").unwrap());
    repo.save(&user).await.unwrap();

    user.subject = "sub-updated".into();
    repo.save(&user).await.unwrap();

    let found = repo.find_by_id(&user.id).await.unwrap().unwrap();
    assert_eq!(found.subject, "sub-updated");
}
