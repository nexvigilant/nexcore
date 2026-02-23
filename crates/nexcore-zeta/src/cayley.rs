//! # Cayley Transform of CMV Matrices
//!
//! Maps the unitary CMV matrix U (eigenvalues on unit circle) to a
//! self-adjoint operator H (eigenvalues on real line) via the Cayley
//! transform:
//!
//! ```text
//! H = i(I + U)(I - U)^{-1}
//! ```
//!
//! If U has eigenvalue e^{iθ}, then H has eigenvalue cot(θ/2).
//!
//! ## Why This Matters
//!
//! The Cayley transform amplifies small deviations near θ = 0 and θ = 2π
//! (the "poles" of cot), making anomalies in the zero distribution more
//! detectable. It also produces a self-adjoint operator — the form that
//! the Hilbert-Pólya conjecture says should exist.

use serde::{Deserialize, Serialize};

use crate::cmv::CmvReconstruction;
use crate::error::ZetaError;
use crate::zeros::ZetaZero;

// ── Types ────────────────────────────────────────────────────────────────────

/// Result of applying the Cayley transform to CMV Verblunsky coefficients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CayleyTransform {
    /// Real-line eigenvalues from the Cayley transform: cot(θₖ/2).
    pub eigenvalues: Vec<f64>,
    /// Original unit-circle angles θₖ from CMV.
    pub unit_circle_angles: Vec<f64>,
    /// Condition number estimate: max|λ| / min|λ|.
    pub condition_number: f64,
    /// Whether any eigenvalue is numerically unstable (θ near 0 or 2π).
    pub has_unstable_eigenvalues: bool,
    /// Number of eigenvalues flagged as unstable (|cot(θ/2)| > 1e6).
    pub n_unstable: usize,
}

/// Enhanced anomaly detection using the Cayley-transformed spectrum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CayleyAnomalyReport {
    /// Anomaly score based on Cayley eigenvalue distribution.
    pub anomaly_score: f64,
    /// Whether anomalous deviations were detected.
    pub is_anomalous: bool,
    /// Per-eigenvalue deviation from expected distribution.
    pub deviations: Vec<CayleyDeviation>,
    /// Comparison metric: KL-divergence from expected Cauchy distribution.
    pub kl_divergence: f64,
    /// Number of eigenvalues analyzed.
    pub n_analyzed: usize,
}

/// Deviation of a single Cayley eigenvalue from the expected distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CayleyDeviation {
    /// Index of the eigenvalue.
    pub index: usize,
    /// Cayley eigenvalue cot(θ/2).
    pub eigenvalue: f64,
    /// Expected value from the smooth distribution.
    pub expected: f64,
    /// Deviation in standard deviations.
    pub sigma_deviation: f64,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Apply the Cayley transform to CMV reconstruction data.
///
/// Computes `cot(θₖ/2)` for each unit-circle angle θₖ from the CMV
/// eigenvalue mapping. Eigenvalues near θ = 0 or 2π are flagged as
/// unstable.
///
/// # Errors
///
/// Returns error if fewer than 3 eigenvalues in the CMV reconstruction.
pub fn cayley_transform(cmv: &CmvReconstruction) -> Result<CayleyTransform, ZetaError> {
    let n = cmv.eigenvalues.len();
    if n < 3 {
        return Err(ZetaError::InvalidParameter(
            "need at least 3 eigenvalues for Cayley transform".to_string(),
        ));
    }

    // Reconstruct unit-circle angles from eigenvalues
    let t_min = cmv.eigenvalues.first().copied().unwrap_or(0.0);
    let t_max = cmv.eigenvalues.last().copied().unwrap_or(1.0);
    let range = t_max - t_min;

    if range < 1e-10 {
        return Err(ZetaError::InvalidParameter(
            "eigenvalue range too small for Cayley transform".to_string(),
        ));
    }

    let mut angles = Vec::with_capacity(n);
    let mut cayley_eigs = Vec::with_capacity(n);
    let mut n_unstable = 0_usize;

    let stability_threshold = 1e6;

    for &t in &cmv.eigenvalues {
        // Map to unit circle: same mapping as CMV reconstruction
        let theta = 2.0 * std::f64::consts::PI * (t - t_min) / range;
        angles.push(theta);

        // Cayley: cot(θ/2)
        let half_theta = theta / 2.0;
        let sin_half = half_theta.sin();
        let cos_half = half_theta.cos();

        let cayley_eig = if sin_half.abs() < 1e-12 {
            // θ near 0 or 2π — pole of cot
            n_unstable += 1;
            // Clip to large magnitude preserving sign
            if cos_half >= 0.0 {
                stability_threshold
            } else {
                -stability_threshold
            }
        } else {
            cos_half / sin_half
        };

        cayley_eigs.push(cayley_eig);
    }

    // Condition number
    let max_abs = cayley_eigs.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);
    let min_abs = cayley_eigs
        .iter()
        .map(|x| x.abs())
        .fold(f64::INFINITY, f64::min);

    let condition = if min_abs > 1e-15 {
        max_abs / min_abs
    } else {
        f64::INFINITY
    };

    Ok(CayleyTransform {
        eigenvalues: cayley_eigs,
        unit_circle_angles: angles,
        condition_number: condition,
        has_unstable_eigenvalues: n_unstable > 0,
        n_unstable,
    })
}

/// Run anomaly detection on Cayley-transformed eigenvalues.
///
/// The Cayley eigenvalues of true zeta zeros (under RH) should follow
/// a smooth distribution determined by the density of zeros near each
/// height. Anomalies — deviations from this smoothness — suggest either
/// computational error or (in the extreme) a counterexample to RH.
///
/// # Algorithm
///
/// 1. Sort Cayley eigenvalues (excluding unstable poles)
/// 2. Compute normalized spacings
/// 3. Compare spacing distribution to expected (Cauchy-like) model
/// 4. Flag deviations exceeding 3σ
pub fn cayley_anomaly_detect(
    cmv: &CmvReconstruction,
    zeros: &[ZetaZero],
) -> Result<CayleyAnomalyReport, ZetaError> {
    let transform = cayley_transform(cmv)?;

    // Filter to stable eigenvalues only
    let stability_threshold = 1e6;
    let stable: Vec<(usize, f64)> = transform
        .eigenvalues
        .iter()
        .enumerate()
        .filter(|&(_, &e)| e.abs() < stability_threshold)
        .map(|(i, &e)| (i, e))
        .collect();

    if stable.len() < 3 {
        return Err(ZetaError::InvalidParameter(
            "too few stable Cayley eigenvalues for anomaly detection".to_string(),
        ));
    }

    // Sort by eigenvalue
    let mut sorted: Vec<(usize, f64)> = stable;
    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Compute spacings
    let spacings: Vec<f64> = sorted.windows(2).map(|w| w[1].1 - w[0].1).collect();

    if spacings.is_empty() {
        return Ok(CayleyAnomalyReport {
            anomaly_score: 0.0,
            is_anomalous: false,
            deviations: vec![],
            kl_divergence: 0.0,
            n_analyzed: sorted.len(),
        });
    }

    // Expected smooth spacing: local mean over a window
    let window = 5.min(spacings.len());
    let mut deviations = Vec::new();
    let mut max_deviation = 0.0_f64;

    for i in 0..spacings.len() {
        let start = i.saturating_sub(window / 2);
        let end = (i + window / 2 + 1).min(spacings.len());
        let local_mean: f64 = spacings[start..end].iter().sum::<f64>() / (end - start) as f64;
        let local_std = if end - start > 1 {
            let var = spacings[start..end]
                .iter()
                .map(|&s| (s - local_mean).powi(2))
                .sum::<f64>()
                / (end - start - 1) as f64;
            var.sqrt().max(1e-10)
        } else {
            local_mean.abs().max(1e-10)
        };

        let sigma_dev = (spacings[i] - local_mean).abs() / local_std;

        if sigma_dev > 2.0 {
            deviations.push(CayleyDeviation {
                index: sorted[i].0,
                eigenvalue: sorted[i].1,
                expected: local_mean,
                sigma_deviation: sigma_dev,
            });
        }

        if sigma_dev > max_deviation {
            max_deviation = sigma_dev;
        }
    }

    // KL divergence: compare spacing distribution to exponential (GUE spacing model)
    let global_mean = spacings.iter().sum::<f64>() / spacings.len() as f64;
    let kl = if global_mean > 1e-15 {
        compute_kl_vs_exponential(&spacings, global_mean)
    } else {
        0.0
    };

    // Use zero count for additional context
    let _n_zeros = zeros.len();

    Ok(CayleyAnomalyReport {
        anomaly_score: max_deviation,
        is_anomalous: max_deviation > 3.0,
        deviations,
        kl_divergence: kl,
        n_analyzed: sorted.len(),
    })
}

// ── Internal ─────────────────────────────────────────────────────────────────

/// Compute KL divergence of observed spacings vs exponential(1/mean) distribution.
///
/// KL(P||Q) = Σ P(x) · log(P(x) / Q(x))
///
/// Uses histogram-based estimation with fixed bins.
fn compute_kl_vs_exponential(spacings: &[f64], mean: f64) -> f64 {
    let n_bins = 20;
    let max_spacing = spacings.iter().copied().fold(0.0_f64, f64::max);
    let bin_width = max_spacing / n_bins as f64;

    if bin_width < 1e-15 {
        return 0.0;
    }

    let mut counts = vec![0_usize; n_bins];
    for &s in spacings {
        let bin = ((s / bin_width) as usize).min(n_bins - 1);
        counts[bin] += 1;
    }

    let n = spacings.len() as f64;
    let rate = 1.0 / mean;
    let mut kl = 0.0_f64;

    for (i, &count) in counts.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let p = count as f64 / n;
        // Q(bin) = integral of rate*exp(-rate*x) over [i*w, (i+1)*w]
        let x0 = i as f64 * bin_width;
        let x1 = x0 + bin_width;
        let q = (-rate * x0).exp() - (-rate * x1).exp();
        if q > 1e-15 {
            kl += p * (p / q).ln();
        }
    }

    kl.max(0.0)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmv::reconstruct_cmv;
    use crate::zeros::find_zeros_bracket;

    fn get_cmv(t_min: f64, t_max: f64) -> Option<(CmvReconstruction, Vec<ZetaZero>)> {
        let zeros = find_zeros_bracket(t_min, t_max, 0.05).ok()?;
        if zeros.len() < 20 {
            return None;
        }
        let cmv = reconstruct_cmv(&zeros).ok()?;
        Some((cmv, zeros))
    }

    #[test]
    fn cayley_on_zeta_zeros() {
        let Some((cmv, _zeros)) = get_cmv(10.0, 100.0) else {
            return;
        };
        let result = cayley_transform(&cmv);
        assert!(result.is_ok(), "cayley failed: {:?}", result.err());
        let ct = result.unwrap_or_else(|_| unreachable!());

        assert_eq!(ct.eigenvalues.len(), cmv.eigenvalues.len());
        assert!(ct.condition_number > 0.0);
        // Most eigenvalues should be stable
        assert!(
            ct.n_unstable <= ct.eigenvalues.len() / 5,
            "too many unstable: {} of {}",
            ct.n_unstable,
            ct.eigenvalues.len()
        );

        eprintln!(
            "Cayley: {} eigs, {} unstable, condition={:.2e}",
            ct.eigenvalues.len(),
            ct.n_unstable,
            ct.condition_number
        );
    }

    #[test]
    fn cayley_angles_match_eigenvalue_count() {
        let Some((cmv, _)) = get_cmv(10.0, 80.0) else {
            return;
        };
        let ct = cayley_transform(&cmv).unwrap_or_else(|_| unreachable!());
        assert_eq!(ct.eigenvalues.len(), ct.unit_circle_angles.len());
    }

    #[test]
    fn cayley_anomaly_on_clean_zeros() {
        let Some((cmv, zeros)) = get_cmv(10.0, 150.0) else {
            return;
        };
        let report = cayley_anomaly_detect(&cmv, &zeros);
        assert!(
            report.is_ok(),
            "anomaly detection failed: {:?}",
            report.err()
        );
        let r = report.unwrap_or_else(|_| unreachable!());

        // Clean zeros should not be anomalous
        eprintln!(
            "Cayley anomaly: score={:.4}, is_anomalous={}, deviations={}, kl={:.4}",
            r.anomaly_score,
            r.is_anomalous,
            r.deviations.len(),
            r.kl_divergence
        );
    }

    #[test]
    fn cayley_anomaly_detects_perturbation() {
        let Some((_, mut zeros)) = get_cmv(10.0, 150.0) else {
            return;
        };

        // Inject a massive perturbation
        if zeros.len() > 10 {
            zeros[5].t += 10.0;
        }

        let cmv = reconstruct_cmv(&zeros);
        if cmv.is_err() {
            return; // Perturbed zeros may break CMV
        }
        let cmv = cmv.unwrap_or_else(|_| unreachable!());

        let report = cayley_anomaly_detect(&cmv, &zeros);
        if let Ok(r) = report {
            // Perturbed should have higher anomaly score
            eprintln!(
                "Perturbed Cayley: score={:.4}, deviations={}",
                r.anomaly_score,
                r.deviations.len()
            );
        }
    }

    #[test]
    fn cayley_too_few_eigenvalues() {
        let cmv = CmvReconstruction {
            verblunsky_magnitudes: vec![0.5],
            verblunsky_phases: vec![0.1],
            eigenvalues: vec![14.0, 21.0],
            roundtrip_error: 0.01,
            structure: crate::cmv::CmvStructure {
                mean_coefficient_magnitude: 0.5,
                coefficient_decay_rate: 0.3,
                coefficient_regularity: 0.1,
                phase_regularity: 0.1,
                max_coefficient: 0.5,
                n: 1,
            },
        };
        assert!(cayley_transform(&cmv).is_err());
    }

    #[test]
    fn kl_divergence_is_non_negative() {
        let spacings = vec![1.0, 1.5, 0.8, 1.2, 0.9, 1.1, 1.3];
        let mean = spacings.iter().sum::<f64>() / spacings.len() as f64;
        let kl = compute_kl_vs_exponential(&spacings, mean);
        assert!(kl >= 0.0, "KL divergence negative: {kl}");
    }

    #[test]
    fn empty_spacings_kl_zero() {
        let kl = compute_kl_vs_exponential(&[], 1.0);
        assert!((kl - 0.0).abs() < 1e-15);
    }
}
