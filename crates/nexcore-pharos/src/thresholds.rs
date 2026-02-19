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
}
