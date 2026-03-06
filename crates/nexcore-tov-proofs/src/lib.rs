//! # Logic Proofs Library
//!
//! A Rust implementation of the Curry-Howard correspondence for automated
//! theorem verification through the type system.
//!
//! ## Overview
//!
//! This crate allows you to express logical propositions as types and proofs
//! as programs. If a Rust function type-checks, the corresponding logical
//! theorem is proven valid (within intuitionistic logic).
//!
//! ## The Correspondence
//!
//! | Logic | Rust |
//! |-------|------|
//! | Proposition | Type |
//! | Proof | Program |
//! | True (⊤) | `()` |
//! | False (⊥) | `Void` |
//! | Conjunction (P ∧ Q) | `And<P, Q>` |
//! | Disjunction (P ∨ Q) | `Or<P, Q>` |
//! | Implication (P → Q) | `fn(P) -> Q` |
//! | Negation (¬P) | `fn(P) -> Void` |
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_tov_proofs::prelude::*;
//!
//! // Define atomic propositions as types
//! struct Raining;
//! struct HaveUmbrella;
//! struct StayDry;
//!
//! // State a theorem as a function signature
//! // This claims: (Raining ∧ HaveUmbrella) → StayDry
//! fn umbrella_theorem(
//!     premise: And<Raining, HaveUmbrella>
//! ) -> StayDry {
//!     // The proof: by our domain axiom, umbrella keeps you dry
//!     StayDry
//! }
//!
//! // The fact that this compiles proves the theorem!
//! ```
//!
//! ## Important: Intuitionistic Logic
//!
//! Rust's type system embodies intuitionistic (constructive) logic, not classical logic.
//! This means some classically valid principles cannot be proven:
//!
//! - **Law of Excluded Middle (LEM)**: P ∨ ¬P — Not provable
//! - **Double Negation Elimination**: ¬¬P → P — Not provable
//! - **Peirce's Law**: ((P → Q) → P) → P — Not provable
//!
//! See the `logic_prelude` module documentation for details.
//!
//! ## Proof Validity Guarantees
//!
//! A proof is valid if the function:
//! - Compiles successfully
//! - Contains no `panic!()`, `todo!()`, `unreachable!()`
//! - Contains no `unsafe` blocks
//! - Terminates (no infinite loops)
//!
//! ## Module Structure
//!
//! - [`logic_prelude`]: Core types (`And`, `Or`, `Void`, etc.)
//! - [`inference_rules`]: Standard inference rules as functions
//! - [`proof_patterns`]: Common proof strategy templates
//! - [`proofs`]: Your custom proofs

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
// Deny unsafe code and panic-inducing constructs to maintain proof soundness
#![deny(unsafe_code)]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod attenuation;
pub mod codex_compliance;
pub mod grounding;
pub mod inference_rules;
pub mod logic_prelude;
pub mod proof_patterns;
pub mod proofs;
pub mod type_level;

#[cfg(feature = "kani")]
pub mod kani_proofs;

/// Prelude module - import this to get all commonly used types and functions.
///
/// # Usage
///
/// ```rust
/// use nexcore_tov_proofs::prelude::*;
/// ```
pub mod prelude {
    pub use crate::codex_compliance::{
        CompareAbsent, Confident, ConfidentProof, GroundsTo, HasTier, IsPrimitive, Tier, Versioned,
    };
    pub use crate::inference_rules::*;
    pub use crate::logic_prelude::*;
    pub use crate::proof_patterns::*;
}

// Re-export commonly used items at crate root
pub use logic_prelude::{And, Exists, Not, Or, Proof, Truth, Void};
