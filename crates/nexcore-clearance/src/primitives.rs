//! # Primitive Foundation
//!
//! Re-exports the T1 Lex Primitiva primitives most relevant to this crate.
//!
//! ## Clearance Primitive Grounding
//!
//! | Domain Concept         | T1 Primitive | Symbol | Justification |
//! |------------------------|-------------|--------|---------------|
//! | Classification level   | Comparison  | κ      | Ordered hierarchy (Public < … < Top Secret) |
//! | Access enforcement     | State       | ς      | AccessMode lifecycle per classification |
//! | Audit trail            | Irreversibility | ∝  | Policy decisions cannot be un-logged |
//! | Policy evaluation      | Causality   | →      | Level causes mode causes enforcement |
//! | Boundary crossing      | Boundary    | ∂      | Cross-classification validation |

pub use nexcore_lex_primitiva::grounding::GroundsTo;
pub use nexcore_lex_primitiva::primitiva::LexPrimitiva;
pub use nexcore_lex_primitiva::primitiva::PrimitiveComposition;
pub use nexcore_lex_primitiva::tier::Tier;
