#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![warn(missing_docs)]

//! # Anti-Vector Computation
//!
//! Structured countermeasures that annihilate harm vectors.
//!
//! An anti-vector is not mere negation (-v). It is an object that, when combined
//! with a harm vector, **annihilates** it — like matter meeting antimatter.
//! Both disappear, and energy (knowledge) is released.
//!
//! ## Three Anti-Vector Types
//!
//! | Type | What It Does | Annihilates |
//! |------|-------------|-------------|
//! | **Mechanistic** | Identifies the minimal intervention that breaks the harm pathway | The causal chain |
//! | **Epistemic** | Constructs the evidence packet that cancels a false signal | The noise |
//! | **Architectural** | Engineers the risk minimization measure that increases d(s) | The exposure |
//!
//! ## Primitive Grounding
//!
//! If a harm vector is `→(cause) × N(magnitude)`, an anti-vector is
//! `→⁻¹(counter-cause) × N(counter-magnitude)` bounded by the same `∂`.
//!
//! The conservation law still holds: `∃ = ∂(×(ς, ∅))`. But now `∃` is
//! protective existence rather than harmful existence.
//!
//! ## Annihilation Semantics
//!
//! When harm vector H meets anti-vector A:
//! - `|H| > |A|` → residual harm = H - A (partial protection)
//! - `|H| = |A|` → annihilation (complete neutralization, knowledge released)
//! - `|H| < |A|` → surplus protection (safety margin increased)
//!
//! The knowledge released on annihilation is the **mechanism understanding**:
//! we now know the pathway well enough to cancel it.

pub mod annihilate;
pub mod classify;
pub mod compute;
pub mod types;

pub use annihilate::*;
pub use classify::*;
pub use compute::*;
pub use types::*;
