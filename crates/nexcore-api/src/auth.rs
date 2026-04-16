//! Authentication middleware for nexcore API
//!
//! Provides API key validation with constant-time comparison.
//! Public routes (`/health`, `/docs`, `/openapi.json`) bypass auth.
//! Protected routes under `/api/v1/` require a valid `X-API-Key` header.
//!
//! ## Configuration
//!
//! Set the `API_KEY` environment variable. If not set, all requests are allowed
//! (development mode with a warning logged on startup).
//!
//! Tier: T2-C (∂ Boundary + ς State)

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::RwLock;

/// Cached API key from environment (loaded once at first request)
static API_KEY: OnceLock<Option<String>> = OnceLock::new();

/// Get the configured API key, caching the result.
fn get_api_key() -> &'static Option<String> {
    API_KEY.get_or_init(|| {
        let key = std::env::var("API_KEY").ok();
        if key.is_none() {
            tracing::warn!(
                "API_KEY not set — running in OPEN MODE (all requests allowed). \
                 Set API_KEY environment variable for production."
            );
        }
        key
    })
}

/// Constant-time comparison to prevent timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// Extract API key from request headers
fn extract_key(headers: &HeaderMap) -> Option<String> {
    if let Some(val) = headers.get("x-api-key") {
        return val.to_str().ok().map(String::from);
    }
    if let Some(val) = headers.get("authorization") {
        if let Ok(s) = val.to_str() {
            if let Some(token) = s.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }
    None
}

/// Extract token from query string (for WebSocket — browser WS API can't set headers).
fn extract_token_from_query(uri: &axum::http::Uri) -> Option<String> {
    uri.query().and_then(|q| {
        q.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            if k == "token" {
                Some(v.to_string())
            } else {
                None
            }
        })
    })
}

/// Middleware that requires a valid API key for protected routes
pub async fn require_api_key(req: Request<Body>, next: Next) -> Response {
    let expected = get_api_key();
    let expected_key = match expected {
        Some(key) => key,
        None => return next.run(req).await,
    };

    // Try headers first, then query param (for WebSocket upgrade requests)
    let provided = extract_key(req.headers()).or_else(|| extract_token_from_query(req.uri()));
    let key = match provided {
        Some(k) => k,
        None => return unauthorized("Missing API key"),
    };

    if validate_key(&key, expected_key).await {
        next.run(req).await
    } else {
        unauthorized("Invalid API key or auth token")
    }
}

/// Validate a token (API key, Guardian key, or Firebase JWT).
/// Used by routes outside the API key middleware (e.g., WebSocket endpoints).
pub async fn validate_token(token: &str) -> bool {
    let expected = get_api_key();
    match expected {
        Some(key) => validate_key(token, key).await,
        None => true, // No API key configured — allow (dev mode)
    }
}

async fn validate_key(provided: &str, expected: &str) -> bool {
    // 1. Global API key — backward compat for internal/MCP tools
    if constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        return true;
    }

    // 2. Guardian per-user API key (grd_ prefix) — production auth
    if provided.starts_with("grd_") {
        return validate_guardian_key(provided).await;
    }

    // 3. JWT token (Firebase ID token) — portal auth
    if provided.starts_with("eyJ") {
        return verify_id_token(provided).await;
    }

    false
}

/// Validate a Guardian API key against the subscription store.
/// Checks: key exists, not revoked, subscription active, rate limit not exceeded.
async fn validate_guardian_key(key: &str) -> bool {
    let store = crate::subscription_store::get_store();
    let store = store.lock().await;

    match store.validate_api_key(key) {
        Ok(Some(user_id)) => {
            // Key valid — check rate limit
            match store.check_rate_limit(&user_id) {
                Ok(true) => true,
                Ok(false) => {
                    tracing::warn!(
                        user_id = user_id.as_str(),
                        "Guardian API rate limit exceeded"
                    );
                    false
                }
                Err(e) => {
                    tracing::error!("Rate limit check error: {e}");
                    false
                }
            }
        }
        Ok(None) => {
            tracing::debug!("Guardian API key not found or revoked");
            false
        }
        Err(e) => {
            tracing::error!("API key validation error: {e}");
            false
        }
    }
}

// =============================================================================
// Firebase JWT Verification (D007-P0-3 — RESOLVED)
// =============================================================================

/// Google's X.509 certificate endpoint for Firebase Auth tokens.
const FIREBASE_CERTS_URL: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

/// Cached JWKS keys: kid → RSA DecodingKey. Refreshed every 60 minutes.
static JWKS_CACHE: OnceLock<RwLock<JwksCache>> = OnceLock::new();

struct JwksCache {
    keys: HashMap<String, jsonwebtoken::DecodingKey>,
    fetched_at: std::time::Instant,
}

impl JwksCache {
    fn new() -> Self {
        Self {
            keys: HashMap::new(),
            fetched_at: std::time::Instant::now()
                .checked_sub(std::time::Duration::from_secs(7200))
                .unwrap_or_else(std::time::Instant::now),
        }
    }

    fn is_stale(&self) -> bool {
        self.fetched_at.elapsed() > std::time::Duration::from_secs(3600)
    }
}

fn get_jwks_cache() -> &'static RwLock<JwksCache> {
    JWKS_CACHE.get_or_init(|| RwLock::new(JwksCache::new()))
}

/// Fetch Google's X.509 certificates and convert to DecodingKeys.
async fn refresh_jwks() -> Result<HashMap<String, jsonwebtoken::DecodingKey>, String> {
    let response = reqwest::get(FIREBASE_CERTS_URL)
        .await
        .map_err(|e| format!("JWKS fetch failed: {e}"))?;

    let certs: HashMap<String, String> = response
        .json()
        .await
        .map_err(|e| format!("JWKS parse failed: {e}"))?;

    let mut keys = HashMap::new();
    for (kid, pem) in &certs {
        match jsonwebtoken::DecodingKey::from_rsa_pem(pem.as_bytes()) {
            Ok(key) => {
                keys.insert(kid.clone(), key);
            }
            Err(e) => {
                tracing::warn!(kid = kid.as_str(), error = %e, "Failed to parse JWKS cert");
            }
        }
    }

    if keys.is_empty() {
        return Err("No valid keys in JWKS response".to_string());
    }

    tracing::info!(key_count = keys.len(), "Refreshed Firebase JWKS keys");
    Ok(keys)
}

/// Get a DecodingKey by kid, refreshing the cache if stale.
async fn get_decoding_key(kid: &str) -> Option<jsonwebtoken::DecodingKey> {
    let cache = get_jwks_cache();

    // Fast path: read lock, key exists and cache is fresh
    {
        let read = cache.read().await;
        if !read.is_stale() {
            if let Some(key) = read.keys.get(kid) {
                return Some(key.clone());
            }
        }
    }

    // Slow path: refresh keys
    match refresh_jwks().await {
        Ok(new_keys) => {
            let result = new_keys.get(kid).cloned();
            let mut write = cache.write().await;
            write.keys = new_keys;
            write.fetched_at = std::time::Instant::now();
            result
        }
        Err(e) => {
            tracing::error!(error = e.as_str(), "Failed to refresh JWKS");
            // Fall back to cached keys even if stale
            let read = cache.read().await;
            read.keys.get(kid).cloned()
        }
    }
}

/// Firebase ID token claims we validate.
#[derive(serde::Deserialize)]
struct FirebaseClaims {
    /// Subject (user ID) — must be non-empty
    sub: String,
    /// Audience — must match our project ID
    aud: String,
    /// Issuer — must be https://securetoken.google.com/{project-id}
    iss: String,
}

/// Verify a Firebase ID token: signature (RS256), issuer, audience, expiry.
///
/// Replaces the D007-P0-3 stub. Now validates:
/// 1. JWT header has a `kid` matching a Google public key
/// 2. RS256 signature is valid
/// 3. Token is not expired (jsonwebtoken checks `exp` automatically)
/// 4. Issuer matches `https://securetoken.google.com/{project-id}`
/// 5. Audience matches `{project-id}`
/// 6. Subject (uid) is non-empty
async fn verify_id_token(token: &str) -> bool {
    // Decode header to get kid (without verification)
    let header = match jsonwebtoken::decode_header(token) {
        Ok(h) => h,
        Err(e) => {
            tracing::debug!(error = %e, "Firebase JWT: invalid header");
            return false;
        }
    };

    let kid = match &header.kid {
        Some(k) => k.clone(),
        None => {
            tracing::debug!("Firebase JWT: missing kid in header");
            return false;
        }
    };

    // Algorithm must be RS256
    if header.alg != jsonwebtoken::Algorithm::RS256 {
        tracing::debug!(alg = ?header.alg, "Firebase JWT: unexpected algorithm");
        return false;
    }

    // Get the matching public key
    let decoding_key = match get_decoding_key(&kid).await {
        Some(k) => k,
        None => {
            tracing::debug!(kid = kid.as_str(), "Firebase JWT: no matching key for kid");
            return false;
        }
    };

    // Determine the expected project ID
    let project_id = std::env::var("FIREBASE_PROJECT_ID")
        .unwrap_or_else(|_| "nexvigilant-digital-clubhouse".to_string());

    // Validate signature + standard claims (exp, iat, nbf)
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&[&project_id]);
    validation.set_issuer(&[format!("https://securetoken.google.com/{project_id}")]);

    match jsonwebtoken::decode::<FirebaseClaims>(token, &decoding_key, &validation) {
        Ok(data) => {
            // Additional check: sub must be non-empty (Firebase requirement)
            if data.claims.sub.is_empty() {
                tracing::debug!("Firebase JWT: empty sub claim");
                return false;
            }
            tracing::debug!(uid = data.claims.sub.as_str(), "Firebase JWT: verified");
            true
        }
        Err(e) => {
            tracing::debug!(error = %e, "Firebase JWT: verification failed");
            false
        }
    }
}

fn unauthorized(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        axum::Json(serde_json::json!({
            "code": "UNAUTHORIZED",
            "message": message
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq_same() {
        assert!(constant_time_eq(b"secret123", b"secret123"));
    }

    #[test]
    fn test_constant_time_eq_different() {
        assert!(!constant_time_eq(b"secret123", b"secret456"));
    }

    #[test]
    fn test_extract_key_bearer() {
        use axum::http::HeaderValue;
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer my-token"));
        assert_eq!(extract_key(&headers), Some("my-token".to_string()));
    }

    #[test]
    fn test_extract_token_from_query() {
        let uri: axum::http::Uri = "/ws?mode=shell&token=abc123&tier=explorer"
            .parse()
            .expect("valid URI");
        assert_eq!(extract_token_from_query(&uri), Some("abc123".to_string()));
    }

    #[test]
    fn test_extract_token_from_query_missing() {
        let uri: axum::http::Uri = "/ws?mode=shell".parse().expect("valid URI");
        assert_eq!(extract_token_from_query(&uri), None);
    }

    #[tokio::test]
    async fn test_verify_id_token_rejects_simple_string() {
        // The old stub accepted "fake.token.jwt" — the new impl must reject it
        assert!(!verify_id_token("fake.token.jwt").await);
    }

    #[tokio::test]
    async fn test_verify_id_token_rejects_empty() {
        assert!(!verify_id_token("").await);
    }

    #[tokio::test]
    async fn test_verify_id_token_rejects_malformed_jwt() {
        // Valid base64 header but garbage payload/signature
        let fake =
            "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.fake_signature";
        assert!(!verify_id_token(fake).await);
    }

    #[test]
    fn test_jwks_cache_starts_stale() {
        let cache = JwksCache::new();
        assert!(cache.is_stale());
        assert!(cache.keys.is_empty());
    }
}
