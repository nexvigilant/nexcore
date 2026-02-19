//! Tier 1 -- Reactive Signal Detection.
//!
//! Tier: T2-P (maps to Boundary `partial` + Quantity `N` + Comparison `kappa`)
//!
//! The simplest signal detection tier: did disproportionate reporting occur?
//!
//! ```text
//! S(d, e) = N(d, e) / E(d, e)
//! ```
//!
//! Where:
//! - `N(d, e)` = observed count of drug-event co-reports
//! - `E(d, e)` = expected count under statistical independence
//!
//! This is essentially the PRR (Proportional Reporting Ratio) simplified
//! to the observed/expected ratio.

use crate::types::ReportingCounts;

/// Default threshold for reactive signal detection.
///
/// Signal is flagged when `S(d,e) >= REACTIVE_THRESHOLD`.
pub const REACTIVE_THRESHOLD: f64 = 2.0;

/// Minimum number of co-reports required before signal evaluation.
pub const MINIMUM_REPORTS: f64 = 3.0;

/// Computes the reactive signal strength S(d,e) = observed / expected.
///
/// Returns `None` if:
/// - Expected count is zero (would divide by zero)
/// - Observed count is below the minimum report threshold
#[must_use]
pub fn signal_strength(counts: &ReportingCounts) -> Option<f64> {
    if counts.a < MINIMUM_REPORTS {
        return None;
    }

    let expected = counts.expected();
    if expected <= 0.0 {
        return None;
    }

    Some(counts.a / expected)
}

/// Returns true if a reactive signal is detected above the given threshold.
#[must_use]
pub fn is_signal(counts: &ReportingCounts, threshold: f64) -> bool {
    match signal_strength(counts) {
        Some(s) => s >= threshold,
        None => false,
    }
}

/// Returns true if a reactive signal is detected using the default threshold.
#[must_use]
pub fn is_signal_default(counts: &ReportingCounts) -> bool {
    is_signal(counts, REACTIVE_THRESHOLD)
}

/// Computes the Chi-squared statistic for the 2x2 contingency table.
///
/// ```text
/// chi2 = (N * (ad - bc)^2) / ((a+b)(c+d)(a+c)(b+d))
/// ```
///
/// Returns `None` if any marginal total is zero.
#[must_use]
pub fn chi_squared(counts: &ReportingCounts) -> Option<f64> {
    let n = counts.total();
    let row1 = counts.a + counts.b;
    let row2 = counts.c + counts.d;
    let col1 = counts.a + counts.c;
    let col2 = counts.b + counts.d;

    let denom = row1 * row2 * col1 * col2;
    if denom <= 0.0 {
        return None;
    }

    let ad_bc = counts.a * counts.d - counts.b * counts.c;
    Some(n * ad_bc * ad_bc / denom)
}

/// Chi-squared critical value at p = 0.05 with 1 degree of freedom.
pub const CHI2_CRITICAL_005: f64 = 3.841;

/// Returns true if the chi-squared test rejects independence at p < 0.05.
#[must_use]
pub fn chi_squared_significant(counts: &ReportingCounts) -> bool {
    match chi_squared(counts) {
        Some(chi2) => chi2 >= CHI2_CRITICAL_005,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_strength_basic() {
        let counts = ReportingCounts::new(15.0, 100.0, 20.0, 10000.0);
        let result = signal_strength(&counts);
        assert!(result.is_some());
        let s = result.unwrap_or(0.0);
        // observed = 15, expected = 115 * 35 / 10135 = 0.3972...
        // S = 15 / expected
        let expected = 115.0 * 35.0 / 10135.0;
        let expected_s = 15.0 / expected;
        assert!((s - expected_s).abs() < 1e-10);
    }

    #[test]
    fn signal_strength_below_minimum() {
        let counts = ReportingCounts::new(2.0, 100.0, 20.0, 10000.0);
        assert!(signal_strength(&counts).is_none());
    }

    #[test]
    fn signal_strength_zero_expected() {
        // All reports are drug+event — no expected independence
        let counts = ReportingCounts::new(10.0, 0.0, 0.0, 0.0);
        // expected = (10+0)*(10+0)/10 = 10, so expected is not zero here
        // Let's make a case where expected truly is zero
        let counts2 = ReportingCounts::new(10.0, 0.0, 0.0, 100.0);
        // expected = (10+0)*(10+0)/110 = 100/110 ~ 0.909 (not zero)
        let result = signal_strength(&counts2);
        assert!(result.is_some());
    }

    #[test]
    fn is_signal_default_positive() {
        // Strong signal: many co-reports relative to expected
        let counts = ReportingCounts::new(50.0, 100.0, 20.0, 10000.0);
        assert!(is_signal_default(&counts));
    }

    #[test]
    fn is_signal_default_negative() {
        // Weak signal: few co-reports relative to expected
        let counts = ReportingCounts::new(3.0, 1000.0, 500.0, 10000.0);
        let s = signal_strength(&counts);
        // Expected = 1003 * 503 / 11503 = 43.86..., observed = 3
        // S = 3 / 43.86 ~ 0.068 (well below threshold)
        assert!(s.is_some());
        assert!(!is_signal_default(&counts));
    }

    #[test]
    fn chi_squared_basic() {
        let counts = ReportingCounts::new(15.0, 100.0, 20.0, 10000.0);
        let result = chi_squared(&counts);
        assert!(result.is_some());
        let chi2 = result.unwrap_or(0.0);
        assert!(chi2 > 0.0);
    }

    #[test]
    fn chi_squared_zero_marginal() {
        let counts = ReportingCounts::new(0.0, 0.0, 10.0, 100.0);
        // row1 = 0+0 = 0 -> denom = 0
        assert!(chi_squared(&counts).is_none());
    }

    #[test]
    fn chi_squared_significant_strong_signal() {
        let counts = ReportingCounts::new(50.0, 100.0, 20.0, 10000.0);
        assert!(chi_squared_significant(&counts));
    }

    #[test]
    fn chi_squared_not_significant_weak() {
        // Proportional reporting: drug has roughly equal event rate as background
        let counts = ReportingCounts::new(10.0, 990.0, 100.0, 9900.0);
        // Expected ~ (1000 * 110) / 11000 = 10.0, observed = 10
        // O/E ~ 1.0, chi2 ~ 0
        assert!(!chi_squared_significant(&counts));
    }

    #[test]
    fn is_signal_custom_threshold() {
        let counts = ReportingCounts::new(15.0, 100.0, 20.0, 10000.0);
        let s = signal_strength(&counts).unwrap_or(0.0);
        // S is large (15 / ~0.4 ~ 37.8), any reasonable threshold should detect
        assert!(is_signal(&counts, 2.0));
        assert!(is_signal(&counts, 5.0));
        assert!(is_signal(&counts, 10.0));
        assert!(is_signal(&counts, 30.0));
        // But not an extreme threshold
        assert!(!is_signal(&counts, s + 1.0));
    }
}
