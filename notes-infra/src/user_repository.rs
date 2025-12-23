//! SQLite implementation of UserRepository

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use notes_domain::{DomainError, DomainResult, User, UserRepository};

/// SQLite adapter for UserRepository
pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Row type for SQLite query results
#[derive(Debug, FromRow)]
struct UserRow {
    id: String,
    subject: String,
    email: String,
    password_hash: Option<String>,
    created_at: String,
}

impl TryFrom<UserRow> for User {
    type Error = DomainError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&row.id)
            .map_err(|e| DomainError::RepositoryError(format!("Invalid UUID: {}", e)))?;
        let created_at = DateTime::parse_from_rfc3339(&row.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                // Fallback for SQLite datetime format
                chrono::NaiveDateTime::parse_from_str(&row.created_at, "%Y-%m-%d %H:%M:%S")
                    .map(|dt| dt.and_utc())
            })
            .map_err(|e| DomainError::RepositoryError(format!("Invalid datetime: {}", e)))?;

        Ok(User::with_id(
            id,
            row.subject,
            row.email,
            row.password_hash,
            created_at,
        ))
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<User>> {
        let id_str = id.to_string();
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, subject, email, password_hash, created_at FROM users WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        row.map(User::try_from).transpose()
    }

    async fn find_by_subject(&self, subject: &str) -> DomainResult<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, subject, email, password_hash, created_at FROM users WHERE subject = ?",
        )
        .bind(subject)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        row.map(User::try_from).transpose()
    }

    async fn find_by_email(&self, email: &str) -> DomainResult<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, subject, email, password_hash, created_at FROM users WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        row.map(User::try_from).transpose()
    }

    async fn save(&self, user: &User) -> DomainResult<()> {
        let id = user.id.to_string();
        let created_at = user.created_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO users (id, subject, email, password_hash, created_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                subject = excluded.subject,
                email = excluded.email,
                password_hash = excluded.password_hash
            "#,
        )
        .bind(&id)
        .bind(&user.subject)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{DatabaseConfig, create_pool, run_migrations};

    async fn setup_test_db() -> SqlitePool {
        let config = DatabaseConfig::in_memory();
        let pool = create_pool(&config).await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_save_and_find_user() {
        let pool = setup_test_db().await;
        let repo = SqliteUserRepository::new(pool);

        let user = User::new("oidc|123", "test@example.com");
        repo.save(&user).await.unwrap();

        let found = repo.find_by_id(user.id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.subject, "oidc|123");
        assert_eq!(found.email, "test@example.com");
        assert!(found.password_hash.is_none());
    }

    #[tokio::test]
    async fn test_save_and_find_user_with_password() {
        let pool = setup_test_db().await;
        let repo = SqliteUserRepository::new(pool);

        let user = User::new_local("local@example.com", "hashed_pw");
        repo.save(&user).await.unwrap();

        let found = repo.find_by_id(user.id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.email, "local@example.com");
        assert_eq!(found.password_hash, Some("hashed_pw".to_string()));
    }

    #[tokio::test]
    async fn test_find_by_subject() {
        let pool = setup_test_db().await;
        let repo = SqliteUserRepository::new(pool);

        let user = User::new("google|456", "user@gmail.com");
        repo.save(&user).await.unwrap();

        let found = repo.find_by_subject("google|456").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user.id);
    }

    #[tokio::test]
    async fn test_delete_user() {
        let pool = setup_test_db().await;
        let repo = SqliteUserRepository::new(pool);

        let user = User::new("test|789", "delete@test.com");
        repo.save(&user).await.unwrap();
        repo.delete(user.id).await.unwrap();

        let found = repo.find_by_id(user.id).await.unwrap();
        assert!(found.is_none());
    }
}
