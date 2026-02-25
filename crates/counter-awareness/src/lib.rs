#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

//! # Counter-Awareness: Detection/Counter-Detection Formalization
//!
//! A primitive-grounded framework for modeling multi-spectral detection
//! and counter-detection systems.
//!
//! ## Architecture
//!
//! ```text
//! primitives   → matrix    → detection → fusion → device
//! (T1 atoms)   (8×8 map)   (single)    (multi)  (stateful)
//! ```
//!
//! ## Core Equation
//!
//! ```text
//! P_detect(sensor, target, range) =
//!     clamp(S_raw × ∏(1 - M[p][c]) / noise_floor, 0, 1) × exp(-α × r/r_max)
//!
//! P_fused = 1 - ∏(1 - P_detect_i)
//! ```
//!
//! ## Lex Primitiva Grounding
//! Crate-level: {κ, μ, N, Σ, ς, π, ∂}
//! - κ (Comparison): threshold-based detection
//! - μ (Mapping): effectiveness matrix
//! - N (Quantity): probabilities, measurements
//! - Σ (Sum): sensor fusion, state enumeration
//! - ς (State): device state machine
//! - π (Persistence): measurement log
//! - ∂ (Boundary): counter-awareness = boundary enforcement

pub mod detection;
pub mod device;
pub mod fusion;
pub mod matrix;
pub mod primitives;
