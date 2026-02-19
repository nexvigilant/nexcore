//! # NexVigilant Core — Theory of Vigilance (ToV)
//!
//! Unified crate combining:
//! - **grounded**: Runtime primitives, types, and traits (S = U x R x T)
//! - **proofs**: Curry-Howard proof verification (theorems as types, proofs as programs)

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

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
