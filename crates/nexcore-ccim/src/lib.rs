//! Capability Compound Interest Machine (CCIM)
//!
//! Sovereign development acceleration modeled as financial instrument.
//!
//! Core equation:
//! ```text
//! C(d) = C₀(1 + ρ)^d + T × [((1 + ρ)^d - 1) / ρ] - W(d)
//! ```
//!
//! Where:
//! - C(d) = Total capability at directive d
//! - C₀ = Starting capability base
//! - ρ = Reinvestment rate (compounding ratio)
//! - T = New tools shipped per directive
//! - W(d) = Cumulative depreciation
//!
//! Grounding: ρ(Recursion) + N(Quantity) + →(Causality) + κ(Comparison) + π(Persistence).

#![forbid(unsafe_code)]
#![cfg_attr(
    not(any(test, clippy)),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]

pub mod assess;
pub mod depreciation;
pub mod error;
pub mod project;
pub mod state;
pub mod types;

// Re-export primary types for convenience.
pub use assess::{CcimAssessment, FireProgress, NcrrResult};
pub use depreciation::{compute_delta_avg, compute_withdrawal, depreciation_alerts};
pub use error::CcimError;
pub use project::{FireProjection, TrajectoryPoint, ccim_equation, rule_of_72, trajectory_project};
pub use state::{CcimState, load_state, save_state};
pub use types::{CompoundingRatio, DepreciationCategory, DepreciationEntry};
