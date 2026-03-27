//! # Crystalbook Numeral System (CNS)
//!
//! Base-9 numeral system derived from the 8 Laws of System Homeostasis + ∅ (Void).
//! Founded 2026-03-26 by Matthew A. Campion, PharmD.
//!
//! A CNS number is simultaneously:
//! 1. **Positional** — standard base-9 arithmetic
//! 2. **Qualitative** — each position scores a law's health
//! 3. **Compositional** — read left-to-right as an operator sequence
//!
//! ## Conservation Law
//!
//! ∃ = ∂(×(ς, ∅))
//!
//! The conservation law IS the number system:
//! - ∃ (Existence) = the number itself
//! - ∂ (Boundary) = the positional structure
//! - ς (State) = the digit values
//! - ∅ (Void) = the zeros
//! - × (Product) = multiplication

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

mod digit;
mod health;
mod number;

pub use digit::{CnsDigit, Polarity, conjugate_pair, is_conjugate};
pub use health::{ConjugateStatus, HealthReport, HealthVector};
pub use number::CnsNumber;

/// The Pleroma — all laws fully satisfied. 8×9⁷ + 8×9⁶ + ... + 8×9⁰ = 43,046,720.
pub const PLEROMA: u64 = 43_046_720;

/// The Staircase — laws in ascending order. 1×9⁰ + 2×9¹ + ... + 8×9⁷ = 42,374,116.
pub const STAIRCASE: u64 = 42_374_116;

/// The Mirror — laws in descending order. 8×9⁰ + 7×9¹ + ... + 1×9⁷ = 6,053,444.
pub const MIRROR: u64 = 6_053_444;

/// The Gap — Pleroma minus Staircase. Digital root VII (Active Maintenance). = 672,604.
pub const GAP: u64 = 672_604;

/// Base of the CNS number system.
pub const BASE: u64 = 9;
