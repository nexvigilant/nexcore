//! # NexVigilant Core — domain-primitives
//!
//! Domain primitive extraction: tier taxonomy (T1/T2-P/T2-C/T3),
//! dependency graphs, and cross-domain transfer confidence.
//!
//! ## Tier System
//!
//! | Tier | Meaning | Avg Transfer |
//! |------|---------|--------------|
//! | **T1** | Universal | 0.87–0.99 |
//! | **T2-P** | Cross-domain primitive | 0.61–0.89 |
//! | **T2-C** | Cross-domain composite | 0.61–0.82 |
//! | **T3** | Domain-specific | N/A |
//!
//! ## Transfer Formula
//!
//! ```text
//! confidence = structural × 0.4 + functional × 0.4 + contextual × 0.2
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_domain_primitives::golden_dome::golden_dome;
//! use nexcore_domain_primitives::taxonomy::Tier;
//!
//! let tax = golden_dome();
//! assert_eq!(tax.by_tier(Tier::T1).len(), 8);
//! assert_eq!(tax.irreducible_atoms().len(), 14);
//!
//! // Decompose T3 → T1 foundations
//! if let Some(tree) = tax.decompose("boost-phase-intercept") {
//!     for leaf in tree.leaves() {
//!         println!("Foundation: {leaf}");
//!     }
//! }
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::disallowed_types,
    reason = "Taxonomy representation uses standard collections for graph and registry structures"
)]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::indexing_slicing,
    reason = "Taxonomy domain involves frequent metric calculations, transfer scoring, and indexing"
)]
#![allow(
    clippy::iter_over_hash_type,
    reason = "Deterministic iteration not required for most aggregate metrics"
)]

pub mod analysis;
pub mod compare;
pub mod cybersecurity;
mod display;
pub mod golden_dome;
pub mod grounding;
pub mod pharmacovigilance;
pub mod registry;
pub mod taxonomy;
pub mod transfer;
pub mod transfer_matrix;
pub mod validation;

pub use analysis::{Bottleneck, bottlenecks, critical_paths, topological_sort};
pub use compare::{TaxonomyComparison, compare};
pub use registry::TaxonomyRegistry;
pub use taxonomy::{DecompositionNode, DomainTaxonomy, Primitive, Tier};
pub use transfer::{DomainTransfer, TransferScore};
pub use transfer_matrix::{Bridge, MatrixCell, TransferMatrix};
