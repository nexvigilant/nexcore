//! # Minesweeper-PV: Constraint Satisfaction Problem Framework
//!
//! A Bayesian belief propagation framework for pharmacovigilance signal investigation
//! prioritization, based on the Minesweeper game metaphor.
//!
//! ## Safety Axioms
//!
//! This module implements the **Investigation Prioritization Axiom** from the
//! Tree of Vigilance (ToV):
//!
//! > "When multiple potential safety signals exist, investigation resources must
//! > be allocated to maximize expected information gain while maintaining
//! > conservation of evidence integrity."
//!
//! ## Conservation Laws
//!
//! The Evidence struct implements Conservation Law #4 (Information Conservation):
//! - Likelihood ratios must be computed from validated statistical measures
//! - Propagated evidence must decay to preserve epistemic boundaries
//!
//! ## Overview
//!
//! The framework models drug-event pairs as cells in a constraint satisfaction grid,
//! where evidence propagates between adjacent cells based on mechanistic, phenotypic,
//! temporal, demographic, and concomitant relationships.
//!
//! ## Key Concepts
//!
//! - **Cell**: A drug-event-population combination with associated belief state
//! - **Evidence**: Observed data that updates belief (PRR, dechallenge/rechallenge, etc.)
//! - **Adjacency**: Relationships between cells that allow evidence propagation
//! - **EIG**: Expected Information Gain for prioritizing investigations
//! - **Campion Signal Theory**: S = U × R × T (Rarity × Recognition × Temporal)
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::pv::minesweeper::{CSPGrid, Evidence, AdjacencyType};
//!
//! // Create a grid for SGLT2 inhibitor class investigation
//! let mut grid = CSPGrid::new();
//!
//! // Add drug-event cells
//! grid.add_cell("canagliflozin", "ketoacidosis", "all");
//! grid.add_cell("dapagliflozin", "ketoacidosis", "all");
//!
//! // Set mechanistic adjacency (same drug class, same event)
//! grid.set_adjacency(
//!     "canagliflozin|ketoacidosis|all",
//!     "dapagliflozin|ketoacidosis|all",
//!     0.85,
//!     Some(AdjacencyType::Mechanistic),
//! );
//!
//! // Investigate canagliflozin with strong evidence
//! let evidence = Evidence::builder()
//!     .prr(3.2)
//!     .chi2(8.5)
//!     .count(12)
//!     .temporal_pattern("strong")
//!     .mechanism("established")
//!     .build();
//!
//! let newly_flagged = grid.propagate_evidence("canagliflozin|ketoacidosis|all", &evidence);
//!
//! // Get investigation priorities by Expected Information Gain
//! let priorities = grid.get_investigation_priorities(10);
//! ```
//!
//! ## References
//!
//! - Campion Signal Theory: S = U × R × T (NexVigilant, 2025)
//! - CSP for Pharmacovigilance (NexVigilant, 2025)
//! - Tree of Vigilance (ToV) Safety Axioms (NexVigilant, 2025)

mod belief;
mod evidence;
mod grid;
mod signal;
mod types;

/// Sentinel Constraint Propagation (ToV §33) - Codex-compliant implementation
pub mod sentinel;

#[cfg(test)]
pub mod experiential_testing;

pub use belief::{BeliefState, Cell};
pub use evidence::Evidence;
pub use grid::CSPGrid;
pub use signal::{CampionSignalResult, cell_to_signal_score};
pub use types::{AdjacencyType, CellStatus, TemporalWindow};

// Re-export constants for external configuration
pub use types::{
    BASE_SIGNAL_RATE, CONFIRM_THRESHOLD, CONFIRM_UNCERTAINTY, DEFAULT_ADJACENCY_WEIGHTS,
    FLAG_THRESHOLD, MIN_EIG_FOR_ACTION, PROPAGATION_DECAY, PROPAGATION_THRESHOLD, UNFLAG_THRESHOLD,
};

// Re-export Sentinel CSP types (ToV §33)
pub use sentinel::{
    // Constants
    D_SAFE_THRESHOLD,
    // Types
    DisproportionalityScores,
    Observable,
    S_CLEAR,
    S_CONFIRM,
    S_HIGH,
    S_MODERATE,
    SafetyMargin,
    SentinelAdjacency,
    SentinelCellState,
    SentinelEdge,
    SentinelGrid,
    SentinelStatus,
    U_NON_RECURRENCE_BITS,
};

#[cfg(test)]
mod tests;
