//! # Guardian Crypto
//!
//! Cryptographic operations for audit trails and 21 CFR Part 11 compliance.
//!
//! TODO: Implement SHA-256 hashing and audit trail signatures.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Placeholder for crypto operations (not yet implemented).
pub struct CryptoEngine;

impl CryptoEngine {
    /// Create a new crypto engine.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for CryptoEngine {
    fn default() -> Self {
        Self::new()
    }
}
