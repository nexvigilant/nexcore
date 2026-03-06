// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # nexcore-retrocasting — Retrocasting Engine
//!
//! Links FAERS post-market safety signals to molecular structures,
//! enabling retrospective identification of structural alerts that
//! could have predicted adverse drug reactions pre-market.
//!
//! ## Pipeline
//!
//! ```text
//! FAERS Signals
//!   ↓ linker::link_signals_to_structures
//! StructuredSignal (signal + CompoundRecord + SMILES)
//!   ↓ cluster::cluster_by_similarity
//! StructuralCluster (groups of structurally similar drugs)
//!   ↓ correlate::correlate_alerts
//! AlertCandidate (fragment → adverse event associations)
//!   ↓ training::generate_training_data
//! TrainingDataset (labeled ML training records)
//! ```
//!
//! ## Feature Flags
//!
//! - `fingerprints`: Enable Morgan/ECFP4 Tanimoto similarity in clustering.
//!   Requires `nexcore-molcore` with Task 7 (fingerprint module) complete.
//!   Without this flag, SMILES character bigram Jaccard similarity is used.
//!
//! ## Primitive Grounding
//!
//! | Module | Tier | Primitives | Meaning |
//! |--------|------|-----------|---------|
//! | `linker` | T3 | σ + → + μ + ∃ | Sequence of causal mappings to existing structures |
//! | `cluster` | T2-C | Σ + κ + ∂ | Sum of comparisons within a similarity boundary |
//! | `correlate` | T3 | ν + κ + → + ∂ | Frequency comparisons causally mapped to boundaries |
//! | `training` | T3 | Σ + π + σ + → | Sum of persisted sequences mapped to labels |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod cluster;
pub mod correlate;
pub mod error;
pub mod linker;
pub mod training;
pub mod types;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use error::{RetrocastError, RetrocastResult};
pub use types::{
    AlertCandidate, RetrocastResult as RetrocastAnalysis, SignalRecord, StructuralCluster,
    StructuredSignal, TrainingDataset, TrainingRecord,
};
