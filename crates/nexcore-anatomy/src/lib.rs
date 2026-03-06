//! # NexVigilant Core — anatomy
//!
//! Structural anatomy analysis for Rust workspaces — dependency graph metrics,
//! layer classification, boundary violation detection, criticality scoring,
//! and health reporting.
//!
//! ## Primitive Foundation
//! - σ (Sequence): Topological ordering, layer hierarchy
//! - μ (Mapping): Crate → layer/score/node classification
//! - κ (Comparison): Ranking, threshold-based tier assignment
//! - ∂ (Boundary): Layer boundaries, violation detection
//! - ρ (Recursion): Cycle detection in dependency graph
//! - N (Quantity): Fan-in/out counts, criticality scores, density
//! - Σ (Sum): Aggregation of metrics across workspace
//! - π (Persistence): Serializable reports
//!
//! ## Architecture (Chomsky Level: Type-1)
//!
//! The dependency graph is a DAG analyzed via Kahn's algorithm (σ + ρ).
//! Layer classification adds context-sensitivity (κ) for boundary enforcement.
//! Generators used: {σ, Σ, ρ, κ} → Type-1 (context-sensitive validation).
//!
//! ## Usage
//!
//! ```no_run
//! use nexcore_anatomy::{DependencyGraph, LayerMap, WorkspaceMetrics, AnatomyReport};
//!
//! // Build from cargo metadata
//! let metadata = cargo_metadata::MetadataCommand::new().exec().unwrap();
//! let graph = DependencyGraph::from_metadata(&metadata);
//! let report = AnatomyReport::from_graph(graph);
//!
//! println!("Health: {}", report.summary.health.label());
//! println!("Violations: {}", report.summary.violation_count);
//! println!("Bottleneck: {} (fan-in: {})", report.summary.bottleneck, report.summary.max_fan_in);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::disallowed_types,
    reason = "Anatomy analysis uses standard collections for graph representation"
)]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::indexing_slicing,
    reason = "Static analysis involves frequent metric calculations and indexing"
)]
#![allow(
    clippy::iter_over_hash_type,
    reason = "Deterministic iteration not required for most graph summary metrics"
)]

pub mod blast_radius;
pub mod chomsky;
pub mod graph;
pub mod layer;
pub mod metrics;
pub mod report;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::blast_radius::{BlastRadius, BlastRadiusReport};
    pub use crate::chomsky::{ChomskyLevel, ChomskyReport, CrateChomskyProfile, Generator};
    pub use crate::graph::{CrateNode, DependencyGraph};
    pub use crate::layer::{BoundaryViolation, Layer, LayerMap};
    pub use crate::metrics::{CriticalityScore, CriticalityTier, WorkspaceMetrics};
    pub use crate::report::{AnatomyReport, HealthStatus, ReportSummary};
}

// Re-export primary types at crate root.
pub use blast_radius::{BlastRadius, BlastRadiusReport};
pub use chomsky::{ChomskyLevel, ChomskyReport, CrateChomskyProfile, Generator};
pub use graph::{CrateNode, DependencyGraph};
pub use layer::{BoundaryViolation, Layer, LayerMap};
pub use metrics::{CriticalityScore, CriticalityTier, WorkspaceMetrics};
pub use report::{AnatomyReport, HealthStatus, ReportSummary};
