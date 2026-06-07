use async_trait::async_trait;
use sqlx::{FromRow, SqlitePool};

use domain::{
    errors::DomainResult,
    user::{
        entity::{User, UserId},
        ports::UserRepository,
        value_objects::{Email, PasswordHash},
    },
};

use crate::db::{RepoExt, parse_dt};

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct UserRow {
    id: String,
    subject: String,
    email: String,
    password_hash: Option<String>,
    created_at: String,
}

impl TryFrom<UserRow> for User {
    type Error = domain::errors::DomainError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        use domain::errors::DomainError;
        let id = UserId::from_uuid(
            uuid::Uuid::parse_str(&row.id)
                .map_err(|e| DomainError::Repository(format!("invalid user uuid: {e}")))?,
        );
        let email = Email::new(&row.email)?;
        let password_hash = row.password_hash.map(PasswordHash::new);
        let created_at = parse_dt(&row.created_at)?;
        Ok(User::from_row(
            id,
            row.subject,
            email,
            password_hash,
            created_at,
        ))
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn find_by_id(&self, id: &UserId) -> DomainResult<Option<User>> {
        let id_str = id.as_uuid().to_string();
        sqlx::query_as::<_, UserRow>(
            "SELECT id, subject, email, password_hash, created_at FROM users WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .repo()?
        .map(User::try_from)
        .transpose()
    }

    async fn find_by_subject(&self, subject: &str) -> DomainResult<Option<User>> {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, subject, email, password_hash, created_at FROM users WHERE subject = ?",
        )
        .bind(subject)
        .fetch_optional(&self.pool)
        .await
        .repo()?
        .map(User::try_from)
        .transpose()
    }

    async fn find_by_email(&self, email: &Email) -> DomainResult<Option<User>> {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, subject, email, password_hash, created_at FROM users WHERE email = ?",
        )
        .bind(email.as_ref())
        .fetch_optional(&self.pool)
        .await
        .repo()?
        .map(User::try_from)
        .transpose()
    }

    async fn save(&self, user: &User) -> DomainResult<()> {
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
        .bind(user.id.as_uuid().to_string())
        .bind(&user.subject)
        .bind(user.email.as_ref())
        .bind(user.password_hash.as_ref().map(PasswordHash::as_str))
        .bind(user.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .repo()
        .map(|_| ())
    }

    async fn delete(&self, id: &UserId) -> DomainResult<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id.as_uuid().to_string())
            .execute(&self.pool)
            .await
            .repo()
            .map(|_| ())
    }
}

#[cfg(test)]
#[path = "tests/user.rs"]
mod tests;
