//! Processor error types — ∂ on the failure path.

use std::fmt;

/// Errors that can occur during processing.
#[derive(Debug, Clone)]
pub enum ProcessorError {
    /// Input rejected by a boundary gate (∂).
    BoundaryRejection {
        /// Which boundary rejected.
        boundary: String,
        /// Why it was rejected.
        reason: String,
    },

    /// The mapping itself failed (μ error).
    TransformError {
        /// Which processor failed.
        processor: String,
        /// What went wrong.
        reason: String,
    },

    /// A pipeline stage failed, with stage index.
    PipelineError {
        /// Zero-based index of the failing stage.
        stage: usize,
        /// The underlying error.
        source: Box<ProcessorError>,
    },
}

impl fmt::Display for ProcessorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BoundaryRejection { boundary, reason } => {
                write!(f, "boundary '{boundary}' rejected: {reason}")
            }
            Self::TransformError { processor, reason } => {
                write!(f, "processor '{processor}' failed: {reason}")
            }
            Self::PipelineError { stage, source } => {
                write!(f, "pipeline stage {stage} failed: {source}")
            }
        }
    }
}

impl std::error::Error for ProcessorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PipelineError { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}
