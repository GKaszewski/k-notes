use anyhow::{Result, anyhow};
use openidconnect::{
    AccessTokenHash, Client, EmptyAdditionalClaims, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    OAuth2TokenResponse, PkceCodeChallenge, Scope, StandardErrorResponse, TokenResponse,
    UserInfoClaims,
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreErrorResponseType,
        CoreGenderClaim, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreProviderMetadata,
        CoreRevocableToken, CoreRevocationErrorResponse, CoreTokenIntrospectionResponse,
        CoreTokenResponse,
    },
    reqwest,
};
use serde::{Deserialize, Serialize};

use crate::config::OidcConfig;

pub type OidcClient = Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    CoreTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

/// Data returned when starting the OIDC authorization flow.
#[derive(Debug, Clone)]
pub struct AuthorizationUrlData {
    pub url: url::Url,
    pub csrf_token: String,
    pub nonce: String,
    pub pkce_verifier: String,
}

/// Verified identity returned after a successful callback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcUser {
    pub subject: String,
    pub email: String,
}

#[derive(Clone)]
pub struct OidcService {
    client: OidcClient,
    resource_id: Option<String>,
}

impl OidcService {
    pub async fn new(config: OidcConfig) -> Result<Self> {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        let provider_metadata = CoreProviderMetadata::discover_async(
            openidconnect::IssuerUrl::new(config.issuer_url)?,
            &http_client,
        )
        .await?;

        let client_secret = config
            .client_secret
            .filter(|s| !s.trim().is_empty())
            .map(openidconnect::ClientSecret::new);

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            openidconnect::ClientId::new(config.client_id),
            client_secret,
        )
        .set_redirect_uri(openidconnect::RedirectUrl::new(config.redirect_url)?);

        Ok(Self {
            client,
            resource_id: config.resource_id,
        })
    }

    pub fn authorization_url(&self) -> AuthorizationUrlData {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (url, csrf_token, nonce) = self
            .client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                openidconnect::CsrfToken::new_random,
                openidconnect::Nonce::new_random,
            )
            .add_scope(Scope::new("profile".into()))
            .add_scope(Scope::new("email".into()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        AuthorizationUrlData {
            url: url.into(),
            csrf_token: csrf_token.secret().clone(),
            nonce: nonce.secret().clone(),
            pkce_verifier: pkce_verifier.secret().clone(),
        }
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        nonce: &str,
        pkce_verifier: &str,
    ) -> Result<OidcUser> {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        let token_response = self
            .client
            .exchange_code(openidconnect::AuthorizationCode::new(code.to_owned()))?
            .set_pkce_verifier(openidconnect::PkceCodeVerifier::new(
                pkce_verifier.to_owned(),
            ))
            .request_async(&http_client)
            .await?;

        let id_token = token_response
            .id_token()
            .ok_or_else(|| anyhow!("server did not return an ID token"))?;

        let mut verifier = self.client.id_token_verifier().clone();
        if let Some(ref rid) = self.resource_id {
            let rid = rid.clone();
            verifier =
                verifier.set_other_audience_verifier_fn(move |aud| aud.as_str() == rid.as_str());
        }

        let oidc_nonce = openidconnect::Nonce::new(nonce.to_owned());
        let claims = id_token.claims(&verifier, &oidc_nonce)?;

        if let Some(expected_hash) = claims.access_token_hash() {
            let actual_hash = AccessTokenHash::from_token(
                token_response.access_token(),
                id_token.signing_alg()?,
                id_token.signing_key(&verifier)?,
            )?;
            if actual_hash != *expected_hash {
                return Err(anyhow!("access token hash mismatch"));
            }
        }

        let email = match claims.email() {
            Some(e) => e.as_str().to_owned(),
            None => {
                tracing::debug!("email absent in ID token, fetching userinfo");
                let userinfo: UserInfoClaims<EmptyAdditionalClaims, CoreGenderClaim> = self
                    .client
                    .user_info(token_response.access_token().clone(), None)?
                    .request_async(&http_client)
                    .await?;
                userinfo
                    .email()
                    .map(|e| e.as_str().to_owned())
                    .ok_or_else(|| anyhow!("no verified email in identity provider response"))?
            }
        };

        Ok(OidcUser {
            subject: claims.subject().to_string(),
            email,
        })
    }
}
