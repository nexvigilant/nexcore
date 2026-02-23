//! Parameter types for vault cryptographic MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Derive a 256-bit key from password + salt using PBKDF2-HMAC-SHA256.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VaultDeriveKeyParams {
    /// Password to derive key from.
    pub password: String,
    /// Base64-encoded 32-byte salt.
    pub salt: String,
    /// PBKDF2 iterations (default: 600000, OWASP 2023 recommendation).
    pub iterations: Option<u32>,
}

/// Encrypt plaintext using AES-256-GCM.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VaultEncryptParams {
    /// Base64-encoded 256-bit key (32 bytes).
    pub key: String,
    /// UTF-8 plaintext to encrypt.
    pub plaintext: String,
}

/// Decrypt ciphertext using AES-256-GCM.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VaultDecryptParams {
    /// Base64-encoded 256-bit key (32 bytes).
    pub key: String,
    /// Base64-encoded 12-byte GCM nonce.
    pub nonce: String,
    /// Base64-encoded ciphertext (includes 16-byte auth tag).
    pub ciphertext: String,
}

/// Generate a cryptographically random 32-byte salt.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VaultGenerateSaltParams {}

/// Get the sample vault configuration TOML.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VaultConfigSampleParams {}
