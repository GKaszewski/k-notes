use notes_infra::{DatabaseConfig, create_pool};

use crate::config::Config;

mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env();
    let nats_client = async_nats::connect(config.broker_url).await?;
    let db_config = DatabaseConfig::new(config.database_url);
    let db_pool = create_pool(&db_config).await?;

    // subscribe to jobs and process them
    Ok(())
}
