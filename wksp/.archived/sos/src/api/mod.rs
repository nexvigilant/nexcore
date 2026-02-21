/// API client layer — μ Mapping (API-to-UI transform) + ∂ Boundary (HTTP edge)
///
/// All nexcore-api communication flows through this module.
/// Base URL and auth token are configurable via Settings page (stored in localStorage).
pub mod academy;
pub mod brain;
pub mod guardian;
pub mod pvos;
pub mod signal;
pub mod skills;

use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

const DEFAULT_API_URL: &str = "http://localhost:3030";
const STORAGE_KEY_API_URL: &str = "sos_api_url";
const STORAGE_KEY_AUTH_TOKEN: &str = "sos_auth_token";

/// Get the configured API base URL (no trailing slash)
pub fn api_base_url() -> String {
    LocalStorage::get::<String>(STORAGE_KEY_API_URL).unwrap_or_else(|_| DEFAULT_API_URL.to_string())
}

/// Get the configured auth token (if any)
pub fn auth_token() -> Option<String> {
    LocalStorage::get::<String>(STORAGE_KEY_AUTH_TOKEN).ok()
}

/// Set the API base URL
pub fn set_api_base_url(url: &str) {
    let _ = LocalStorage::set(STORAGE_KEY_API_URL, url.trim_end_matches('/'));
}

/// Set the auth token
pub fn set_auth_token(token: &str) {
    let _ = LocalStorage::set(STORAGE_KEY_AUTH_TOKEN, token);
}

/// Clear auth token
pub fn clear_auth_token() {
    LocalStorage::delete(STORAGE_KEY_AUTH_TOKEN);
}

/// Build full URL from path
pub fn url(path: &str) -> String {
    format!("{}{}", api_base_url(), path)
}

/// Common error type for API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<gloo_net::Error> for ApiError {
    fn from(err: gloo_net::Error) -> Self {
        ApiError {
            message: format!("Network error: {err}"),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError {
            message: format!("Parse error: {err}"),
        }
    }
}
