//! Custom Proofs Module
//!
//! This module contains example proofs demonstrating how to use the
//! Curry-Howard correspondence for theorem verification.
//!
//! # Organization
//!
//! - `propositional`: Basic propositional logic proofs
//! - `predicate`: Predicate logic with quantifiers
//! - `examples`: Practical examples and demonstrations
//! - `vigilance`: Theory of Vigilance axiom encoding

pub mod examples;
pub mod predicate;
pub mod propositional;
pub mod vigilance;

// Re-export for convenience
pub use predicate::*;
pub use propositional::*;
