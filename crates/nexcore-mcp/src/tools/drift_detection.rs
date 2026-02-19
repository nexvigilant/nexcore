//! Statistical Drift Detection — KS test, PSI, Jensen-Shannon divergence.
//!
//! Inspired by AI Engineering Bible Section 32 (Model Monitoring & Drift Detection):
//! provides formal statistical tests to detect concept drift, data drift, and
//! label shift in production AI systems.
//!
//! # Algorithms
//!
//! 1. **KS Test** (Kolmogorov-Smirnov): Non-parametric test comparing two empirical
//!    CDFs. Returns D-statistic (max CDF difference) and approximate p-value.
//!
//! 2. **PSI** (Population Stability Index): Measures population shift using
//!    binned distributions. Industry thresholds: <0.1 (stable), 0.1-0.25 (moderate),
//!    >0.25 (significant drift).
//!
//! 3. **JSD** (Jensen-Shannon Divergence): Symmetric divergence between two
//!    probability distributions. Bounded [0, 1] in bits.
//!
//! # T1 Grounding: ν(Frequency) + κ(Comparison) + ∂(Boundary) + N(Quantity)
//! - ν: Distribution frequency counting
//! - κ: Statistical comparison between distributions
//! - ∂: Significance boundaries and drift thresholds
//! - N: Numeric test statistics

use crate::params::{DriftDetectParams, DriftJsdParams, DriftKsTestParams, DriftPsiParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ============================================================================
// Statistical Helpers
// ============================================================================

/// Compute the empirical CDF of a sorted sample at a given value.
fn ecdf(sorted: &[f64], x: f64) -> f64 {
    let count = sorted.partition_point(|v| *v <= x);
    count as f64 / sorted.len() as f64
}

/// Kolmogorov-Smirnov D-statistic between two samples.
fn ks_statistic(ref_sorted: &[f64], cur_sorted: &[f64]) -> f64 {
    // Merge all unique values and compute max CDF difference
    let mut all_values: Vec<f64> = Vec::with_capacity(ref_sorted.len() + cur_sorted.len());
    all_values.extend_from_slice(ref_sorted);
    all_values.extend_from_slice(cur_sorted);
    all_values.sort_by(|a, b| a.total_cmp(b));
    all_values.dedup();

    let mut d_max = 0.0_f64;
    for &x in &all_values {
        let cdf_ref = ecdf(ref_sorted, x);
        let cdf_cur = ecdf(cur_sorted, x);
        let d = (cdf_ref - cdf_cur).abs();
        if d > d_max {
            d_max = d;
        }
    }
    d_max
}

/// Approximate p-value for KS test using the asymptotic formula.
/// Uses the Kolmogorov distribution approximation.
fn ks_pvalue(d: f64, n1: usize, n2: usize) -> f64 {
    let n_eff = (n1 as f64 * n2 as f64) / (n1 as f64 + n2 as f64);
    let lambda = (n_eff.sqrt() + 0.12 + 0.11 / n_eff.sqrt()) * d;

    if lambda < 0.001 {
        return 1.0;
    }

    // Asymptotic series: P(D > d) ≈ 2 * sum_{k=1}^{inf} (-1)^{k+1} * exp(-2k^2*lambda^2)
    let mut p = 0.0;
    let lambda_sq = lambda * lambda;
    for k in 1..=100 {
        let k_f = k as f64;
        let term = (-1.0_f64).powi(k as i32 + 1) * (-2.0 * k_f * k_f * lambda_sq).exp();
        p += term;
        if term.abs() < 1e-12 {
            break;
        }
    }
    (2.0 * p).clamp(0.0, 1.0)
}

/// Bin raw data into histogram proportions.
fn bin_data(data: &[f64], n_bins: usize) -> Vec<f64> {
    if data.is_empty() || n_bins == 0 {
        return vec![0.0; n_bins.max(1)];
    }

    let min_val = data.iter().copied().fold(f64::INFINITY, f64::min);
    let max_val = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    if (max_val - min_val).abs() < f64::EPSILON {
        let mut bins = vec![0.0; n_bins];
        bins[0] = 1.0;
        return bins;
    }

    let bin_width = (max_val - min_val) / n_bins as f64;
    let mut counts = vec![0_u64; n_bins];

    for &val in data {
        let idx = ((val - min_val) / bin_width) as usize;
        let idx = idx.min(n_bins - 1);
        counts[idx] += 1;
    }

    let total = data.len() as f64;
    counts.iter().map(|&c| c as f64 / total).collect()
}

/// Compute PSI from two bin proportion vectors.
fn psi_from_bins(reference: &[f64], current: &[f64]) -> f64 {
    let eps = 1e-6;
    reference
        .iter()
        .zip(current.iter())
        .map(|(&r, &c)| {
            let r = r.max(eps);
            let c = c.max(eps);
            (c - r) * (c / r).ln()
        })
        .sum()
}

/// KL divergence: D_KL(P || Q) = sum(p * ln(p/q))
fn kl_divergence(p: &[f64], q: &[f64]) -> f64 {
    let eps = 1e-12;
    p.iter()
        .zip(q.iter())
        .map(|(&pi, &qi)| {
            let pi = pi.max(eps);
            let qi = qi.max(eps);
            pi * (pi / qi).ln()
        })
        .sum()
}

/// Jensen-Shannon Divergence: JSD(P, Q) = 0.5 * D_KL(P || M) + 0.5 * D_KL(Q || M)
/// where M = 0.5 * (P + Q)
fn jsd(p: &[f64], q: &[f64], base2: bool) -> f64 {
    let m: Vec<f64> = p
        .iter()
        .zip(q.iter())
        .map(|(&pi, &qi)| 0.5 * (pi + qi))
        .collect();

    let jsd_nat = 0.5 * kl_divergence(p, &m) + 0.5 * kl_divergence(q, &m);

    if base2 {
        jsd_nat / 2.0_f64.ln() // Convert nats to bits
    } else {
        jsd_nat
    }
}

/// Normalize a distribution to sum to 1.0.
fn normalize_dist(dist: &[f64]) -> Vec<f64> {
    let sum: f64 = dist.iter().sum();
    if sum > 0.0 {
        dist.iter().map(|&v| v / sum).collect()
    } else {
        vec![1.0 / dist.len() as f64; dist.len()]
    }
}

// ============================================================================
// MCP Tools
// ============================================================================

/// `drift_ks_test` — Kolmogorov-Smirnov two-sample test for distribution shift.
///
/// Compares reference and current sample distributions. Returns D-statistic
/// (max CDF difference) and approximate p-value. Drift detected if p < alpha.
pub fn drift_ks_test(params: DriftKsTestParams) -> Result<CallToolResult, McpError> {
    if params.reference.is_empty() || params.current.is_empty() {
        return Err(McpError::invalid_params(
            "Both reference and current must have at least one value".to_string(),
            None,
        ));
    }

    let alpha = params.alpha.unwrap_or(0.05);

    let mut ref_sorted = params.reference.clone();
    ref_sorted.sort_by(|a, b| a.total_cmp(b));
    let mut cur_sorted = params.current.clone();
    cur_sorted.sort_by(|a, b| a.total_cmp(b));

    let d = ks_statistic(&ref_sorted, &cur_sorted);
    let p = ks_pvalue(d, params.reference.len(), params.current.len());
    let drift_detected = p < alpha;

    // Critical value approximation: c(alpha) * sqrt((n1+n2)/(n1*n2))
    let n1 = params.reference.len() as f64;
    let n2 = params.current.len() as f64;
    let c_alpha = match alpha {
        a if a <= 0.01 => 1.628,
        a if a <= 0.05 => 1.358,
        a if a <= 0.10 => 1.224,
        _ => 1.073,
    };
    let d_critical = c_alpha * ((n1 + n2) / (n1 * n2)).sqrt();

    let ref_mean: f64 = params.reference.iter().sum::<f64>() / n1;
    let cur_mean: f64 = params.current.iter().sum::<f64>() / n2;
    let ref_std = (params
        .reference
        .iter()
        .map(|x| (x - ref_mean).powi(2))
        .sum::<f64>()
        / n1)
        .sqrt();
    let cur_std = (params
        .current
        .iter()
        .map(|x| (x - cur_mean).powi(2))
        .sum::<f64>()
        / n2)
        .sqrt();

    let result = json!({
        "test": "kolmogorov_smirnov_two_sample",
        "d_statistic": (d * 10000.0).round() / 10000.0,
        "d_critical": (d_critical * 10000.0).round() / 10000.0,
        "p_value": (p * 10000.0).round() / 10000.0,
        "alpha": alpha,
        "drift_detected": drift_detected,
        "verdict": if drift_detected { "DRIFT" } else { "STABLE" },
        "reference": {
            "n": params.reference.len(),
            "mean": (ref_mean * 1000.0).round() / 1000.0,
            "std": (ref_std * 1000.0).round() / 1000.0,
        },
        "current": {
            "n": params.current.len(),
            "mean": (cur_mean * 1000.0).round() / 1000.0,
            "std": (cur_std * 1000.0).round() / 1000.0,
        },
        "interpretation": if drift_detected {
            format!("D={:.4} > D_critical={:.4} (p={:.4} < alpha={:.2}): distributions differ significantly", d, d_critical, p, alpha)
        } else {
            format!("D={:.4} <= D_critical={:.4} (p={:.4} >= alpha={:.2}): no significant difference", d, d_critical, p, alpha)
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `drift_psi` — Population Stability Index for distribution shift.
///
/// PSI thresholds (industry standard):
/// - < 0.1: No significant shift
/// - 0.1 - 0.25: Moderate shift (investigate)
/// - > 0.25: Significant shift (action required)
pub fn drift_psi(params: DriftPsiParams) -> Result<CallToolResult, McpError> {
    let n_bins = params.bins.unwrap_or(10);
    let raw = params.raw_data.unwrap_or(false);

    let (ref_bins, cur_bins) = if raw {
        if params.reference.is_empty() || params.current.is_empty() {
            return Err(McpError::invalid_params(
                "Both reference and current must have values for raw data mode".to_string(),
                None,
            ));
        }
        (
            bin_data(&params.reference, n_bins),
            bin_data(&params.current, n_bins),
        )
    } else {
        if params.reference.len() != params.current.len() {
            return Err(McpError::invalid_params(
                "Reference and current bin counts must match".to_string(),
                None,
            ));
        }
        (
            normalize_dist(&params.reference),
            normalize_dist(&params.current),
        )
    };

    let psi = psi_from_bins(&ref_bins, &cur_bins);

    let severity = if psi < 0.1 {
        "stable"
    } else if psi < 0.25 {
        "moderate"
    } else {
        "significant"
    };

    let verdict = if psi < 0.1 {
        "NO_DRIFT"
    } else if psi < 0.25 {
        "MODERATE_DRIFT"
    } else {
        "SIGNIFICANT_DRIFT"
    };

    // Per-bin contributions
    let eps = 1e-6;
    let bin_details: Vec<serde_json::Value> = ref_bins
        .iter()
        .zip(cur_bins.iter())
        .enumerate()
        .map(|(i, (&r, &c))| {
            let r_safe = r.max(eps);
            let c_safe = c.max(eps);
            let contribution = (c_safe - r_safe) * (c_safe / r_safe).ln();
            json!({
                "bin": i,
                "reference": (r * 10000.0).round() / 10000.0,
                "current": (c * 10000.0).round() / 10000.0,
                "psi_contribution": (contribution * 10000.0).round() / 10000.0,
            })
        })
        .collect();

    let result = json!({
        "test": "population_stability_index",
        "psi": (psi * 10000.0).round() / 10000.0,
        "severity": severity,
        "verdict": verdict,
        "thresholds": {
            "stable": "< 0.1",
            "moderate": "0.1 - 0.25",
            "significant": "> 0.25",
        },
        "bins": n_bins,
        "bin_details": bin_details,
        "interpretation": format!("PSI={:.4}: {} population shift", psi, severity),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `drift_jsd` — Jensen-Shannon Divergence between two distributions.
///
/// Symmetric measure bounded [0, 1] in bits. Values near 0 indicate similar
/// distributions; near 1 indicates maximally different.
pub fn drift_jsd(params: DriftJsdParams) -> Result<CallToolResult, McpError> {
    if params.p.len() != params.q.len() {
        return Err(McpError::invalid_params(
            "Distributions P and Q must have the same length".to_string(),
            None,
        ));
    }
    if params.p.is_empty() {
        return Err(McpError::invalid_params(
            "Distributions must not be empty".to_string(),
            None,
        ));
    }

    let base2 = params.base2.unwrap_or(true);

    let p = normalize_dist(&params.p);
    let q = normalize_dist(&params.q);

    let jsd_val = jsd(&p, &q, base2);
    let jsd_sqrt = jsd_val.sqrt(); // Jensen-Shannon distance

    let severity = if jsd_val < 0.05 {
        "minimal"
    } else if jsd_val < 0.2 {
        "moderate"
    } else if jsd_val < 0.5 {
        "substantial"
    } else {
        "severe"
    };

    let result = json!({
        "test": "jensen_shannon_divergence",
        "jsd": (jsd_val * 100000.0).round() / 100000.0,
        "js_distance": (jsd_sqrt * 100000.0).round() / 100000.0,
        "unit": if base2 { "bits" } else { "nats" },
        "max_value": if base2 { 1.0 } else { 2.0_f64.ln() },
        "severity": severity,
        "interpretation": format!(
            "JSD={:.5} {}: {} divergence between distributions",
            jsd_val,
            if base2 { "bits" } else { "nats" },
            severity
        ),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `drift_detect` — Composite drift detection running KS + PSI + JSD.
///
/// Provides a unified drift verdict combining all three statistical tests.
pub fn drift_detect(params: DriftDetectParams) -> Result<CallToolResult, McpError> {
    if params.reference.is_empty() || params.current.is_empty() {
        return Err(McpError::invalid_params(
            "Both reference and current must have values".to_string(),
            None,
        ));
    }

    let alpha = params.alpha.unwrap_or(0.05);
    let psi_threshold = params.psi_threshold.unwrap_or(0.25);
    let n_bins = params.bins.unwrap_or(10);

    // KS Test
    let mut ref_sorted = params.reference.clone();
    ref_sorted.sort_by(|a, b| a.total_cmp(b));
    let mut cur_sorted = params.current.clone();
    cur_sorted.sort_by(|a, b| a.total_cmp(b));

    let d = ks_statistic(&ref_sorted, &cur_sorted);
    let p = ks_pvalue(d, params.reference.len(), params.current.len());
    let ks_drift = p < alpha;

    // PSI
    let ref_bins = bin_data(&params.reference, n_bins);
    let cur_bins = bin_data(&params.current, n_bins);
    let psi = psi_from_bins(&ref_bins, &cur_bins);
    let psi_drift = psi >= psi_threshold;

    // JSD
    let p_dist = normalize_dist(&ref_bins);
    let q_dist = normalize_dist(&cur_bins);
    let jsd_val = jsd(&p_dist, &q_dist, true);

    // Unified verdict: drift if 2+ tests agree
    let drift_signals = [ks_drift, psi_drift, jsd_val > 0.2];
    let drift_count = drift_signals.iter().filter(|&&x| x).count();

    let verdict = match drift_count {
        0 => "STABLE",
        1 => "WATCH",
        2 => "DRIFT_LIKELY",
        _ => "DRIFT_CONFIRMED",
    };

    let severity = match drift_count {
        0 => "none",
        1 => "low",
        2 => "medium",
        _ => "high",
    };

    let recommended_action = match drift_count {
        0 => "No action needed. Continue monitoring.",
        1 => "Minor anomaly detected. Increase monitoring frequency.",
        2 => "Likely drift. Investigate data pipeline and feature distributions.",
        _ => "Confirmed drift. Trigger retraining pipeline or rollback to previous model.",
    };

    let result = json!({
        "composite_drift_detection": {
            "verdict": verdict,
            "severity": severity,
            "drift_signals": drift_count,
            "total_tests": 3,
            "recommended_action": recommended_action,
        },
        "ks_test": {
            "d_statistic": (d * 10000.0).round() / 10000.0,
            "p_value": (p * 10000.0).round() / 10000.0,
            "alpha": alpha,
            "drift": ks_drift,
        },
        "psi": {
            "value": (psi * 10000.0).round() / 10000.0,
            "threshold": psi_threshold,
            "drift": psi_drift,
        },
        "jsd": {
            "value": (jsd_val * 100000.0).round() / 100000.0,
            "unit": "bits",
            "drift": jsd_val > 0.2,
        },
        "sample_sizes": {
            "reference": params.reference.len(),
            "current": params.current.len(),
        },
        "bins": n_bins,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
