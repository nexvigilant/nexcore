//! # Signal Atoms (L1)
//!
//! Pure, single-responsibility calculation atoms.
//! Each atom MUST be < 20 LOC to satisfy UACA Purity.

use crate::SafetyMargin;

/// Atom: Calculate axiomatic safety distance (L1)
pub fn calculate_safety_distance(prr: f64, ror: f64, ic: f64, eb: f64, n: u64) -> f32 {
    SafetyMargin::calculate(prr, ror, ic, eb, n as u32).distance as f32
}

/// Atom: Score epistemic trust for signal atoms (L1)
pub fn score_signal_trust(n: u64) -> f64 {
    // ToV Axiom: Trust scales with log of evidence
    (n as f64).ln_1p() / 5.0f64.ln_1p().min(1.0)
}
