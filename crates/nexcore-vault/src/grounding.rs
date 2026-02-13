//! # GroundsTo implementations for nexcore-vault types
//!
//! Connects encrypted secret management types to the Lex Primitiva type system.
//!
//! ## Domain Signature
//!
//! - **∂ (Boundary)**: cryptographic boundaries, access control
//! - **π (Persistence)**: encrypted storage on disk
//! - **μ (Mapping)**: SecretName → EncryptedValue

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::error::VaultError;
use crate::types::{PlaintextExport, Salt, SecretName, VaultEntry, VaultFile};

// ---------------------------------------------------------------------------
// T2-P: Cryptographic primitives
// ---------------------------------------------------------------------------

/// Salt: T2-P (∂ + N), dominant ∂
///
/// Cryptographic salt (32 bytes). Boundary-dominant: provides
/// the cryptographic boundary for key derivation.
impl GroundsTo for Salt {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- crypto boundary
            LexPrimitiva::Quantity, // N -- 32 bytes of entropy
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// SecretName: T2-P (λ + ∂), dominant λ
///
/// Validated secret name. Location-dominant: addressing secrets.
impl GroundsTo for SecretName {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // λ -- secret address
            LexPrimitiva::Boundary, // ∂ -- character validation
        ])
        .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T2-C: Encrypted entry
// ---------------------------------------------------------------------------

/// VaultEntry: T2-C (∂ + π + ς + N), dominant ∂
///
/// A single encrypted vault entry: nonce, ciphertext, timestamps.
/// Boundary-dominant: encryption IS a cryptographic boundary.
impl GroundsTo for VaultEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,    // ∂ -- encryption boundary
            LexPrimitiva::Persistence, // π -- stored on disk
            LexPrimitiva::State,       // ς -- created/updated timestamps
            LexPrimitiva::Quantity,    // N -- nonce, ciphertext bytes
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T3: Domain aggregates
// ---------------------------------------------------------------------------

/// VaultFile: T3 (∂ + π + μ + ς + N + ∃), dominant ∂
///
/// The encrypted vault file format. Boundary-dominant: everything
/// is wrapped in a cryptographic boundary (PBKDF2 + AES-256-GCM).
impl GroundsTo for VaultFile {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,    // ∂ -- AES-256-GCM encryption
            LexPrimitiva::Persistence, // π -- file storage
            LexPrimitiva::Mapping,     // μ -- name → entry
            LexPrimitiva::State,       // ς -- version state
            LexPrimitiva::Quantity,    // N -- version number
            LexPrimitiva::Existence,   // ∃ -- entry presence check
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// PlaintextExport: T2-C (μ + π + N + ∃), dominant μ
///
/// Plaintext export format for import/export. Mapping-dominant:
/// name → value mapping without encryption.
impl GroundsTo for PlaintextExport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,     // μ -- name → value
            LexPrimitiva::Persistence, // π -- serialization format
            LexPrimitiva::Quantity,    // N -- version number
            LexPrimitiva::Existence,   // ∃ -- secret presence
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// VaultError: T2-C (∂ + ∅ + π + →), dominant ∂
///
/// Vault errors: auth failures, crypto failures, not found, already exists.
impl GroundsTo for VaultError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,    // ∂ -- auth/crypto boundaries
            LexPrimitiva::Void,        // ∅ -- not found
            LexPrimitiva::Persistence, // π -- I/O failures
            LexPrimitiva::Causality,   // → -- operation failures
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn salt_is_boundary_dominant() {
        assert_eq!(Salt::dominant_primitive(), Some(LexPrimitiva::Boundary));
        assert_eq!(Salt::tier(), Tier::T2Primitive);
    }

    #[test]
    fn secret_name_is_location_dominant() {
        assert_eq!(
            SecretName::dominant_primitive(),
            Some(LexPrimitiva::Location)
        );
    }

    #[test]
    fn vault_entry_is_boundary_dominant() {
        assert_eq!(
            VaultEntry::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(VaultEntry::tier(), Tier::T2Composite);
    }

    #[test]
    fn vault_file_is_t3() {
        assert_eq!(VaultFile::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            VaultFile::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn plaintext_export_is_mapping_dominant() {
        assert_eq!(
            PlaintextExport::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn vault_error_is_boundary_dominant() {
        assert_eq!(
            VaultError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }
}
