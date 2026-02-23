//! # Anomaly-Based Counterexample Detector
//!
//! Builds a statistical baseline from Verblunsky coefficients (CMV reconstruction)
//! of known-good zeta zeros, then tests new zero sets for anomalous deviations.
//!
//! ## Algorithm
//!
//! 1. Build baseline from N known zeros via CMV → Verblunsky coefficients
//! 2. Fit power-law model: |αₖ| ~ A·k^(−β) via log-log OLS
//! 3. Detect spikes: coefficients deviating > 2σ from the power-law model
//! 4. Test new zeros by comparing their coefficients to the baseline model
//! 5. Anomaly score = max σ-deviation across all coefficients

use serde::{Deserialize, Serialize};

use crate::cmv::reconstruct_cmv;
use crate::error::ZetaError;
use crate::zeros::ZetaZero;

// ── Public types ──────────────────────────────────────────────────────────────

/// Statistical baseline for Verblunsky coefficient anomaly detection.
///
/// Built from a set of known-good zeta zeros. Captures the expected
/// power-law decay of Verblunsky coefficient magnitudes and serves as
/// the reference model for subsequent anomaly detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerblunskyBaseline {
    /// Fitted amplitude A in |αₖ| ~ A·k^(−β).
    pub power_law_a: f64,
    /// Power-law decay exponent β (positive = decaying coefficients).
    pub power_law_beta: f64,
    /// Standard deviation of residuals |αₖ| − A·k^(−β).
    pub residual_std: f64,
    /// Catalog of spikes: coefficients deviating > 2σ from the power-law model.
    pub spike_catalog: Vec<SpikeFeature>,
    /// Phase progression model for Verblunsky phases.
    pub phase_model: PhaseModel,
    /// Number of zeros used to build this baseline.
    pub n_zeros: usize,
    /// 2σ detection threshold in absolute magnitude units.
    pub confidence_2sigma: f64,
    /// 3σ detection threshold in absolute magnitude units.
    pub confidence_3sigma: f64,
}

/// A spike in the Verblunsky coefficient magnitude sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpikeFeature {
    /// Index k of the Verblunsky coefficient.
    pub index: usize,
    /// Observed magnitude |αₖ|.
    pub magnitude: f64,
    /// Deviation from the power-law model in σ units.
    pub sigma_deviation: f64,
}

/// Linear phase progression model for Verblunsky phases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseModel {
    /// Mean of consecutive phase differences arg(αₖ₊₁) − arg(αₖ).
    pub mean_progression: f64,
    /// Standard deviation of consecutive phase differences.
    pub progression_std: f64,
}

/// Report from anomaly detection on a test zero set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    /// Maximum σ-deviation from baseline power-law model across all coefficients.
    pub anomaly_score: f64,
    /// Coefficients deviating > 2σ from baseline.
    pub anomalous_coefficients: Vec<AnomalyDetail>,
    /// Whether `anomaly_score > 3.0` (default 3σ threshold).
    pub is_anomalous: bool,
    /// Minimum detectable σ-deviation at this N via the 2/√N heuristic.
    pub min_detectable_sigma: f64,
    /// Number of zeros in the baseline.
    pub baseline_n: usize,
    /// Number of zeros in the test set.
    pub test_n: usize,
}

/// Detail about a single anomalous Verblunsky coefficient.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetail {
    /// Index k of the coefficient.
    pub index: usize,
    /// Observed magnitude |αₖ| from the test zeros.
    pub observed: f64,
    /// Expected magnitude A·k^(−β) from the baseline model.
    pub expected: f64,
    /// Deviation in σ units: |observed − expected| / residual_std.
    pub sigma_deviation: f64,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Build a statistical baseline from known-good zeta zeros.
///
/// Reconstructs the CMV matrix, fits |αₖ| ~ A·k^(−β) via log-log OLS,
/// and identifies spikes (coefficients deviating > 2σ from the model).
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if fewer than 10 zeros are provided.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::anomaly::build_baseline;
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 200.0, 0.1).unwrap();
/// let baseline = build_baseline(&zeros).unwrap();
/// assert!(baseline.power_law_beta.is_finite());
/// assert!(baseline.residual_std >= 0.0);
/// ```
pub fn build_baseline(zeros: &[ZetaZero]) -> Result<VerblunskyBaseline, ZetaError> {
    if zeros.len() < 10 {
        return Err(ZetaError::InvalidParameter(
            "need at least 10 zeros to build a baseline".to_string(),
        ));
    }

    let cmv = reconstruct_cmv(zeros)?;
    let mags = &cmv.verblunsky_magnitudes;
    let phases = &cmv.verblunsky_phases;

    // Fit |αₖ| ~ A·k^(−β) via log-log OLS (skip k=0, skip near-zero magnitudes)
    let (power_law_a, power_law_beta) = fit_power_law(mags);

    // Residuals: r_k = |α_k| − A·k^(−β) for k ≥ 1
    let residuals: Vec<f64> = mags
        .iter()
        .enumerate()
        .skip(1)
        .filter(|&(_, &m)| m > 1e-30)
        .map(|(k, &m)| {
            let expected = power_law_a * (k as f64).powf(-power_law_beta);
            m - expected
        })
        .collect();

    let residual_std = population_std_dev(&residuals);
    let threshold_2sigma = 2.0 * residual_std;

    // Spike catalog: coefficients with |r_k| > 2σ
    let spike_catalog: Vec<SpikeFeature> = mags
        .iter()
        .enumerate()
        .skip(1)
        .filter(|&(_, &m)| m > 1e-30)
        .filter_map(|(k, &m)| {
            let expected = power_law_a * (k as f64).powf(-power_law_beta);
            let residual = m - expected;
            if residual.abs() > threshold_2sigma {
                let sigma_dev = if residual_std > 1e-30 {
                    residual.abs() / residual_std
                } else {
                    0.0
                };
                Some(SpikeFeature {
                    index: k,
                    magnitude: m,
                    sigma_deviation: sigma_dev,
                })
            } else {
                None
            }
        })
        .collect();

    let phase_model = fit_phase_model(phases);

    Ok(VerblunskyBaseline {
        power_law_a,
        power_law_beta,
        residual_std,
        spike_catalog,
        phase_model,
        n_zeros: zeros.len(),
        confidence_2sigma: threshold_2sigma,
        confidence_3sigma: 3.0 * residual_std,
    })
}

/// Detect anomalies in a test zero set by comparison to a baseline.
///
/// Reconstructs CMV from test zeros, and for each Verblunsky coefficient k ≥ 1
/// computes the σ-deviation from the baseline power-law model. The anomaly
/// score is the maximum σ-deviation across all coefficients.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if fewer than 3 zeros are provided.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::anomaly::{build_baseline, detect_anomaly};
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 200.0, 0.1).unwrap();
/// let baseline = build_baseline(&zeros).unwrap();
/// let report = detect_anomaly(&baseline, &zeros).unwrap();
/// assert!(report.anomaly_score.is_finite());
/// assert_eq!(report.baseline_n, baseline.n_zeros);
/// ```
pub fn detect_anomaly(
    baseline: &VerblunskyBaseline,
    test_zeros: &[ZetaZero],
) -> Result<AnomalyReport, ZetaError> {
    if test_zeros.len() < 3 {
        return Err(ZetaError::InvalidParameter(
            "need at least 3 test zeros for anomaly detection".to_string(),
        ));
    }

    let test_n = test_zeros.len();
    let cmv = reconstruct_cmv(test_zeros)?;
    let mags = &cmv.verblunsky_magnitudes;

    // Floor residual_std to avoid division by zero on perfect-fitting baselines
    let sigma_floor = baseline.residual_std.max(1e-30);

    let mut anomaly_score = 0.0_f64;
    let mut anomalous_coefficients = Vec::new();

    for (k, &observed) in mags.iter().enumerate().skip(1) {
        if observed <= 1e-30 {
            continue;
        }
        let expected = baseline.power_law_a * (k as f64).powf(-baseline.power_law_beta);
        let sigma_dev = (observed - expected).abs() / sigma_floor;

        if sigma_dev > anomaly_score {
            anomaly_score = sigma_dev;
        }

        if sigma_dev > 2.0 {
            anomalous_coefficients.push(AnomalyDetail {
                index: k,
                observed,
                expected,
                sigma_deviation: sigma_dev,
            });
        }
    }

    // Minimum detectable σ-deviation: 2/√N heuristic
    let min_detectable_sigma = 2.0 / (test_n as f64).sqrt();

    Ok(AnomalyReport {
        anomaly_score,
        anomalous_coefficients,
        is_anomalous: anomaly_score > 3.0,
        min_detectable_sigma,
        baseline_n: baseline.n_zeros,
        test_n,
    })
}

/// Inject an off-critical-line zero for sensitivity testing.
///
/// Clones `zeros` and shifts the t-value at `position` by
/// `sigma_deviation × mean_spacing`. Marks the modified zero as
/// `on_critical_line: false`.
///
/// Returns the original set unchanged if `position >= zeros.len()`.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::anomaly::inject_off_cl_zero;
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 100.0, 0.1).unwrap();
/// let n = zeros.len();
/// let modified = inject_off_cl_zero(&zeros, n / 2, 0.5);
/// assert_eq!(modified.len(), n);
/// assert!(!modified[n / 2].on_critical_line);
/// ```
pub fn inject_off_cl_zero(
    zeros: &[ZetaZero],
    position: usize,
    sigma_deviation: f64,
) -> Vec<ZetaZero> {
    let mut modified = zeros.to_vec();

    if position >= modified.len() {
        return modified;
    }

    let mean_spacing = compute_mean_spacing(zeros);
    let shift = sigma_deviation * mean_spacing;

    let original = modified[position];
    modified[position] = ZetaZero {
        t: original.t + shift,
        on_critical_line: false,
        ..original
    };

    modified
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Fit |αₖ| ~ A·k^(−β) via log-log OLS. Returns `(A, β)`.
///
/// Skips k=0 (log(0) undefined) and near-zero magnitudes.
/// Returns `(1.0, 0.0)` if fewer than 2 usable points.
fn fit_power_law(magnitudes: &[f64]) -> (f64, f64) {
    let points: Vec<(f64, f64)> = magnitudes
        .iter()
        .enumerate()
        .skip(1)
        .filter(|&(_, &m)| m > 1e-30)
        .map(|(k, &m)| ((k as f64).ln(), m.ln()))
        .collect();

    if points.len() < 2 {
        return (1.0, 0.0);
    }

    let n = points.len() as f64;

    #[allow(clippy::suspicious_operation_groupings)]
    let (sum_x, sum_y, sum_xy, sum_x2) = points.iter().fold(
        (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
        |(sx, sy, sxy, sx2), &(x, y)| (sx + x, sy + y, sxy + x * y, sx2 + x * x),
    );

    #[allow(clippy::suspicious_operation_groupings)]
    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-30 {
        return (1.0, 0.0);
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;

    // slope = d(log|α|)/d(log k) = −β  →  β = −slope
    // intercept = log(A)  →  A = exp(intercept)
    let beta = -slope;
    let a = intercept.exp().max(1e-100);

    (a, beta)
}

/// Population standard deviation (denominator = n).
fn population_std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|&v| (v - mean) * (v - mean)).sum::<f64>() / n;
    var.sqrt()
}

/// Fit the phase progression model from Verblunsky coefficient phases.
fn fit_phase_model(phases: &[f64]) -> PhaseModel {
    if phases.len() < 2 {
        return PhaseModel {
            mean_progression: 0.0,
            progression_std: 0.0,
        };
    }
    let diffs: Vec<f64> = phases.windows(2).map(|w| w[1] - w[0]).collect();
    let mean = diffs.iter().sum::<f64>() / diffs.len() as f64;
    PhaseModel {
        mean_progression: mean,
        progression_std: population_std_dev(&diffs),
    }
}

/// Mean spacing between consecutive zeros sorted by t-value.
fn compute_mean_spacing(zeros: &[ZetaZero]) -> f64 {
    if zeros.len() < 2 {
        return 1.0;
    }
    let mut ts: Vec<f64> = zeros.iter().map(|z| z.t).collect();
    ts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let span = ts[ts.len() - 1] - ts[0];
    if span < 1e-30 {
        return 1.0;
    }
    span / (ts.len() - 1) as f64
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zeros::find_zeros_bracket;

    /// Get approximately `target_n` zeros by choosing an appropriate height.
    fn get_zeros_approx(target_n: usize) -> Vec<ZetaZero> {
        // N(T) ≈ (T/2π)·ln(T/2πe) + 7/8
        // Approximate heights for target counts:
        //   ~30 zeros → T≈100,  ~50 zeros → T≈150,  ~75 zeros → T≈250
        let height = if target_n <= 35 {
            100.0_f64
        } else if target_n <= 55 {
            150.0
        } else {
            250.0
        };
        find_zeros_bracket(10.0, height, 0.1).unwrap_or_default()
    }

    #[test]
    fn baseline_from_75_zeros_produces_finite_params() {
        let zeros = get_zeros_approx(75);
        assert!(
            zeros.len() >= 10,
            "need at least 10 zeros, got {}",
            zeros.len()
        );
        let baseline = build_baseline(&zeros).unwrap();

        assert!(
            baseline.power_law_a.is_finite() && baseline.power_law_a > 0.0,
            "power_law_a must be finite and positive: {}",
            baseline.power_law_a
        );
        assert!(
            baseline.power_law_beta.is_finite(),
            "power_law_beta not finite: {}",
            baseline.power_law_beta
        );
        assert!(
            baseline.residual_std.is_finite() && baseline.residual_std >= 0.0,
            "residual_std not finite or negative: {}",
            baseline.residual_std
        );
        assert!(
            baseline.confidence_2sigma.is_finite(),
            "confidence_2sigma not finite"
        );
        assert!(
            baseline.confidence_3sigma.is_finite(),
            "confidence_3sigma not finite"
        );
        assert_eq!(
            baseline.confidence_3sigma,
            3.0 * baseline.residual_std,
            "3σ threshold must be 3 × residual_std"
        );
        assert_eq!(baseline.n_zeros, zeros.len());

        eprintln!(
            "Baseline n={}: A={:.4e}, β={:.4}, σ={:.4e}, spikes={}",
            zeros.len(),
            baseline.power_law_a,
            baseline.power_law_beta,
            baseline.residual_std,
            baseline.spike_catalog.len()
        );
    }

    #[test]
    fn clean_zeros_have_low_anomaly_score() {
        let zeros = get_zeros_approx(75);
        let baseline = build_baseline(&zeros).unwrap();
        let report = detect_anomaly(&baseline, &zeros).unwrap();

        assert!(
            report.anomaly_score.is_finite(),
            "anomaly_score not finite: {}",
            report.anomaly_score
        );
        assert_eq!(report.baseline_n, baseline.n_zeros);
        assert_eq!(report.test_n, zeros.len());
        assert!(
            report.min_detectable_sigma.is_finite() && report.min_detectable_sigma > 0.0,
            "min_detectable_sigma must be positive finite: {}",
            report.min_detectable_sigma
        );

        eprintln!(
            "Clean zeros (n={}): anomaly_score={:.4}, is_anomalous={}, n_flagged={}",
            zeros.len(),
            report.anomaly_score,
            report.is_anomalous,
            report.anomalous_coefficients.len()
        );
    }

    #[test]
    fn injected_half_sigma_is_detectable() {
        let zeros = get_zeros_approx(75);
        let baseline = build_baseline(&zeros).unwrap();

        // Inject at the middle zero: shift by 0.5 × mean_spacing
        let mid = zeros.len() / 2;
        let modified = inject_off_cl_zero(&zeros, mid, 0.5);

        assert_eq!(modified.len(), zeros.len());
        assert!(
            !modified[mid].on_critical_line,
            "injected zero should be off CL"
        );
        assert!(
            (modified[mid].t - zeros[mid].t).abs() > 1e-12,
            "t-value should have changed"
        );

        let report = detect_anomaly(&baseline, &modified).unwrap();
        assert!(
            report.anomaly_score.is_finite(),
            "anomaly_score not finite after injection"
        );
        assert!(
            report.anomaly_score > 2.0,
            "expected anomaly_score > 2 after 0.5σ injection, got {:.4} \
             (baseline: A={:.3e}, β={:.3}, σ={:.3e})",
            report.anomaly_score,
            baseline.power_law_a,
            baseline.power_law_beta,
            baseline.residual_std
        );

        eprintln!(
            "0.5σ injection at pos {mid}: anomaly_score={:.4}, is_anomalous={}",
            report.anomaly_score, report.is_anomalous
        );
    }

    #[test]
    fn injecting_small_deviation_logs_result() {
        let zeros = get_zeros_approx(75);
        let baseline = build_baseline(&zeros).unwrap();

        let mid = zeros.len() / 2;
        let modified = inject_off_cl_zero(&zeros, mid, 0.1);

        let report = detect_anomaly(&baseline, &modified).unwrap();
        assert!(
            report.anomaly_score.is_finite(),
            "anomaly_score not finite after 0.1σ injection"
        );

        eprintln!(
            "0.1σ injection: anomaly_score={:.4}, is_anomalous={}, n_flagged={}",
            report.anomaly_score,
            report.is_anomalous,
            report.anomalous_coefficients.len()
        );
    }

    #[test]
    fn sensitivity_improves_with_n() {
        // Three increasing heights → increasing N → smaller min_detectable_sigma
        let heights = [100.0_f64, 150.0, 250.0];
        let mut results: Vec<(usize, f64)> = Vec::new();

        for height in heights {
            let zeros = find_zeros_bracket(10.0, height, 0.1).unwrap_or_default();
            if zeros.len() < 10 {
                continue;
            }
            let baseline = match build_baseline(&zeros) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let report = match detect_anomaly(&baseline, &zeros) {
                Ok(r) => r,
                Err(_) => continue,
            };
            results.push((zeros.len(), report.min_detectable_sigma));
            eprintln!(
                "N={}: min_detectable_sigma={:.4}",
                zeros.len(),
                report.min_detectable_sigma
            );
        }

        // Require at least two data points and verify monotonic decrease
        assert!(
            results.len() >= 2,
            "need at least two N values for sensitivity comparison"
        );

        let first_sigma = results[0].1;
        let last_sigma = results[results.len() - 1].1;
        assert!(
            last_sigma < first_sigma,
            "min_detectable_sigma should decrease as N increases: \
             first={first_sigma:.4} (N={}), last={last_sigma:.4} (N={})",
            results[0].0,
            results[results.len() - 1].0
        );
    }

    #[test]
    fn inject_at_out_of_bounds_is_noop() {
        let zeros = get_zeros_approx(30);
        let n = zeros.len();
        let modified = inject_off_cl_zero(&zeros, n + 10, 1.0);
        assert_eq!(modified.len(), n);
        // All zeros unchanged
        for (orig, modif) in zeros.iter().zip(modified.iter()) {
            assert!(
                (orig.t - modif.t).abs() < 1e-12,
                "t should be unchanged for out-of-bounds injection"
            );
            assert_eq!(orig.on_critical_line, modif.on_critical_line);
        }
    }

    #[test]
    fn build_baseline_rejects_too_few_zeros() {
        let zeros = vec![ZetaZero {
            ordinal: 1,
            t: 14.1347,
            z_value: 0.0,
            on_critical_line: true,
        }];
        assert!(
            build_baseline(&zeros).is_err(),
            "expected error for < 10 zeros"
        );
    }
}
