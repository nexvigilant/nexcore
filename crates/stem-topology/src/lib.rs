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
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod betti;
pub mod diagram;
pub mod filtration;
pub mod persistence;
pub mod simplex;

pub use betti::{betti_numbers, BettiNumbers};
pub use diagram::{PersistenceDiagram, PersistencePoint};
pub use filtration::{vietoris_rips, DistanceMatrix};
pub use persistence::compute_persistence;
pub use simplex::{Simplex, SimplicialComplex};
