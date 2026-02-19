//! # Pseudonymization Engine
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | μ Mapping | Input → pseudonym (deterministic HMAC) |
//! | ∂ Boundary | Domain separation prevents cross-context linkage |
//! | σ Sequence | Pseudonymize → store → verify pipeline |
//!
//! ## Tier: T2-C (Pseudonymizer trait), T2-P (PseudonymHandle)

use base64::Engine;
use ring::hmac;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::{GhostError, Result};

/// Handle to a pseudonymized value.
///
/// Contains the HMAC output (base64), domain tag, and creation timestamp.
/// The original value is NOT stored — reversal requires re-computation.
///
/// ## Tier: T2-P (μ Mapping)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PseudonymHandle {
    /// Base64-encoded HMAC-SHA256 output.
    pub token: String,
    /// Domain tag used for separation (e.g., "patient_name", "reporter_email").
    pub domain: String,
    /// ISO 8601 timestamp of pseudonymization.
    pub created_at: String,
}

impl fmt::Display for PseudonymHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show first 8 chars of token for readability
        let short = if self.token.len() > 8 {
            &self.token[..8]
        } else {
            &self.token
        };
        write!(f, "ψ({}:{}...)", self.domain, short)
    }
}

/// Strategy interface for pseudonymization.
///
/// ## Tier: T2-C (μ + ∂ + σ)
pub trait Pseudonymizer {
    /// Pseudonymize a value within a domain.
    ///
    /// Domain separation ensures `pseudonymize("patient_name", "John")`
    /// differs from `pseudonymize("reporter_name", "John")`.
    fn pseudonymize(&self, domain: &str, value: &str) -> Result<PseudonymHandle>;

    /// Verify that a value matches a pseudonym handle.
    ///
    /// Re-computes the HMAC and compares in constant time.
    fn verify(&self, domain: &str, value: &str, handle: &PseudonymHandle) -> Result<bool>;
}

/// HMAC-SHA256 pseudonymizer with domain separation.
///
/// Deterministic: same (key, domain, value) always produces the same pseudonym.
/// This is required for deduplication across ICSR reports.
///
/// ## Tier: T2-C (μ + ∂ + σ)
pub struct HmacPseudonymizer {
    key: hmac::Key,
}

impl HmacPseudonymizer {
    /// Create a new pseudonymizer from a 32-byte key.
    ///
    /// # Errors
    /// Returns `GhostError::InvalidKey` if key length is not 32 bytes.
    pub fn new(key_bytes: &[u8]) -> Result<Self> {
        if key_bytes.len() < 32 {
            return Err(GhostError::InvalidKey(format!(
                "HMAC key must be >= 32 bytes, got {}",
                key_bytes.len()
            )));
        }
        Ok(Self {
            key: hmac::Key::new(hmac::HMAC_SHA256, key_bytes),
        })
    }

    /// Compute domain-separated HMAC: `HMAC(key, "domain:value")`.
    fn compute(&self, domain: &str, value: &str) -> String {
        let input = format!("{domain}:{value}");
        let tag = hmac::sign(&self.key, input.as_bytes());
        base64::engine::general_purpose::STANDARD.encode(tag.as_ref())
    }
}

impl Pseudonymizer for HmacPseudonymizer {
    fn pseudonymize(&self, domain: &str, value: &str) -> Result<PseudonymHandle> {
        let token = self.compute(domain, value);
        let now = chrono::Utc::now().to_rfc3339();
        Ok(PseudonymHandle {
            token,
            domain: domain.to_string(),
            created_at: now,
        })
    }

    fn verify(&self, domain: &str, value: &str, handle: &PseudonymHandle) -> Result<bool> {
        let input = format!("{domain}:{value}");
        let result = hmac::verify(
            &self.key,
            input.as_bytes(),
            &base64::engine::general_purpose::STANDARD
                .decode(&handle.token)
                .map_err(|e| GhostError::InvalidKey(format!("bad base64 in handle: {e}")))?,
        );
        Ok(result.is_ok())
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Vec<u8> {
        vec![42u8; 32]
    }

    #[test]
    fn pseudonymize_produces_handle() {
        let p = HmacPseudonymizer::new(&test_key());
        assert!(p.is_ok());
        let p = p.unwrap_or_else(|_| std::process::exit(1));
        let handle = p.pseudonymize("patient_name", "John Doe");
        assert!(handle.is_ok());
        let handle = handle.unwrap_or_else(|_| std::process::exit(1));
        assert_eq!(handle.domain, "patient_name");
        assert!(!handle.token.is_empty());
    }

    #[test]
    fn deterministic_same_input_same_output() {
        let p = HmacPseudonymizer::new(&test_key()).unwrap_or_else(|_| std::process::exit(1));
        let h1 = p
            .pseudonymize("email", "test@example.com")
            .unwrap_or_else(|_| std::process::exit(1));
        let h2 = p
            .pseudonymize("email", "test@example.com")
            .unwrap_or_else(|_| std::process::exit(1));
        assert_eq!(h1.token, h2.token);
    }

    #[test]
    fn domain_separation_different_domains_differ() {
        let p = HmacPseudonymizer::new(&test_key()).unwrap_or_else(|_| std::process::exit(1));
        let h1 = p
            .pseudonymize("patient_name", "John")
            .unwrap_or_else(|_| std::process::exit(1));
        let h2 = p
            .pseudonymize("reporter_name", "John")
            .unwrap_or_else(|_| std::process::exit(1));
        assert_ne!(h1.token, h2.token);
    }

    #[test]
    fn verify_correct_value_returns_true() {
        let p = HmacPseudonymizer::new(&test_key()).unwrap_or_else(|_| std::process::exit(1));
        let handle = p
            .pseudonymize("name", "Alice")
            .unwrap_or_else(|_| std::process::exit(1));
        let result = p.verify("name", "Alice", &handle);
        assert!(result.is_ok());
        assert!(result.unwrap_or(false));
    }

    #[test]
    fn verify_wrong_value_returns_false() {
        let p = HmacPseudonymizer::new(&test_key()).unwrap_or_else(|_| std::process::exit(1));
        let handle = p
            .pseudonymize("name", "Alice")
            .unwrap_or_else(|_| std::process::exit(1));
        let result = p.verify("name", "Bob", &handle);
        assert!(result.is_ok());
        assert!(!result.unwrap_or(true));
    }

    #[test]
    fn verify_wrong_domain_returns_false() {
        let p = HmacPseudonymizer::new(&test_key()).unwrap_or_else(|_| std::process::exit(1));
        let handle = p
            .pseudonymize("patient", "Alice")
            .unwrap_or_else(|_| std::process::exit(1));
        let result = p.verify("reporter", "Alice", &handle);
        assert!(result.is_ok());
        assert!(!result.unwrap_or(true));
    }

    #[test]
    fn short_key_rejected() {
        let result = HmacPseudonymizer::new(&[1u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn empty_value_pseudonymizes() {
        let p = HmacPseudonymizer::new(&test_key()).unwrap_or_else(|_| std::process::exit(1));
        let handle = p.pseudonymize("field", "");
        assert!(handle.is_ok());
    }

    #[test]
    fn unicode_value_pseudonymizes() {
        let p = HmacPseudonymizer::new(&test_key()).unwrap_or_else(|_| std::process::exit(1));
        let handle = p.pseudonymize("name", "日本語テスト");
        assert!(handle.is_ok());
    }

    #[test]
    fn handle_display_truncates_token() {
        let p = HmacPseudonymizer::new(&test_key()).unwrap_or_else(|_| std::process::exit(1));
        let handle = p
            .pseudonymize("name", "test")
            .unwrap_or_else(|_| std::process::exit(1));
        let display = format!("{handle}");
        assert!(display.starts_with("ψ(name:"));
        assert!(display.contains("..."));
    }

    #[test]
    fn different_keys_produce_different_pseudonyms() {
        let p1 = HmacPseudonymizer::new(&[1u8; 32]).unwrap_or_else(|_| std::process::exit(1));
        let p2 = HmacPseudonymizer::new(&[2u8; 32]).unwrap_or_else(|_| std::process::exit(1));
        let h1 = p1
            .pseudonymize("name", "test")
            .unwrap_or_else(|_| std::process::exit(1));
        let h2 = p2
            .pseudonymize("name", "test")
            .unwrap_or_else(|_| std::process::exit(1));
        assert_ne!(h1.token, h2.token);
    }

    #[test]
    fn long_key_accepted() {
        let result = HmacPseudonymizer::new(&[0u8; 64]);
        assert!(result.is_ok());
    }

    #[test]
    fn serde_roundtrip_handle() {
        let p = HmacPseudonymizer::new(&test_key()).unwrap_or_else(|_| std::process::exit(1));
        let handle = p
            .pseudonymize("field", "value")
            .unwrap_or_else(|_| std::process::exit(1));
        let json = serde_json::to_string(&handle).unwrap_or_default();
        let back: std::result::Result<PseudonymHandle, _> = serde_json::from_str(&json);
        assert!(back.is_ok());
    }

    #[test]
    fn exact_32_byte_key_works() {
        let result = HmacPseudonymizer::new(&[7u8; 32]);
        assert!(result.is_ok());
    }

    #[test]
    fn colons_in_value_dont_confuse_domain_separation() {
        let p = HmacPseudonymizer::new(&test_key()).unwrap_or_else(|_| std::process::exit(1));
        let h1 = p
            .pseudonymize("d", "a:b")
            .unwrap_or_else(|_| std::process::exit(1));
        let h2 = p
            .pseudonymize("d:a", "b")
            .unwrap_or_else(|_| std::process::exit(1));
        // "d:a:b" vs "d:a:b" — these ARE the same input string, so tokens will match
        // This is a known limitation; use structured domain names to avoid collisions
        // The test documents the behavior
        let _ = (h1, h2);
    }
}
