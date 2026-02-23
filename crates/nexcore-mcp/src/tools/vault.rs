//! Vault cryptographic MCP tools — AES-256-GCM encryption + PBKDF2 key derivation.
//!
//! Pure-function wrappers for nexcore-vault's stateless crypto primitives:
//! key derivation, symmetric encryption/decryption, salt generation.

use nexcore_vault::cipher;
use nexcore_vault::config::VaultConfig;
use nexcore_vault::kdf;
use nexcore_vault::types::Salt;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::vault::{
    VaultConfigSampleParams, VaultDecryptParams, VaultDeriveKeyParams, VaultEncryptParams,
    VaultGenerateSaltParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

/// Encode bytes as lowercase hex string.
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Decode hex string to bytes.
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.trim();
    if hex.len() % 2 != 0 {
        return Err("hex string must have even length".to_string());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| format!("invalid hex: {e}")))
        .collect()
}

/// Parse a hex-encoded 32-byte key.
fn parse_key(hex: &str) -> Result<[u8; 32], String> {
    let bytes = hex_to_bytes(hex)?;
    if bytes.len() != 32 {
        return Err(format!(
            "key must be 32 bytes (64 hex chars), got {}",
            bytes.len()
        ));
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

const DEFAULT_ITERATIONS: u32 = 600_000;

// ── Tools ────────────────────────────────────────────────────────────────

/// Derive a 256-bit key from password + salt using PBKDF2-HMAC-SHA256.
pub fn vault_derive_key(p: VaultDeriveKeyParams) -> Result<CallToolResult, McpError> {
    let iterations = p.iterations.unwrap_or(DEFAULT_ITERATIONS);

    let salt = Salt(p.salt);
    let salt_bytes = match salt.to_bytes() {
        Ok(b) => b,
        Err(e) => return err_result(&format!("invalid base64 salt: {e}")),
    };

    match kdf::derive_key(p.password.as_bytes(), &salt_bytes, iterations) {
        Ok(key) => ok_json(json!({
            "key_hex": bytes_to_hex(&key),
            "iterations": iterations,
            "algorithm": "PBKDF2-HMAC-SHA256",
            "key_bits": 256,
        })),
        Err(e) => err_result(&format!("key derivation failed: {e}")),
    }
}

/// Encrypt plaintext using AES-256-GCM.
pub fn vault_encrypt(p: VaultEncryptParams) -> Result<CallToolResult, McpError> {
    let key = match parse_key(&p.key) {
        Ok(k) => k,
        Err(e) => return err_result(&e),
    };

    match cipher::encrypt(&key, p.plaintext.as_bytes()) {
        Ok((nonce_b64, ciphertext_b64)) => ok_json(json!({
            "nonce": nonce_b64,
            "ciphertext": ciphertext_b64,
            "algorithm": "AES-256-GCM",
            "plaintext_bytes": p.plaintext.len(),
        })),
        Err(e) => err_result(&format!("encryption failed: {e}")),
    }
}

/// Decrypt ciphertext using AES-256-GCM.
pub fn vault_decrypt(p: VaultDecryptParams) -> Result<CallToolResult, McpError> {
    let key = match parse_key(&p.key) {
        Ok(k) => k,
        Err(e) => return err_result(&e),
    };

    match cipher::decrypt(&key, &p.nonce, &p.ciphertext) {
        Ok(plaintext_bytes) => {
            let plaintext = String::from_utf8(plaintext_bytes)
                .unwrap_or_else(|e| format!("<non-utf8: {} bytes>", e.into_bytes().len()));
            ok_json(json!({
                "plaintext": plaintext,
                "algorithm": "AES-256-GCM",
            }))
        }
        Err(e) => err_result(&format!("decryption failed: {e}")),
    }
}

/// Generate a cryptographically random 32-byte salt.
pub fn vault_generate_salt(_p: VaultGenerateSaltParams) -> Result<CallToolResult, McpError> {
    match kdf::generate_salt() {
        Ok(salt) => ok_json(json!({
            "salt_base64": salt.0,
            "bytes": 32,
        })),
        Err(e) => err_result(&format!("salt generation failed: {e}")),
    }
}

/// Get sample vault configuration TOML.
pub fn vault_config_sample(_p: VaultConfigSampleParams) -> Result<CallToolResult, McpError> {
    let toml = VaultConfig::sample_toml();
    ok_json(json!({
        "toml": toml,
        "default_iterations": DEFAULT_ITERATIONS,
        "algorithm": "PBKDF2-HMAC-SHA256 + AES-256-GCM",
    }))
}
