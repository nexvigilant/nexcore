//! Multiplicity adjustment procedures per ICH E9(R1) §2.2.5.
//!
//! Tier: T2-P (κ+N+∂ — Multiplicity Correction)
//!
//! Implements: Bonferroni, Holm (step-down), Hochberg (step-up), Benjamini-Hochberg (FDR).

use crate::types::AdjustedResult;

// ── Bonferroni ────────────────────────────────────────────────────────────────

/// Bonferroni correction — divide alpha by number of tests.
///
/// Most conservative; controls FWER strongly. Each hypothesis is tested at `α/m`.
///
/// # Arguments
/// - `p_values`: Slice of raw p-values (one per hypothesis)
/// - `alpha`: Desired family-wise error rate
pub fn bonferroni_adjust(p_values: &[f64], alpha: f64) -> Vec<AdjustedResult> {
    let m = p_values.len();
    let threshold = alpha / m as f64;
    p_values
        .iter()
        .map(|&p| AdjustedResult {
            original_p: p,
            adjusted_threshold: threshold,
            significant: p < threshold,
            method: "Bonferroni".into(),
        })
        .collect()
}

// ── Holm (Step-Down) ─────────────────────────────────────────────────────────

/// Holm step-down procedure — uniformly more powerful than Bonferroni.
///
/// Sort p-values ascending. The k-th smallest is compared to `α / (m - k + 1)`.
/// Once we fail to reject, all remaining are also non-significant.
///
/// # Arguments
/// - `p_values`: Slice of raw p-values
/// - `alpha`: Family-wise error rate
pub fn holm_adjust(p_values: &[f64], alpha: f64) -> Vec<AdjustedResult> {
    let m = p_values.len();
    // Create sorted index pairs (value, original_index)
    let mut indexed: Vec<(f64, usize)> = p_values
        .iter()
        .copied()
        .enumerate()
        .map(|(i, p)| (p, i))
        .collect();
    indexed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut results = vec![
        AdjustedResult {
            original_p: 0.0,
            adjusted_threshold: 0.0,
            significant: false,
            method: "Holm".into(),
        };
        m
    ];

    let mut rejected_so_far = true;
    for (rank, &(p, orig_idx)) in indexed.iter().enumerate() {
        let threshold = alpha / (m - rank) as f64;
        let significant = rejected_so_far && p < threshold;
        if !significant {
            rejected_so_far = false;
        }
        results[orig_idx] = AdjustedResult {
            original_p: p,
            adjusted_threshold: threshold,
            significant,
            method: "Holm".into(),
        };
    }
    results
}

// ── Hochberg (Step-Up) ────────────────────────────────────────────────────────

/// Hochberg step-up procedure — controls FWER, more powerful than Holm under independence.
///
/// Sort p-values descending. Accept all hypotheses at or below the first one that
/// satisfies `p_k ≤ α / (m - k + 1)` (counting from largest).
///
/// # Arguments
/// - `p_values`: Slice of raw p-values
/// - `alpha`: Family-wise error rate
pub fn hochberg_adjust(p_values: &[f64], alpha: f64) -> Vec<AdjustedResult> {
    let m = p_values.len();
    let mut indexed: Vec<(f64, usize)> = p_values
        .iter()
        .copied()
        .enumerate()
        .map(|(i, p)| (p, i))
        .collect();
    indexed.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)); // descending

    let mut results = vec![
        AdjustedResult {
            original_p: 0.0,
            adjusted_threshold: 0.0,
            significant: false,
            method: "Hochberg".into(),
        };
        m
    ];

    let mut accept_remaining = false;
    for (rank, &(p, orig_idx)) in indexed.iter().enumerate() {
        // rank 0 = largest p-value, threshold = α/1; rank m-1 = smallest, threshold = α/m
        let threshold = alpha / (rank + 1) as f64;
        let significant = accept_remaining || p <= threshold;
        if significant {
            accept_remaining = true;
        }
        results[orig_idx] = AdjustedResult {
            original_p: p,
            adjusted_threshold: threshold,
            significant,
            method: "Hochberg".into(),
        };
    }
    results
}

// ── Benjamini-Hochberg (FDR) ──────────────────────────────────────────────────

/// Benjamini-Hochberg procedure — controls False Discovery Rate (FDR).
///
/// Less conservative than FWER methods; preferred for exploratory analyses.
/// Sort p-values ascending. The k-th sorted p-value is compared to `k * α / m`.
///
/// # Arguments
/// - `p_values`: Slice of raw p-values
/// - `alpha`: Desired false discovery rate
pub fn benjamini_hochberg_adjust(p_values: &[f64], alpha: f64) -> Vec<AdjustedResult> {
    let m = p_values.len();
    let mut indexed: Vec<(f64, usize)> = p_values
        .iter()
        .copied()
        .enumerate()
        .map(|(i, p)| (p, i))
        .collect();
    indexed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Find the largest k such that p_(k) ≤ k * alpha / m
    let mut max_sig_rank: Option<usize> = None;
    for (rank, &(p, _)) in indexed.iter().enumerate() {
        let threshold = (rank + 1) as f64 * alpha / m as f64;
        if p <= threshold {
            max_sig_rank = Some(rank);
        }
    }

    let mut results = vec![
        AdjustedResult {
            original_p: 0.0,
            adjusted_threshold: 0.0,
            significant: false,
            method: "Benjamini-Hochberg".into(),
        };
        m
    ];

    for (rank, &(p, orig_idx)) in indexed.iter().enumerate() {
        let threshold = (rank + 1) as f64 * alpha / m as f64;
        let significant = max_sig_rank.map_or(false, |max| rank <= max);
        results[orig_idx] = AdjustedResult {
            original_p: p,
            adjusted_threshold: threshold,
            significant,
            method: "Benjamini-Hochberg".into(),
        };
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bonferroni() {
        // With 3 tests at alpha=0.05, threshold = 0.0167
        let adjusted = bonferroni_adjust(&[0.01, 0.03, 0.04], 0.05);
        assert!(
            adjusted[0].significant,
            "0.01 < 0.0167 — should be significant"
        );
        assert!(
            !adjusted[1].significant,
            "0.03 > 0.0167 — should not be significant"
        );
        assert!(
            !adjusted[2].significant,
            "0.04 > 0.0167 — should not be significant"
        );
    }

    #[test]
    fn test_bonferroni_threshold() {
        let adjusted = bonferroni_adjust(&[0.01, 0.03, 0.04], 0.05);
        let expected = 0.05 / 3.0;
        for r in &adjusted {
            assert!((r.adjusted_threshold - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn test_holm_stepdown() {
        // Holm is less conservative than Bonferroni.
        // p=[0.01, 0.02, 0.04]: Holm finds all 3 significant; Bonferroni finds only 1.
        // Holm thresholds for m=3: rank1→0.05/3=0.0167, rank2→0.05/2=0.025, rank3→0.05/1=0.05
        let adjusted = holm_adjust(&[0.01, 0.02, 0.04], 0.05);
        assert!(
            adjusted[0].significant,
            "0.01 < 0.0167 should be significant by Holm"
        );
        assert!(
            adjusted[1].significant,
            "0.02 < 0.025 should be significant by Holm"
        );
    }

    #[test]
    fn test_holm_stops_at_first_nonsig() {
        // 0.01 sig, 0.04 not sig, then 0.05 also not sig (step-down stops)
        let adjusted = holm_adjust(&[0.01, 0.04, 0.05], 0.05);
        assert!(adjusted[0].significant);
        assert!(!adjusted[1].significant);
        assert!(!adjusted[2].significant);
    }

    #[test]
    fn test_hochberg_step_up() {
        let adjusted = hochberg_adjust(&[0.01, 0.03, 0.04], 0.05);
        // All three significant since smallest p=0.01 < 0.05/3
        assert!(adjusted[0].significant);
    }

    #[test]
    fn test_benjamini_hochberg_fdr() {
        // BH at alpha=0.05 for 3 tests
        // Sorted: 0.01 ≤ 1*0.05/3=0.0167 sig, 0.03 ≤ 2*0.05/3=0.0333 sig, 0.04 ≤ 3*0.05/3=0.05 sig
        let adjusted = benjamini_hochberg_adjust(&[0.01, 0.03, 0.04], 0.05);
        assert!(adjusted[0].significant);
        assert!(adjusted[1].significant);
        assert!(adjusted[2].significant);
    }

    #[test]
    fn test_bh_vs_bonferroni_power() {
        // BH should flag more results than Bonferroni for borderline p-values
        let p_values = [0.01, 0.03, 0.04];
        let bonferroni = bonferroni_adjust(&p_values, 0.05);
        let bh = benjamini_hochberg_adjust(&p_values, 0.05);
        let sig_bonf: usize = bonferroni.iter().filter(|r| r.significant).count();
        let sig_bh: usize = bh.iter().filter(|r| r.significant).count();
        assert!(
            sig_bh >= sig_bonf,
            "BH should have >= significant results vs Bonferroni"
        );
    }

    #[test]
    fn test_all_methods_preserve_length() {
        let p = vec![0.01, 0.02, 0.05, 0.10];
        assert_eq!(bonferroni_adjust(&p, 0.05).len(), 4);
        assert_eq!(holm_adjust(&p, 0.05).len(), 4);
        assert_eq!(hochberg_adjust(&p, 0.05).len(), 4);
        assert_eq!(benjamini_hochberg_adjust(&p, 0.05).len(), 4);
    }
}
