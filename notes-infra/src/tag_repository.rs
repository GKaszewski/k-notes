//! SQLite implementation of TagRepository

use async_trait::async_trait;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use notes_domain::{DomainError, DomainResult, Tag, TagName, TagRepository};

/// SQLite adapter for TagRepository
pub struct SqliteTagRepository {
    pool: SqlitePool,
}

impl SqliteTagRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct TagRow {
    id: String,
    name: String,
    user_id: String,
}

impl TryFrom<TagRow> for Tag {
    type Error = DomainError;

    fn try_from(row: TagRow) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&row.id)
            .map_err(|e| DomainError::RepositoryError(format!("Invalid UUID: {}", e)))?;
        let user_id = Uuid::parse_str(&row.user_id)
            .map_err(|e| DomainError::RepositoryError(format!("Invalid UUID: {}", e)))?;

        // Parse TagName from stored string - was validated when originally stored
        let name = TagName::try_from(row.name)
            .map_err(|e| DomainError::RepositoryError(format!("Invalid tag name in DB: {}", e)))?;

        Ok(Tag::with_id(id, name, user_id))
    }
}

#[async_trait]
impl TagRepository for SqliteTagRepository {
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Tag>> {
        let id_str = id.to_string();
        let row: Option<TagRow> = sqlx::query_as("SELECT id, name, user_id FROM tags WHERE id = ?")
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        row.map(Tag::try_from).transpose()
    }

    async fn find_by_user(&self, user_id: Uuid) -> DomainResult<Vec<Tag>> {
        let user_id_str = user_id.to_string();
        let rows: Vec<TagRow> =
            sqlx::query_as("SELECT id, name, user_id FROM tags WHERE user_id = ? ORDER BY name")
                .bind(&user_id_str)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        rows.into_iter().map(Tag::try_from).collect()
    }

    async fn find_by_name(&self, user_id: Uuid, name: &str) -> DomainResult<Option<Tag>> {
        let user_id_str = user_id.to_string();
        let row: Option<TagRow> =
            sqlx::query_as("SELECT id, name, user_id FROM tags WHERE user_id = ? AND name = ?")
                .bind(&user_id_str)
                .bind(name)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        row.map(Tag::try_from).transpose()
    }

    async fn save(&self, tag: &Tag) -> DomainResult<()> {
        let id = tag.id.to_string();
        let user_id = tag.user_id.to_string();

        sqlx::query(
            r#"
            INSERT INTO tags (id, name, user_id)
            VALUES (?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET name = excluded.name
            "#,
        )
        .bind(&id)
        .bind(tag.name.as_ref()) // Use .as_ref() to get the inner &str
        .bind(&user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    async fn add_to_note(&self, tag_id: Uuid, note_id: Uuid) -> DomainResult<()> {
        let tag_id_str = tag_id.to_string();
        let note_id_str = note_id.to_string();

        sqlx::query("INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?, ?)")
            .bind(&note_id_str)
            .bind(&tag_id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    async fn remove_from_note(&self, tag_id: Uuid, note_id: Uuid) -> DomainResult<()> {
        let tag_id_str = tag_id.to_string();
        let note_id_str = note_id.to_string();

        sqlx::query("DELETE FROM note_tags WHERE note_id = ? AND tag_id = ?")
            .bind(&note_id_str)
            .bind(&tag_id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_note(&self, note_id: Uuid) -> DomainResult<Vec<Tag>> {
        let note_id_str = note_id.to_string();
        let rows: Vec<TagRow> = sqlx::query_as(
            r#"
            SELECT t.id, t.name, t.user_id
            FROM tags t
            INNER JOIN note_tags nt ON t.id = nt.tag_id
            WHERE nt.note_id = ?
            ORDER BY t.name
            "#,
        )
        .bind(&note_id_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        rows.into_iter().map(Tag::try_from).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::run_migrations;
    use crate::user_repository::SqliteUserRepository;
    use k_core::db::DatabaseConfig;
    use notes_domain::{Email, User, UserRepository};

    async fn setup_test_db() -> SqlitePool {
        let config = DatabaseConfig::in_memory();
        let pool = k_core::db::connect(&config).await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool.sqlite_pool().unwrap().clone()
    }

    async fn create_test_user(pool: &SqlitePool) -> User {
        let user_repo = SqliteUserRepository::new(pool.clone());
        let email = Email::try_from("test@example.com").unwrap();
        let user = User::new("test|user", email);
        user_repo.save(&user).await.unwrap();
        user
    }

    #[tokio::test]
    async fn test_save_and_find_tag() {
        let pool = setup_test_db().await;
        let user = create_test_user(&pool).await;
        let repo = SqliteTagRepository::new(pool);

        let name = TagName::try_from("work").unwrap();
        let tag = Tag::new(name, user.id);
        repo.save(&tag).await.unwrap();

        let found = repo.find_by_id(tag.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name_str(), "work");
    }

    #[tokio::test]
    async fn test_find_by_name() {
        let pool = setup_test_db().await;
        let user = create_test_user(&pool).await;
        let repo = SqliteTagRepository::new(pool);

        let name = TagName::try_from("important").unwrap();
        let tag = Tag::new(name, user.id);
        repo.save(&tag).await.unwrap();

        let found = repo.find_by_name(user.id, "important").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, tag.id);
    }

    #[tokio::test]
    async fn test_find_by_user() {
        let pool = setup_test_db().await;
        let user = create_test_user(&pool).await;
        let repo = SqliteTagRepository::new(pool);

        let name_alpha = TagName::try_from("alpha").unwrap();
        let name_beta = TagName::try_from("beta").unwrap();
        repo.save(&Tag::new(name_alpha, user.id)).await.unwrap();
        repo.save(&Tag::new(name_beta, user.id)).await.unwrap();

        let tags = repo.find_by_user(user.id).await.unwrap();
        assert_eq!(tags.len(), 2);
        // Should be sorted alphabetically
        assert_eq!(tags[0].name_str(), "alpha");
        assert_eq!(tags[1].name_str(), "beta");
    }
}
