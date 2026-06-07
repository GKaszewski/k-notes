/// Config for OIDC. Validated when constructing OidcService.
#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_url: String,
    /// Optional audience / resource ID for token validation.
    pub resource_id: Option<String>,
}

/// Config for JWT. Validated when constructing JwtValidator.
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: Option<String>,
    pub audience: Option<String>,
    pub expiry_hours: u64,
}

impl JwtConfig {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            issuer: None,
            audience: None,
            expiry_hours: 24,
        }
    }
}
