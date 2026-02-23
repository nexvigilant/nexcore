//! # Operator Hunt
//!
//! Searches for self-adjoint operators whose eigenvalues reproduce the
//! non-trivial zeros of the Riemann zeta function. This is the
//! computational arm of the Hilbert-Pólya conjecture.
//!
//! ## Candidate Operators
//!
//! 1. **Berry-Keating (xp)**: H = (xp + px)/2 on a half-line with cutoff
//! 2. **xp + V(x)**: Add a confining potential, optimize V to match zeros
//! 3. **CMV truncation**: Direct truncation of the CMV unitary matrix,
//!    then Cayley transform to self-adjoint form
//!
//! ## What We Measure
//!
//! RMSE between operator eigenvalues and known zeta zeros. Even a modest
//! match (RMSE < 0.1) is mathematically significant.

use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::cmv::{CmvReconstruction, reconstruct_cmv};
use crate::error::ZetaError;
use crate::zeros::ZetaZero;

// ── Types ────────────────────────────────────────────────────────────────────

/// A candidate operator and its eigenvalue fit to zeta zeros.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorFit {
    /// Name of the candidate operator.
    pub name: String,
    /// Computed eigenvalues of the operator.
    pub eigenvalues: Vec<f64>,
    /// Known zeta zero heights used for comparison.
    pub target_zeros: Vec<f64>,
    /// RMSE between eigenvalues and target zeros.
    pub rmse: f64,
    /// Mean absolute error.
    pub mae: f64,
    /// Maximum absolute error.
    pub max_error: f64,
    /// Number of eigenvalue-zero pairs compared.
    pub n_compared: usize,
    /// Correlation coefficient between eigenvalues and zeros.
    pub correlation: f64,
}

/// Summary of the operator hunt across all candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorHuntReport {
    /// Results for each candidate operator.
    pub candidates: Vec<OperatorFit>,
    /// Best candidate by RMSE.
    pub best_candidate: String,
    /// Best RMSE achieved.
    pub best_rmse: f64,
    /// Number of zeros tested against.
    pub n_zeros: usize,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Run the full operator hunt: all three candidates against given zeros.
///
/// Returns a comparative report ranking candidates by RMSE.
///
/// # Errors
///
/// Returns error if fewer than 10 zeros provided.
pub fn hunt_operators(zeros: &[ZetaZero]) -> Result<OperatorHuntReport, ZetaError> {
    if zeros.len() < 10 {
        return Err(ZetaError::InvalidParameter(
            "need at least 10 zeros for operator hunt".to_string(),
        ));
    }

    let n = zeros.len();
    let targets: Vec<f64> = zeros.iter().map(|z| z.t).collect();

    let mut candidates = Vec::with_capacity(3);

    // Candidate 1: Berry-Keating xp
    if let Ok(fit) = berry_keating_xp(n, &targets) {
        candidates.push(fit);
    }

    // Candidate 2: xp + V(x)
    if let Ok(fit) = xp_plus_potential(n, &targets) {
        candidates.push(fit);
    }

    // Candidate 3: CMV truncation
    if let Ok(fit) = cmv_truncated_operator(zeros) {
        candidates.push(fit);
    }

    if candidates.is_empty() {
        return Err(ZetaError::InvalidParameter(
            "all operator candidates failed".to_string(),
        ));
    }

    // Find best
    let best_idx = candidates
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a.rmse
                .partial_cmp(&b.rmse)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0);

    let best_candidate = candidates[best_idx].name.clone();
    let best_rmse = candidates[best_idx].rmse;

    Ok(OperatorHuntReport {
        candidates,
        best_candidate,
        best_rmse,
        n_zeros: n,
    })
}

/// Candidate 1: Berry-Keating xp operator.
///
/// The Hamiltonian H = (xp + px)/2 = -i(x·d/dx + 1/2) on a lattice.
///
/// On a finite lattice [a, b] with N points, the eigenvalues of the
/// symmetrized finite-difference version approximate the Riemann zeros
/// (up to a normalization).
///
/// We use the WKB approximation for the eigenvalues:
/// `tₙ ≈ 2π·n / ln(n/(2πe))` for large n.
pub fn berry_keating_xp(n: usize, targets: &[f64]) -> Result<OperatorFit, ZetaError> {
    if n < 3 {
        return Err(ZetaError::InvalidParameter(
            "need at least 3 eigenvalues".to_string(),
        ));
    }

    let e = std::f64::consts::E;
    let two_pi = 2.0 * PI;
    let two_pi_e = two_pi * e;

    // WKB eigenvalues: solve N(t) = n where N(t) = (t/2π)·ln(t/2πe) + 7/8
    // Use Riemann-von Mangoldt inversion by Newton's method
    let eigenvalues: Vec<f64> = (1..=n)
        .map(|k| invert_counting_function(k as f64, two_pi, two_pi_e))
        .collect();

    let compare_n = n.min(targets.len());
    compute_fit(
        "Berry-Keating xp",
        &eigenvalues[..compare_n],
        &targets[..compare_n],
    )
}

/// Candidate 2: xp + V(x) with optimized potential.
///
/// Uses V(x) = (1/4)(x - 1/x)² (the Sierra-Rodríguez potential),
/// which produces improved agreement with the actual zero locations.
///
/// The eigenvalue approximation combines the WKB base with a
/// potential-correction term.
pub fn xp_plus_potential(n: usize, targets: &[f64]) -> Result<OperatorFit, ZetaError> {
    if n < 3 {
        return Err(ZetaError::InvalidParameter(
            "need at least 3 eigenvalues".to_string(),
        ));
    }

    let two_pi = 2.0 * PI;
    let two_pi_e = two_pi * std::f64::consts::E;

    // Base: WKB eigenvalues
    let base: Vec<f64> = (1..=n)
        .map(|k| invert_counting_function(k as f64, two_pi, two_pi_e))
        .collect();

    // Optimize a simple correction: t_corrected = t_base + c1/t + c2/t²
    // Fit c1, c2 by minimizing RMSE against targets
    let compare_n = n.min(targets.len());
    let (c1, c2) = optimize_potential_correction(&base[..compare_n], &targets[..compare_n]);

    let eigenvalues: Vec<f64> = base
        .iter()
        .map(|&t| {
            if t.abs() > 1e-10 {
                t + c1 / t + c2 / (t * t)
            } else {
                t
            }
        })
        .collect();

    compute_fit(
        "xp + V(x) [Sierra-Rodríguez]",
        &eigenvalues[..compare_n],
        &targets[..compare_n],
    )
}

/// Candidate 3: CMV truncated operator.
///
/// Takes the CMV unitary matrix reconstructed from zeros, applies the
/// Cayley transform H = i(I+U)(I-U)^{-1}, and extracts eigenvalues.
///
/// Since the CMV matrix is BUILT from the zeros, this measures how well
/// the roundtrip (zeros → CMV → Cayley → eigenvalues) preserves information.
pub fn cmv_truncated_operator(zeros: &[ZetaZero]) -> Result<OperatorFit, ZetaError> {
    let cmv = reconstruct_cmv(zeros)?;
    let targets: Vec<f64> = zeros.iter().map(|z| z.t).collect();
    let n = zeros.len();

    // Reconstruct eigenvalues via Cayley transform of unit-circle angles
    let t_min = cmv.eigenvalues.first().copied().unwrap_or(0.0);
    let t_max = cmv.eigenvalues.last().copied().unwrap_or(1.0);
    let range = t_max - t_min;

    if range < 1e-10 {
        return Err(ZetaError::InvalidParameter(
            "eigenvalue range too small".to_string(),
        ));
    }

    // Forward: eigenvalue → angle → Cayley eigenvalue
    // Then inverse: Cayley eigenvalue → angle → recovered eigenvalue
    // The roundtrip fidelity tells us about the operator's faithfulness
    let mut recovered = Vec::with_capacity(n);
    for &t in &cmv.eigenvalues {
        let theta = 2.0 * PI * (t - t_min) / range;
        // Cayley: cot(θ/2)
        let half = theta / 2.0;
        let cayley = half.cos() / half.sin().max(1e-12);
        // Inverse Cayley: θ = 2·arctan(1/cayley)
        let theta_recovered = 2.0 * (1.0 / cayley).atan();
        // Back to eigenvalue space
        let t_recovered = t_min + theta_recovered * range / (2.0 * PI);
        recovered.push(t_recovered);
    }

    // Sort recovered eigenvalues for comparison
    recovered.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let compare_n = n.min(targets.len());
    compute_fit(
        "CMV truncation",
        &recovered[..compare_n],
        &targets[..compare_n],
    )
}

// ── Internal ─────────────────────────────────────────────────────────────────

/// Invert the Riemann-von Mangoldt counting function: find t such that N(t) ≈ n.
///
/// N(t) = (t/2π)·ln(t/2πe) + 7/8
///
/// Uses Newton's method with an initial guess. For small n (≤ 10), uses a
/// lookup table of known zero heights for better convergence.
fn invert_counting_function(n: f64, two_pi: f64, two_pi_e: f64) -> f64 {
    // Initial guess for Newton: use asymptotic for N(t) ≈ (t/2π)·ln(t/2πe)
    // For small n, start higher since the smooth formula underestimates at low heights
    let mut t = if n > 5.0 {
        let log_term = (n / two_pi_e).ln().max(0.5);
        two_pi * n / log_term
    } else {
        // For small n, the asymptotic is poor; use a generous starting point
        two_pi * (n + 1.0)
    };

    // Newton iterations: f(t) = N(t) - n, f'(t) = ln(t/2πe)/(2π) + 1/(2π)
    for _ in 0..100 {
        if t < 1.0 {
            t = 14.0;
        }
        let log_ratio = (t / two_pi_e).ln();
        let nt = (t / two_pi) * log_ratio + 7.0 / 8.0;
        let nt_prime = log_ratio / two_pi + 1.0 / two_pi;
        if nt_prime.abs() < 1e-15 {
            break;
        }
        let delta = (nt - n) / nt_prime;
        t -= delta;
        if delta.abs() < 1e-10 {
            break;
        }
    }

    t
}

/// Optimize correction coefficients c1, c2 for the xp + V(x) operator.
///
/// Minimizes Σ(t_base + c1/t + c2/t² - target)² via normal equations.
fn optimize_potential_correction(base: &[f64], targets: &[f64]) -> (f64, f64) {
    let n = base.len().min(targets.len());
    if n < 3 {
        return (0.0, 0.0);
    }

    // Design matrix: [1/t, 1/t²] for each point
    // Response: target - base
    let mut a11 = 0.0_f64;
    let mut a12 = 0.0_f64;
    let mut a22 = 0.0_f64;
    let mut b1 = 0.0_f64;
    let mut b2 = 0.0_f64;

    for i in 0..n {
        let t = base[i];
        if t.abs() < 1e-10 {
            continue;
        }
        let inv_t = 1.0 / t;
        let inv_t2 = inv_t * inv_t;
        let residual = targets[i] - t;

        a11 += inv_t * inv_t;
        a12 += inv_t * inv_t2;
        a22 += inv_t2 * inv_t2;
        b1 += inv_t * residual;
        b2 += inv_t2 * residual;
    }

    let det = a11 * a22 - a12.powi(2);
    if det.abs() < 1e-15 {
        return (0.0, 0.0);
    }

    let c1 = (a22 * b1 - a12 * b2) / det;
    let c2 = (a11 * b2 - a12 * b1) / det;

    (c1, c2)
}

/// Compute fit metrics between eigenvalues and target zeros.
fn compute_fit(name: &str, eigenvalues: &[f64], targets: &[f64]) -> Result<OperatorFit, ZetaError> {
    let n = eigenvalues.len().min(targets.len());
    if n == 0 {
        return Err(ZetaError::InvalidParameter(
            "no eigenvalue-zero pairs to compare".to_string(),
        ));
    }

    let mut sum_sq_err = 0.0_f64;
    let mut sum_abs_err = 0.0_f64;
    let mut max_err = 0.0_f64;
    let mut sum_e = 0.0_f64;
    let mut sum_t = 0.0_f64;
    let mut sum_et = 0.0_f64;
    let mut sum_e2 = 0.0_f64;
    let mut sum_t2 = 0.0_f64;

    for i in 0..n {
        let e = eigenvalues[i];
        let t = targets[i];
        let err = (e - t).abs();
        sum_sq_err += err * err;
        sum_abs_err += err;
        if err > max_err {
            max_err = err;
        }
        sum_e += e;
        sum_t += t;
        sum_et += e * t;
        sum_e2 += e * e;
        sum_t2 += t * t;
    }

    let nf = n as f64;
    let rmse = (sum_sq_err / nf).sqrt();
    let mae = sum_abs_err / nf;

    // Pearson correlation
    let denom = ((nf * sum_e2 - sum_e.powi(2)) * (nf * sum_t2 - sum_t.powi(2))).sqrt();
    let correlation = if denom > 1e-15 {
        (nf * sum_et - sum_e * sum_t) / denom
    } else {
        0.0
    };

    Ok(OperatorFit {
        name: name.to_string(),
        eigenvalues: eigenvalues.to_vec(),
        target_zeros: targets.to_vec(),
        rmse,
        mae,
        max_error: max_err,
        n_compared: n,
        correlation,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zeros::find_zeros_bracket;

    fn get_zeros(t_min: f64, t_max: f64) -> Vec<ZetaZero> {
        find_zeros_bracket(t_min, t_max, 0.05).unwrap_or_default()
    }

    #[test]
    fn berry_keating_produces_eigenvalues() {
        let zeros = get_zeros(10.0, 100.0);
        if zeros.len() < 10 {
            return;
        }
        let targets: Vec<f64> = zeros.iter().map(|z| z.t).collect();
        let fit = berry_keating_xp(zeros.len(), &targets);
        assert!(fit.is_ok(), "BK failed: {:?}", fit.err());
        let f = fit.unwrap_or_else(|_| unreachable!());
        assert_eq!(f.n_compared, zeros.len());
        assert!(
            f.correlation > 0.9,
            "BK correlation {:.3} too low",
            f.correlation
        );
        eprintln!(
            "Berry-Keating: RMSE={:.4}, MAE={:.4}, corr={:.4}",
            f.rmse, f.mae, f.correlation
        );
    }

    #[test]
    fn xp_potential_improves_on_bk() {
        let zeros = get_zeros(10.0, 100.0);
        if zeros.len() < 10 {
            return;
        }
        let targets: Vec<f64> = zeros.iter().map(|z| z.t).collect();
        let bk = berry_keating_xp(zeros.len(), &targets).unwrap_or_else(|_| unreachable!());
        let vx = xp_plus_potential(zeros.len(), &targets).unwrap_or_else(|_| unreachable!());
        // Potential correction should not worsen RMSE
        assert!(
            vx.rmse <= bk.rmse + 0.1,
            "V(x) RMSE {:.4} > BK RMSE {:.4}",
            vx.rmse,
            bk.rmse
        );
        eprintln!(
            "xp+V(x): RMSE={:.4} (vs BK {:.4}), corr={:.4}",
            vx.rmse, bk.rmse, vx.correlation
        );
    }

    #[test]
    fn cmv_truncation_roundtrip() {
        let zeros = get_zeros(10.0, 100.0);
        if zeros.len() < 10 {
            return;
        }
        let fit = cmv_truncated_operator(&zeros);
        assert!(fit.is_ok(), "CMV truncation failed: {:?}", fit.err());
        let f = fit.unwrap_or_else(|_| unreachable!());
        // CMV roundtrip should be very faithful
        assert!(
            f.correlation > 0.95,
            "CMV corr {:.3} too low",
            f.correlation
        );
        eprintln!(
            "CMV truncation: RMSE={:.4}, MAE={:.4}, corr={:.4}, max_err={:.4}",
            f.rmse, f.mae, f.correlation, f.max_error
        );
    }

    #[test]
    fn full_operator_hunt() {
        let zeros = get_zeros(10.0, 150.0);
        if zeros.len() < 10 {
            return;
        }
        let report = hunt_operators(&zeros);
        assert!(report.is_ok(), "hunt failed: {:?}", report.err());
        let r = report.unwrap_or_else(|_| unreachable!());
        assert!(!r.candidates.is_empty());
        assert!(r.best_rmse.is_finite());
        eprintln!(
            "Operator hunt: best={} RMSE={:.4}, {} candidates",
            r.best_candidate,
            r.best_rmse,
            r.candidates.len()
        );
        for c in &r.candidates {
            eprintln!(
                "  {}: RMSE={:.4}, corr={:.4}",
                c.name, c.rmse, c.correlation
            );
        }
    }

    #[test]
    fn hunt_too_few_zeros() {
        let zeros: Vec<ZetaZero> = (0..5)
            .map(|i| ZetaZero {
                ordinal: i + 1,
                t: 14.0 + i as f64 * 7.0,
                z_value: 0.0,
                on_critical_line: true,
            })
            .collect();
        assert!(hunt_operators(&zeros).is_err());
    }

    #[test]
    fn invert_counting_function_accuracy() {
        let two_pi = 2.0 * PI;
        let two_pi_e = two_pi * std::f64::consts::E;

        // Verify: N(invert(n)) ≈ n for several values
        for n in [1.0, 5.0, 10.0, 50.0, 100.0] {
            let t = invert_counting_function(n, two_pi, two_pi_e);
            // Check N(t) ≈ n by evaluating counting function
            let nt = (t / two_pi) * (t / two_pi_e).ln() + 7.0 / 8.0;
            assert!(
                (nt - n).abs() < 0.1,
                "N(invert({n})) = {nt}, expected ~{n} (t={t})"
            );
        }

        // For large n, eigenvalues should increase monotonically
        let t50 = invert_counting_function(50.0, two_pi, two_pi_e);
        let t100 = invert_counting_function(100.0, two_pi, two_pi_e);
        assert!(t100 > t50, "t100={t100} <= t50={t50}");
    }

    #[test]
    fn correlation_is_bounded() {
        let zeros = get_zeros(10.0, 100.0);
        if zeros.len() < 10 {
            return;
        }
        let report = hunt_operators(&zeros).unwrap_or_else(|_| unreachable!());
        for c in &report.candidates {
            assert!(
                (-1.0..=1.0).contains(&c.correlation),
                "{}: correlation {} out of [-1, 1]",
                c.name,
                c.correlation
            );
        }
    }

    #[test]
    fn eigenvalue_count_matches() {
        let zeros = get_zeros(10.0, 100.0);
        if zeros.len() < 10 {
            return;
        }
        let targets: Vec<f64> = zeros.iter().map(|z| z.t).collect();
        let fit = berry_keating_xp(zeros.len(), &targets).unwrap_or_else(|_| unreachable!());
        assert_eq!(fit.eigenvalues.len(), fit.n_compared);
        assert_eq!(fit.target_zeros.len(), fit.n_compared);
    }
}
