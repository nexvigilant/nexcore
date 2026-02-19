//! Bonferroni Family-Wise Error Rate (FWER) Correction
//!
//! Controls the probability of making one or more false discoveries.
//! Most conservative multiple testing correction - use when false positives
//! are costly (e.g., regulatory decisions, label changes).
//!
//! # Algorithm
//!
//! Given m hypothesis tests:
//!
//! ```text
//! Adjusted p-value = min(p × m, 1.0)
//! Reject if: p ≤ α/m
//! ```
//!
//! # When to Use
//!
//! - **High-stakes decisions**: Regulatory actions, label changes
//! - **Few comparisons**: Bonferroni is reasonable for m < 20
//! - **Strong control needed**: When ANY false positive is unacceptable
//!
//! # When NOT to Use
//!
//! - **Large-scale screening**: Too conservative, misses true signals
//! - **Exploratory analysis**: Use Benjamini-Hochberg instead
//!
//! # Variants
//!
//! This module also implements:
//! - **Holm-Bonferroni**: Step-down procedure, uniformly more powerful
//! - **Šidák**: Assumes independence, slightly less conservative
//!
//! # References
//!
//! - Bonferroni CE (1936). "Teoria statistica delle classi e calcolo delle probabilità."
//!   Pubblicazioni del R Istituto Superiore di Scienze Economiche e Commerciali di Firenze 8:3-62.
//!
//! - Holm S (1979). "A simple sequentially rejective multiple test procedure."
//!   Scandinavian Journal of Statistics 6(2):65-70.
//!   JSTOR: [4615733](https://www.jstor.org/stable/4615733)
//!
//! - Šidák Z (1967). "Rectangular confidence regions for the means of multivariate normal
//!   distributions." Journal of the American Statistical Association 62(318):626-633.
//!   DOI: [10.1080/01621459.1967.10482935](https://doi.org/10.1080/01621459.1967.10482935)

use serde::{Deserialize, Serialize};

/// Result of Bonferroni-family adjustment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BonferroniResult {
    /// Original p-values (in input order)
    pub p_values: Vec<f64>,
    /// Adjusted p-values (in input order)
    pub adjusted_p_values: Vec<f64>,
    /// Whether each hypothesis is rejected
    pub rejected: Vec<bool>,
    /// Number of rejections
    pub n_rejected: usize,
    /// Alpha level used
    pub alpha: f64,
    /// Method used for adjustment
    pub method: BonferroniMethod,
}

/// Bonferroni-family correction methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BonferroniMethod {
    /// Standard Bonferroni: p_adj = p × m
    Bonferroni,
    /// Holm step-down: more powerful than Bonferroni
    Holm,
    /// Šidák: assumes independence, p_adj = 1 - (1-p)^m
    Sidak,
}

/// Apply standard Bonferroni correction.
///
/// # Arguments
///
/// * `p_values` - Vector of raw p-values
/// * `alpha` - Significance level (typically 0.05)
///
/// # Complexity
///
/// - **Time**: O(n)
/// - **Space**: O(n)
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::adjustment::bonferroni::bonferroni_adjust;
///
/// let p_values = vec![0.001, 0.01, 0.03, 0.04, 0.05];
/// let result = bonferroni_adjust(&p_values, 0.05);
///
/// // p=0.001 (adjusted: 0.005 ≤ 0.05) and p=0.01 (adjusted: 0.05 ≤ 0.05) are rejected
/// assert_eq!(result.n_rejected, 2);
/// ```
#[must_use]
pub fn bonferroni_adjust(p_values: &[f64], alpha: f64) -> BonferroniResult {
    let m = p_values.len();

    if m == 0 {
        return BonferroniResult {
            p_values: vec![],
            adjusted_p_values: vec![],
            rejected: vec![],
            n_rejected: 0,
            alpha,
            method: BonferroniMethod::Bonferroni,
        };
    }

    let m_f64 = m as f64;
    let adjusted_p_values: Vec<f64> = p_values.iter().map(|&p| (p * m_f64).min(1.0)).collect();

    let rejected: Vec<bool> = adjusted_p_values.iter().map(|&p| p <= alpha).collect();

    let n_rejected = rejected.iter().filter(|&&r| r).count();

    BonferroniResult {
        p_values: p_values.to_vec(),
        adjusted_p_values,
        rejected,
        n_rejected,
        alpha,
        method: BonferroniMethod::Bonferroni,
    }
}

/// Apply Holm-Bonferroni step-down correction.
///
/// More powerful than standard Bonferroni while still controlling FWER.
///
/// # Algorithm
///
/// 1. Sort p-values: p₍₁₎ ≤ p₍₂₎ ≤ ... ≤ p₍ₘ₎
/// 2. Find smallest k where p₍ₖ₎ > α/(m-k+1)
/// 3. Reject hypotheses 1, ..., k-1
///
/// # Complexity
///
/// - **Time**: O(n log n) due to sorting
/// - **Space**: O(n)
#[must_use]
pub fn holm_adjust(p_values: &[f64], alpha: f64) -> BonferroniResult {
    let m = p_values.len();

    if m == 0 {
        return BonferroniResult {
            p_values: vec![],
            adjusted_p_values: vec![],
            rejected: vec![],
            n_rejected: 0,
            alpha,
            method: BonferroniMethod::Holm,
        };
    }

    // Sort with original indices
    let mut indexed: Vec<(usize, f64)> = p_values.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let m_f64 = m as f64;

    // Compute adjusted p-values (step-down)
    let mut adj_sorted = vec![0.0; m];
    adj_sorted[0] = (indexed[0].1 * m_f64).min(1.0);

    for i in 1..m {
        let multiplier = m_f64 - i as f64;
        let current_adj = indexed[i].1 * multiplier;
        // Ensure monotonicity
        adj_sorted[i] = current_adj.max(adj_sorted[i - 1]).min(1.0);
    }

    // Map back to original order
    let mut adjusted_p_values = vec![0.0; m];
    let mut rejected = vec![false; m];

    for (sorted_idx, &(orig_idx, _)) in indexed.iter().enumerate() {
        adjusted_p_values[orig_idx] = adj_sorted[sorted_idx];
        rejected[orig_idx] = adj_sorted[sorted_idx] <= alpha;
    }

    let n_rejected = rejected.iter().filter(|&&r| r).count();

    BonferroniResult {
        p_values: p_values.to_vec(),
        adjusted_p_values,
        rejected,
        n_rejected,
        alpha,
        method: BonferroniMethod::Holm,
    }
}

/// Apply Šidák correction (assumes independence).
///
/// Less conservative than Bonferroni when tests are independent.
///
/// # Formula
///
/// ```text
/// α_adj = 1 - (1 - α)^(1/m)
/// p_adj = 1 - (1 - p)^m
/// ```
///
/// # Complexity
///
/// - **Time**: O(n)
/// - **Space**: O(n)
#[must_use]
pub fn sidak_adjust(p_values: &[f64], alpha: f64) -> BonferroniResult {
    let m = p_values.len();

    if m == 0 {
        return BonferroniResult {
            p_values: vec![],
            adjusted_p_values: vec![],
            rejected: vec![],
            n_rejected: 0,
            alpha,
            method: BonferroniMethod::Sidak,
        };
    }

    let m_f64 = m as f64;

    // Adjusted p-value: 1 - (1-p)^m
    let adjusted_p_values: Vec<f64> = p_values
        .iter()
        .map(|&p| {
            if p >= 1.0 {
                1.0
            } else {
                (1.0 - (1.0 - p).powf(m_f64)).min(1.0)
            }
        })
        .collect();

    let rejected: Vec<bool> = adjusted_p_values.iter().map(|&p| p <= alpha).collect();

    let n_rejected = rejected.iter().filter(|&&r| r).count();

    BonferroniResult {
        p_values: p_values.to_vec(),
        adjusted_p_values,
        rejected,
        n_rejected,
        alpha,
        method: BonferroniMethod::Sidak,
    }
}

/// Get the Bonferroni-corrected significance threshold.
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::adjustment::bonferroni::bonferroni_threshold;
///
/// let threshold = bonferroni_threshold(0.05, 100);
/// assert!((threshold - 0.0005).abs() < 1e-10);
/// ```
#[must_use]
pub fn bonferroni_threshold(alpha: f64, n_tests: usize) -> f64 {
    if n_tests == 0 {
        return alpha;
    }
    alpha / n_tests as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bonferroni_basic() {
        let p_values = vec![0.001, 0.01, 0.03, 0.04, 0.05];
        let result = bonferroni_adjust(&p_values, 0.05);

        // Adjusted p-values = original × 5
        assert!((result.adjusted_p_values[0] - 0.005).abs() < 1e-10);
        assert!((result.adjusted_p_values[1] - 0.05).abs() < 1e-10);
        assert!((result.adjusted_p_values[2] - 0.15).abs() < 1e-10);

        // Only first one rejected at α = 0.05
        assert!(result.rejected[0]);
        assert!(result.rejected[1]); // 0.05 <= 0.05
        assert!(!result.rejected[2]);

        assert_eq!(result.n_rejected, 2);
    }

    #[test]
    fn test_bonferroni_capped_at_one() {
        let p_values = vec![0.5, 0.6, 0.7];
        let result = bonferroni_adjust(&p_values, 0.05);

        // All adjusted p-values should be capped at 1.0
        for &adj_p in &result.adjusted_p_values {
            assert!(adj_p <= 1.0);
        }
    }

    #[test]
    fn test_holm_more_powerful() {
        // Holm should reject at least as many as Bonferroni
        let p_values = vec![0.001, 0.01, 0.02, 0.03, 0.04];

        let bonf = bonferroni_adjust(&p_values, 0.05);
        let holm = holm_adjust(&p_values, 0.05);

        assert!(holm.n_rejected >= bonf.n_rejected);
    }

    #[test]
    fn test_holm_step_down() {
        let p_values = vec![0.01, 0.02, 0.03, 0.04, 0.05];
        let result = holm_adjust(&p_values, 0.05);

        // Holm multipliers: 5, 4, 3, 2, 1
        // Adjusted: 0.05, 0.08, 0.09, 0.08 → 0.09, 0.05 → 0.09
        // (ensuring monotonicity)
        assert!(result.rejected[0]); // 0.01 × 5 = 0.05 ≤ 0.05
    }

    #[test]
    fn test_sidak_less_conservative() {
        let p_values = vec![0.01, 0.02, 0.03];

        let bonf = bonferroni_adjust(&p_values, 0.05);
        let sidak = sidak_adjust(&p_values, 0.05);

        // Šidák adjusted p-values should be slightly smaller than Bonferroni
        for i in 0..p_values.len() {
            assert!(sidak.adjusted_p_values[i] <= bonf.adjusted_p_values[i]);
        }
    }

    #[test]
    fn test_sidak_formula() {
        let p_values = vec![0.01];
        let result = sidak_adjust(&p_values, 0.05);

        // With m=1, Šidák = Bonferroni = original
        assert!((result.adjusted_p_values[0] - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_empty_input() {
        let p_values: Vec<f64> = vec![];

        let bonf = bonferroni_adjust(&p_values, 0.05);
        let holm = holm_adjust(&p_values, 0.05);
        let sidak = sidak_adjust(&p_values, 0.05);

        assert_eq!(bonf.n_rejected, 0);
        assert_eq!(holm.n_rejected, 0);
        assert_eq!(sidak.n_rejected, 0);
    }

    #[test]
    fn test_threshold() {
        assert!((bonferroni_threshold(0.05, 100) - 0.0005).abs() < 1e-10);
        assert!((bonferroni_threshold(0.05, 1) - 0.05).abs() < 1e-10);
        assert!((bonferroni_threshold(0.05, 0) - 0.05).abs() < 1e-10);
    }
}
