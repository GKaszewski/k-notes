/// Application-level configuration. Auth and infra adapter config lives in their own crates.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub base_url: String,
    pub smart: SmartConfig,
    /// When false the `/auth/register` endpoint returns 403.
    pub allow_registration: bool,
}

#[derive(Debug, Clone)]
pub struct SmartConfig {
    pub neighbour_limit: usize,
    pub min_similarity: f32,
}

impl Default for SmartConfig {
    fn default() -> Self {
        Self {
            neighbour_limit: 10,
            min_similarity: 0.7,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".into()),
            smart: SmartConfig::default(),
            allow_registration: true,
        }
    }
}
