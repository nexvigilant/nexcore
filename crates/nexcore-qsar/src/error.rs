// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Error types for QSAR toxicity prediction operations.

use nexcore_error::Error;

/// Errors that can occur during QSAR prediction.
#[derive(Debug, Error)]
pub enum QsarError {
    /// SMILES string could not be parsed into a valid molecule.
    #[error("SMILES parse error: {0}")]
    SmilesParse(String),

    /// A descriptor required by the model could not be calculated.
    #[error("descriptor calculation failed: {0}")]
    DescriptorFailed(String),

    /// The requested model endpoint is not yet available.
    #[error("model not available: {0}")]
    ModelNotAvailable(String),
}

/// Convenience alias for `Result<T, QsarError>`.
pub type QsarResult<T> = Result<T, QsarError>;
