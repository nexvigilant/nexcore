// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Code generation errors.
//!
//! ## Tier: T2-P (∂ + Σ)

use thiserror::Error;

/// Code generation error.
#[derive(Error, Debug)]
pub enum CodegenError {
    #[error("Unsupported construct: {0}")]
    Unsupported(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Unknown identifier: {0}")]
    UnknownIdentifier(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl CodegenError {
    /// Create an unsupported construct error
    #[must_use]
    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self::Unsupported(msg.into())
    }

    /// Create a backend error
    #[must_use]
    pub fn backend(msg: impl Into<String>) -> Self {
        Self::Backend(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let e = CodegenError::unsupported("closures");
        assert!(e.to_string().contains("closures"));
    }

    #[test]
    fn test_type_mismatch() {
        let e = CodegenError::TypeMismatch {
            expected: "N".to_string(),
            actual: "S".to_string(),
        };
        assert!(e.to_string().contains("expected N"));
        assert!(e.to_string().contains("got S"));
    }
}
