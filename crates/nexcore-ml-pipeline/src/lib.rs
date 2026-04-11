//! # NexVigilant Core — ML Pipeline
//!
//! Autonomous machine learning pipeline for PV signal detection.
//! Builds on `nexcore-dtree` to provide a random forest ensemble
//! trained on FAERS disproportionality features.
//!
//! ## Pipeline Stages
//!
//! 1. **Feature extraction** — FAERS contingency → 12-element PV feature vector
//! 2. **Training** — Random forest (bagged decision trees with feature subsampling)
//! 3. **Evaluation** — AUC, precision, recall, F1, confusion matrix
//! 4. **Persistence** — Versioned JSON model artifacts
//! 5. **Prediction** — Score new drug-event pairs
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | contingency table → features → prediction |
//! | T1: Recursion (ρ) | decision tree traversal, forest ensemble |
//! | T1: Sequence (σ) | pipeline stage ordering |
//! | T1: State (ς) | model parameters, training metrics |
//! | T1: Persistence (π) | serialized model artifacts |
//! | T1: Quantity (N) | vote counting, bootstrap sampling, metrics |
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use nexcore_ml_pipeline::prelude::*;
//!
//! // Build dataset from raw FAERS data
//! // let result = pipeline::run(&raw_data, &labels, PipelineConfig::default());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod ensemble;
pub mod evaluate;
pub mod feature;
pub mod persist;
pub mod pipeline;
pub mod types;

/// Convenience prelude for common imports.
pub mod prelude {
    pub use crate::ensemble::RandomForest;
    pub use crate::evaluate::{compare_baseline, compute_auc, compute_metrics};
    pub use crate::feature::{extract_features, feature_names};
    pub use crate::persist::ModelArtifact;
    pub use crate::pipeline::{self, PipelineConfig};
    pub use crate::types::{
        ContingencyTable, Dataset, FEATURE_NAMES, ForestConfig, Metrics, OutcomeBreakdown,
        PipelineResult, Prediction, RawPairData, ReporterBreakdown, Sample, TemporalData,
    };
}
