//! # Primitive Foundation
//!
//! Re-exports the T1 Lex Primitiva primitives most relevant to this crate.
//!
//! This module is only available when the `grounding` feature is enabled.
//!
//! ## STOS Primitive Grounding (Quindecet — all 15 operational T1 primitives)
//!
//! | STOS Layer | Dominant Primitive | Symbol |
//! |------------|-------------------|--------|
//! | STOS-ST (State Registry)     | State       | ς |
//! | STOS-TR (Transition Engine)  | Causality   | → |
//! | STOS-BD (Boundary Manager)   | Boundary    | ∂ |
//! | STOS-GD (Guard Evaluator)    | Comparison  | κ |
//! | STOS-CT (Count Metrics)      | Quantity    | N |
//! | STOS-SQ (Sequence Controller)| Sequence    | σ |
//! | STOS-RC (Recursion Detector) | Recursion   | ρ |
//! | STOS-VD (Void Cleaner)       | Void        | ∅ |
//! | STOS-PR (Persist Store)      | Persistence | π |
//! | STOS-EX (Existence Validator)| Existence   | ∃ |
//! | STOS-AG (Aggregate Coord.)   | Sum         | Σ |
//! | STOS-TM (Temporal Scheduler) | Frequency   | ν |
//! | STOS-LC (Location Router)    | Location    | λ |
//! | STOS-IR (Irreversibility Auditor) | Irreversibility | ∝ |
//! | STOS-MP (Mapping Transformer)| Mapping     | μ |

pub use nexcore_lex_primitiva::grounding::GroundsTo;
pub use nexcore_lex_primitiva::primitiva::LexPrimitiva;
pub use nexcore_lex_primitiva::primitiva::PrimitiveComposition;
pub use nexcore_lex_primitiva::tier::Tier;
