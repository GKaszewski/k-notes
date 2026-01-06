#[cfg(feature = "smart-features")]
use notes_infra::factory::{EmbeddingProvider, VectorProvider};
use serde::{Deserialize, Serialize};
use std::env;

/// Authentication mode - determines how the API authenticates requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    /// Session-based authentication using cookies (default for backward compatibility)
    #[default]
    Session,
    /// JWT-based authentication using Bearer tokens
    Jwt,
    /// Support both session and JWT authentication (try JWT first, then session)
    Both,
}

impl AuthMode {
    /// Parse auth mode from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "jwt" => AuthMode::Jwt,
            "both" => AuthMode::Both,
            _ => AuthMode::Session,
        }
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub session_secret: String,
    pub cors_allowed_origins: Vec<String>,
    pub allow_registration: bool,
    #[cfg(feature = "smart-features")]
    pub embedding_provider: EmbeddingProvider,
    #[cfg(feature = "smart-features")]
    pub vector_provider: VectorProvider,
    pub broker_url: String,

    pub secure_cookie: bool,

    pub db_max_connections: u32,

    pub db_min_connections: u32,

    // OIDC configuration
    pub oidc_issuer: Option<String>,
    pub oidc_client_id: Option<String>,
    pub oidc_client_secret: Option<String>,
    pub oidc_redirect_url: Option<String>,
    pub oidc_resource_id: Option<String>,

    // Auth mode configuration
    pub auth_mode: AuthMode,

    // JWT configuration
    pub jwt_secret: Option<String>,
    pub jwt_issuer: Option<String>,
    pub jwt_audience: Option<String>,
    pub jwt_expiry_hours: u64,

    /// Whether the application is running in production mode
    pub is_production: bool,

    /// Frontend URL for OIDC redirect (defaults to first CORS origin)
    pub frontend_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            database_url: "sqlite:data.db?mode=rwc".to_string(),
            session_secret: "k-notes-super-secret-key-must-be-at-least-64-bytes-long!!!!"
                .to_string(),
            cors_allowed_origins: vec!["http://localhost:5173".to_string()],
            allow_registration: true,
            #[cfg(feature = "smart-features")]
            embedding_provider: EmbeddingProvider::FastEmbed,
            #[cfg(feature = "smart-features")]
            vector_provider: VectorProvider::Qdrant {
                url: "http://localhost:6334".to_string(),
                collection: "notes".to_string(),
            },
            broker_url: "nats://localhost:4222".to_string(),
            secure_cookie: false,
            db_max_connections: 5,
            db_min_connections: 1,
            oidc_issuer: None,
            oidc_client_id: None,
            oidc_client_secret: None,
            oidc_redirect_url: None,
            oidc_resource_id: None,
            auth_mode: AuthMode::Session,
            jwt_secret: None,
            jwt_issuer: None,
            jwt_audience: None,
            jwt_expiry_hours: 24,
            is_production: false,
            frontend_url: "http://localhost:5173".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        // Load .env file if it exists, ignore errors if it doesn't
        let _ = dotenvy::dotenv();

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data.db?mode=rwc".to_string());

        let session_secret = env::var("SESSION_SECRET").unwrap_or_else(|_| {
            "k-notes-super-secret-key-must-be-at-least-64-bytes-long!!!!".to_string()
        });

        let cors_origins_str = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());

        let cors_allowed_origins = cors_origins_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let allow_registration = env::var("ALLOW_REGISTRATION")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(true);

        #[cfg(feature = "smart-features")]
        let embedding_provider = match env::var("EMBEDDING_PROVIDER").unwrap_or_default().as_str() {
            // Future: "ollama" => EmbeddingProvider::Ollama(...),
            _ => EmbeddingProvider::FastEmbed,
        };

        #[cfg(feature = "smart-features")]
        let vector_provider = match env::var("VECTOR_PROVIDER").unwrap_or_default().as_str() {
            // Future: "postgres" => ...
            _ => VectorProvider::Qdrant {
                url: env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string()),
                collection: env::var("QDRANT_COLLECTION").unwrap_or_else(|_| "notes".to_string()),
            },
        };

        let broker_url =
            env::var("BROKER_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

        let secure_cookie = env::var("SECURE_COOKIE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);

        let db_max_connections = env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        let db_min_connections = env::var("DB_MIN_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let oidc_issuer = env::var("OIDC_ISSUER").ok();
        let oidc_client_id = env::var("OIDC_CLIENT_ID").ok();
        let oidc_client_secret = env::var("OIDC_CLIENT_SECRET").ok();
        let oidc_redirect_url = env::var("OIDC_REDIRECT_URL").ok();
        let oidc_resource_id = env::var("OIDC_RESOURCE_ID").ok();

        // Auth mode configuration
        let auth_mode = env::var("AUTH_MODE")
            .map(|s| AuthMode::from_str(&s))
            .unwrap_or_default();

        // JWT configuration
        let jwt_secret = env::var("JWT_SECRET").ok();
        let jwt_issuer = env::var("JWT_ISSUER").ok();
        let jwt_audience = env::var("JWT_AUDIENCE").ok();
        let jwt_expiry_hours = env::var("JWT_EXPIRY_HOURS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);

        let is_production = env::var("PRODUCTION")
            .or_else(|_| env::var("RUST_ENV"))
            .map(|v| v.to_lowercase() == "production" || v == "1" || v == "true")
            .unwrap_or(false);

        Self {
            host,
            port,
            database_url,
            session_secret,
            cors_allowed_origins,
            allow_registration,
            #[cfg(feature = "smart-features")]
            embedding_provider,
            #[cfg(feature = "smart-features")]
            vector_provider,
            broker_url,
            secure_cookie,
            db_max_connections,
            db_min_connections,
            oidc_issuer,
            oidc_client_id,
            oidc_client_secret,
            oidc_redirect_url,
            oidc_resource_id,
            auth_mode,
            jwt_secret,
            jwt_issuer,
            jwt_audience,
            jwt_expiry_hours,
            is_production,
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
        }
    }
}
