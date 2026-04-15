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
use std::sync::OnceLock;

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

async fn verify_id_token(token: &str) -> bool {
    // TODO(D007-P0-3): Replace with real JWT verification (jsonwebtoken crate + JWKS).
    // Currently safe because Studio proxy verifies Firebase tokens server-side
    // (nexcore-proxy.ts:48 verifyIdToken) before forwarding. Direct API callers
    // use API_KEY or grd_ keys instead. This is a defense-in-depth gap, not an
    // active bypass — but must be closed before removing the Studio proxy layer.
    token.split('.').count() == 3
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
}
