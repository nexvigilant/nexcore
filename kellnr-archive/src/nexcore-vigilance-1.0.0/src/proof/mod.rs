//! # Formal Proof Infrastructure (Curry-Howard Correspondence)
//!
//! This module provides type-level encoding of Theory of Vigilance propositions
//! and proofs using the Curry-Howard correspondence. If code compiles, the
//! theorem is proven (within intuitionistic logic).
//!
//! ## The Correspondence
//!
//! | Logic | Rust |
//! |-------|------|
//! | Proposition | Type |
//! | Proof | Program/Value |
//! | True (⊤) | `()` |
//! | False (⊥) | `Void` |
//! | Conjunction (P ∧ Q) | `And<P, Q>` |
//! | Disjunction (P ∨ Q) | `Or<P, Q>` |
//! | Implication (P → Q) | `fn(P) -> Q` |
//! | Negation (¬P) | `fn(P) -> Void` |
//! | Universal (∀x.P(x)) | Generic function |
//! | Existential (∃x.P(x)) | `Exists<Witness, Property>` |
//!
//! ## Modules
//!
//! - [`logic`]: Core logical types and inference rules
//! - [`type_level`]: Compile-time ToV constraint validation
//! - [`attenuation`]: Attenuation Theorem (T10.2) implementation
//! - [`axioms`]: Full ToV axiom encodings
//!
//! ## Proof Validity Guarantees
//!
//! A proof is valid if the function:
//! - Compiles successfully
//! - Contains no `panic!()`, `todo!()`, `unreachable!()`
//! - Contains no `unsafe` blocks
//! - Terminates (no infinite loops)
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::proof::prelude::*;
//!
//! // Define atomic propositions
//! struct SystemSafe;
//! struct ConstraintsSatisfied;
//!
//! // Prove: ConstraintsSatisfied → SystemSafe (by ToV Axiom 3)
//! fn safety_from_constraints(_: ConstraintsSatisfied) -> SystemSafe {
//!     SystemSafe // Axiom: constraint satisfaction implies safety
//! }
//!
//! // The fact that this compiles proves the theorem!
//! ```

pub mod attenuation;
pub mod axioms;
pub mod logic;
pub mod type_level;

/// Prelude - import for common proof types and functions.
pub mod prelude {
    pub use super::logic::*;
    pub use super::type_level::*;
}

// Re-export commonly used items at module root
pub use logic::{And, Exists, Not, Or, Proof, Truth, Void};
pub use type_level::{
    BoundedProbability, NonRecurrenceThreshold, ValidatedDomainIndex, ValidatedHarmTypeIndex,
    ValidatedLawIndex, ValidatedLevel,
};
