//! # GROUNDED
//!
//! **G**iving **R**easoning **O**bservable, **U**nified, **N**ested,
//! **D**evelopmental, **E**vidence-based **D**ynamics
//!
//! A foundational library for AI-first development that introduces feedback loops
//! between reasoning and reality.
//!
//! ## Core Components
//!
//! - [`Uncertain<T>`] — A value paired with confidence. Forces explicit handling
//!   of epistemic uncertainty at the type level.
//! - [`Confidence`] — Bounded [0.0, 1.0] measure with multiplicative composition.
//! - [`ConfidenceBand`] — Discrete bands (High/Medium/Low/Negligible) for exhaustive matching.
//! - [`GroundedLoop`] — The minimum viable primitive: hypothesis → experiment → outcome → learning → persist.
//! - [`uncertain_match!`] — Macro enforcing exhaustive confidence band handling.
//! - [`verify!`] — Macro for specification-based runtime verification.
//!
//! ## T1 Primitive Grounding
//!
//! | Component | Primitives |
//! |-----------|-----------|
//! | Uncertain | ×(Product), N(Quantity), ∂(Boundary) |
//! | Confidence | N(Quantity), ∂(Boundary) |
//! | ConfidenceBand | Σ(Sum) |
//! | GroundedLoop | ρ(Recursion), →(Causality), π(Persistence), ς(State) |
//! | Hypothesis | →(Causality), κ(Comparison) |
//! | Experiment | ∃(Existence), μ(Mapping) |
//! | Outcome | →(Causality), ∂(Boundary) |
//! | Learning | μ(Mapping), π(Persistence) |
//! | verify! | κ(Comparison), ∂(Boundary) |
//! | EvidenceChain | σ(Sequence), →(Causality), N(Quantity) |
//! | SqliteStore | π(Persistence), λ(Location) |
//! | BashWorld | →(Causality) |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

mod chain;
mod confidence;
mod error;
mod feedback;
mod macros;
pub mod skill;
mod store;
mod uncertain;
mod world;

pub use chain::{EvidenceChain, EvidenceStep};
pub use confidence::{Confidence, ConfidenceBand};
pub use error::GroundedError;
pub use feedback::{
    Context, ExperienceStore, Experiment, GroundedLoop, Hypothesis, Learning, MemoryStore, Outcome,
    Verdict, World,
};
pub use store::SqliteStore;
pub use uncertain::Uncertain;
pub use world::{BashWorld, MockWorld};
