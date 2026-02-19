//! # NexVigilant Core — measure
//!
//! Mathematically rigorous crate and workspace quality measurement.
//!
//! ## Domains
//!
//! - **Information Theory**: Shannon entropy, mutual information, NCD, redundancy
//! - **Graph Theory**: DAG analysis, Tarjan SCC, Brandes betweenness centrality
//! - **Statistics**: Poisson CI, Bayesian posterior, Welch t-test, OLS regression
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | counts → entropy, features → scores |
//! | T1: State (ς) | measurement snapshot at point in time |
//! | T1: Sequence (σ) | time-series of measurements for trend tracking |
//! | T1: Comparison (κ) | drift detection, health thresholds |
//! | T1: Boundary (δ) | normalized scores clamped to [0,1] |
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_measure::prelude::*;
//!
//! // Shannon entropy of module sizes
//! let counts = vec![100, 200, 50, 300];
//! let h = entropy::shannon_entropy(&counts).unwrap();
//! assert!(h.value() > 0.0);
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod bridge;
pub mod collect;
pub mod composite;
pub mod entropy;
pub mod error;
pub mod graph;
pub mod grounding;
pub mod history;
pub mod skill;
pub mod skill_graph;
pub mod stats;
pub mod types;

/// Convenience prelude for common imports.
pub mod prelude {
    pub use crate::bridge;
    pub use crate::collect;
    pub use crate::composite;
    pub use crate::entropy;
    pub use crate::error::{MeasureError, MeasureResult};
    pub use crate::graph::{self, DepGraph};
    pub use crate::history;
    pub use crate::skill;
    pub use crate::skill_graph;
    pub use crate::stats;
    pub use crate::types::*;
}
