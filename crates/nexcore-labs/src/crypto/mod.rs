//! # Guardian Crypto
//!
//! Cryptographic operations for audit trails and 21 CFR Part 11 compliance.
//!
//! # Sovereignty Decision Required
//!
//! SHA-256 hashing needed for audit trail signatures. Per DP §2.2, crypto is an
//! essential domain where audited external code is justified (cannot implement
//! spec-compliant SHA-256 in <500 LOC). Recommended: `sha2` crate from RustCrypto
//! (pure Rust, audited, zero unsafe). Requires CEO confirmation before adding dep.
//!
//! Existing MCP tool `foundation_sha256` provides runtime hashing but is not
//! importable as a library dependency from this crate.

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
