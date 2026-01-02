//! Database connection pool management

use k_core::db::DatabasePool;

/// Run database migrations
pub async fn run_migrations(pool: &DatabasePool) -> Result<(), sqlx::Error> {
    match pool {
        #[cfg(feature = "sqlite")]
        DatabasePool::Sqlite(pool) => {
            // Point specifically to the sqlite folder
            sqlx::migrate!("../migrations").run(pool).await?;
        }
        #[cfg(feature = "postgres")]
        DatabasePool::Postgres(pool) => {
            // Point specifically to the postgres folder
            sqlx::migrate!("../migrations_postgres").run(pool).await?;
        }
    }

    tracing::info!("Database migrations completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_in_memory_pool() {
        let config = k_core::db::DatabaseConfig::in_memory();
        let pool = k_core::db::connect(&config).await;
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_run_migrations() {
        let config = k_core::db::DatabaseConfig::in_memory();
        let pool = k_core::db::connect(&config).await.unwrap();
        let result = run_migrations(&pool).await;
        assert!(result.is_ok());
    }
}
