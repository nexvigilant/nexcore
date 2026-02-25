//! # NexVigilant Core — grammar-lab: Grammar Algorithms for State Functions
//!
//! Experimental laboratory proving that grammar IS math by climbing
//! the Chomsky hierarchy one T1 primitive at a time.
//!
//! ## The Chomsky ↔ Lex Primitiva Correspondence
//!
//! | Experiment | Chomsky Level | Primitives | STEM Traits | Capability Gained |
//! |-----------|---------------|------------|-------------|-------------------|
//! | `exp1_regular` | Type-3 (Regular) | σ + Σ | Transit, Classify, Superpose | Flat sequence recognition |
//! | `exp2_context_free` | Type-2 (CF) | + ρ | Infer, Associate | Nested state parsing |
//! | `exp3_context_sensitive` | Type-1 (CS) | + κ | Membership, Bound, Preserve | Context-dependent guards |
//! | `exp4_unrestricted` | Type-0 (Unrestricted) | + ∃ | Normalize, Experiment, Prove | Full state interpretation |
//!
//! Each experiment adds exactly ONE primitive, gaining exactly ONE Chomsky level.
//!
//! ## Key Insight
//!
//! Adding one T1 primitive to a grammar is equivalent to:
//! - One level in the Chomsky hierarchy
//! - One automaton upgrade (FA → PDA → LBA → TM)
//! - One qualitative capability jump
//!
//! The 16 Lex Primitiva symbols are a grammar over computation itself.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

pub mod exp1_regular;
pub mod exp2_context_free;
pub mod exp3_context_sensitive;
pub mod exp4_unrestricted;
pub mod grounding;
