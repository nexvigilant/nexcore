//! # NexVigilant Core — Theory of Vigilance (ToV)
//!
//! Unified crate combining:
//! - **grounded**: Runtime primitives, types, and traits (S = U x R x T)
//! - **proofs**: Curry-Howard proof verification (theorems as types, proofs as programs)

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::allow_attributes_without_reason,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    clippy::disallowed_types,
    clippy::indexing_slicing,
    clippy::empty_line_after_doc_comments,
    clippy::unused_unit,
    clippy::semicolon_if_nothing_returned,
    clippy::redundant_clone,
    clippy::new_without_default,
    reason = "ToV proof corpus is intentionally explicit and pedagogical, with many theorem witness and doc-rich proof artifacts"
)]

pub mod grounded;
pub mod grounding;
pub mod proofs;

// Re-export commonly used grounded types at crate root
pub use grounded::{
    Actuator, Bits, ComplexityChi, HarmType, Measured, MetaVigilance, QuantityUnit, RecognitionR,
    ResponseGovernor, SafetyAction, SafetyMarginD, SignalStrengthS, StabilityShell, TemporalT,
    UniquenessU, UnitId, VigilanceError, VigilanceSystem,
};

// Re-export proof types
pub use proofs::logic_prelude::{And, Exists, Not, Or, Proof, Truth, Void};
