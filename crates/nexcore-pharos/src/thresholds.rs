//! Signal detection thresholds for PHAROS filtering.
//!
//! Primitive composition: ∂(Boundary) + κ(Comparison) + N(Quantity)

use serde::{Deserialize, Serialize};

/// Configurable thresholds for signal detection gating.
///
/// A drug-event pair must exceed ALL enabled thresholds to be classified
/// as a PHAROS signal worthy of Guardian injection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalThresholds {
    /// Minimum PRR point estimate.
    pub min_prr: f64,

    /// Minimum ROR lower 95% CI (must be > 1.0 for statistical significance).
    pub min_ror_lower_ci: f64,

    /// Minimum IC025 (lower bound of Information Component).
    pub min_ic025: f64,

    /// Minimum EB05 (lower bound of Empirical Bayes).
    pub min_eb05: f64,

    /// Minimum case count for a signal to be actionable.
    pub min_cases: u64,

    /// Minimum number of algorithms that must flag the pair.
    pub min_algorithms_flagged: u32,
}

impl Default for SignalThresholds {
    fn default() -> Self {
        Self {
            min_prr: 2.0,
            min_ror_lower_ci: 1.0,
            min_ic025: 0.0,
            min_eb05: 2.0,
            min_cases: 3,
            min_algorithms_flagged: 2,
        }
    }
}

impl SignalThresholds {
    /// Strict mode — higher thresholds, fewer false positives.
    pub fn strict() -> Self {
        Self {
            min_prr: 3.0,
            min_ror_lower_ci: 2.0,
            min_ic025: 1.0,
            min_eb05: 3.0,
            min_cases: 5,
            min_algorithms_flagged: 3,
        }
    }

    /// Sensitive mode — lower thresholds, catches more signals.
    pub fn sensitive() -> Self {
        Self {
            min_prr: 1.5,
            min_ror_lower_ci: 1.0,
            min_ic025: -0.5,
            min_eb05: 1.5,
            min_cases: 2,
            min_algorithms_flagged: 1,
        }
    }

    /// Check if a signal result passes all thresholds.
    pub fn passes(
        &self,
        prr: f64,
        ror_lower: f64,
        ic025: f64,
        eb05: f64,
        cases: u64,
        algorithms_flagged: u32,
    ) -> bool {
        cases >= self.min_cases
            && algorithms_flagged >= self.min_algorithms_flagged
            && prr >= self.min_prr
            && ror_lower >= self.min_ror_lower_ci
            && ic025 >= self.min_ic025
            && eb05 >= self.min_eb05
    }

    /// Compute the boundary sharpness score (∂-score) for a signal.
    ///
    /// The Rosetta encoding of signal strength: all four detection algorithms
    /// measure the same boundary (drug-event association) from different angles.
    /// PRR and ROR are frequentist ratio lenses. IC is information-theoretic.
    /// EBGM is empirical Bayes. Each contributes proportionally to how far
    /// above its threshold the signal reaches.
    ///
    /// Returns a continuous score in `[0, ∞)`:
    /// - `0.0`: at threshold (boundary barely visible — Row 3, faint ring)
    /// - `0.5`: moderate (boundary forming)
    /// - `1.0`: strong (boundary crystallized — Row 3, solid ring)
    /// - `2.0+`: exceptional (boundary blazing — Row 4, existence confirmed)
    pub fn boundary_score(
        &self,
        prr: f64,
        ror_lower: f64,
        ic025: f64,
        eb05: f64,
        cases: u64,
        algorithms_flagged: u32,
    ) -> f64 {
        // Each dimension: how far above its threshold (ratio for positive
        // thresholds, additive for IC which can be zero/negative).
        let prr_excess = if self.min_prr > 0.0 {
            ((prr / self.min_prr) - 1.0).max(0.0)
        } else {
            prr.max(0.0)
        };
        let ror_excess = if self.min_ror_lower_ci > 0.0 {
            ((ror_lower / self.min_ror_lower_ci) - 1.0).max(0.0)
        } else {
            ror_lower.max(0.0)
        };
        let ic_excess = (ic025 - self.min_ic025).max(0.0);
        let eb_excess = if self.min_eb05 > 0.0 {
            ((eb05 / self.min_eb05) - 1.0).max(0.0)
        } else {
            eb05.max(0.0)
        };

        // Algorithm agreement: fraction of measurement angles that see the boundary
        let algo_weight = (algorithms_flagged as f64) / 4.0;

        // Zero observations = no boundary to measure.
        if cases == 0 {
            return 0.0;
        }

        // Case resolution: log scale, saturates at ~100 cases.
        // More observations = sharper image of the boundary.
        let case_weight = ((cases as f64).ln() / 100_f64.ln()).min(1.0);

        // Unified ∂-score: mean dimensional excess × agreement × resolution
        let dimensional_avg = (prr_excess + ror_excess + ic_excess + eb_excess) / 4.0;
        dimensional_avg * algo_weight * (0.5 + 0.5 * case_weight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_thresholds_pass() {
        let t = SignalThresholds::default();
        assert!(t.passes(3.0, 1.5, 0.5, 2.5, 10, 3));
    }

    #[test]
    fn test_default_thresholds_fail_low_prr() {
        let t = SignalThresholds::default();
        assert!(!t.passes(1.0, 1.5, 0.5, 2.5, 10, 3));
    }

    #[test]
    fn test_default_thresholds_fail_low_cases() {
        let t = SignalThresholds::default();
        assert!(!t.passes(3.0, 1.5, 0.5, 2.5, 1, 3));
    }

    #[test]
    fn test_default_thresholds_fail_low_algorithms() {
        let t = SignalThresholds::default();
        assert!(!t.passes(3.0, 1.5, 0.5, 2.5, 10, 1));
    }

    #[test]
    fn test_strict_thresholds() {
        let t = SignalThresholds::strict();
        // Passes strict
        assert!(t.passes(4.0, 2.5, 1.5, 3.5, 10, 4));
        // Fails strict but would pass default
        assert!(!t.passes(2.5, 1.5, 0.5, 2.5, 4, 2));
    }

    #[test]
    fn test_sensitive_thresholds() {
        let t = SignalThresholds::sensitive();
        // Passes sensitive but would fail default
        assert!(t.passes(1.5, 1.0, -0.3, 1.5, 2, 1));
    }

    #[test]
    fn test_boundary_score_strong_signal() {
        let t = SignalThresholds::default();
        // Strong signal well above thresholds
        let score = t.boundary_score(5.0, 3.0, 1.5, 4.0, 50, 4);
        assert!(
            score > 1.0,
            "Strong signal should have ∂-score > 1.0, got {score}"
        );
    }

    #[test]
    fn test_boundary_score_weak_signal() {
        let t = SignalThresholds::default();
        // Signal barely above thresholds
        let score = t.boundary_score(2.1, 1.1, 0.1, 2.1, 3, 2);
        assert!(
            score < 0.25,
            "Weak signal should have ∂-score < 0.25, got {score}"
        );
    }

    #[test]
    fn test_boundary_score_critical_signal() {
        let t = SignalThresholds::default();
        // Blazing signal across all dimensions
        let score = t.boundary_score(10.0, 5.0, 3.0, 8.0, 200, 4);
        assert!(
            score > 2.0,
            "Critical signal should have ∂-score > 2.0, got {score}"
        );
    }

    #[test]
    fn test_boundary_score_zero_cases() {
        let t = SignalThresholds::default();
        let score = t.boundary_score(5.0, 3.0, 1.5, 4.0, 0, 4);
        assert!(
            score.abs() < f64::EPSILON,
            "Zero cases = no boundary observation, got {score}"
        );
    }

    #[test]
    fn test_boundary_score_monotonic_with_cases() {
        let t = SignalThresholds::default();
        let s10 = t.boundary_score(5.0, 3.0, 1.5, 4.0, 10, 4);
        let s100 = t.boundary_score(5.0, 3.0, 1.5, 4.0, 100, 4);
        assert!(
            s100 > s10,
            "More cases should sharpen boundary: {s100} > {s10}"
        );
    }
}
