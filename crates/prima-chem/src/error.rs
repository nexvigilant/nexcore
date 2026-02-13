// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Error types for prima-chem.
//!
//! ## Primitive Grounding: ∂ (Boundary)
//!
//! Errors represent boundaries between valid and invalid molecular states.

use thiserror::Error;

/// Chemistry error type.
///
/// ## Tier: T2-P (∂ + σ)
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ChemError {
    /// Invalid element symbol.
    #[error("Invalid element: {0}")]
    InvalidElement(String),

    /// Invalid SMILES string.
    #[error("Invalid SMILES at position {position}: {message}")]
    InvalidSmiles { position: usize, message: String },

    /// Invalid bond order.
    #[error("Invalid bond order: {0}")]
    InvalidBondOrder(u8),

    /// Atom not found.
    #[error("Atom not found: {0}")]
    AtomNotFound(usize),

    /// Invalid valence.
    #[error("Invalid valence for {element}: expected {expected}, got {actual}")]
    InvalidValence {
        element: String,
        expected: u8,
        actual: u8,
    },

    /// Geometry error.
    #[error("Geometry error: {0}")]
    Geometry(String),

    /// Reaction error.
    #[error("Reaction error: {0}")]
    Reaction(String),
}

/// Result type for chemistry operations.
pub type ChemResult<T> = Result<T, ChemError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let e = ChemError::InvalidElement("Xx".to_string());
        assert!(e.to_string().contains("Xx"));
    }

    #[test]
    fn test_smiles_error() {
        let e = ChemError::InvalidSmiles {
            position: 5,
            message: "unexpected character".to_string(),
        };
        assert!(e.to_string().contains("position 5"));
    }
}
