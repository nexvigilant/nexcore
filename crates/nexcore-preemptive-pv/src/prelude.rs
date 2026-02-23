//! # Prelude
//!
//! Convenient re-exports for `use nexcore_preemptive_pv::prelude::*`.
//!
//! Provides the most commonly used types from this crate along with
//! the Lex Primitiva grounding types needed for primitive analysis.

// Core types
pub use crate::types::{
    Decision, DrugEventPair, GibbsParams, InterventionResult, NoiseParams, ReportingCounts,
    ReportingDataPoint, SafetyLambda, Seriousness,
};

// Predictive tier
pub use crate::predictive::{PredictiveConfig, PredictiveResult};

// Preemptive tier
pub use crate::preemptive::{PreemptiveConfig, PreemptiveResult};

// Grounding
pub use nexcore_lex_primitiva::grounding::GroundsTo;
pub use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
pub use nexcore_lex_primitiva::tier::Tier;
