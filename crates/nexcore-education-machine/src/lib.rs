//! # NexVigilant Core — Education Machine
//!
//! General-purpose education engine that teaches any subject via primitive decomposition.
//!
//! ## Primitive Foundation
//!
//! Education = **σμρςNκ** (T3, 6 primitives, dominant σ Sequence)
//!
//! | Primitive | Role |
//! |-----------|------|
//! | **σ** Sequence | Curriculum ordering, phase progression |
//! | **μ** Mapping | Concepts → lessons, domain → primitives |
//! | **ρ** Recursion | Spaced repetition review cycles |
//! | **ς** State | Learner state, enrollment tracking |
//! | **N** Quantity | Mastery probability, difficulty, grades |
//! | **κ** Comparison | Assessment, verdict thresholds |
//!
//! ## 5-Phase Learning FSM
//!
//! ```text
//! Discover → Extract → Practice → Assess → Master
//!    ↑          │          │          │        │
//!    └──────────┴──────────┴──────────┴────────┘  (backward to Discover)
//! ```
//!
//! - **Discover** — Survey domain, identify concepts
//! - **Extract** — Decompose to T1/T2/T3 primitives
//! - **Practice** — Recursive application with spaced repetition
//! - **Assess** — Bayesian mastery evaluation P(M|D)
//! - **Master** — Certified competency (verdict >= 0.85)
//!
//! ## Bayesian Mastery Engine
//!
//! ```text
//! P(M|D) = alpha / (alpha + beta)
//! ```
//!
//! - Correct answer at difficulty d → alpha += d
//! - Incorrect answer at difficulty d → beta += (1-d)
//! - Verdict: >= 0.85 MASTERED, >= 0.50 DEVELOPING, < 0.50 REMEDIATE

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![warn(missing_docs)]

pub mod assessment;
pub mod error;
pub mod grounding;
pub mod learner;
pub mod lesson;
pub mod prelude;
pub mod spaced_repetition;
pub mod state_machine;
pub mod subject;
pub mod types;
