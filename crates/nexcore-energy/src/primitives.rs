//! # Primitive Foundation
//!
//! Re-exports the T1 Lex Primitiva primitives most relevant to this crate.
//!
//! ## Energy Charge Primitive Grounding
//!
//! | Domain Concept | T1 Primitive | Symbol | Justification |
//! |----------------|-------------|--------|---------------|
//! | Token budget   | Quantity    | N      | Countable resource under conservation |
//! | EC thresholds  | Comparison  | κ      | Regime classification at boundary values |
//! | Pool state     | State       | ς      | Metabolic regime (Anabolic/Catabolic/…) |
//! | Recycling      | Causality   | →      | tADP → tATP via compression/caching |
//! | Waste ratio    | Irreversibility | ∝  | Irreversible token degradation |

pub use nexcore_lex_primitiva::grounding::GroundsTo;
pub use nexcore_lex_primitiva::primitiva::LexPrimitiva;
pub use nexcore_lex_primitiva::primitiva::PrimitiveComposition;
pub use nexcore_lex_primitiva::tier::Tier;
