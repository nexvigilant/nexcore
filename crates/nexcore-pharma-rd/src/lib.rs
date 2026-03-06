//! # NexVigilant Core — pharma-rd
//!
//! Predictive pharmaceutical R&D taxonomy encoded as typed Rust.
//!
//! ## What This Crate Models
//!
//! The complete conceptual vocabulary of predictive pharmaceutical research
//! and development, from target identification through post-market pharmacovigilance.
//! Every concept is grounded to Lex Primitiva symbols with tier classification
//! derived from unique symbol count.
//!
//! ## Taxonomy (62 concepts)
//!
//! | Tier | Count | Examples |
//! |------|-------|---------|
//! | T1 | 16 | Lex Primitiva symbols (σ, μ, ς, ...) |
//! | T2-P | 24 | `BindingAffinity`, `Toxicity`, `Signal`, `Randomization` |
//! | T2-C | 14 | `AdmetProfile`, `ClinicalTrialDesign`, `LeadOptimization` |
//! | T3 | 8 | `IndApplication`, `NdaSubmission`, `Rems` |
//!
//! ## Chomsky Grammar Classification
//!
//! Each R&D pipeline stage maps to a Chomsky level:
//!
//! | Stage | Level | Automaton |
//! |-------|-------|-----------|
//! | Hit Finding | Type-3 | Finite Automaton |
//! | ADMET Prediction | Type-2 | Pushdown Automaton |
//! | Target ID | Type-1 | Linear Bounded |
//! | Clinical Trials (adaptive) | Type-0 | Turing Machine |
//!
//! ## Transfer Confidence
//!
//! Three-dimensional formula: `TC = structural * 0.4 + functional * 0.4 + contextual * 0.2`
//!
//! Strongest corridors: Pharma -> Biotech (clinical design), Pharma -> Agrochemical (SAR).
//!
//! ## Signature
//!
//! Two symbols define pharma R&D:
//! - **N (Quantity)**: present in 9/9 stages — fundamentally quantitative
//! - **∂ (Boundary)**: present in 7/9 stages — a field defined by thresholds

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod chomsky;
pub mod grounding;
pub mod lex;
pub mod pipeline;
pub mod taxonomy;
pub mod transfer;

// Re-exports for convenience
pub use chomsky::{ChomskyLevel, Generator, classify_generators, overengineering};
pub use lex::{LexSymbol, PrimitiveComposition, Tier};
pub use pipeline::PipelineStage;
pub use taxonomy::{
    PharmaComposite, PharmaDomainConcept, PharmaPrimitive, TaxonomySummary, taxonomy_summary,
};
pub use transfer::{
    TransferConfidence, TransferDomain, lookup_transfer, strongest_transfers, transfer_matrix,
    weakest_transfers,
};
