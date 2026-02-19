//! # Confidence Interval Type
//!
//! Reusable confidence interval with point estimate.
//! Replaces ad-hoc `(lower_ci, upper_ci)` field pairs across signal detection.
//!
//! # Tier: T2-C (κ + N + ∂)
//! Composed of Comparison (κ), Quantity (N), and Boundary (∂).

use serde::{Deserialize, Serialize};

/// Confidence interval with point estimate.
///
/// # Examples
///
/// ```
/// use nexcore_constants::ConfidenceInterval;
///
/// let ci = ConfidenceInterval::new(2.5, 1.2, 4.8, 0.05);
/// assert!((ci.width() - 3.6).abs() < 1e-10);
/// assert!(ci.contains(3.0));
/// assert!(!ci.contains(0.5));
/// assert!(ci.is_significant(1.0)); // lower > 1.0
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Point estimate (e.g., PRR, ROR, odds ratio).
    pub point_estimate: f64,
    /// Lower bound of the interval.
    pub lower: f64,
    /// Upper bound of the interval.
    pub upper: f64,
    /// Significance level (0.05 = 95% CI).
    pub alpha: f64,
}

impl ConfidenceInterval {
    /// Create a new confidence interval.
    #[must_use]
    pub const fn new(point_estimate: f64, lower: f64, upper: f64, alpha: f64) -> Self {
        Self {
            point_estimate,
            lower,
            upper,
            alpha,
        }
    }

    /// Width of the interval (upper - lower).
    #[must_use]
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }

    /// Whether `value` falls within [lower, upper].
    #[must_use]
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }

    /// Whether the lower bound exceeds `threshold`.
    ///
    /// For signal detection: `ci.is_significant(1.0)` means the entire
    /// interval is above the null value.
    #[must_use]
    pub fn is_significant(&self, threshold: f64) -> bool {
        self.lower > threshold
    }

    /// Confidence level as percentage (e.g., 95.0 for alpha=0.05).
    #[must_use]
    pub fn confidence_level_pct(&self) -> f64 {
        (1.0 - self.alpha) * 100.0
    }
}

impl Default for ConfidenceInterval {
    fn default() -> Self {
        Self {
            point_estimate: 0.0,
            lower: 0.0,
            upper: 0.0,
            alpha: 0.05,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_calculation() {
        let ci = ConfidenceInterval::new(5.0, 2.0, 8.0, 0.05);
        assert!((ci.width() - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn contains_within_bounds() {
        let ci = ConfidenceInterval::new(3.0, 1.0, 5.0, 0.05);
        assert!(ci.contains(1.0));
        assert!(ci.contains(3.0));
        assert!(ci.contains(5.0));
        assert!(!ci.contains(0.9));
        assert!(!ci.contains(5.1));
    }

    #[test]
    fn is_significant_above_null() {
        let sig = ConfidenceInterval::new(2.5, 1.2, 4.8, 0.05);
        assert!(sig.is_significant(1.0));

        let not_sig = ConfidenceInterval::new(1.5, 0.8, 2.2, 0.05);
        assert!(!not_sig.is_significant(1.0));
    }

    #[test]
    fn confidence_level() {
        let ci = ConfidenceInterval::new(1.0, 0.5, 1.5, 0.05);
        assert!((ci.confidence_level_pct() - 95.0).abs() < f64::EPSILON);

        let ci99 = ConfidenceInterval::new(1.0, 0.3, 1.7, 0.01);
        assert!((ci99.confidence_level_pct() - 99.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_is_zero_with_95pct() {
        let ci = ConfidenceInterval::default();
        assert!((ci.point_estimate - 0.0).abs() < f64::EPSILON);
        assert!((ci.alpha - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn serde_roundtrip() {
        let ci = ConfidenceInterval::new(2.5, 1.2, 4.8, 0.05);
        let json = serde_json::to_string(&ci);
        assert!(json.is_ok());
        let parsed: Result<ConfidenceInterval, _> = serde_json::from_str(&json.unwrap_or_default());
        assert!(parsed.is_ok());
        let rt = parsed.unwrap_or_default();
        assert!((rt.point_estimate - 2.5).abs() < f64::EPSILON);
    }
}
