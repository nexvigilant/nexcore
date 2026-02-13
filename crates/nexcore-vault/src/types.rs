//! Domain types for nexcore-vault.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: State (ς) | Encrypted vault blob |
//! | T1: Mapping (μ) | SecretName → EncryptedValue |
//! | T1: Exists (∃) | Secret presence check |
//! | T2-P: Salt | Newtype over `[u8; 32]` |
//! | T2-P: SecretName | Newtype over `String` |

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Tier: T2-P — Cryptographic salt (32 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Salt(pub String);

impl Salt {
    /// Create a Salt from raw bytes, encoding as base64.
    #[must_use]
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        use base64::Engine;
        Self(base64::engine::general_purpose::STANDARD.encode(bytes))
    }

    /// Decode the salt back to raw bytes.
    ///
    /// # Errors
    /// Returns `VaultError::Base64` if the stored string is invalid base64.
    pub fn to_bytes(&self) -> crate::error::Result<Vec<u8>> {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(&self.0)
            .map_err(|e| crate::error::VaultError::Base64(e.to_string()))
    }
}

/// Tier: T2-P — Secret name (validated string).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SecretName(pub String);

impl SecretName {
    /// Create a new secret name from a string.
    ///
    /// # Errors
    /// Returns `VaultError::Config` if the name is empty or contains invalid characters.
    pub fn new(name: impl Into<String>) -> crate::error::Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(crate::error::VaultError::Config(
                "secret name must not be empty".into(),
            ));
        }
        // Allow alphanumeric, hyphens, underscores, dots
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(crate::error::VaultError::Config(format!(
                "secret name '{name}' contains invalid characters (allowed: alphanumeric, -, _, .)"
            )));
        }
        Ok(Self(name))
    }

    /// Get the inner string reference.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SecretName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Tier: T2-C — A single encrypted vault entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    /// Base64-encoded 12-byte nonce.
    pub nonce: String,
    /// Base64-encoded ciphertext (includes 16-byte auth tag).
    pub ciphertext: String,
    /// When this secret was first created.
    pub created_at: String,
    /// When this secret was last updated.
    pub updated_at: String,
}

/// Tier: T3 — The encrypted vault file format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultFile {
    /// Format version (always 1).
    pub version: u32,
    /// Base64-encoded 32-byte salt for PBKDF2.
    pub salt: Salt,
    /// Encrypted entries keyed by secret name.
    pub entries: BTreeMap<String, VaultEntry>,
}

impl VaultFile {
    /// Create a new empty vault file with the given salt.
    #[must_use]
    pub fn new(salt: Salt) -> Self {
        Self {
            version: 1,
            salt,
            entries: BTreeMap::new(),
        }
    }
}

/// Plaintext export format for import/export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaintextExport {
    /// Format version.
    pub version: u32,
    /// Plaintext entries keyed by secret name.
    pub secrets: BTreeMap<String, String>,
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_name_valid() {
        assert!(SecretName::new("api_key").is_ok());
        assert!(SecretName::new("my-secret.v2").is_ok());
        assert!(SecretName::new("DATABASE_URL").is_ok());
    }

    #[test]
    fn secret_name_rejects_empty() {
        assert!(SecretName::new("").is_err());
    }

    #[test]
    fn secret_name_rejects_spaces() {
        assert!(SecretName::new("my secret").is_err());
    }

    #[test]
    fn secret_name_rejects_special_chars() {
        assert!(SecretName::new("my@secret").is_err());
        assert!(SecretName::new("secret/path").is_err());
    }

    #[test]
    fn salt_round_trip() {
        let bytes = [42u8; 32];
        let salt = Salt::from_bytes(&bytes);
        let decoded = salt.to_bytes();
        assert!(decoded.is_ok());
        let decoded = decoded.unwrap_or_default();
        assert_eq!(decoded.len(), 32);
        assert_eq!(decoded[0], 42);
    }

    #[test]
    fn vault_file_new() {
        let salt = Salt::from_bytes(&[0u8; 32]);
        let vf = VaultFile::new(salt);
        assert_eq!(vf.version, 1);
        assert!(vf.entries.is_empty());
    }

    #[test]
    fn secret_name_display() {
        let name = SecretName::new("test_key").unwrap_or(SecretName("fallback".into()));
        assert_eq!(format!("{name}"), "test_key");
    }

    #[test]
    fn vault_file_serialization() {
        let salt = Salt::from_bytes(&[1u8; 32]);
        let vf = VaultFile::new(salt);
        let json = serde_json::to_string(&vf);
        assert!(json.is_ok());
        let json = json.unwrap_or_default();
        assert!(json.contains("\"version\":1"));
    }
}
