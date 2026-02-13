//! # NexVigilant Core — aggregate
//!
//! T1 fold/aggregate/recursive combinators that fill the three weakest
//! primitives in the nexcore workspace: **Σ (Sum)**, **ρ (Recursion)**, **κ (Comparison)**.
//!
//! ## Modules
//!
//! - [`fold`] — Generic fold combinators (Σ): sum, product, mean, variance, min, max
//! - [`tree`] — Recursive tree traversal (ρ): tree_fold, tree_depth, tree_flatten
//! - [`ranked`] — Comparison-based ranking (κ): top_n, percentile, outlier detection
//!
//! ## Primitive Coverage
//!
//! | Primitive | Module | Manifestation |
//! |-----------|--------|---------------|
//! | Σ (Sum) | fold | `Fold` trait, `fold_all`, `SumFold` |
//! | ρ (Recursion) | tree | `tree_fold`, `tree_depth`, `tree_count` |
//! | κ (Comparison) | ranked | `Ranked` (Ord impl), `top_n`, `percentile` |
//! | σ (Sequence) | fold | Single-pass iteration |
//! | N (Quantity) | ranked | Numeric values and counts |
//! | ∝ (Proportion) | ranked | Normalization, percentiles |
//! | ∂ (Boundary) | ranked | Outlier fences, range checks |
//! | ς (State) | fold | Accumulator state |
//! | λ (Location) | tree | Node identity |
//!
//! ## Tier: T2-C (Σ + ρ + κ + σ + N)
//!
//! ## Lifecycle
//! - **begins**: Aggregation operation initiated with input data
//! - **exists**: Computation state maintained through fold/traverse
//! - **changes**: Each step transforms accumulator or traversal state
//! - **persists**: Results serializable via serde
//! - **ends**: Final aggregated value returned to caller
//!
//! ## Example
//!
//! ```
//! use nexcore_aggregate::fold::{SumFold, Fold, fold_all};
//! use nexcore_aggregate::tree::{SimpleNode, tree_fold, combine_sum, TraversalConfig};
//! use nexcore_aggregate::ranked::{rank, top_n, percentile};
//!
//! // Σ: Fold a sequence
//! let data = vec![1.0, 2.0, 3.0];
//! assert!((SumFold.fold(&data) - 6.0).abs() < f64::EPSILON);
//!
//! // ρ: Recursive tree fold
//! let tree = SimpleNode::branch("root", 1.0, vec![
//!     SimpleNode::leaf("a", 2.0),
//!     SimpleNode::leaf("b", 3.0),
//! ]);
//! let total = tree_fold(&tree, &combine_sum, &TraversalConfig::default());
//! assert!((total.unwrap_or(0.0) - 6.0).abs() < f64::EPSILON);
//!
//! // κ: Ranked comparison
//! let items = vec![("x", 10.0), ("y", 30.0), ("z", 20.0)];
//! let top = top_n(&items, 2);
//! assert_eq!(top[0].name, "y");
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(missing_docs)]

pub mod error;
pub mod fold;
pub mod grounding;
pub mod ranked;
pub mod tree;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::error::AggregateError;
    pub use crate::fold::{
        CountFold, Fold, FoldResults, MaxFold, MeanAccumulator, MeanFold, MinFold, ProductFold,
        SumFold, VarianceAccumulator, VarianceFold, fold_all,
    };
    pub use crate::ranked::{
        OutlierDirection, Ranked, bottom_n, detect_outliers, iqr, normalize, percentile, quartiles,
        rank, top_n,
    };
    pub use crate::tree::{
        SimpleNode, TraversalConfig, TreeNode, combine_max, combine_mean, combine_sum,
        combine_weighted, tree_count, tree_depth, tree_flatten, tree_fold, tree_max,
    };
}
