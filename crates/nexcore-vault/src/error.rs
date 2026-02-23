//! Error types for nexcore-vault.
//!
//! Uses `thiserror` for library errors, following nexcore conventions.

use std::path::PathBuf;

/// Vault error hierarchy.
#[derive(Debug, nexcore_error::Error)]
pub enum VaultError {
    /// I/O error with path context.
    #[error("I/O error at {path:?}: {source}")]
    Io {
        /// Path involved in the error.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// JSON serialization/deserialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML deserialization failed.
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Configuration error.
    #[error("config error: {0}")]
    Config(String),

    /// Cryptographic operation failed.
    #[error("crypto error: {0}")]
    Crypto(String),

    /// Authentication failed (wrong password or tampered data).
    #[error("authentication failed: wrong password or corrupted vault")]
    AuthFailed,

    /// Secret not found.
    #[error("secret not found: {0}")]
    NotFound(String),

    /// Vault already exists (on init).
    #[error("vault already exists at {0:?}")]
    AlreadyExists(PathBuf),

    /// Vault does not exist (on open).
    #[error("vault not found at {0:?}")]
    VaultNotFound(PathBuf),

    /// Password required but not provided.
    #[error("password required: set NEXCORE_VAULT_PASSWORD or provide interactively")]
    PasswordRequired,

    /// Base64 decoding failed.
    #[error("base64 decode error: {0}")]
    Base64(String),

    /// Invalid vault file format.
    #[error("invalid vault format: {0}")]
    InvalidFormat(String),
}

/// Convenience result type.
pub type Result<T> = std::result::Result<T, VaultError>;

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_auth_failed() {
        let e = VaultError::AuthFailed;
        assert!(e.to_string().contains("authentication failed"));
    }

    #[test]
    fn error_display_not_found() {
        let e = VaultError::NotFound("api_key".into());
        assert!(e.to_string().contains("api_key"));
    }

    #[test]
    fn error_display_io() {
        let e = VaultError::Io {
            path: PathBuf::from("/tmp/test"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "gone"),
        };
        let msg = e.to_string();
        assert!(msg.contains("/tmp/test"));
        assert!(msg.contains("gone"));
    }

    #[test]
    fn error_display_crypto() {
        let e = VaultError::Crypto("bad key".into());
        assert!(e.to_string().contains("bad key"));
    }

    #[test]
    fn result_type_alias_works() {
        let ok: Result<u32> = Ok(42);
        assert!(ok.is_ok());
        let err: Result<u32> = Err(VaultError::AuthFailed);
        assert!(err.is_err());
    }
}
