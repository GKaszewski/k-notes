use std::sync::Arc;

use crate::config::{AuthMode, Config};
use notes_domain::{NoteRepository, NoteService, TagRepository, TagService, UserService};

#[cfg(feature = "auth-jwt")]
use notes_infra::auth::jwt::{JwtConfig, JwtValidator};
#[cfg(feature = "auth-oidc")]
use notes_infra::auth::oidc::OidcService;

/// Application state holding all dependencies
#[derive(Clone)]
pub struct AppState {
    pub note_repo: Arc<dyn NoteRepository>,
    pub tag_repo: Arc<dyn TagRepository>,
    #[cfg(feature = "smart-features")]
    pub link_repo: Arc<dyn notes_domain::ports::LinkRepository>,
    pub note_service: Arc<NoteService>,
    pub tag_service: Arc<TagService>,
    pub user_service: Arc<UserService>,
    pub config: Config,
    #[cfg(feature = "auth-oidc")]
    pub oidc_service: Option<Arc<OidcService>>,
    #[cfg(feature = "auth-jwt")]
    pub jwt_validator: Option<Arc<JwtValidator>>,
}

impl AppState {
    pub async fn new(
        note_repo: Arc<dyn NoteRepository>,
        tag_repo: Arc<dyn TagRepository>,
        #[cfg(feature = "smart-features")] link_repo: Arc<dyn notes_domain::ports::LinkRepository>,
        note_service: Arc<NoteService>,
        tag_service: Arc<TagService>,
        user_service: Arc<UserService>,
        config: Config,
    ) -> anyhow::Result<Self> {
        #[cfg(feature = "auth-oidc")]
        let oidc_service = if let (Some(issuer), Some(id), secret, Some(redirect), resource_id) = (
            &config.oidc_issuer,
            &config.oidc_client_id,
            &config.oidc_client_secret,
            &config.oidc_redirect_url,
            &config.oidc_resource_id,
        ) {
            tracing::info!("Initializing OIDC service with issuer: {}", issuer);

            // Construct newtypes from config strings
            let issuer_url = notes_domain::IssuerUrl::new(issuer)
                .map_err(|e| anyhow::anyhow!("Invalid OIDC issuer URL: {}", e))?;
            let client_id = notes_domain::ClientId::new(id)
                .map_err(|e| anyhow::anyhow!("Invalid OIDC client ID: {}", e))?;
            let client_secret = secret.as_ref().map(|s| notes_domain::ClientSecret::new(s));
            let redirect_url = notes_domain::RedirectUrl::new(redirect)
                .map_err(|e| anyhow::anyhow!("Invalid OIDC redirect URL: {}", e))?;
            let resource = resource_id
                .as_ref()
                .map(|r| notes_domain::ResourceId::new(r))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid OIDC resource ID: {}", e))?;

            Some(Arc::new(
                OidcService::new(issuer_url, client_id, client_secret, redirect_url, resource)
                    .await?,
            ))
        } else {
            None
        };

        #[cfg(feature = "auth-jwt")]
        let jwt_validator = if matches!(config.auth_mode, AuthMode::Jwt | AuthMode::Both) {
            // Use provided secret or fall back to a development secret
            let secret = if let Some(ref s) = config.jwt_secret {
                if s.is_empty() { None } else { Some(s.clone()) }
            } else {
                None
            };

            let secret = match secret {
                Some(s) => s,
                None => {
                    if config.is_production {
                        anyhow::bail!(
                            "JWT_SECRET is required when AUTH_MODE is 'jwt' or 'both' in production"
                        );
                    }
                    // Use a development-only default secret
                    tracing::warn!(
                        "⚠️  JWT_SECRET not set - using insecure development secret. DO NOT USE IN PRODUCTION!"
                    );
                    "k-template-dev-secret-not-for-production-use-only".to_string()
                }
            };

            tracing::info!("Initializing JWT validator");
            let jwt_config = JwtConfig::new(
                secret,
                config.jwt_issuer.clone(),
                config.jwt_audience.clone(),
                Some(config.jwt_expiry_hours),
                config.is_production,
            )?;
            Some(Arc::new(JwtValidator::new(jwt_config)))
        } else {
            None
        };

        Ok(Self {
            note_repo,
            tag_repo,
            #[cfg(feature = "smart-features")]
            link_repo,
            note_service,
            tag_service,
            user_service,
            config,
            #[cfg(feature = "auth-oidc")]
            oidc_service,
            #[cfg(feature = "auth-jwt")]
            jwt_validator,
        })
    }
}
