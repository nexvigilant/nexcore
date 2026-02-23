// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Error types for the structural alert library.

use nexcore_error::Error;

/// Errors produced by the structural alert library.
#[derive(Debug, Error)]
pub enum AlertError {
    /// The input SMILES string could not be parsed into a molecule.
    #[error("SMILES parse error: {0}")]
    SmilesParse(String),

    /// A requested alert identifier was not found in the library.
    #[error("alert not found: {0}")]
    AlertNotFound(String),

    /// A built-in or custom alert pattern is syntactically invalid.
    #[error("invalid pattern: {0}")]
    InvalidPattern(String),
}

/// Convenience alias for `Result<T, AlertError>`.
pub type AlertResult<T> = Result<T, AlertError>;
