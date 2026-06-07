use chrono::{DateTime, Utc};
pub use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::str::FromStr;

use domain::errors::DomainError;

pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Parse a datetime string from SQLite (RFC3339 or naive format).
pub(crate) fn parse_dt(s: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map(|dt| dt.and_utc())
        })
        .map_err(|e| DomainError::Repository(format!("invalid datetime '{s}': {e}")))
}

/// Map a sqlx error to DomainError::Repository.
pub(crate) trait RepoExt<T> {
    fn repo(self) -> Result<T, DomainError>;
}

impl<T> RepoExt<T> for Result<T, sqlx::Error> {
    fn repo(self) -> Result<T, DomainError> {
        self.map_err(|e| DomainError::Repository(e.to_string()))
    }
}
