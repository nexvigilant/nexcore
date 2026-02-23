//! Error types for the cognitive engine.
//!
//! # Meta-cognitive observation
//!
//! Error handling IS part of cognition. When I (Claude) encounter malformed input,
//! I don't crash — I produce a structured explanation of what went wrong and why.
//! This module captures that pattern: every failure mode is typed, every error
//! carries enough context to diagnose the root cause.
//!
//! # T1 Primitive grounding
//!
//! - `∂` (Boundary): errors arise at violated boundaries
//! - `κ` (Comparison): shape mismatches are comparison failures
//! - `→` (Causality): error chains preserve causal history via `source`

use nexcore_error::Error;

/// All cognitive engine errors.
#[derive(Debug, Error)]
pub enum CognitionError {
    /// Tensor shape mismatch — two tensors can't interact.
    /// Grounded in `∂` (boundary violation) + `κ` (comparison failure).
    #[error("shape mismatch: expected {expected:?}, got {got:?} in {operation}")]
    ShapeMismatch {
        expected: Vec<usize>,
        got: Vec<usize>,
        operation: &'static str,
    },

    /// Dimension out of range for a tensor operation.
    #[error("dimension {dim} out of range for tensor with {ndim} dimensions in {operation}")]
    DimensionOutOfRange {
        dim: usize,
        ndim: usize,
        operation: &'static str,
    },

    /// Empty tensor where content was required.
    /// Grounded in `∅` (void) — absence where existence was expected.
    #[error("empty tensor in {operation}: {reason}")]
    EmptyTensor {
        operation: &'static str,
        reason: &'static str,
    },

    /// Numerical instability detected (NaN, Inf).
    /// Grounded in `∂` (boundary) — values escaped the valid range.
    #[error("numerical instability in {operation}: {detail}")]
    NumericalInstability {
        operation: &'static str,
        detail: String,
    },

    /// Configuration error — invalid hyperparameters.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// Generation halted — max tokens reached or stop condition met.
    #[error("generation halted: {reason}")]
    GenerationHalted { reason: String },

    /// Vocabulary error — token ID out of range.
    #[error("token id {id} out of vocabulary range [0, {vocab_size})")]
    TokenOutOfRange { id: usize, vocab_size: usize },
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, CognitionError>;
