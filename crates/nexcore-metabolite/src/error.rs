// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Error types for the metabolite prediction engine.

use nexcore_error::Error;

/// Errors produced by the metabolite prediction engine.
#[derive(Debug, Error)]
pub enum MetaboliteError {
    /// The input SMILES string could not be parsed into a valid molecule.
    #[error("SMILES parse error: {0}")]
    SmilesParse(String),

    /// A prediction step failed for an internal reason.
    #[error("prediction failed: {0}")]
    PredictionFailed(String),
}

/// Convenience alias for `Result<T, MetaboliteError>`.
pub type MetaboliteResult<T> = Result<T, MetaboliteError>;
