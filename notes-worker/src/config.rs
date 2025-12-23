#[derive(Debug, Clone)]
pub struct Config {
    pub broker_url: String,
    pub database_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            broker_url: "nats://localhost:4222".to_string(),
            database_url: "sqlite::memory:".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        Self {
            broker_url: std::env::var("BROKER_URL").unwrap_or("nats://localhost:4222".to_string()),
            database_url: std::env::var("DATABASE_URL").unwrap_or("sqlite::memory:".to_string()),
        }
    }
}
