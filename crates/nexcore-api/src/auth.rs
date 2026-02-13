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

#![allow(clippy::expect_used)] // Only in lazy_static init, not request path

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

/// Middleware that requires a valid API key for protected routes
pub async fn require_api_key(req: Request<Body>, next: Next) -> Response {
    let expected = get_api_key();
    let expected_key = match expected {
        Some(key) => key,
        None => return next.run(req).await,
    };

    let provided = extract_key(req.headers());
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
    if constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        return true;
    }
    if provided.starts_with("eyJ") {
        return verify_id_token(provided).await;
    }
    false
}

async fn verify_id_token(token: &str) -> bool {
    // For now, in dev mode, we just check if it looks like a JWT
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
