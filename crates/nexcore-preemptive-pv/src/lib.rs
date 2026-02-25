//! # Preemptive Pharmacovigilance (`nexcore-preemptive-pv`)
//!
//! Three-tier signal detection system implementing the Preemptive PV Equation:
//!
//! ```text
//! Psi(d, e, t) = DeltaG(d,e) * Gamma(d,e,t) * Omega(e) * [1 - eta(d,t)]
//! ```
//!
//! ## The Three Tiers
//!
//! | Tier | Name | Equation | Question |
//! |------|------|----------|----------|
//! | 1 | Reactive | `S = N/E` | Did harm occur? |
//! | 2 | Predictive | `Psi = DeltaG * Gamma * (1-eta)` | Will harm occur? |
//! | 3 | Preemptive | `Pi = Psi * Omega - C(I)` | Can I prevent irreversible harm? |
//!
//! ## Modules
//!
//! - [`types`] -- Core domain types (Seriousness, DrugEventPair, Decision, etc.)
//! - [`gibbs`] -- Signal emergence feasibility (DeltaG, thermodynamic analogy)
//! - [`trajectory`] -- Temporal trajectory with Hill amplification (Gamma)
//! - [`severity`] -- Irreversibility-weighted severity (Omega)
//! - [`noise`] -- Noise floor correction (eta, Nernst-inspired sigmoid)
//! - [`reactive`] -- Tier 1 reactive signal detection (S = N/E)
//! - [`predictive`] -- Tier 2 predictive signal (Psi)
//! - [`preemptive`] -- Tier 3 preemptive decision engine (Pi)
//! - [`intervention`] -- Competitive inhibition intervention model
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_preemptive_pv::preemptive;
//! use nexcore_preemptive_pv::types::*;
//!
//! let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
//! let data = vec![
//!     ReportingDataPoint::new(1.0, 2.0),
//!     ReportingDataPoint::new(2.0, 5.0),
//!     ReportingDataPoint::new(3.0, 11.0),
//! ];
//! let noise = NoiseParams::new(25.0, 50.0);
//!
//! let result = preemptive::evaluate_default(&gibbs, &data, &noise, Seriousness::Fatal);
//! assert!(result.decision.requires_intervention());
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    reason = "Preemptive PV domain equations use closed-domain data structures and bounded numeric derivations"
)]

pub mod types;

pub mod gibbs;
pub mod grounding;
pub mod noise;
pub mod severity;
pub mod trajectory;

pub mod intervention;
pub mod predictive;
pub mod preemptive;
pub mod reactive;

pub mod composites;
pub mod prelude;
pub mod primitives;
pub mod transfer;

// Re-export key types at crate root for convenience.
pub use types::{
    Decision, DrugEventPair, GibbsParams, InterventionResult, NoiseParams, ReportingCounts,
    ReportingDataPoint, SafetyLambda, Seriousness,
};
