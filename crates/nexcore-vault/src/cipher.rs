//! AES-256-GCM authenticated encryption via ring.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Sequence (σ) | Encrypt → store (nonce, ciphertext) → decrypt |
//! | T1: State (ς) | Ciphertext + auth tag |

use ring::aead::{AES_256_GCM, Aad, LessSafeKey, NONCE_LEN, Nonce, UnboundKey};
use ring::rand::{SecureRandom, SystemRandom};

use nexcore_codec::base64 as b64;

use crate::error::{Result, VaultError};

/// Encrypt plaintext using AES-256-GCM.
///
/// Returns `(nonce_b64, ciphertext_b64)` where both are base64-encoded.
/// The ciphertext includes the 16-byte authentication tag appended by ring.
///
/// # Errors
/// Returns `VaultError::Crypto` if encryption fails or nonce generation fails.
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<(String, String)> {
    let unbound_key = UnboundKey::new(&AES_256_GCM, key)
        .map_err(|_| VaultError::Crypto("failed to create AES-256-GCM key".into()))?;
    let sealing_key = LessSafeKey::new(unbound_key);

    // Generate unique 12-byte nonce
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| VaultError::Crypto("failed to generate nonce".into()))?;

    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    // ring appends the auth tag to the buffer in-place
    let mut in_out = plaintext.to_vec();
    sealing_key
        .seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| VaultError::Crypto("AES-256-GCM encryption failed".into()))?;

    let nonce_b64 = b64::encode(nonce_bytes);
    let ciphertext_b64 = b64::encode(&in_out);

    Ok((nonce_b64, ciphertext_b64))
}

/// Decrypt ciphertext using AES-256-GCM.
///
/// Takes base64-encoded nonce and ciphertext (with appended auth tag).
///
/// # Errors
/// Returns `VaultError::AuthFailed` if decryption fails (wrong key or tampered).
/// Returns `VaultError::Base64` if base64 decoding fails.
pub fn decrypt(key: &[u8; 32], nonce_b64: &str, ciphertext_b64: &str) -> Result<Vec<u8>> {
    let nonce_bytes: Vec<u8> =
        b64::decode(nonce_b64).map_err(|e| VaultError::Base64(format!("nonce: {e}")))?;

    if nonce_bytes.len() != NONCE_LEN {
        return Err(VaultError::InvalidFormat(format!(
            "nonce must be {NONCE_LEN} bytes, got {}",
            nonce_bytes.len()
        )));
    }

    let mut nonce_arr = [0u8; NONCE_LEN];
    nonce_arr.copy_from_slice(&nonce_bytes);

    let mut ciphertext =
        b64::decode(ciphertext_b64).map_err(|e| VaultError::Base64(format!("ciphertext: {e}")))?;

    let unbound_key = UnboundKey::new(&AES_256_GCM, key)
        .map_err(|_| VaultError::Crypto("failed to create AES-256-GCM key".into()))?;
    let opening_key = LessSafeKey::new(unbound_key);

    let nonce = Nonce::assume_unique_for_key(nonce_arr);

    let plaintext = opening_key
        .open_in_place(nonce, Aad::empty(), &mut ciphertext)
        .map_err(|_| VaultError::AuthFailed)?;

    Ok(plaintext.to_vec())
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [42u8; 32]
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let key = test_key();
        let plaintext = b"hello, vault!";

        let result = encrypt(&key, plaintext);
        assert!(result.is_ok());
        let (nonce, ciphertext) = result.unwrap_or(("".into(), "".into()));

        let decrypted = decrypt(&key, &nonce, &ciphertext);
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap_or_default(), plaintext);
    }

    #[test]
    fn encrypt_decrypt_empty() {
        let key = test_key();
        let plaintext = b"";

        let result = encrypt(&key, plaintext);
        assert!(result.is_ok());
        let (nonce, ciphertext) = result.unwrap_or(("".into(), "".into()));

        let decrypted = decrypt(&key, &nonce, &ciphertext);
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap_or_default(), plaintext);
    }

    #[test]
    fn encrypt_decrypt_large() {
        let key = test_key();
        let plaintext = vec![0xABu8; 10_000];

        let result = encrypt(&key, &plaintext);
        assert!(result.is_ok());
        let (nonce, ciphertext) = result.unwrap_or(("".into(), "".into()));

        let decrypted = decrypt(&key, &nonce, &ciphertext);
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap_or_default(), plaintext);
    }

    #[test]
    fn wrong_key_fails() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let plaintext = b"secret data";

        let result = encrypt(&key1, plaintext);
        assert!(result.is_ok());
        let (nonce, ciphertext) = result.unwrap_or(("".into(), "".into()));

        let decrypted = decrypt(&key2, &nonce, &ciphertext);
        assert!(decrypted.is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let key = test_key();
        let plaintext = b"tamper test";

        let result = encrypt(&key, plaintext);
        assert!(result.is_ok());
        let (nonce, ciphertext) = result.unwrap_or(("".into(), "".into()));

        // Tamper with the ciphertext by modifying the base64
        let mut raw = b64::decode(&ciphertext).unwrap_or_default();
        if !raw.is_empty() {
            raw[0] ^= 0xFF;
        }
        let tampered = b64::encode(&raw);

        let decrypted = decrypt(&key, &nonce, &tampered);
        assert!(decrypted.is_err());
    }

    #[test]
    fn invalid_nonce_length_fails() {
        let key = test_key();
        let bad_nonce = b64::encode([0u8; 5]); // Wrong length

        let result = decrypt(&key, &bad_nonce, "AAAA");
        assert!(result.is_err());
    }

    #[test]
    fn unique_nonces_per_encryption() {
        let key = test_key();
        let plaintext = b"same data";

        let r1 = encrypt(&key, plaintext);
        let r2 = encrypt(&key, plaintext);
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        let (nonce1, _) = r1.unwrap_or(("".into(), "".into()));
        let (nonce2, _) = r2.unwrap_or(("".into(), "".into()));

        // Nonces should be different (overwhelmingly likely)
        assert_ne!(nonce1, nonce2);
    }
}
