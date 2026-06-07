use sqlx::SqlitePool;

use domain::{
    note::{
        entity::{Note, NoteFilter},
        ports::NoteRepository,
        value_objects::NoteTitle,
    },
    user::{entity::User, ports::UserRepository, value_objects::Email},
};

use crate::{db::run_migrations, note::SqliteNoteRepository, user::SqliteUserRepository};

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
    let repo = SqliteNoteRepository::new(p);

    let note = Note::new(user.id, NoteTitle::new("Hello").ok(), "world".to_string());
    repo.save(&note).await.unwrap();

    let found = repo.find_by_id(&note.id).await.unwrap().unwrap();
    assert_eq!(found.content, "world");
    assert_eq!(found.title.as_ref().unwrap().as_ref(), "Hello");
}

#[tokio::test]
async fn save_note_without_title() {
    let p = pool().await;
    let user = seed_user(&p).await;
    let repo = SqliteNoteRepository::new(p);

    let note = Note::new(user.id, None, "no title".to_string());
    repo.save(&note).await.unwrap();

    let found = repo.find_by_id(&note.id).await.unwrap().unwrap();
    assert!(found.title.is_none());
}

#[tokio::test]
async fn find_by_user_with_pinned_filter() {
    let p = pool().await;
    let user = seed_user(&p).await;
    let repo = SqliteNoteRepository::new(p);

    let mut pinned = Note::new(user.id, None, "pinned".to_string());
    pinned.set_pinned(true);
    repo.save(&pinned).await.unwrap();
    repo.save(&Note::new(user.id, None, "normal".to_string()))
        .await
        .unwrap();

    let results = repo
        .find_by_user(&user.id, NoteFilter::default().pinned())
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content, "pinned");
}

#[tokio::test]
async fn delete_removes_note() {
    let p = pool().await;
    let user = seed_user(&p).await;
    let repo = SqliteNoteRepository::new(p);

    let note = Note::new(user.id, None, "bye".to_string());
    repo.save(&note).await.unwrap();
    repo.delete(&note.id).await.unwrap();

    assert!(repo.find_by_id(&note.id).await.unwrap().is_none());
}

#[tokio::test]
async fn save_and_find_versions() {
    let p = pool().await;
    let user = seed_user(&p).await;
    let repo = SqliteNoteRepository::new(p);

    let note = Note::new(user.id, None, "v1".to_string());
    repo.save(&note).await.unwrap();

    let version = domain::note::entity::NoteVersion::snapshot(&note);
    repo.save_version(&version).await.unwrap();

    let versions = repo.find_versions(&note.id).await.unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].content, "v1");
}

#[tokio::test]
async fn upsert_updates_note() {
    let p = pool().await;
    let user = seed_user(&p).await;
    let repo = SqliteNoteRepository::new(p);

    let mut note = Note::new(user.id, None, "original".to_string());
    repo.save(&note).await.unwrap();

    note.set_content("updated");
    repo.save(&note).await.unwrap();

    let found = repo.find_by_id(&note.id).await.unwrap().unwrap();
    assert_eq!(found.content, "updated");
}
