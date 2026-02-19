//! # Spatial Bridge: signal → stem-math
//!
//! Expresses signal detection thresholds as `Neighborhood` containment checks
//! and defines `ContingencyMetric` for measuring divergence between contingency tables.
//!
//! ## Primitive Foundation
//!
//! Signal detection is fundamentally about spatial proximity:
//! - "Is this observed PRR inside the signal neighborhood?" = Neighborhood containment
//! - "How different are these two drug-event profiles?" = Metric distance
//!
//! The Evans thresholds (PRR >= 2.0, chi² >= 3.841, n >= 3) define closed
//! neighborhoods in a multi-dimensional signal space.

use stem_math::spatial::{Dimension, Distance, Metric, Neighborhood};

use crate::core::ContingencyTable;

// ============================================================================
// Signal threshold neighborhoods
// ============================================================================

/// Evans default threshold: PRR >= 2.0 (closed neighborhood from 2.0 outward).
///
/// A signal is detected when the observed PRR is at or beyond the boundary.
/// We model this as: Distance from zero >= 2.0.
pub fn evans_prr_neighborhood() -> Neighborhood {
    Neighborhood::closed(Distance::new(2.0))
}

/// Strict threshold: PRR >= 3.0
pub fn strict_prr_neighborhood() -> Neighborhood {
    Neighborhood::closed(Distance::new(3.0))
}

/// Sensitive threshold: PRR >= 1.5
pub fn sensitive_prr_neighborhood() -> Neighborhood {
    Neighborhood::closed(Distance::new(1.5))
}

/// Chi-square threshold neighborhoods.
pub fn evans_chi_neighborhood() -> Neighborhood {
    Neighborhood::closed(Distance::new(3.841)) // p=0.05
}

/// Strict chi-square threshold (p=0.01).
pub fn strict_chi_neighborhood() -> Neighborhood {
    Neighborhood::closed(Distance::new(6.635))
}

/// Sensitive chi-square threshold (p=0.10).
pub fn sensitive_chi_neighborhood() -> Neighborhood {
    Neighborhood::closed(Distance::new(2.706))
}

/// Check if a PRR value is a detected signal under the given threshold profile.
///
/// This replaces ad-hoc `prr >= threshold` checks with formal neighborhood containment.
pub fn prr_is_signal(prr: f64, threshold: &Neighborhood) -> bool {
    // A signal is detected when the PRR distance from zero >= threshold radius
    // For closed neighborhoods: prr >= radius
    let d = Distance::new(prr);
    // Invert containment: signal detected when PRR is OUTSIDE the "safe" neighborhood
    // OR equivalently: when PRR is at or beyond the signal boundary
    d.value() >= threshold.radius.value()
}

/// Signal space dimensionality.
///
/// The signal detection space has 5 independent axes:
/// PRR, ROR, IC, EBGM, chi-square.
pub const SIGNAL_DIMENSION: Dimension = Dimension::new(5);

/// Contingency table dimensionality.
///
/// A 2x2 contingency table has 4 cells, but one degree of freedom
/// (row/column totals constrain it), so effective dimension = 3.
pub const CONTINGENCY_DIMENSION: Dimension = Dimension::SPACE_3D;

// ============================================================================
// ContingencyMetric: Distance between two contingency tables
// ============================================================================

/// Metric over contingency tables measuring divergence in reporting rates.
///
/// Distance = |PRR(a) - PRR(b)| where PRR = (a/a+b) / (c/c+d).
/// This captures how differently two drug-event pairs report relative to background.
///
/// Tier: T2-C (N Quantity + kappa Comparison + mu Mapping + partial Boundary)
pub struct ContingencyMetric;

impl ContingencyMetric {
    /// Compute PRR from a contingency table's cells.
    ///
    /// PRR = [a/(a+b)] / [c/(c+d)]
    fn prr(table: &ContingencyTable) -> f64 {
        let a = table.a as f64;
        let b = table.b as f64;
        let c = table.c as f64;
        let d = table.d as f64;

        let num = a / (a + b).max(f64::EPSILON);
        let den = c / (c + d).max(f64::EPSILON);

        if den < f64::EPSILON {
            return 0.0;
        }
        num / den
    }
}

impl Metric for ContingencyMetric {
    type Element = ContingencyTable;

    fn distance(&self, a: &ContingencyTable, b: &ContingencyTable) -> Distance {
        let prr_a = Self::prr(a);
        let prr_b = Self::prr(b);
        Distance::new((prr_a - prr_b).abs())
    }
}

// ============================================================================
// Threshold profile as a set of neighborhoods
// ============================================================================

/// A complete signal detection threshold profile expressed as neighborhoods.
///
/// Each metric has its own closed neighborhood defining the signal boundary.
pub struct ThresholdProfile {
    /// PRR signal boundary
    pub prr: Neighborhood,
    /// Chi-square signal boundary
    pub chi_square: Neighborhood,
    /// Minimum case count boundary
    pub min_cases: Neighborhood,
}

impl ThresholdProfile {
    /// Evans (default) thresholds.
    pub fn evans() -> Self {
        Self {
            prr: evans_prr_neighborhood(),
            chi_square: evans_chi_neighborhood(),
            min_cases: Neighborhood::closed(Distance::new(3.0)),
        }
    }

    /// Strict thresholds.
    pub fn strict() -> Self {
        Self {
            prr: strict_prr_neighborhood(),
            chi_square: strict_chi_neighborhood(),
            min_cases: Neighborhood::closed(Distance::new(5.0)),
        }
    }

    /// Sensitive thresholds.
    pub fn sensitive() -> Self {
        Self {
            prr: sensitive_prr_neighborhood(),
            chi_square: sensitive_chi_neighborhood(),
            min_cases: Neighborhood::closed(Distance::new(2.0)),
        }
    }

    /// Check if a signal (prr, chi_sq, n) passes all thresholds.
    pub fn is_signal(&self, prr: f64, chi_sq: f64, n: u64) -> bool {
        prr_is_signal(prr, &self.prr)
            && prr_is_signal(chi_sq, &self.chi_square)
            && prr_is_signal(n as f64, &self.min_cases)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn table(a: u64, b: u64, c: u64, d: u64) -> ContingencyTable {
        ContingencyTable { a, b, c, d }
    }

    // ===== Neighborhood threshold tests =====

    #[test]
    fn evans_prr_boundary() {
        assert!(prr_is_signal(2.0, &evans_prr_neighborhood()));
        assert!(prr_is_signal(3.5, &evans_prr_neighborhood()));
        assert!(!prr_is_signal(1.9, &evans_prr_neighborhood()));
    }

    #[test]
    fn strict_prr_boundary() {
        assert!(prr_is_signal(3.0, &strict_prr_neighborhood()));
        assert!(!prr_is_signal(2.9, &strict_prr_neighborhood()));
    }

    #[test]
    fn sensitive_prr_boundary() {
        assert!(prr_is_signal(1.5, &sensitive_prr_neighborhood()));
        assert!(!prr_is_signal(1.4, &sensitive_prr_neighborhood()));
    }

    // ===== ContingencyMetric tests =====

    #[test]
    fn metric_non_negative() {
        let m = ContingencyMetric;
        let t1 = table(15, 100, 20, 10000);
        let t2 = table(5, 200, 10, 5000);
        assert!(m.distance(&t1, &t2).value() >= 0.0);
    }

    #[test]
    fn metric_identity() {
        let m = ContingencyMetric;
        let t = table(15, 100, 20, 10000);
        assert!(m.distance(&t, &t).approx_eq(&Distance::ZERO, f64::EPSILON));
    }

    #[test]
    fn metric_symmetry() {
        let m = ContingencyMetric;
        let t1 = table(15, 100, 20, 10000);
        let t2 = table(5, 200, 10, 5000);
        assert!(m.is_symmetric(&t1, &t2, f64::EPSILON));
    }

    // ===== ThresholdProfile tests =====

    #[test]
    fn evans_profile_detects_signal() {
        let profile = ThresholdProfile::evans();
        // Strong signal: PRR=7.5, chi²=high, n=15
        assert!(profile.is_signal(7.5, 50.0, 15));
    }

    #[test]
    fn evans_profile_rejects_weak() {
        let profile = ThresholdProfile::evans();
        // Weak signal: PRR below threshold
        assert!(!profile.is_signal(1.5, 50.0, 15));
    }

    #[test]
    fn strict_more_conservative() {
        let evans = ThresholdProfile::evans();
        let strict = ThresholdProfile::strict();
        // PRR=2.5 passes Evans but not Strict
        assert!(evans.is_signal(2.5, 5.0, 5));
        assert!(!strict.is_signal(2.5, 5.0, 5));
    }

    #[test]
    fn sensitive_more_permissive() {
        let evans = ThresholdProfile::evans();
        let sensitive = ThresholdProfile::sensitive();
        // PRR=1.6 fails Evans but passes Sensitive
        assert!(!evans.is_signal(1.6, 3.0, 3));
        assert!(sensitive.is_signal(1.6, 3.0, 3));
    }

    // ===== Dimension constants =====

    #[test]
    fn signal_dimension_is_5() {
        assert_eq!(SIGNAL_DIMENSION.rank(), 5);
    }

    #[test]
    fn contingency_dimension_is_3() {
        assert_eq!(CONTINGENCY_DIMENSION.rank(), 3);
    }
}
