use async_trait::async_trait;
use sqlx::{FromRow, SqlitePool};

use domain::{
    auth::{RefreshSession, RefreshSessionId, ports::RefreshSessionRepository},
    errors::DomainResult,
    user::UserId,
};

use crate::db::{RepoExt, parse_dt};

pub struct SqliteRefreshSessionRepository {
    pool: SqlitePool,
}

impl SqliteRefreshSessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct RefreshSessionRow {
    id: String,
    user_id: String,
    token: String,
    expires_at: String,
    created_at: String,
}

impl TryFrom<RefreshSessionRow> for RefreshSession {
    type Error = domain::errors::DomainError;

    fn try_from(row: RefreshSessionRow) -> Result<Self, Self::Error> {
        use domain::errors::DomainError;
        let id = RefreshSessionId::from_uuid(
            uuid::Uuid::parse_str(&row.id)
                .map_err(|e| DomainError::Repository(format!("invalid session uuid: {e}")))?,
        );
        let user_id = UserId::from_uuid(
            uuid::Uuid::parse_str(&row.user_id)
                .map_err(|e| DomainError::Repository(format!("invalid user uuid: {e}")))?,
        );
        let expires_at = parse_dt(&row.expires_at)?;
        let created_at = parse_dt(&row.created_at)?;
        Ok(RefreshSession::from_persistence(
            id, user_id, row.token, expires_at, created_at,
        ))
    }
}

#[async_trait]
impl RefreshSessionRepository for SqliteRefreshSessionRepository {
    async fn create(&self, session: &RefreshSession) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO refresh_sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(session.id().as_uuid().to_string())
        .bind(session.user_id().as_uuid().to_string())
        .bind(session.token())
        .bind(session.expires_at().to_rfc3339())
        .bind(session.created_at().to_rfc3339())
        .execute(&self.pool)
        .await
        .repo()
        .map(|_| ())
    }

    async fn find_by_token(&self, token: &str) -> DomainResult<Option<RefreshSession>> {
        sqlx::query_as::<_, RefreshSessionRow>(
            "SELECT id, user_id, token, expires_at, created_at FROM refresh_sessions WHERE token = ?",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .repo()?
        .map(RefreshSession::try_from)
        .transpose()
    }

    async fn revoke(&self, token: &str) -> DomainResult<()> {
        sqlx::query("DELETE FROM refresh_sessions WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await
            .repo()
            .map(|_| ())
    }

    async fn revoke_all_for_user(&self, user_id: &UserId) -> DomainResult<()> {
        sqlx::query("DELETE FROM refresh_sessions WHERE user_id = ?")
            .bind(user_id.as_uuid().to_string())
            .execute(&self.pool)
            .await
            .repo()
            .map(|_| ())
    }

    async fn delete_expired(&self) -> DomainResult<u64> {
        let result = sqlx::query("DELETE FROM refresh_sessions WHERE expires_at < ?")
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(&self.pool)
            .await
            .repo()?;
        Ok(result.rows_affected())
    }
}
