use sqlx::SqlitePool;

use domain::{
    tag::{entity::Tag, ports::TagRepository, value_objects::TagName},
    user::entity::{User, UserId},
};

use crate::{db::run_migrations, tag::SqliteTagRepository, user::SqliteUserRepository};
use domain::user::{ports::UserRepository, value_objects::Email};

async fn pool() -> SqlitePool {
    let p = SqlitePool::connect("sqlite::memory:").await.unwrap();
    run_migrations(&p).await.unwrap();
    p
}

async fn seed_user(pool: &SqlitePool) -> User {
    let repo = SqliteUserRepository::new(pool.clone());
    let user = User::new_oidc("sub", Email::new("u@example.com").unwrap());
    repo.save(&user).await.unwrap();
    user
}

#[tokio::test]
async fn save_and_find_by_id() {
    let p = pool().await;
    let user = seed_user(&p).await;
    let repo = SqliteTagRepository::new(p);

    let tag = Tag::new(TagName::new("work").unwrap(), user.id);
    repo.save(&tag).await.unwrap();

    let found = repo.find_by_id(&tag.id).await.unwrap().unwrap();
    assert_eq!(found.name.as_ref(), "work");
}

#[tokio::test]
async fn find_by_name() {
    let p = pool().await;
    let user = seed_user(&p).await;
    let repo = SqliteTagRepository::new(p);

    let tag = Tag::new(TagName::new("rust").unwrap(), user.id);
    repo.save(&tag).await.unwrap();

    let found = repo
        .find_by_name(&user.id, &TagName::new("rust").unwrap())
        .await
        .unwrap();
    assert_eq!(found.unwrap().id, tag.id);
}

#[tokio::test]
async fn find_by_user_returns_sorted() {
    let p = pool().await;
    let user = seed_user(&p).await;
    let repo = SqliteTagRepository::new(p);

    repo.save(&Tag::new(TagName::new("zebra").unwrap(), user.id))
        .await
        .unwrap();
    repo.save(&Tag::new(TagName::new("alpha").unwrap(), user.id))
        .await
        .unwrap();

    let tags = repo.find_by_user(&user.id).await.unwrap();
    assert_eq!(tags[0].name.as_ref(), "alpha");
    assert_eq!(tags[1].name.as_ref(), "zebra");
}

#[tokio::test]
async fn delete_removes_tag() {
    let p = pool().await;
    let user = seed_user(&p).await;
    let repo = SqliteTagRepository::new(p);

    let tag = Tag::new(TagName::new("gone").unwrap(), user.id);
    repo.save(&tag).await.unwrap();
    repo.delete(&tag.id).await.unwrap();

    assert!(repo.find_by_id(&tag.id).await.unwrap().is_none());
}
