use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use domain::user::entity::User;

use crate::config::JwtConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("token creation failed: {0}")]
    Creation(#[from] jsonwebtoken::errors::Error),
    #[error("token expired")]
    Expired,
    #[error("invalid token: {0}")]
    Invalid(String),
}

pub struct JwtValidator {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtValidator {
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        let mut validation = Validation::new(Algorithm::HS256);
        if let Some(ref iss) = config.issuer {
            validation.set_issuer(&[iss]);
        }
        if let Some(ref aud) = config.audience {
            validation.set_audience(&[aud]);
        }

        Self {
            config,
            encoding_key,
            decoding_key,
            validation,
        }
    }

    pub fn create_token(&self, user: &User) -> Result<String, JwtError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_secs() as usize;

        let claims = JwtClaims {
            sub: user.id.as_uuid().to_string(),
            email: user.email.as_ref().to_string(),
            exp: now + self.config.expiry_hours as usize * 3600,
            iat: now,
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
        };

        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key)
            .map_err(JwtError::Creation)
    }

    pub fn validate_token(&self, token: &str) -> Result<JwtClaims, JwtError> {
        decode::<JwtClaims>(token, &self.decoding_key, &self.validation)
            .map(|td| td.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::Expired,
                _ => JwtError::Invalid(e.to_string()),
            })
    }
}

impl std::fmt::Debug for JwtValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtValidator")
            .field("issuer", &self.config.issuer)
            .field("expiry_hours", &self.config.expiry_hours)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[path = "tests/jwt.rs"]
mod tests;
