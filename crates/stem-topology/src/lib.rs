//! stem-topology — Topological Data Analysis (TDA) for pharmacovigilance.
//!
//! Detects stable vs ephemeral PV signals across temporal snapshots using
//! persistent homology. High-persistence Betti-0 features = stable standalone
//! signals. High-persistence Betti-1 features = stable drug-class effects.
//!
//! ## T1 Grounding
//! - Simplex → N (quantity)
//! - Filtration → σ (sequence)
//! - Persistence → π (persistence) + ∝ (irreversibility of stable features)
//! - Betti → Σ (sum/count)

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod betti;
pub mod diagram;
pub mod filtration;
pub mod grounding;
pub mod persistence;
pub mod simplex;

pub use betti::{BettiNumbers, betti_numbers};
pub use diagram::{PersistenceDiagram, PersistencePoint};
pub use filtration::{DistanceMatrix, vietoris_rips};
pub use persistence::compute_persistence;
pub use simplex::{Simplex, SimplicialComplex};
