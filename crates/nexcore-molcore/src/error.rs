// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Error types for molecular core operations.

use nexcore_error::Error;

/// Errors from molecular core operations.
#[derive(Debug, Error)]
pub enum MolcoreError {
    /// Invalid SMILES syntax.
    #[error("Invalid SMILES at position {position}: {message}")]
    InvalidSmiles { position: usize, message: String },

    /// Unexpected end of SMILES string.
    #[error("Unexpected end of SMILES string")]
    UnexpectedEnd,

    /// Ring closure mismatch.
    #[error("Unclosed ring at digit {0}")]
    UnclosedRing(u8),

    /// Unmatched branch parenthesis.
    #[error("Unmatched parenthesis in SMILES")]
    UnmatchedParen,

    /// Invalid element symbol.
    #[error("Unknown element: {0}")]
    UnknownElement(String),

    /// Valence violation.
    #[error("Valence violation at atom {atom_idx}: expected <= {expected}, got {actual}")]
    ValenceViolation {
        atom_idx: usize,
        expected: u8,
        actual: u8,
    },

    /// Descriptor calculation error.
    #[error("Descriptor error: {0}")]
    DescriptorError(String),

    /// Fingerprint error.
    #[error("Fingerprint error: {0}")]
    FingerprintError(String),

    /// Substructure matching error.
    #[error("Substructure error: {0}")]
    SubstructureError(String),

    /// Chemistry error from prima-chem.
    #[error(transparent)]
    ChemError(#[from] prima_chem::ChemError),
}

/// Result type for molecular core operations.
pub type MolcoreResult<T> = Result<T, MolcoreError>;
