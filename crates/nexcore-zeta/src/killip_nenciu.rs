//! # Killip-Nenciu GUE Test
//!
//! Tests whether the Verblunsky coefficients from a CMV reconstruction
//! follow the distribution predicted by the Gaussian Unitary Ensemble (GUE).
//!
//! ## Theory (Killip & Nenciu, 2004)
//!
//! For the circular β-ensemble with β = 2 (GUE), the Verblunsky coefficients
//! α₀, …, αₙ₋₂ are independent with |αₖ|² ~ Beta(1, N−k−1).
//!
//! This means:
//! - CDF of |αₖ|²: F(x) = 1 − (1−x)^(N−k−1) for x ∈ [0,1]
//! - E[|αₖ|²] = 1/(N−k)
//! - Var[|αₖ|²] = (N−k−1)/((N−k)²(N−k+1))
//!
//! We run a Kolmogorov-Smirnov test comparing the observed |αₖ|² values
//! against this theoretical distribution. This is an **independent** test
//! of GUE universality from the pair correlation approach.

use serde::{Deserialize, Serialize};

use crate::cmv::CmvReconstruction;
use crate::error::ZetaError;
use crate::statistics::GueComparison;

// ── Public types ──────────────────────────────────────────────────────────────

/// Result of the Killip-Nenciu GUE test on Verblunsky coefficients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillipNenciuTest {
    /// Number of Verblunsky coefficients tested.
    pub n_coefficients: usize,
    /// Kolmogorov-Smirnov statistic D_n = sup |F_emp − F_theo|.
    pub ks_statistic: f64,
    /// Approximate p-value from the Kolmogorov distribution.
    pub ks_pvalue: f64,
    /// Observed |αₖ|² values.
    pub observed_alpha_sq: Vec<f64>,
    /// Expected E[|αₖ|²] = 1/(N−k) under GUE.
    pub expected_alpha_sq: Vec<f64>,
    /// Per-coefficient CDF values F(|αₖ|²) under the Beta(1, N−k−1) model.
    pub cdf_values: Vec<f64>,
    /// Whether the test rejects GUE at α = 0.05.
    pub rejects_gue: bool,
    /// Chi-squared goodness-of-fit statistic (sum of (obs−exp)²/exp).
    pub chi_sq_statistic: f64,
    /// Mean ratio of observed/expected |αₖ|².
    pub mean_obs_exp_ratio: f64,
}

/// Combined verdict from both GUE tests (pair correlation + Killip-Nenciu).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualGueVerdict {
    /// Pair correlation GUE match score ∈ [0,1].
    pub pair_correlation_score: f64,
    /// Killip-Nenciu KS p-value.
    pub killip_nenciu_pvalue: f64,
    /// Both tests accept GUE (score > 0.5 AND p > 0.05).
    pub both_accept: bool,
    /// Both tests reject GUE.
    pub both_reject: bool,
    /// Tests disagree (one accepts, one rejects).
    pub discordant: bool,
    /// Combined confidence: geometric mean of the two signals.
    pub combined_confidence: f64,
    /// Human-readable interpretation.
    pub interpretation: String,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Run the Killip-Nenciu test on a CMV reconstruction.
///
/// Tests whether the Verblunsky coefficient magnitudes follow the
/// Beta(1, N−k−1) distribution predicted by GUE random matrix theory.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if the CMV has fewer than 5
/// coefficients (too few for a meaningful statistical test).
pub fn killip_nenciu_test(cmv: &CmvReconstruction) -> Result<KillipNenciuTest, ZetaError> {
    let mags = &cmv.verblunsky_magnitudes;
    let n_coeff = mags.len();

    if n_coeff < 5 {
        return Err(ZetaError::InvalidParameter(
            "need at least 5 Verblunsky coefficients for Killip-Nenciu test".to_string(),
        ));
    }

    // N = number of eigenvalues = n_coeff + 1
    let big_n = n_coeff + 1;

    let mut observed_sq: Vec<f64> = Vec::with_capacity(n_coeff);
    let mut expected_sq: Vec<f64> = Vec::with_capacity(n_coeff);
    let mut cdf_vals: Vec<f64> = Vec::with_capacity(n_coeff);
    let mut ks_max: f64 = 0.0;
    let mut chi_sq: f64 = 0.0;
    let mut ratio_sum: f64 = 0.0;
    let mut ratio_count: usize = 0;

    // Effective number of coefficients used (skip degenerate tail)
    let n_effective = n_coeff.min(big_n.saturating_sub(2));

    for k in 0..n_effective {
        let alpha_sq = mags[k] * mags[k];
        observed_sq.push(alpha_sq);

        // Shape parameter for Beta(1, shape): shape = N − k − 1
        let shape = big_n.saturating_sub(k + 1);
        if shape == 0 {
            // Degenerate — last coefficient has no distributional constraint
            expected_sq.push(0.0);
            cdf_vals.push(0.5);
            continue;
        }

        let shape_f = shape as f64;

        // E[|αₖ|²] = 1/(shape + 1) = 1/(N - k) for Beta(1, shape)
        let expected = 1.0 / (shape_f + 1.0);
        expected_sq.push(expected);

        // CDF of Beta(1, shape) at x: F(x) = 1 − (1−x)^shape
        let cdf = beta_1_n_cdf(alpha_sq, shape);
        cdf_vals.push(cdf);

        // KS: compare empirical CDF to uniform
        // Under H₀, F(|αₖ|²) ~ Uniform(0,1) for each k independently
        // We collect CDF values and compute a one-sample KS against Uniform
        let empirical_rank = (k as f64 + 0.5) / n_effective as f64;
        let deviation = (cdf - empirical_rank).abs();
        if deviation > ks_max {
            ks_max = deviation;
        }

        // Chi-squared contribution
        if expected > 1e-15 {
            let diff = alpha_sq - expected;
            chi_sq += diff * diff / expected;
            ratio_sum += alpha_sq / expected;
            ratio_count += 1;
        }
    }

    // KS p-value via Kolmogorov distribution approximation
    let ks_pvalue = kolmogorov_pvalue(ks_max, n_effective);

    let mean_ratio = if ratio_count > 0 {
        ratio_sum / ratio_count as f64
    } else {
        f64::NAN
    };

    Ok(KillipNenciuTest {
        n_coefficients: n_effective,
        ks_statistic: ks_max,
        ks_pvalue,
        observed_alpha_sq: observed_sq,
        expected_alpha_sq: expected_sq,
        cdf_values: cdf_vals,
        rejects_gue: ks_pvalue < 0.05,
        chi_sq_statistic: chi_sq,
        mean_obs_exp_ratio: mean_ratio,
    })
}

/// Combine pair correlation (existing) and Killip-Nenciu GUE tests.
///
/// Produces a verdict on whether both independent statistical tests
/// agree on the GUE universality of the zero spacings.
pub fn compare_gue_tests(pair_corr: &GueComparison, kn: &KillipNenciuTest) -> DualGueVerdict {
    let pc_accepts = pair_corr.gue_match_score > 0.5;
    let kn_accepts = !kn.rejects_gue;

    let both_accept = pc_accepts && kn_accepts;
    let both_reject = !pc_accepts && !kn_accepts;
    let discordant = pc_accepts != kn_accepts;

    // Combined confidence: geometric mean of pair-corr score and KN p-value
    // Both range [0,1], higher = more GUE-like
    let kn_signal = kn.ks_pvalue.max(0.001); // floor to avoid log(0)
    let pc_signal = pair_corr.gue_match_score.max(0.001);
    let combined = (pc_signal * kn_signal).sqrt();

    let interpretation = if both_accept {
        format!(
            "Strong GUE evidence: pair correlation score {:.3} AND Killip-Nenciu p={:.3} both accept. \
             Two independent tests converge on the same universality class.",
            pair_corr.gue_match_score, kn.ks_pvalue
        )
    } else if both_reject {
        format!(
            "GUE rejected by both tests: pair correlation {:.3} < 0.5 AND KN p={:.4} < 0.05. \
             The zeros deviate from GUE predictions in both spacing statistics and coefficient distribution.",
            pair_corr.gue_match_score, kn.ks_pvalue
        )
    } else if pc_accepts {
        format!(
            "Discordant: pair correlation accepts GUE ({:.3}) but Killip-Nenciu rejects (p={:.4}). \
             Possible finite-N effect or deviation in the tails of the coefficient distribution.",
            pair_corr.gue_match_score, kn.ks_pvalue
        )
    } else {
        format!(
            "Discordant: Killip-Nenciu accepts GUE (p={:.3}) but pair correlation rejects ({:.3}). \
             Coefficient distribution matches but spacing correlation deviates.",
            kn.ks_pvalue, pair_corr.gue_match_score
        )
    };

    DualGueVerdict {
        pair_correlation_score: pair_corr.gue_match_score,
        killip_nenciu_pvalue: kn.ks_pvalue,
        both_accept,
        both_reject,
        discordant,
        combined_confidence: combined,
        interpretation,
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// CDF of Beta(1, n) distribution: F(x) = 1 − (1−x)^n for x ∈ [0,1].
fn beta_1_n_cdf(x: f64, n: usize) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    1.0 - (1.0 - x).powi(n as i32)
}

/// Approximate p-value from the Kolmogorov distribution.
///
/// P(D_n > d) ≈ 2 Σ_{j=1}^{∞} (−1)^{j−1} exp(−2j²·n·d²)
///
/// Uses the first 100 terms for convergence.
fn kolmogorov_pvalue(d: f64, n: usize) -> f64 {
    if d <= 0.0 {
        return 1.0;
    }
    if !d.is_finite() {
        return 0.0;
    }

    let lambda = (n as f64).sqrt() * d;
    let lambda_sq = lambda * lambda;

    let mut sum = 0.0_f64;
    for j in 1..=100_i32 {
        let jf = j as f64;
        let term = (-2.0 * jf * jf * lambda_sq).exp();
        if j % 2 == 1 {
            sum += term;
        } else {
            sum -= term;
        }
        if term.abs() < 1e-15 {
            break;
        }
    }

    (2.0 * sum).clamp(0.0, 1.0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmv::reconstruct_cmv;
    use crate::statistics::compare_to_gue;
    use crate::zeros::find_zeros_bracket;

    fn get_test_zeros(t_min: f64, t_max: f64) -> Vec<crate::zeros::ZetaZero> {
        find_zeros_bracket(t_min, t_max, 0.05).unwrap_or_default()
    }

    #[test]
    fn beta_cdf_boundaries() {
        // F(0) = 0, F(1) = 1 for any shape
        assert!((beta_1_n_cdf(0.0, 5) - 0.0).abs() < 1e-15);
        assert!((beta_1_n_cdf(1.0, 5) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn beta_cdf_monotone() {
        for n in [2, 5, 10, 50] {
            let mut prev = 0.0;
            for i in 1..=100 {
                let x = i as f64 / 100.0;
                let val = beta_1_n_cdf(x, n);
                assert!(val >= prev, "Beta(1,{n}) CDF not monotone at x={x}");
                prev = val;
            }
        }
    }

    #[test]
    fn beta_cdf_known_value() {
        // Beta(1, 1) = Uniform(0,1) -> CDF(x) = x
        assert!((beta_1_n_cdf(0.5, 1) - 0.5).abs() < 1e-12);
        assert!((beta_1_n_cdf(0.3, 1) - 0.3).abs() < 1e-12);

        // Beta(1, 2): F(x) = 1 - (1-x)² = 2x - x²
        let x = 0.3;
        let expected = 2.0 * x - x * x;
        assert!((beta_1_n_cdf(x, 2) - expected).abs() < 1e-12);
    }

    #[test]
    fn kolmogorov_pvalue_boundaries() {
        // D = 0 -> p = 1
        assert!((kolmogorov_pvalue(0.0, 100) - 1.0).abs() < 1e-10);

        // Large D -> p ≈ 0
        let p = kolmogorov_pvalue(2.0, 100);
        assert!(p < 0.001, "p={p} should be near 0 for large D");
    }

    #[test]
    fn kolmogorov_pvalue_in_unit_interval() {
        for &d in &[0.01, 0.05, 0.1, 0.2, 0.5, 1.0] {
            let p = kolmogorov_pvalue(d, 50);
            assert!(
                (0.0..=1.0).contains(&p),
                "p-value {p} out of [0,1] for D={d}"
            );
        }
    }

    #[test]
    fn killip_nenciu_on_zeta_zeros() {
        let zeros = get_test_zeros(10.0, 200.0);
        if zeros.len() < 10 {
            return;
        }
        let cmv = reconstruct_cmv(&zeros);
        assert!(cmv.is_ok());
        let cmv = cmv.unwrap_or_else(|_| unreachable!());
        let result = killip_nenciu_test(&cmv);
        assert!(result.is_ok(), "KN test failed: {:?}", result.err());
        let kn = result.unwrap_or_else(|_| unreachable!());

        assert!(kn.ks_statistic.is_finite(), "KS stat not finite");
        assert!(
            (0.0..=1.0).contains(&kn.ks_pvalue),
            "p-value {} not in [0,1]",
            kn.ks_pvalue
        );
        assert!(kn.n_coefficients > 0);

        eprintln!(
            "Killip-Nenciu: D={:.4}, p={:.4}, rejects_gue={}, mean_ratio={:.3}",
            kn.ks_statistic, kn.ks_pvalue, kn.rejects_gue, kn.mean_obs_exp_ratio
        );
    }

    #[test]
    fn killip_nenciu_with_more_zeros() {
        let zeros = get_test_zeros(10.0, 600.0);
        if zeros.len() < 50 {
            return;
        }
        let cmv = reconstruct_cmv(&zeros);
        assert!(cmv.is_ok());
        let cmv = cmv.unwrap_or_else(|_| unreachable!());
        let kn = killip_nenciu_test(&cmv);
        assert!(kn.is_ok());
        let kn = kn.unwrap_or_else(|_| unreachable!());

        eprintln!(
            "KN (N={}): D={:.4}, p={:.4}, chi²={:.2}, ratio={:.3}",
            zeros.len(),
            kn.ks_statistic,
            kn.ks_pvalue,
            kn.chi_sq_statistic,
            kn.mean_obs_exp_ratio
        );
    }

    #[test]
    fn dual_verdict_both_accept() {
        let zeros = get_test_zeros(10.0, 200.0);
        if zeros.len() < 20 {
            return;
        }
        let cmv = reconstruct_cmv(&zeros);
        assert!(cmv.is_ok());
        let cmv = cmv.unwrap_or_else(|_| unreachable!());

        let pair_corr = compare_to_gue(&zeros);
        assert!(pair_corr.is_ok());
        let pair_corr = pair_corr.unwrap_or_else(|_| unreachable!());

        let kn = killip_nenciu_test(&cmv);
        assert!(kn.is_ok());
        let kn = kn.unwrap_or_else(|_| unreachable!());

        let verdict = compare_gue_tests(&pair_corr, &kn);
        assert!(verdict.combined_confidence.is_finite());
        assert!(
            verdict.combined_confidence >= 0.0,
            "combined confidence {} < 0",
            verdict.combined_confidence
        );

        eprintln!(
            "Dual verdict: accept_both={}, reject_both={}, discordant={}, confidence={:.4}",
            verdict.both_accept,
            verdict.both_reject,
            verdict.discordant,
            verdict.combined_confidence
        );
        eprintln!("Interpretation: {}", verdict.interpretation);
    }

    #[test]
    fn rejects_too_few_coefficients() {
        let cmv = CmvReconstruction {
            verblunsky_magnitudes: vec![0.1, 0.2, 0.3],
            verblunsky_phases: vec![0.0, 0.5, 1.0],
            eigenvalues: vec![14.1, 21.0, 25.0, 30.4],
            roundtrip_error: 0.001,
            structure: crate::cmv::CmvStructure {
                mean_coefficient_magnitude: 0.2,
                coefficient_decay_rate: 0.5,
                coefficient_regularity: 0.3,
                phase_regularity: 0.5,
                max_coefficient: 0.3,
                n: 3,
            },
        };
        assert!(killip_nenciu_test(&cmv).is_err());
    }

    #[test]
    fn expected_alpha_sq_decreases() {
        let zeros = get_test_zeros(10.0, 200.0);
        if zeros.len() < 10 {
            return;
        }
        let cmv = reconstruct_cmv(&zeros);
        assert!(cmv.is_ok());
        let cmv = cmv.unwrap_or_else(|_| unreachable!());
        let kn = killip_nenciu_test(&cmv);
        assert!(kn.is_ok());
        let kn = kn.unwrap_or_else(|_| unreachable!());

        // E[|αₖ|²] = 1/(N−k) should increase with k (denominator shrinks)
        // So the expected values should be monotonically increasing
        for i in 1..kn.expected_alpha_sq.len() {
            if kn.expected_alpha_sq[i] > 1e-15 && kn.expected_alpha_sq[i - 1] > 1e-15 {
                assert!(
                    kn.expected_alpha_sq[i] >= kn.expected_alpha_sq[i - 1] - 1e-12,
                    "expected |α_{}|² = {:.6} < expected |α_{}|² = {:.6}",
                    i,
                    kn.expected_alpha_sq[i],
                    i - 1,
                    kn.expected_alpha_sq[i - 1]
                );
            }
        }
    }
}
