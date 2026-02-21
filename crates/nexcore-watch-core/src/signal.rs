#![allow(dead_code)]
//! Signal detection for wearable deployment.
//!
//! ## Primitive Grounding
//! - κ (Comparison): all disproportionality metrics compare observed vs expected
//! - N (Quantity): 2×2 contingency table cell counts (a, b, c, d)
//! - ν (Frequency): rate-based metrics (PRR, ROR)
//! - Σ (Sum): Chi-squared summation over cells
//! - ρ (Recursion): logarithmic transforms (IC uses log₂)
//! - ∂ (Boundary): threshold checks for signal/no-signal decision
//!
//! ## Tier: T2-C (κ + N + ν + Σ + ρ + ∂)
//!
//! ## Grammar: Type-1 (context-sensitive)
//! The threshold check requires context (profile selection) beyond
//! the context-free computation of individual metrics.
//!
//! ## 2×2 Contingency Table
//! ```text
//!              | Event+ | Event- |
//!   Drug+      |   a    |   b    |  a+b
//!   Drug-      |   c    |   d    |  c+d
//!              | a+c    | b+d    |  N=a+b+c+d
//! ```

use serde::{Deserialize, Serialize};

/// Disproportionality signal result from 2×2 contingency table.
///
/// ## Primitive Grounding
/// - κ (Comparison): each metric compares observed vs expected frequency
/// - N (Quantity): cell counts and total N
/// - ν (Frequency): reporting rates
/// - ∃ (Existence): signal_detected = ∃(signal)
///
/// ## Tier: T2-C (κ + N + ν + ∃)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalResult {
    /// Drug name — λ (Location): identifies subject
    pub drug: String,
    /// Adverse event term — λ (Location): identifies object
    pub event: String,
    /// Proportional Reporting Ratio — κ (Comparison): (a/(a+b)) / (c/(c+d))
    pub prr: f64,
    /// Reporting Odds Ratio — κ (Comparison): (a×d) / (b×c)
    pub ror: f64,
    /// Information Component — κ + ρ (Comparison + Recursion): log₂(observed/expected)
    pub ic: f64,
    /// Empirical Bayesian Geometric Mean — κ + ν (Comparison + Frequency): a/E
    pub ebgm: f64,
    /// Pearson Chi-Squared statistic — κ + Σ (Comparison + Sum): Σ(O-E)²/E
    pub chi_squared: f64,
    /// Signal detected flag — ∃ (Existence): does signal exist at default thresholds
    pub signal_detected: bool,
}

impl SignalResult {
    /// Compute PRR: (a/(a+b)) / (c/(c+d))
    ///
    /// Proportional Reporting Ratio — the proportion of reports for the drug
    /// that mention the event, divided by the same proportion for all other drugs.
    ///
    /// ## Primitive: κ (Comparison)
    /// ## Tier: T1 — single primitive operation
    ///
    /// Returns ∅ (None) if denominators ≤ 0.
    pub fn compute_prr(a: f64, b: f64, c: f64, d: f64) -> Option<f64> {
        // ∂ (Boundary): reject negative cell values — invalid input
        if a < 0.0 || b < 0.0 || c < 0.0 || d < 0.0 {
            return None; // ∅ (Void): negative counts are nonsensical
        }
        let denom_left = a + b; // N: drug+ total
        let denom_right = c + d; // N: drug- total
        if denom_left <= 0.0 || denom_right <= 0.0 {
            return None; // ∅ (Void): undefined
        }
        let rate_drug = a / denom_left; // ν: reporting rate for drug
        let rate_other = c / denom_right; // ν: reporting rate for others
        if rate_other <= 0.0 {
            return None; // ∅ (Void): zero denominator
        }
        Some(rate_drug / rate_other) // κ: compare rates
    }

    /// Compute ROR: (a×d) / (b×c)
    ///
    /// Reporting Odds Ratio — odds of event in drug reports vs non-drug reports.
    ///
    /// ## Primitive: κ (Comparison)
    /// ## Tier: T1 — single primitive operation
    ///
    /// Returns ∅ (None) if denominator ≤ 0.
    pub fn compute_ror(a: f64, b: f64, c: f64, d: f64) -> Option<f64> {
        // ∂ (Boundary): reject negative cell values
        if a < 0.0 || b < 0.0 || c < 0.0 || d < 0.0 {
            return None;
        }
        let denom = b * c; // N: cross-product denominator
        if denom <= 0.0 {
            return None; // ∅ (Void): undefined
        }
        Some((a * d) / denom) // κ: odds comparison
    }

    /// Compute IC: log₂(a × N / ((a+b) × (a+c)))
    ///
    /// Information Component (Bayesian) — measures information content of the
    /// drug-event association. IC > 0 means observed > expected.
    ///
    /// ## Primitives: κ (Comparison) + ρ (Recursion via log₂)
    /// ## Tier: T2-P (κ + ρ) — two primitives composed
    ///
    /// Returns ∅ (None) if expected frequency ≤ 0 or ratio ≤ 0.
    pub fn compute_ic(a: f64, b: f64, c: f64, d: f64) -> Option<f64> {
        // ∂ (Boundary): reject negative cell values
        if a < 0.0 || b < 0.0 || c < 0.0 || d < 0.0 {
            return None;
        }
        let n = a + b + c + d; // N: total reports
        let row_total = a + b; // N: drug+ row
        let col_total = a + c; // N: event+ column
        if n <= 0.0 || row_total <= 0.0 || col_total <= 0.0 {
            return None; // ∅ (Void): undefined
        }
        let expected = (row_total * col_total) / n; // ν: expected frequency
        if expected <= 0.0 {
            return None; // ∅ (Void): zero expected
        }
        let observed = a; // N: observed count
        if observed <= 0.0 {
            return None; // ∅ (Void): no observations
        }
        Some((observed / expected).ln() / core::f64::consts::LN_2) // ρ: log₂ transform
    }

    /// Compute EBGM: a / E where E = (a+b)(a+c) / (a+b+c+d)
    ///
    /// Empirical Bayesian Geometric Mean — ratio of observed to expected,
    /// the core of the Multi-item Gamma Poisson Shrinker (MGPS).
    ///
    /// ## Primitives: κ (Comparison) + ν (Frequency)
    /// ## Tier: T2-P (κ + ν) — observed vs expected frequency comparison
    ///
    /// Returns ∅ (None) if expected ≤ 0.
    pub fn compute_ebgm(a: f64, b: f64, c: f64, d: f64) -> Option<f64> {
        // ∂ (Boundary): reject negative cell values
        if a < 0.0 || b < 0.0 || c < 0.0 || d < 0.0 {
            return None;
        }
        let n = a + b + c + d; // N: total
        let row_total = a + b; // N: drug+ row
        let col_total = a + c; // N: event+ column
        if n <= 0.0 {
            return None; // ∅ (Void): no data
        }
        let expected = (row_total * col_total) / n; // ν: expected frequency
        if expected <= 0.0 {
            return None; // ∅ (Void): zero expected
        }
        Some(a / expected) // κ: observed/expected ratio
    }

    /// Compute Pearson Chi-Squared: Σ((O-E)²/E) across all four cells.
    ///
    /// Tests independence between drug exposure and adverse event.
    /// χ² ≥ 3.841 → p < 0.05 (significant at 1 df).
    ///
    /// ## Primitives: κ (Comparison) + Σ (Sum) + N (Quantity)
    /// ## Tier: T2-C (κ + Σ + N) — summation of comparisons across cells
    ///
    /// Returns ∅ (None) if any expected value ≤ 0 or N ≤ 0.
    pub fn compute_chi_squared(a: f64, b: f64, c: f64, d: f64) -> Option<f64> {
        // ∂ (Boundary): reject negative cell values
        if a < 0.0 || b < 0.0 || c < 0.0 || d < 0.0 {
            return None;
        }
        let n = a + b + c + d; // N: total
        if n <= 0.0 {
            return None; // ∅ (Void): no data
        }

        let row1 = a + b; // N: drug+ row marginal
        let row2 = c + d; // N: drug- row marginal
        let col1 = a + c; // N: event+ column marginal
        let col2 = b + d; // N: event- column marginal

        // Expected values for each cell — ν (Frequency): E = row×col / N
        let e_a = (row1 * col1) / n;
        let e_b = (row1 * col2) / n;
        let e_c = (row2 * col1) / n;
        let e_d = (row2 * col2) / n;

        // ∂ (Boundary): all expected values must be > 0
        if e_a <= 0.0 || e_b <= 0.0 || e_c <= 0.0 || e_d <= 0.0 {
            return None; // ∅ (Void): chi-squared undefined
        }

        // Σ (Sum): accumulate (O-E)²/E across all four cells
        let chi_sq = ((a - e_a).powi(2) / e_a)
            + ((b - e_b).powi(2) / e_b)
            + ((c - e_c).powi(2) / e_c)
            + ((d - e_d).powi(2) / e_d);

        Some(chi_sq) // κ: final test statistic
    }

    /// Compute all five metrics at once from a 2×2 table.
    ///
    /// ## Primitive: σ (Sequence) — ordered computation pipeline
    /// ## Tier: T3 (σ + κ + N + ν + Σ + ρ + ∂) — full domain composition
    ///
    /// Pipeline: PRR →σ ROR →σ IC →σ EBGM →σ χ² →σ threshold check
    pub fn compute_all(drug: &str, event: &str, a: f64, b: f64, c: f64, d: f64) -> Self {
        let prr = Self::compute_prr(a, b, c, d).unwrap_or(0.0);
        let ror = Self::compute_ror(a, b, c, d).unwrap_or(0.0);
        let ic = Self::compute_ic(a, b, c, d).unwrap_or(0.0);
        let ebgm = Self::compute_ebgm(a, b, c, d).unwrap_or(0.0);
        let chi_squared = Self::compute_chi_squared(a, b, c, d).unwrap_or(0.0);

        // ∂ (Boundary): default threshold check
        // PRR ≥ 2.0 AND χ² ≥ 3.841 AND n ≥ 3
        let signal_detected = prr >= 2.0 && chi_squared >= 3.841 && a >= 3.0;

        Self {
            drug: drug.to_string(),
            event: event.to_string(),
            prr,
            ror,
            ic,
            ebgm,
            chi_squared,
            signal_detected,
        }
    }

    /// Check if signal meets default thresholds.
    ///
    /// Default: PRR ≥ 2.0, χ² ≥ 3.841, n ≥ 3
    ///
    /// ## Primitive: ∂ (Boundary) + κ (Comparison)
    /// ## Tier: T2-P (∂ + κ)
    pub fn meets_default_thresholds(&self, n: u32) -> bool {
        self.prr >= 2.0 && self.chi_squared >= 3.841 && n >= 3
    }

    /// Check if signal meets sensitive thresholds (for fatal/life-threatening).
    ///
    /// Sensitive: PRR ≥ 1.5, χ² ≥ 2.706, n ≥ 2
    ///
    /// ## Primitive: ∂ (Boundary) + κ (Comparison)
    /// ## Tier: T2-P (∂ + κ)
    ///
    /// P0 Patient Safety: lower thresholds catch weak signals early.
    pub fn meets_sensitive_thresholds(&self, n: u32) -> bool {
        self.prr >= 1.5 && self.chi_squared >= 2.706 && n >= 2
    }

    /// Check if signal meets strict thresholds (high specificity).
    ///
    /// Strict: PRR ≥ 3.0, χ² ≥ 6.635, n ≥ 5
    ///
    /// ## Primitive: ∂ (Boundary) + κ (Comparison)
    /// ## Tier: T2-P (∂ + κ)
    ///
    /// Reduces false positives at cost of sensitivity.
    pub fn meets_strict_thresholds(&self, n: u32) -> bool {
        self.prr >= 3.0 && self.chi_squared >= 6.635 && n >= 5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════
    // PRR Tests — κ (Comparison)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn prr_known_signal() {
        // Classic 2×2: a=15, b=100, c=20, d=10000
        // PRR = (15/115) / (20/10020) = 0.1304 / 0.001996 ≈ 65.3
        let result = SignalResult::compute_prr(15.0, 100.0, 20.0, 10000.0);
        assert!(result.is_some());
        let prr = result.unwrap_or(0.0);
        assert!(prr > 60.0, "PRR should be > 60, got {prr}");
        assert!(prr < 70.0, "PRR should be < 70, got {prr}");
    }

    #[test]
    fn prr_no_signal() {
        // Balanced table: equal proportions → PRR ≈ 1.0
        let result = SignalResult::compute_prr(10.0, 90.0, 100.0, 900.0);
        assert!(result.is_some());
        let prr = result.unwrap_or(0.0);
        assert!((prr - 1.0).abs() < 0.01, "PRR should be ≈1.0, got {prr}");
    }

    #[test]
    fn prr_zero_denominator_returns_none() {
        // ∅ (Void): a+b = 0 → undefined
        assert!(SignalResult::compute_prr(0.0, 0.0, 10.0, 100.0).is_none());
        // ∅ (Void): c+d = 0 → undefined
        assert!(SignalResult::compute_prr(10.0, 100.0, 0.0, 0.0).is_none());
    }

    #[test]
    fn prr_zero_c_returns_none() {
        // c = 0 means rate_other = 0 → division by zero
        assert!(SignalResult::compute_prr(10.0, 90.0, 0.0, 1000.0).is_none());
    }

    // ═══════════════════════════════════════════════════════════
    // ROR Tests — κ (Comparison)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn ror_known_signal() {
        // ROR = (15×10000) / (100×20) = 150000/2000 = 75.0
        let result = SignalResult::compute_ror(15.0, 100.0, 20.0, 10000.0);
        assert!(result.is_some());
        let ror = result.unwrap_or(0.0);
        assert!((ror - 75.0).abs() < 0.01, "ROR should be 75.0, got {ror}");
    }

    #[test]
    fn ror_no_signal() {
        // Balanced: ROR = (10×900) / (90×100) = 9000/9000 = 1.0
        let result = SignalResult::compute_ror(10.0, 90.0, 100.0, 900.0);
        assert!(result.is_some());
        let ror = result.unwrap_or(0.0);
        assert!((ror - 1.0).abs() < 0.01, "ROR should be ≈1.0, got {ror}");
    }

    #[test]
    fn ror_zero_denominator_returns_none() {
        // ∅ (Void): b×c = 0
        assert!(SignalResult::compute_ror(10.0, 0.0, 10.0, 100.0).is_none());
        assert!(SignalResult::compute_ror(10.0, 100.0, 0.0, 100.0).is_none());
    }

    // ═══════════════════════════════════════════════════════════
    // IC Tests — κ (Comparison) + ρ (Recursion via log₂)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn ic_positive_signal() {
        // a=15, b=100, c=20, d=10000
        // E = (115 × 35) / 10135 = 0.3972
        // IC = log₂(15 / 0.3972) ≈ log₂(37.76) ≈ 5.24
        let result = SignalResult::compute_ic(15.0, 100.0, 20.0, 10000.0);
        assert!(result.is_some());
        let ic = result.unwrap_or(0.0);
        assert!(ic > 4.0, "IC should be > 4.0 for strong signal, got {ic}");
    }

    #[test]
    fn ic_balanced_near_zero() {
        // Balanced table: IC ≈ 0 (no information gain)
        let result = SignalResult::compute_ic(10.0, 90.0, 100.0, 900.0);
        assert!(result.is_some());
        let ic = result.unwrap_or(99.0);
        assert!(
            ic.abs() < 0.1,
            "IC should be ≈0 for balanced table, got {ic}"
        );
    }

    #[test]
    fn ic_zero_observed_returns_none() {
        // ∅ (Void): a = 0 → log(0) undefined
        assert!(SignalResult::compute_ic(0.0, 100.0, 20.0, 10000.0).is_none());
    }

    #[test]
    fn ic_zero_total_returns_none() {
        assert!(SignalResult::compute_ic(0.0, 0.0, 0.0, 0.0).is_none());
    }

    // ═══════════════════════════════════════════════════════════
    // EBGM Tests — κ (Comparison) + ν (Frequency)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn ebgm_known_signal() {
        // E = (115 × 35) / 10135 ≈ 0.3972
        // EBGM = 15 / 0.3972 ≈ 37.76
        let result = SignalResult::compute_ebgm(15.0, 100.0, 20.0, 10000.0);
        assert!(result.is_some());
        let ebgm = result.unwrap_or(0.0);
        assert!(ebgm > 35.0, "EBGM should be > 35, got {ebgm}");
        assert!(ebgm < 40.0, "EBGM should be < 40, got {ebgm}");
    }

    #[test]
    fn ebgm_balanced_near_one() {
        // Balanced: EBGM ≈ 1.0 (observed ≈ expected)
        let result = SignalResult::compute_ebgm(10.0, 90.0, 100.0, 900.0);
        assert!(result.is_some());
        let ebgm = result.unwrap_or(99.0);
        assert!((ebgm - 1.0).abs() < 0.01, "EBGM should be ≈1.0, got {ebgm}");
    }

    #[test]
    fn ebgm_zero_total_returns_none() {
        assert!(SignalResult::compute_ebgm(0.0, 0.0, 0.0, 0.0).is_none());
    }

    // ═══════════════════════════════════════════════════════════
    // Chi-Squared Tests — κ (Comparison) + Σ (Sum)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn chi_squared_significant() {
        // a=15, b=100, c=20, d=10000
        // Strong association → χ² >> 3.841
        let result = SignalResult::compute_chi_squared(15.0, 100.0, 20.0, 10000.0);
        assert!(result.is_some());
        let chi2 = result.unwrap_or(0.0);
        assert!(chi2 > 3.841, "χ² should exceed 3.841 (p<0.05), got {chi2}");
    }

    #[test]
    fn chi_squared_balanced_near_zero() {
        // Perfectly balanced → χ² ≈ 0
        let result = SignalResult::compute_chi_squared(10.0, 90.0, 100.0, 900.0);
        assert!(result.is_some());
        let chi2 = result.unwrap_or(99.0);
        assert!(chi2 < 0.1, "χ² should be ≈0 for balanced table, got {chi2}");
    }

    #[test]
    fn chi_squared_zero_marginal_returns_none() {
        // ∅ (Void): row marginal = 0
        assert!(SignalResult::compute_chi_squared(0.0, 0.0, 10.0, 100.0).is_none());
    }

    // ═══════════════════════════════════════════════════════════
    // compute_all Tests — σ (Sequence) pipeline
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn compute_all_strong_signal() {
        let result = SignalResult::compute_all("Aspirin", "GI Bleed", 15.0, 100.0, 20.0, 10000.0);
        assert_eq!(result.drug, "Aspirin");
        assert_eq!(result.event, "GI Bleed");
        assert!(result.prr > 2.0, "PRR should indicate signal");
        assert!(result.ror > 1.0, "ROR should indicate signal");
        assert!(result.ic > 0.0, "IC should be positive");
        assert!(result.ebgm > 2.0, "EBGM should indicate signal");
        assert!(result.chi_squared > 3.841, "χ² should be significant");
        assert!(result.signal_detected, "Signal should be detected");
    }

    #[test]
    fn compute_all_no_signal() {
        let result = SignalResult::compute_all("Placebo", "Headache", 10.0, 90.0, 100.0, 900.0);
        assert!(!result.signal_detected, "No signal for balanced table");
    }

    // ═══════════════════════════════════════════════════════════
    // Threshold Tests — ∂ (Boundary)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn default_thresholds_met() {
        let result = SignalResult::compute_all("DrugX", "EventY", 15.0, 100.0, 20.0, 10000.0);
        assert!(result.meets_default_thresholds(15));
    }

    #[test]
    fn default_thresholds_not_met_low_n() {
        let result = SignalResult {
            drug: String::new(),
            event: String::new(),
            prr: 5.0,
            ror: 5.0,
            ic: 2.0,
            ebgm: 3.0,
            chi_squared: 10.0,
            signal_detected: true,
        };
        assert!(!result.meets_default_thresholds(2), "n=2 < 3 threshold");
    }

    #[test]
    fn sensitive_thresholds_lower_bar() {
        let result = SignalResult {
            drug: String::new(),
            event: String::new(),
            prr: 1.8, // Above 1.5 sensitive, below 2.0 default
            ror: 1.5,
            ic: 0.5,
            ebgm: 1.5,
            chi_squared: 3.0, // Above 2.706 sensitive, below 3.841 default
            signal_detected: false,
        };
        assert!(
            result.meets_sensitive_thresholds(2),
            "Should pass sensitive"
        );
        assert!(!result.meets_default_thresholds(2), "Should fail default");
    }

    #[test]
    fn strict_thresholds_high_bar() {
        let result = SignalResult {
            drug: String::new(),
            event: String::new(),
            prr: 2.5, // Above 2.0 default, below 3.0 strict
            ror: 3.0,
            ic: 1.5,
            ebgm: 2.5,
            chi_squared: 5.0, // Above 3.841 default, below 6.635 strict
            signal_detected: true,
        };
        assert!(result.meets_default_thresholds(5), "Should pass default");
        assert!(!result.meets_strict_thresholds(5), "Should fail strict");
    }

    // ═══════════════════════════════════════════════════════════
    // Edge Cases — ∅ (Void) + ∂ (Boundary)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn negative_values_handled() {
        // Negative values should return None (invalid input)
        assert!(SignalResult::compute_prr(-1.0, 10.0, 10.0, 100.0).is_none());
    }

    #[test]
    fn very_large_values_no_overflow() {
        let result = SignalResult::compute_prr(1e10, 1e12, 1e8, 1e14);
        assert!(result.is_some());
        let prr = result.unwrap_or(0.0);
        assert!(prr.is_finite(), "PRR should be finite for large values");
    }

    #[test]
    fn all_ones_table() {
        // Minimal symmetric table: a=b=c=d=1
        let result = SignalResult::compute_all("A", "B", 1.0, 1.0, 1.0, 1.0);
        assert!((result.prr - 1.0).abs() < 0.01, "PRR should be 1.0");
        assert!((result.ror - 1.0).abs() < 0.01, "ROR should be 1.0");
        assert!(result.ic.abs() < 0.01, "IC should be ≈0");
        assert!((result.ebgm - 1.0).abs() < 0.01, "EBGM should be 1.0");
        assert!(result.chi_squared.abs() < 0.01, "χ² should be ≈0");
        assert!(!result.signal_detected, "No signal for symmetric table");
    }
}
