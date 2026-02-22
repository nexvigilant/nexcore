//! # PV Signal Gate — Energy-Gated Signal Review Depth
//!
//! Maps metabolic regime to pharmacovigilance analysis depth.
//! Crisis mode runs minimal screening; Anabolic mode runs full panel + causality.
//!
//! ## Innovation Scan 001 — Goal 5 (Score: 7.60)
//!
//! ```text
//! Energy Charge → Regime → SignalReviewDepth → algorithm subset
//! ```
//!
//! ## ToV Alignment: V2 Hierarchy
//! Foundation (energy) properly governs Domain (PV) execution depth.
//!
//! ## Tier: T2-P (ς + κ + → + N + ∂)

use crate::Regime;
use serde::{Deserialize, Serialize};
use std::fmt;

// ─── Signal Review Depth ───────────────────────────────────────────────────

/// Signal review depth determined by energy charge.
///
/// Each tier defines which PV algorithms run, estimated token cost,
/// and whether causality assessment is permitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalReviewDepth {
    /// Crisis mode: PRR screening only (cheapest).
    /// Catches only the strongest disproportionality signals.
    Basic,
    /// Catabolic mode: PRR + ROR (minimal disproportionality pair).
    /// Sufficient for routine batch screening.
    Standard,
    /// Homeostatic mode: Full 5-algorithm panel (PRR, ROR, IC, EBGM, Chi-sq).
    /// Standard pharmacovigilance signal detection.
    Full,
    /// Anabolic mode: Full panel + causality assessment + narrative generation.
    /// Maximum depth — only when energy budget permits.
    Comprehensive,
}

impl SignalReviewDepth {
    /// Derive review depth from metabolic regime.
    #[must_use]
    pub const fn from_regime(regime: Regime) -> Self {
        match regime {
            Regime::Crisis => Self::Basic,
            Regime::Catabolic => Self::Standard,
            Regime::Homeostatic => Self::Full,
            Regime::Anabolic => Self::Comprehensive,
        }
    }

    /// Derive review depth directly from energy charge.
    #[must_use]
    pub fn from_ec(ec: f64) -> Self {
        Self::from_regime(Regime::from_ec(ec))
    }

    /// Which signal detection algorithms to run at this depth.
    #[must_use]
    pub const fn algorithms(&self) -> &'static [&'static str] {
        match self {
            Self::Basic => &["PRR"],
            Self::Standard => &["PRR", "ROR"],
            Self::Full => &["PRR", "ROR", "IC", "EBGM", "ChiSquare"],
            Self::Comprehensive => &[
                "PRR",
                "ROR",
                "IC",
                "EBGM",
                "ChiSquare",
                "Naranjo",
                "WHO-UMC",
            ],
        }
    }

    /// Number of algorithms at this depth.
    #[must_use]
    pub const fn algorithm_count(&self) -> usize {
        match self {
            Self::Basic => 1,
            Self::Standard => 2,
            Self::Full => 5,
            Self::Comprehensive => 7,
        }
    }

    /// Estimated token cost for running this depth of analysis.
    #[must_use]
    pub const fn estimated_token_cost(&self) -> u64 {
        match self {
            Self::Basic => 500,
            Self::Standard => 1_500,
            Self::Full => 5_000,
            Self::Comprehensive => 15_000,
        }
    }

    /// Whether causality assessment (Naranjo, WHO-UMC) is permitted.
    #[must_use]
    pub const fn allows_causality(&self) -> bool {
        matches!(self, Self::Comprehensive)
    }

    /// Whether narrative generation is permitted.
    #[must_use]
    pub const fn allows_narrative(&self) -> bool {
        matches!(self, Self::Comprehensive)
    }

    /// Minimum PRR threshold at this depth.
    /// Crisis mode uses a higher threshold (3.0) to reduce noise.
    #[must_use]
    pub const fn min_prr_threshold(&self) -> f64 {
        match self {
            Self::Basic => 3.0,       // Only strong signals in crisis
            Self::Standard => 2.0,    // Standard threshold
            Self::Full => 2.0,        // Standard threshold
            Self::Comprehensive => 1.5, // More sensitive when budget allows
        }
    }

    /// Human-readable label for display.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Basic => "Basic (PRR only)",
            Self::Standard => "Standard (PRR+ROR)",
            Self::Full => "Full (5-algorithm panel)",
            Self::Comprehensive => "Comprehensive (panel+causality)",
        }
    }
}

impl fmt::Display for SignalReviewDepth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{} algorithms, ~{} tokens]",
            self.label(),
            self.algorithm_count(),
            self.estimated_token_cost()
        )
    }
}

// ─── Gate Decision ─────────────────────────────────────────────────────────

/// Result of an energy gate check for PV signal analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDecision {
    /// Current energy charge.
    pub energy_charge: f64,
    /// Derived metabolic regime.
    pub regime: Regime,
    /// Selected review depth.
    pub depth: SignalReviewDepth,
    /// Whether the gate allows proceeding (false only if EC = 0).
    pub proceed: bool,
    /// Estimated token cost for the selected depth.
    pub estimated_cost: u64,
}

/// Check the energy gate before running PV signal analysis.
///
/// Returns a `GateDecision` indicating which algorithms to run
/// based on current energy charge.
///
/// # Example
///
/// ```
/// use nexcore_energy::pv_gate;
///
/// let decision = pv_gate::check(0.75);
/// assert_eq!(decision.depth, pv_gate::SignalReviewDepth::Full);
/// assert!(decision.proceed);
/// assert_eq!(decision.depth.algorithm_count(), 5);
/// ```
#[must_use]
pub fn check(energy_charge: f64) -> GateDecision {
    let regime = Regime::from_ec(energy_charge);
    let depth = SignalReviewDepth::from_regime(regime);

    GateDecision {
        energy_charge,
        regime,
        depth,
        proceed: energy_charge > 0.0,
        estimated_cost: depth.estimated_token_cost(),
    }
}

/// Check whether a specific analysis depth is affordable at the current EC.
///
/// Returns `true` if the energy charge supports at least this depth level.
#[must_use]
pub fn can_afford(energy_charge: f64, desired: SignalReviewDepth) -> bool {
    let available = SignalReviewDepth::from_ec(energy_charge);
    available.algorithm_count() >= desired.algorithm_count()
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crisis_gets_basic() {
        let decision = check(0.30);
        assert_eq!(decision.regime, Regime::Crisis);
        assert_eq!(decision.depth, SignalReviewDepth::Basic);
        assert_eq!(decision.depth.algorithm_count(), 1);
        assert!(!decision.depth.allows_causality());
        assert!(decision.proceed);
    }

    #[test]
    fn test_catabolic_gets_standard() {
        let decision = check(0.60);
        assert_eq!(decision.regime, Regime::Catabolic);
        assert_eq!(decision.depth, SignalReviewDepth::Standard);
        assert_eq!(decision.depth.algorithm_count(), 2);
    }

    #[test]
    fn test_homeostatic_gets_full() {
        let decision = check(0.75);
        assert_eq!(decision.regime, Regime::Homeostatic);
        assert_eq!(decision.depth, SignalReviewDepth::Full);
        assert_eq!(decision.depth.algorithm_count(), 5);
    }

    #[test]
    fn test_anabolic_gets_comprehensive() {
        let decision = check(0.90);
        assert_eq!(decision.regime, Regime::Anabolic);
        assert_eq!(decision.depth, SignalReviewDepth::Comprehensive);
        assert_eq!(decision.depth.algorithm_count(), 7);
        assert!(decision.depth.allows_causality());
        assert!(decision.depth.allows_narrative());
    }

    #[test]
    fn test_zero_ec_does_not_proceed() {
        let decision = check(0.0);
        assert!(!decision.proceed);
    }

    #[test]
    fn test_can_afford() {
        assert!(can_afford(0.90, SignalReviewDepth::Full));
        assert!(!can_afford(0.30, SignalReviewDepth::Full));
        assert!(can_afford(0.60, SignalReviewDepth::Standard));
    }

    #[test]
    fn test_prr_threshold_stricter_in_crisis() {
        assert!(SignalReviewDepth::Basic.min_prr_threshold() > SignalReviewDepth::Full.min_prr_threshold());
    }

    #[test]
    fn test_display() {
        let depth = SignalReviewDepth::Full;
        let display = format!("{depth}");
        assert!(display.contains("5 algorithms"));
    }
}
