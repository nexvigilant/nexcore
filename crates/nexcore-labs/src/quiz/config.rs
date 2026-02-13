//! Configuration for the quiz platform.
//!
//! Loads configuration from environment variables.
//! Follows the 12-factor app methodology.

use std::env;

use super::error::{QuizError, QuizResult};

/// Main configuration struct for the quiz platform.
///
/// All sensitive fields MUST be set via environment variables.
/// Call [`QuizConfig::from_env`] to load, then [`QuizConfig::validate`] to verify.
#[derive(Debug, Clone)]
pub struct QuizConfig {
    // === Application ===
    /// Base URL for the application.
    pub root_address: Option<String>,

    /// Secret key for JWT signing. **Required.**
    pub secret_key: Option<String>,

    /// Access token expiration in minutes.
    pub access_token_expire_minutes: i64,

    // === External Services (all from env vars) ===
    /// Primary storage connection (from env).
    pub primary_store: Option<String>,

    /// Cache connection (from env).
    pub cache_store: Option<String>,

    /// Cache expiry in seconds.
    pub cache_expiry: u64,

    // === Game Settings ===
    /// Game session TTL in seconds (default: 7200 = 2 hours).
    pub game_session_ttl: u64,

    // === Storage ===
    /// Storage backend: "local" or "s3".
    pub storage_backend: StorageBackend,

    /// Local storage path (when backend is "local").
    pub storage_path: Option<String>,

    /// S3 access key (when backend is "s3").
    pub s3_access_key: Option<String>,

    /// S3 secret key (when backend is "s3").
    pub s3_secret_key: Option<String>,

    /// S3 bucket name (when backend is "s3").
    pub s3_bucket_name: Option<String>,

    /// S3 base URL for public access.
    pub s3_base_url: Option<String>,

    /// S3 region.
    pub s3_region: Option<String>,

    /// Free storage limit per user in bytes (default: 50MB).
    pub free_storage_limit: i64,

    // === Search ===
    /// MeiliSearch URL.
    pub meilisearch_url: Option<String>,

    /// MeiliSearch API key.
    pub meilisearch_key: Option<String>,

    /// MeiliSearch index name.
    pub meilisearch_index: Option<String>,

    // === OAuth Providers ===
    /// Google OAuth client ID.
    pub google_client_id: Option<String>,

    /// Google OAuth client secret.
    pub google_client_secret: Option<String>,

    /// GitHub OAuth client ID.
    pub github_client_id: Option<String>,

    /// GitHub OAuth client secret.
    pub github_client_secret: Option<String>,

    // === Captcha ===
    /// hCaptcha secret key.
    pub hcaptcha_key: Option<String>,

    /// reCAPTCHA secret key.
    pub recaptcha_key: Option<String>,

    // === Admin ===
    /// List of moderator usernames.
    pub mods: Vec<String>,

    /// Whether new user registration is disabled.
    pub registration_disabled: bool,
}

/// Storage backend type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StorageBackend {
    /// Local filesystem storage.
    #[default]
    Local,
    /// Amazon S3 (or compatible) storage.
    S3,
}

impl QuizConfig {
    /// Load configuration from environment variables.
    ///
    /// # Required Environment Variables
    ///
    /// - `DATABASE_URL` - PostgreSQL connection string
    /// - `REDIS_URL` - Redis connection string
    /// - `SECRET_KEY` - JWT signing secret (min 32 chars)
    pub fn from_env() -> QuizResult<Self> {
        // Required variables - read from env
        let primary_store = env::var("DATABASE_URL").ok();
        let cache_store = env::var("REDIS_URL").ok();
        let secret_key = env::var("SECRET_KEY").ok();

        // Validate required vars
        if primary_store.is_none() {
            return Err(QuizError::Validation(
                "DATABASE_URL environment variable required".into(),
            ));
        }
        if cache_store.is_none() {
            return Err(QuizError::Validation(
                "REDIS_URL environment variable required".into(),
            ));
        }
        if secret_key.is_none() {
            return Err(QuizError::Validation(
                "SECRET_KEY environment variable required".into(),
            ));
        }

        // Optional with defaults
        let root_address = env::var("ROOT_ADDRESS").ok();

        let access_token_expire_minutes = env::var("ACCESS_TOKEN_EXPIRE_MINUTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        let cache_expiry = env::var("CACHE_EXPIRY")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

        let game_session_ttl = env::var("GAME_SESSION_TTL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(7200);

        // Storage
        let storage_backend = match env::var("STORAGE_BACKEND")
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Ok("s3") => StorageBackend::S3,
            _ => StorageBackend::Local,
        };

        let storage_path = env::var("STORAGE_PATH").ok();

        let free_storage_limit = env::var("FREE_STORAGE_LIMIT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(50 * 1024 * 1024);

        let meilisearch_index = env::var("MEILISEARCH_INDEX").ok();

        let mods = env::var("MODS")
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let registration_disabled = env::var("REGISTRATION_DISABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or_default();

        Ok(Self {
            primary_store,
            cache_store,
            secret_key,
            root_address,
            access_token_expire_minutes,
            cache_expiry,
            game_session_ttl,
            storage_backend,
            storage_path,
            s3_access_key: env::var("S3_ACCESS_KEY").ok(),
            s3_secret_key: env::var("S3_SECRET_KEY").ok(),
            s3_bucket_name: env::var("S3_BUCKET_NAME").ok(),
            s3_base_url: env::var("S3_BASE_URL").ok(),
            s3_region: env::var("S3_REGION").ok(),
            free_storage_limit,
            meilisearch_url: env::var("MEILISEARCH_URL").ok(),
            meilisearch_key: env::var("MEILISEARCH_KEY").ok(),
            meilisearch_index,
            google_client_id: env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET").ok(),
            github_client_id: env::var("GITHUB_CLIENT_ID").ok(),
            github_client_secret: env::var("GITHUB_CLIENT_SECRET").ok(),
            hcaptcha_key: env::var("HCAPTCHA_KEY").ok(),
            recaptcha_key: env::var("RECAPTCHA_KEY").ok(),
            mods,
            registration_disabled,
        })
    }

    /// Get primary store URL (PostgreSQL).
    ///
    /// # Errors
    ///
    /// Returns `QuizError::Validation` if DATABASE_URL was not set.
    pub fn pg_url(&self) -> QuizResult<&str> {
        self.primary_store
            .as_deref()
            .ok_or_else(|| QuizError::Validation("DATABASE_URL not configured".into()))
    }

    /// Get cache store URL (Redis).
    ///
    /// # Errors
    ///
    /// Returns `QuizError::Validation` if REDIS_URL was not set.
    pub fn redis_url(&self) -> QuizResult<&str> {
        self.cache_store
            .as_deref()
            .ok_or_else(|| QuizError::Validation("REDIS_URL not configured".into()))
    }

    /// Get secret key.
    ///
    /// # Errors
    ///
    /// Returns `QuizError::Validation` if SECRET_KEY was not set.
    pub fn secret(&self) -> QuizResult<&str> {
        self.secret_key
            .as_deref()
            .ok_or_else(|| QuizError::Validation("SECRET_KEY not configured".into()))
    }

    /// Validate the configuration for production use.
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if let Some(ref key) = self.secret_key {
            if key.len() < 32 {
                issues.push("SECRET_KEY should be at least 32 characters".into());
            }
        }

        if self.storage_backend == StorageBackend::S3 {
            if self.s3_access_key.is_none() {
                issues.push("S3_ACCESS_KEY is required when using S3 storage".into());
            }
            if self.s3_secret_key.is_none() {
                issues.push("S3_SECRET_KEY is required when using S3 storage".into());
            }
            if self.s3_bucket_name.is_none() {
                issues.push("S3_BUCKET_NAME is required when using S3 storage".into());
            }
        }

        issues
    }

    /// Check if Google OAuth is configured.
    pub fn has_google_oauth(&self) -> bool {
        self.google_client_id.is_some() && self.google_client_secret.is_some()
    }

    /// Check if GitHub OAuth is configured.
    pub fn has_github_oauth(&self) -> bool {
        self.github_client_id.is_some() && self.github_client_secret.is_some()
    }

    /// Check if captcha is configured.
    pub fn has_captcha(&self) -> bool {
        self.hcaptcha_key.is_some() || self.recaptcha_key.is_some()
    }

    /// Check if MeiliSearch is configured.
    pub fn has_search(&self) -> bool {
        self.meilisearch_url.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_backend_default() {
        assert_eq!(StorageBackend::default(), StorageBackend::Local);
    }
}
