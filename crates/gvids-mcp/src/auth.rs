//! Google OAuth2 authentication for Slides/Vids API.
//!
//! Credential search order:
//! 1. `GOOGLE_APPLICATION_CREDENTIALS` env var (service account JSON)
//! 2. `~/.config/gvids-mcp/service-account.json` (service account fallback)
//! 3. `~/.config/gcloud/application_default_credentials.json` (ADC — user or SA)
//!
//! Token lifecycle FSM: NoToken → Active(expires_at) → Expired → Refreshing → Active
//! Tier: T2-C (ς State + σ Sequence + ∂ Boundary + π Persistence)

use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::types::ServiceAccountKey;

/// Google OAuth2 token endpoint.
const TOKEN_URI: &str = "https://oauth2.googleapis.com/token";

/// Scopes for Google Slides + Drive (needed for Vids which may require drive access).
const SLIDES_SCOPES: &str =
    "https://www.googleapis.com/auth/presentations https://www.googleapis.com/auth/drive";

/// Safety margin before token expiry (refresh 5 minutes early).
const REFRESH_MARGIN_SECS: i64 = 300;

/// Errors that can occur during authentication.
#[derive(Debug, nexcore_error::Error)]
pub enum AuthError {
    #[error("no credentials found: checked GOOGLE_APPLICATION_CREDENTIALS, {0}, and gcloud ADC")]
    KeyNotFound(String),
    #[error("failed to read credentials file: {0}")]
    KeyRead(String),
    #[error("failed to parse credentials file: {0}")]
    KeyParse(String),
    #[error("JWT encoding failed: {0}")]
    JwtEncode(String),
    #[error("token exchange failed: {0}")]
    TokenExchange(String),
    #[error("token response missing access_token")]
    MissingAccessToken,
}

/// Detected credential type.
#[derive(Debug, Clone)]
enum CredentialSource {
    /// Service account: uses JWT → access token exchange.
    ServiceAccount(ServiceAccountKey),
    /// Authorized user: uses refresh_token → access token exchange.
    AuthorizedUser(UserCredentials),
}

/// User credentials from `gcloud auth application-default login`.
#[derive(Debug, Clone, Deserialize)]
struct UserCredentials {
    client_id: String,
    client_secret: String,
    refresh_token: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    credential_type: Option<String>,
}

/// JWT claims for Google OAuth2 service account flow.
#[derive(Debug, Serialize)]
struct JwtClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: i64,
    exp: i64,
}

/// Cached access token.
#[derive(Debug, Clone)]
struct TokenCache {
    access_token: String,
    expires_at: i64,
}

/// Token exchange response from Google.
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: Option<i64>,
    #[allow(dead_code)]
    token_type: Option<String>,
}

/// Thread-safe token manager with auto-refresh.
/// Supports both service account (JWT) and user credential (refresh token) flows.
#[derive(Clone)]
pub struct AuthManager {
    source: Arc<CredentialSource>,
    token: Arc<RwLock<Option<TokenCache>>>,
    http: reqwest::Client,
    /// Quota project for `x-goog-user-project` header (required for ADC).
    quota_project: Option<String>,
}

impl AuthManager {
    /// Create a new AuthManager by detecting and loading credentials.
    pub async fn new() -> Result<Self, AuthError> {
        let (source, quota_project) = load_credentials().await?;
        match &source {
            CredentialSource::ServiceAccount(key) => {
                info!(client_email = %key.client_email, "Using service account credentials");
            }
            CredentialSource::AuthorizedUser(_) => {
                info!(
                    quota_project = quota_project.as_deref().unwrap_or("none"),
                    "Using authorized_user (gcloud ADC) credentials"
                );
            }
        }
        Ok(Self {
            source: Arc::new(source),
            token: Arc::new(RwLock::new(None)),
            http: reqwest::Client::new(),
            quota_project,
        })
    }

    /// Returns the quota project ID, if available from credentials.
    pub fn quota_project(&self) -> Option<&str> {
        self.quota_project.as_deref()
    }

    /// Get a valid access token, refreshing if needed.
    pub async fn get_token(&self) -> Result<String, AuthError> {
        // Fast path: check if cached token is still valid.
        {
            let guard = self.token.read().await;
            if let Some(ref cached) = *guard {
                let now = Utc::now().timestamp();
                if now < cached.expires_at - REFRESH_MARGIN_SECS {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // Slow path: acquire write lock and refresh.
        let mut guard = self.token.write().await;

        // Double-check after acquiring write lock.
        if let Some(ref cached) = *guard {
            let now = Utc::now().timestamp();
            if now < cached.expires_at - REFRESH_MARGIN_SECS {
                return Ok(cached.access_token.clone());
            }
        }

        debug!("Refreshing Google Vids access token");
        let new_token = self.acquire_token().await?;
        let access_token = new_token.access_token.clone();
        let expires_in = new_token.expires_in.unwrap_or(3600);
        let expires_at = Utc::now().timestamp() + expires_in;

        *guard = Some(TokenCache {
            access_token: access_token.clone(),
            expires_at,
        });

        info!(expires_in_secs = expires_in, "Access token refreshed");
        Ok(access_token)
    }

    /// Acquire a token using the appropriate flow.
    async fn acquire_token(&self) -> Result<TokenResponse, AuthError> {
        match self.source.as_ref() {
            CredentialSource::ServiceAccount(key) => self.exchange_jwt(key).await,
            CredentialSource::AuthorizedUser(creds) => self.refresh_user_token(creds).await,
        }
    }

    /// Service Account flow: Build a signed JWT and exchange it for an access token.
    async fn exchange_jwt(&self, key: &ServiceAccountKey) -> Result<TokenResponse, AuthError> {
        let now = Utc::now().timestamp();
        let claims = JwtClaims {
            iss: key.client_email.clone(),
            scope: SLIDES_SCOPES.to_string(),
            aud: TOKEN_URI.to_string(),
            iat: now,
            exp: now + 3600,
        };

        let mut header = Header::new(Algorithm::RS256);
        if let Some(ref kid) = key.private_key_id {
            header.kid = Some(kid.clone());
        }

        let encoding_key = EncodingKey::from_rsa_pem(key.private_key.as_bytes())
            .map_err(|e| AuthError::JwtEncode(e.to_string()))?;

        let jwt = jsonwebtoken::encode(&header, &claims, &encoding_key)
            .map_err(|e| AuthError::JwtEncode(e.to_string()))?;

        let resp = self
            .http
            .post(TOKEN_URI)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| AuthError::TokenExchange(e.to_string()))?;

        self.parse_token_response(resp).await
    }

    /// User credential flow: Use refresh_token to get a new access token.
    async fn refresh_user_token(
        &self,
        creds: &UserCredentials,
    ) -> Result<TokenResponse, AuthError> {
        let resp = self
            .http
            .post(TOKEN_URI)
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", &creds.client_id),
                ("client_secret", &creds.client_secret),
                ("refresh_token", &creds.refresh_token),
            ])
            .send()
            .await
            .map_err(|e| AuthError::TokenExchange(e.to_string()))?;

        self.parse_token_response(resp).await
    }

    /// Parse a token response from Google's OAuth2 endpoint.
    async fn parse_token_response(
        &self,
        resp: reqwest::Response,
    ) -> Result<TokenResponse, AuthError> {
        let status = resp.status();
        if !status.is_success() {
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read body>".into());
            return Err(AuthError::TokenExchange(format!("HTTP {status}: {body}")));
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| AuthError::TokenExchange(e.to_string()))?;

        if token_resp.access_token.is_empty() {
            return Err(AuthError::MissingAccessToken);
        }

        Ok(token_resp)
    }
}

/// Load credentials from available sources.
/// Returns the credential source and optional quota project ID.
async fn load_credentials() -> Result<(CredentialSource, Option<String>), AuthError> {
    // 1. GOOGLE_APPLICATION_CREDENTIALS env var
    if let Ok(path) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        let path = PathBuf::from(&path);
        if path.exists() {
            return load_credential_file(&path).await;
        }
        warn!(
            path = %path.display(),
            "GOOGLE_APPLICATION_CREDENTIALS set but file not found, trying fallback"
        );
    }

    // 2. gvids-mcp service account
    let sa_path = gvids_sa_fallback();
    if sa_path.exists() {
        return load_credential_file(&sa_path).await;
    }

    // 3. gcloud ADC
    let adc_path = gcloud_adc_path();
    if adc_path.exists() {
        return load_credential_file(&adc_path).await;
    }

    Err(AuthError::KeyNotFound(sa_path.display().to_string()))
}

/// Load and detect credential type from a JSON file.
/// Returns the credential source and optional quota project ID.
async fn load_credential_file(
    path: &PathBuf,
) -> Result<(CredentialSource, Option<String>), AuthError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| AuthError::KeyRead(e.to_string()))?;

    let raw: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| AuthError::KeyParse(e.to_string()))?;

    let cred_type = raw
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Extract quota_project_id if present (common in ADC files).
    let quota_project = raw
        .get("quota_project_id")
        .and_then(|v| v.as_str())
        .map(String::from);

    match cred_type {
        "service_account" => {
            let key: ServiceAccountKey =
                serde_json::from_str(&content).map_err(|e| AuthError::KeyParse(e.to_string()))?;
            info!(path = %path.display(), "Loaded service account key");
            Ok((CredentialSource::ServiceAccount(key), quota_project))
        }
        "authorized_user" => {
            let creds: UserCredentials =
                serde_json::from_str(&content).map_err(|e| AuthError::KeyParse(e.to_string()))?;
            info!(path = %path.display(), quota_project = ?quota_project, "Loaded authorized_user credentials");
            Ok((CredentialSource::AuthorizedUser(creds), quota_project))
        }
        other => Err(AuthError::KeyParse(format!(
            "unsupported credential type: {other} (expected service_account or authorized_user)"
        ))),
    }
}

/// Default fallback path for service account key.
fn gvids_sa_fallback() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("gvids-mcp")
        .join("service-account.json")
}

/// gcloud Application Default Credentials path.
fn gcloud_adc_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("gcloud")
        .join("application_default_credentials.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jwt_claims_serialize() {
        let claims = JwtClaims {
            iss: "test@test.iam.gserviceaccount.com".into(),
            scope: SLIDES_SCOPES.into(),
            aud: TOKEN_URI.into(),
            iat: 1000,
            exp: 4600,
        };
        let json = serde_json::to_string(&claims).expect("serialize claims");
        assert!(json.contains("presentations"));
        assert!(json.contains("test@test.iam.gserviceaccount.com"));
    }

    #[test]
    fn gvids_sa_fallback_contains_gvids_mcp() {
        let path = gvids_sa_fallback();
        assert!(
            path.to_string_lossy().contains("gvids-mcp"),
            "fallback path should contain gvids-mcp"
        );
    }

    #[test]
    fn gcloud_adc_path_contains_gcloud() {
        let path = gcloud_adc_path();
        assert!(
            path.to_string_lossy().contains("gcloud"),
            "ADC path should contain gcloud"
        );
    }

    #[test]
    fn token_response_deserialize() {
        let json = r#"{"access_token":"ya29.test","expires_in":3600,"token_type":"Bearer"}"#;
        let resp: TokenResponse = serde_json::from_str(json).expect("deserialize token response");
        assert_eq!(resp.access_token, "ya29.test");
        assert_eq!(resp.expires_in, Some(3600));
    }

    #[test]
    fn slides_scope_contains_presentations() {
        assert!(SLIDES_SCOPES.contains("presentations"));
    }

    #[test]
    fn slides_scope_contains_drive() {
        assert!(SLIDES_SCOPES.contains("drive"));
    }
}
