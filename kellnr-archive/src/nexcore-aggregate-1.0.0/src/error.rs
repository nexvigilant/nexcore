//! Error types for nexcore-aggregate
//!
//! ## Tier: T1 (∂ + ∅)
//!
//! ## Lifecycle
//! - **begins**: Error variant constructed
//! - **exists**: Carried through Result chain
//! - **changes**: Never (errors are immutable values)
//! - **persists**: Displayed via Display impl
//! - **ends**: Consumed by caller

/// Errors that can occur during aggregation operations.
#[derive(Debug, thiserror::Error)]
pub enum AggregateError {
    /// Empty input where at least one element is required.
    #[error("empty input: {context}")]
    EmptyInput {
        /// Description of what operation needed non-empty input.
        context: String,
    },

    /// Cycle detected during recursive traversal.
    #[error("cycle detected at depth {depth}: {node}")]
    CycleDetected {
        /// The node where the cycle was detected.
        node: String,
        /// Depth in the traversal when cycle was found.
        depth: usize,
    },

    /// Maximum recursion depth exceeded.
    #[error("max recursion depth {max_depth} exceeded at node: {node}")]
    MaxDepthExceeded {
        /// The node where depth was exceeded.
        node: String,
        /// The configured maximum depth.
        max_depth: usize,
    },
}
