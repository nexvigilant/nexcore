//! Key Derivation Function — PBKDF2-HMAC-SHA256 via ring.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Sequence (σ) | Iterated hash stretching |
//! | T1: State (ς) | Derived key material |
//! | T2-P: Salt | 32-byte random salt |

use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use std::num::NonZeroU32;

use crate::error::{Result, VaultError};
use crate::types::Salt;

/// Length of derived keys (256-bit for AES-256).
pub const KEY_LEN: usize = 32;

/// Length of salt in bytes.
pub const SALT_LEN: usize = 32;

/// Derive a 256-bit key from a password and salt using PBKDF2-HMAC-SHA256.
///
/// # Errors
/// Returns `VaultError::Crypto` if the iteration count is zero.
pub fn derive_key(password: &[u8], salt: &[u8], iterations: u32) -> Result<[u8; KEY_LEN]> {
    let iterations = NonZeroU32::new(iterations)
        .ok_or_else(|| VaultError::Crypto("PBKDF2 iteration count must be non-zero".into()))?;

    let mut key = [0u8; KEY_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt,
        password,
        &mut key,
    );

    Ok(key)
}

/// Verify a password against a known key derivation (constant-time).
///
/// # Errors
/// Returns `VaultError::AuthFailed` if verification fails.
pub fn verify_key(
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    expected_key: &[u8],
) -> Result<()> {
    let iterations = NonZeroU32::new(iterations)
        .ok_or_else(|| VaultError::Crypto("PBKDF2 iteration count must be non-zero".into()))?;

    pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt,
        password,
        expected_key,
    )
    .map_err(|_| VaultError::AuthFailed)
}

/// Generate a cryptographically random 32-byte salt.
///
/// # Errors
/// Returns `VaultError::Crypto` if the system random source fails.
pub fn generate_salt() -> Result<Salt> {
    let rng = SystemRandom::new();
    let mut salt_bytes = [0u8; SALT_LEN];
    rng.fill(&mut salt_bytes)
        .map_err(|_| VaultError::Crypto("failed to generate random salt".into()))?;
    Ok(Salt::from_bytes(&salt_bytes))
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_key_deterministic() {
        let password = b"test-password";
        let salt = [1u8; 32];
        let key1 = derive_key(password, &salt, 1000);
        let key2 = derive_key(password, &salt, 1000);
        assert!(key1.is_ok());
        assert!(key2.is_ok());
        assert_eq!(
            key1.unwrap_or([0u8; KEY_LEN]),
            key2.unwrap_or([1u8; KEY_LEN])
        );
    }

    #[test]
    fn different_salts_different_keys() {
        let password = b"test-password";
        let salt1 = [1u8; 32];
        let salt2 = [2u8; 32];
        let key1 = derive_key(password, &salt1, 1000);
        let key2 = derive_key(password, &salt2, 1000);
        assert!(key1.is_ok());
        assert!(key2.is_ok());
        assert_ne!(
            key1.unwrap_or([0u8; KEY_LEN]),
            key2.unwrap_or([0u8; KEY_LEN])
        );
    }

    #[test]
    fn different_passwords_different_keys() {
        let salt = [1u8; 32];
        let key1 = derive_key(b"password1", &salt, 1000);
        let key2 = derive_key(b"password2", &salt, 1000);
        assert!(key1.is_ok());
        assert!(key2.is_ok());
        assert_ne!(
            key1.unwrap_or([0u8; KEY_LEN]),
            key2.unwrap_or([0u8; KEY_LEN])
        );
    }

    #[test]
    fn verify_key_accepts_correct() {
        let password = b"test-password";
        let salt = [1u8; 32];
        let key = derive_key(password, &salt, 1000);
        assert!(key.is_ok());
        let key = key.unwrap_or([0u8; KEY_LEN]);
        let result = verify_key(password, &salt, 1000, &key);
        assert!(result.is_ok());
    }

    #[test]
    fn verify_key_rejects_wrong_password() {
        let salt = [1u8; 32];
        let key = derive_key(b"correct", &salt, 1000);
        assert!(key.is_ok());
        let key = key.unwrap_or([0u8; KEY_LEN]);
        let result = verify_key(b"wrong", &salt, 1000, &key);
        assert!(result.is_err());
    }

    #[test]
    fn generate_salt_produces_valid_salt() {
        let salt = generate_salt();
        assert!(salt.is_ok());
        let salt = salt.unwrap_or(Salt("".into()));
        let bytes = salt.to_bytes();
        assert!(bytes.is_ok());
        assert_eq!(bytes.unwrap_or_default().len(), SALT_LEN);
    }

    #[test]
    fn generate_salt_unique() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        assert!(salt1.is_ok());
        assert!(salt2.is_ok());
        // Overwhelmingly likely to be different
        assert_ne!(
            salt1.unwrap_or(Salt("a".into())).0,
            salt2.unwrap_or(Salt("a".into())).0
        );
    }
}
