//! Benjamini-Hochberg False Discovery Rate (FDR) Correction
//!
//! Controls the expected proportion of false discoveries among all rejected hypotheses.
//! Less conservative than Bonferroni, making it preferred for large-scale screening.
//!
//! # Algorithm
//!
//! Given m hypothesis tests with p-values p₁, p₂, ..., pₘ:
//!
//! 1. Sort p-values: p₍₁₎ ≤ p₍₂₎ ≤ ... ≤ p₍ₘ₎
//! 2. Find largest k where p₍ₖ₎ ≤ (k/m) × α
//! 3. Reject all hypotheses with p₍ᵢ₎ ≤ p₍ₖ₎
//!
//! Equivalently, compute adjusted p-values (q-values):
//! ```text
//! q₍ᵢ₎ = min(p₍ᵢ₎ × m/i, q₍ᵢ₊₁₎)  for i = m-1, ..., 1
//! q₍ₘ₎ = p₍ₘ₎
//! ```
//!
//! # When to Use
//!
//! - **Large-scale screening**: Thousands of drug-event pairs
//! - **Discovery-oriented**: Finding signals for further investigation
//! - **Resource-constrained**: When you can tolerate some false positives
//!
//! # References
//!
//! - Benjamini Y, Hochberg Y (1995). "Controlling the false discovery rate: a practical
//!   and powerful approach to multiple testing." Journal of the Royal Statistical Society
//!   Series B 57(1):289-300. DOI: [10.1111/j.2517-6161.1995.tb02031.x](https://doi.org/10.1111/j.2517-6161.1995.tb02031.x)
//!
//! - Benjamini Y, Yekutieli D (2001). "The control of the false discovery rate in multiple
//!   testing under dependency." The Annals of Statistics 29(4):1165-1188.
//!   DOI: [10.1214/aos/1013699998](https://doi.org/10.1214/aos/1013699998)
//!
//! - Storey JD (2002). "A direct approach to false discovery rates." Journal of the Royal
//!   Statistical Society Series B 64(3):479-498.
//!   DOI: [10.1111/1467-9868.00346](https://doi.org/10.1111/1467-9868.00346)

use serde::{Deserialize, Serialize};

/// Result of Benjamini-Hochberg FDR adjustment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BHResult {
    /// Original p-values (in input order)
    pub p_values: Vec<f64>,
    /// Adjusted p-values / q-values (in input order)
    pub q_values: Vec<f64>,
    /// Whether each hypothesis is rejected at the specified FDR level
    pub rejected: Vec<bool>,
    /// Number of rejections
    pub n_rejected: usize,
    /// FDR level used
    pub fdr_level: f64,
    /// Critical p-value threshold (largest p-value that is rejected)
    pub critical_value: Option<f64>,
}

/// Apply Benjamini-Hochberg FDR correction to a set of p-values.
///
/// # Arguments
///
/// * `p_values` - Vector of raw p-values from multiple hypothesis tests
/// * `fdr_level` - Desired false discovery rate (typically 0.05 or 0.10)
///
/// # Returns
///
/// `BHResult` containing adjusted p-values and rejection decisions.
///
/// # Complexity
///
/// - **Time**: O(n log n) due to sorting
/// - **Space**: O(n) for storing results
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::adjustment::benjamini_hochberg::bh_adjust;
///
/// let p_values = vec![0.001, 0.008, 0.039, 0.041, 0.042, 0.06, 0.074, 0.205, 0.212, 0.216];
/// let result = bh_adjust(&p_values, 0.05);
///
/// println!("Rejected {} of {} hypotheses", result.n_rejected, p_values.len());
/// for (i, (q, rejected)) in result.q_values.iter().zip(&result.rejected).enumerate() {
///     if *rejected {
///         println!("  Hypothesis {}: q = {:.4} (significant)", i, q);
///     }
/// }
/// ```
#[must_use]
pub fn bh_adjust(p_values: &[f64], fdr_level: f64) -> BHResult {
    let m = p_values.len();

    if m == 0 {
        return BHResult {
            p_values: vec![],
            q_values: vec![],
            rejected: vec![],
            n_rejected: 0,
            fdr_level,
            critical_value: None,
        };
    }

    // Create indexed p-values for sorting
    let mut indexed: Vec<(usize, f64)> = p_values.iter().copied().enumerate().collect();

    // Sort by p-value (ascending)
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Compute adjusted p-values (q-values) using step-up procedure
    let m_f64 = m as f64;
    let mut q_sorted: Vec<f64> = vec![0.0; m];

    // Start from the largest p-value
    q_sorted[m - 1] = indexed[m - 1].1.min(1.0);

    // Work backwards, ensuring monotonicity
    for i in (0..m - 1).rev() {
        let rank = (i + 1) as f64;
        let adjusted = indexed[i].1 * m_f64 / rank;
        q_sorted[i] = adjusted.min(q_sorted[i + 1]).min(1.0);
    }

    // Find critical value (BH threshold)
    let mut critical_value: Option<f64> = None;
    for i in (0..m).rev() {
        let rank = (i + 1) as f64;
        let threshold = rank / m_f64 * fdr_level;
        if indexed[i].1 <= threshold {
            critical_value = Some(indexed[i].1);
            break;
        }
    }

    // Map results back to original order
    let mut q_values = vec![0.0; m];
    let mut rejected = vec![false; m];

    for (sorted_idx, &(orig_idx, _)) in indexed.iter().enumerate() {
        q_values[orig_idx] = q_sorted[sorted_idx];
        rejected[orig_idx] = q_sorted[sorted_idx] <= fdr_level;
    }

    let n_rejected = rejected.iter().filter(|&&r| r).count();

    BHResult {
        p_values: p_values.to_vec(),
        q_values,
        rejected,
        n_rejected,
        fdr_level,
        critical_value,
    }
}

/// Compute q-values only without rejection decisions.
///
/// Lighter-weight version when you only need adjusted p-values.
///
/// # Complexity
///
/// - **Time**: O(n log n)
/// - **Space**: O(n)
#[must_use]
pub fn compute_q_values(p_values: &[f64]) -> Vec<f64> {
    let m = p_values.len();
    if m == 0 {
        return vec![];
    }

    let mut indexed: Vec<(usize, f64)> = p_values.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let m_f64 = m as f64;
    let mut q_sorted: Vec<f64> = vec![0.0; m];

    q_sorted[m - 1] = indexed[m - 1].1.min(1.0);

    for i in (0..m - 1).rev() {
        let rank = (i + 1) as f64;
        let adjusted = indexed[i].1 * m_f64 / rank;
        q_sorted[i] = adjusted.min(q_sorted[i + 1]).min(1.0);
    }

    let mut q_values = vec![0.0; m];
    for (sorted_idx, &(orig_idx, _)) in indexed.iter().enumerate() {
        q_values[orig_idx] = q_sorted[sorted_idx];
    }

    q_values
}

/// Batch rejection at a given FDR level.
///
/// Returns indices of rejected hypotheses.
///
/// # Complexity
///
/// - **Time**: O(n log n)
/// - **Space**: O(n)
#[must_use]
pub fn bh_reject(p_values: &[f64], fdr_level: f64) -> Vec<usize> {
    let result = bh_adjust(p_values, fdr_level);
    result
        .rejected
        .iter()
        .enumerate()
        .filter(|(_, r)| **r)
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bh_basic() {
        // Test BH procedure
        // p-values sorted: 0.001, 0.008, 0.039, 0.041, 0.042, 0.06, 0.074, 0.205, 0.212, 0.216
        // BH threshold for rank i (m=10, α=0.05): i/10 × 0.05
        // Rank 1: 0.001 ≤ 0.005 ✓
        // Rank 2: 0.008 ≤ 0.01 ✓
        // Rank 3: 0.039 > 0.015 ✗
        // So largest k where p_(k) ≤ (k/m)×α is k=2
        let p_values = vec![
            0.001, 0.008, 0.039, 0.041, 0.042, 0.06, 0.074, 0.205, 0.212, 0.216,
        ];
        let result = bh_adjust(&p_values, 0.05);

        // First 2 should be rejected at FDR = 0.05
        assert!(result.rejected[0]); // p=0.001
        assert!(result.rejected[1]); // p=0.008
        assert!(!result.rejected[2]); // p=0.039 - not rejected (0.039 > 0.015)

        assert_eq!(result.n_rejected, 2);
    }

    #[test]
    fn test_bh_all_significant() {
        let p_values = vec![0.001, 0.002, 0.003, 0.004, 0.005];
        let result = bh_adjust(&p_values, 0.05);

        assert!(result.rejected.iter().all(|&r| r));
        assert_eq!(result.n_rejected, 5);
    }

    #[test]
    fn test_bh_none_significant() {
        let p_values = vec![0.5, 0.6, 0.7, 0.8, 0.9];
        let result = bh_adjust(&p_values, 0.05);

        assert!(result.rejected.iter().all(|&r| !r));
        assert_eq!(result.n_rejected, 0);
        assert!(result.critical_value.is_none());
    }

    #[test]
    fn test_bh_empty_input() {
        let p_values: Vec<f64> = vec![];
        let result = bh_adjust(&p_values, 0.05);

        assert!(result.p_values.is_empty());
        assert!(result.q_values.is_empty());
        assert_eq!(result.n_rejected, 0);
    }

    #[test]
    fn test_bh_single_value() {
        let p_values = vec![0.03];
        let result = bh_adjust(&p_values, 0.05);

        assert!(result.rejected[0]);
        assert_eq!(result.q_values[0], 0.03);
    }

    #[test]
    fn test_q_values_monotonic() {
        let p_values = vec![0.01, 0.02, 0.03, 0.04, 0.05];
        let q_values = compute_q_values(&p_values);

        // Q-values should be monotonically increasing when p-values are sorted
        for i in 1..q_values.len() {
            assert!(
                q_values[i] >= q_values[i - 1],
                "Q-values should be monotonically non-decreasing"
            );
        }
    }

    #[test]
    fn test_q_values_bounded() {
        let p_values = vec![0.5, 0.6, 0.7, 0.8, 0.99];
        let q_values = compute_q_values(&p_values);

        // Q-values should never exceed 1.0
        for &q in &q_values {
            assert!(q <= 1.0, "Q-value {} exceeds 1.0", q);
        }
    }

    #[test]
    fn test_bh_reject_indices() {
        let p_values = vec![0.001, 0.5, 0.002, 0.6, 0.003];
        let rejected = bh_reject(&p_values, 0.05);

        assert_eq!(rejected.len(), 3);
        assert!(rejected.contains(&0));
        assert!(rejected.contains(&2));
        assert!(rejected.contains(&4));
    }

    #[test]
    fn test_preserves_original_order() {
        let p_values = vec![0.5, 0.001, 0.3, 0.002];
        let result = bh_adjust(&p_values, 0.05);

        // Original p-values should be preserved in order
        assert_eq!(result.p_values, p_values);

        // Rejection should correspond to correct positions
        assert!(!result.rejected[0]); // 0.5 - not rejected
        assert!(result.rejected[1]); // 0.001 - rejected
        assert!(!result.rejected[2]); // 0.3 - not rejected
        assert!(result.rejected[3]); // 0.002 - rejected
    }
}
