#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

//! # `NexVigilant` Core — Stoichiometry
//!
//! Stoichiometric engine for encoding domain concepts as balanced primitive
//! equations using the 15 operational Lex Primitiva as the periodic table.
//!
//! ## Overview
//!
//! - **Encode**: concept + definition -> balanced equation (mass-conserving)
//! - **Decode**: balanced equation -> "What is X?" (Jeopardy-style)
//! - **Balance**: verify/enforce primitive conservation across equation sides
//! - **Dictionary**: built-in PV/regulatory terms + extensible registry
//! - **Sisters**: detect concepts with equivalent primitive decomposition
//!
//! ## Chemistry Analogy
//!
//! | Chemistry | Stoichiometry Crate |
//! |-----------|---------------------|
//! | Periodic table (118 elements) | 15 operational Lex Primitiva |
//! | Chemical equation | `BalancedEquation` |
//! | Reactants | Definition words (each a `MolecularFormula`) |
//! | Products | Concept (`ConceptFormula`) |
//! | Mass conservation | `BalanceProof` (primitive counts match) |
//! | Isomers | Sister concepts (same primitives, different arrangement) |

pub mod balance;
pub mod codec;
pub mod decomposer;
pub mod dictionary;
pub mod equation;
pub mod error;
pub mod inventory;
pub mod jeopardy;
pub mod mass_state;
pub mod prelude;
pub mod seed;
pub mod sister;

// Re-export foundation types for single-import convenience
pub use nexcore_lex_primitiva::molecular_weight::{AtomicMass, MolecularFormula, MolecularWeight};
pub use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
pub use nexcore_lex_primitiva::state_mode::StateMode;
pub use nexcore_lex_primitiva::tier::Tier;
pub use nexcore_primitives::{Confidence, Measured};
