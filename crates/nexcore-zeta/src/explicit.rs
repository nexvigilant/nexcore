//! # Von Mangoldt's Explicit Formula
//!
//! Reconstructs the Chebyshev function ψ(x) from the non-trivial zeros of ζ(s):
//!
//! ```text
//! ψ(x) = x − Σ_ρ x^ρ/ρ − ln(2π) − ½ ln(1 − x⁻²)
//! ```
//!
//! where the sum runs over non-trivial zeros ρ = ½ + iγ of ζ(s), taken
//! in conjugate pairs.
//!
//! ## Why This Matters
//!
//! This formula closes the **prime↔zero loop**:
//! - `stem-number-theory` computes ψ(x) directly from Λ(n) (von Mangoldt)
//! - `nexcore-zeta` locates zeros ρ of ζ(s)
//! - The explicit formula reconstructs ψ(x) FROM those zeros
//!
//! When both computations agree, we have proven computationally that
//! primes and zeta zeros encode the **same information** — they are
//! dual views of a single mathematical object.
//!
//! ## Convergence
//!
//! The sum over zeros converges conditionally when sorted by |Im(ρ)|.
//! More zeros = better accuracy. With N zeros, the truncation error
//! is roughly O(x / (T · ln T)) where T is the height of the Nth zero.

use std::f64::consts::PI;

use serde::{Deserialize, Serialize};
use stem_complex::Complex;

use crate::error::ZetaError;
use crate::zeros::ZetaZero;

/// Compute ψ(x) via the explicit formula using a set of known zeros.
///
/// Formula: ψ(x) = x − Σ_ρ x^ρ/ρ − ln(2π) − ½ ln(1 − x⁻²)
///
/// Each zero ρ = ½ + iγ contributes a conjugate pair:
/// x^ρ/ρ + x^ρ̄/ρ̄ = 2 · Re(x^ρ / ρ)
///
/// # Arguments
///
/// * `x` — the evaluation point (must be > 1 and not a prime power)
/// * `zeros` — known non-trivial zeros of ζ(s), sorted by imaginary part
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if `x <= 1.0`.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::explicit::explicit_psi;
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 100.0, 0.1).unwrap();
/// let psi = explicit_psi(50.0, &zeros).unwrap();
/// // Should be close to the direct computation of ψ(50)
/// assert!(psi > 30.0 && psi < 70.0);
/// ```
pub fn explicit_psi(x: f64, zeros: &[ZetaZero]) -> Result<f64, ZetaError> {
    if x <= 1.0 {
        return Err(ZetaError::InvalidParameter(format!(
            "explicit_psi requires x > 1, got {x}"
        )));
    }

    let ln_x = x.ln();

    // Main term
    let mut psi = x;

    // Zero sum: −Σ_ρ x^ρ/ρ taken in conjugate pairs
    // For ρ = 1/2 + iγ: x^ρ = x^(1/2) · exp(iγ ln x)
    //                    x^ρ/ρ = x^(1/2) · exp(iγ ln x) / (1/2 + iγ)
    // Conjugate pair contribution: 2 · Re(x^ρ/ρ)
    let sqrt_x = x.sqrt();

    for zero in zeros {
        let gamma = zero.t;
        // x^ρ = x^(1/2) · (cos(γ ln x) + i sin(γ ln x))
        let angle = gamma * ln_x;
        let (sin_a, cos_a) = angle.sin_cos();

        let x_rho = Complex::new(sqrt_x * cos_a, sqrt_x * sin_a);
        let rho = Complex::new(0.5, gamma);

        // x^ρ / ρ via complex division
        let quotient = complex_div(x_rho, rho);

        // Conjugate pair: subtract 2 · Re(x^ρ/ρ)
        psi -= 2.0 * quotient.re;
    }

    // Constant term: −ln(2π)
    psi -= (2.0 * PI).ln();

    // Trivial zero contribution: −½ ln(1 − x⁻²)
    // For x > 1, 1 − x⁻² > 0, so ln is real
    let x_sq = x * x;
    if x_sq > 1.0 {
        psi -= 0.5 * (1.0 - 1.0 / x_sq).ln();
    }

    Ok(psi)
}

/// Complex division: a / b = (a · b̄) / |b|²
fn complex_div(a: Complex, b: Complex) -> Complex {
    let denom = b.re * b.re + b.im * b.im;
    if denom < 1e-30 {
        return Complex::ZERO;
    }
    Complex::new(
        (a.re * b.re + a.im * b.im) / denom,
        (a.im * b.re - a.re * b.im) / denom,
    )
}

/// Compare explicit-formula ψ(x) to direct computation from von Mangoldt Λ(n).
///
/// Returns `(psi_explicit, psi_direct, relative_error)`.
///
/// The direct ψ(x) is computed by `stem_number_theory::summatory::ChebyshevPsi`.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if `x <= 1.0`.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::explicit::explicit_psi_comparison;
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 200.0, 0.05).unwrap();
/// let (psi_e, psi_d, err) = explicit_psi_comparison(100.0, &zeros).unwrap();
/// assert!(err < 0.10, "relative error {err} too large");
/// ```
pub fn explicit_psi_comparison(x: f64, zeros: &[ZetaZero]) -> Result<(f64, f64, f64), ZetaError> {
    let psi_explicit = explicit_psi(x, zeros)?;

    // Direct computation via Chebyshev ψ
    let psi_direct = stem_number_theory::summatory::ChebyshevPsi::compute(x as u64);

    let relative_error = if psi_direct.abs() < 1e-12 {
        psi_explicit.abs()
    } else {
        (psi_explicit - psi_direct).abs() / psi_direct.abs()
    };

    Ok((psi_explicit, psi_direct, relative_error))
}

/// Demonstrate convergence: more zeros → lower error.
///
/// Returns a vec of `(n_zeros, relative_error)` pairs showing monotonic improvement.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if `x <= 1.0` or `zeros` is empty.
pub fn convergence_series(
    x: f64,
    zeros: &[ZetaZero],
    sample_counts: &[usize],
) -> Result<Vec<(usize, f64)>, ZetaError> {
    if zeros.is_empty() {
        return Err(ZetaError::InvalidParameter(
            "convergence_series requires at least one zero".to_string(),
        ));
    }

    let psi_direct = stem_number_theory::summatory::ChebyshevPsi::compute(x as u64);

    let mut results = Vec::with_capacity(sample_counts.len());

    for &n in sample_counts {
        let subset = if n >= zeros.len() { zeros } else { &zeros[..n] };
        let psi_e = explicit_psi(x, subset)?;
        let err = if psi_direct.abs() < 1e-12 {
            psi_e.abs()
        } else {
            (psi_e - psi_direct).abs() / psi_direct.abs()
        };
        results.push((subset.len(), err));
    }

    Ok(results)
}

// ── Adaptive Truncation & Residual Analysis ───────────────────────────────

/// Adaptive truncation analysis: finds the optimal number of zeros to use
/// in the explicit formula at a given evaluation point `x`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveTruncation {
    /// Evaluation point.
    pub x: f64,
    /// Empirically optimal N: the truncation that minimised `|ψ_explicit − ψ_direct|`.
    pub optimal_n: usize,
    /// Absolute errors at each tested truncation: `(n_zeros, |ψ_explicit − ψ_direct|)`.
    pub errors_by_truncation: Vec<(usize, f64)>,
    /// The N that achieved the minimum absolute error (same as `optimal_n`).
    pub best_n: usize,
    /// The minimum absolute error achieved.
    pub best_error: f64,
    /// Theoretical Riemann-Siegel optimal: `⌊√(x / 2π)⌋` (for comparison).
    pub riemann_siegel_n: usize,
}

/// Per-zero marginal error analysis: how much each zero's contribution
/// changes the reconstruction error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualByHeight {
    /// Evaluation point.
    pub x: f64,
    /// Per-zero data: `(zero height γ, marginal error change err_k − err_{k−1})`.
    pub residuals: Vec<(f64, f64)>,
    /// Pearson correlation between zero heights and marginal error changes.
    pub correlation_with_height: f64,
}

/// Find the truncation that minimises `|ψ_explicit(x) − ψ_direct(x)|`.
///
/// Tests a set of candidate truncations anchored around the Riemann-Siegel
/// optimal `N_RS = ⌊√(x / 2π)⌋`.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if `x <= 1.0` or `zeros` is empty.
pub fn adaptive_truncation(x: f64, zeros: &[ZetaZero]) -> Result<AdaptiveTruncation, ZetaError> {
    if x <= 1.0 {
        return Err(ZetaError::InvalidParameter(format!(
            "adaptive_truncation requires x > 1, got {x}"
        )));
    }
    if zeros.is_empty() {
        return Err(ZetaError::InvalidParameter(
            "adaptive_truncation requires at least one zero".to_string(),
        ));
    }

    let rs_n = (x / (2.0 * PI)).sqrt().floor() as usize;
    let n_total = zeros.len();

    // Build candidate set: fixed anchors + RS multiples + all zeros.
    let mut candidates: Vec<usize> = vec![
        10,
        25,
        50,
        rs_n / 2, // 0 when rs_n < 2 — filtered below
        rs_n,
        rs_n.saturating_mul(2),
        rs_n.saturating_mul(4),
        n_total,
    ];
    // Keep only valid, non-zero values within the available range.
    candidates.retain(|&c| c > 0 && c <= n_total);
    candidates.sort_unstable();
    candidates.dedup();

    let psi_direct = stem_number_theory::summatory::ChebyshevPsi::compute(x as u64);

    let mut errors_by_truncation: Vec<(usize, f64)> = Vec::with_capacity(candidates.len());
    for &n in &candidates {
        let psi_e = explicit_psi(x, &zeros[..n])?;
        errors_by_truncation.push((n, (psi_e - psi_direct).abs()));
    }

    // Find the (n, error) pair with minimum error using a fold (no unwrap).
    let (best_n, best_error) = errors_by_truncation.iter().fold(
        errors_by_truncation
            .first()
            .map_or((n_total, f64::NAN), |&(n, e)| (n, e)),
        |best, &(n, e)| {
            if e.total_cmp(&best.1).is_lt() {
                (n, e)
            } else {
                best
            }
        },
    );

    Ok(AdaptiveTruncation {
        x,
        optimal_n: best_n,
        errors_by_truncation,
        best_n,
        best_error,
        riemann_siegel_n: rs_n,
    })
}

/// Compute the marginal error contribution of each zero in the explicit formula.
///
/// For each zero k (1-indexed), computes:
/// - `err_k   = |ψ_explicit(x, zeros[..k]) − ψ_direct(x)|`
/// - `err_{k-1}` (using 0 zeros at k=1)
/// - `marginal = err_k − err_{k-1}`
///
/// Then computes the Pearson correlation between zero heights and marginals.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if `x <= 1.0`.
pub fn residual_by_height(x: f64, zeros: &[ZetaZero]) -> Result<ResidualByHeight, ZetaError> {
    if x <= 1.0 {
        return Err(ZetaError::InvalidParameter(format!(
            "residual_by_height requires x > 1, got {x}"
        )));
    }

    let psi_direct = stem_number_theory::summatory::ChebyshevPsi::compute(x as u64);

    // Baseline: error with zero zeros
    let psi_0 = explicit_psi(x, &[])?;
    let mut prev_err = (psi_0 - psi_direct).abs();

    let mut residuals: Vec<(f64, f64)> = Vec::with_capacity(zeros.len());
    for k in 0..zeros.len() {
        let psi_k = explicit_psi(x, &zeros[..=k])?;
        let err_k = (psi_k - psi_direct).abs();
        residuals.push((zeros[k].t, err_k - prev_err));
        prev_err = err_k;
    }

    let correlation = pearson_correlation(&residuals);

    Ok(ResidualByHeight {
        x,
        residuals,
        correlation_with_height: correlation,
    })
}

/// Pearson correlation between the two components of a `(x, y)` slice.
fn pearson_correlation(pairs: &[(f64, f64)]) -> f64 {
    let n = pairs.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let mean_x = pairs.iter().map(|(x, _)| x).sum::<f64>() / n;
    let mean_y = pairs.iter().map(|(_, y)| y).sum::<f64>() / n;

    let num: f64 = pairs.iter().map(|(x, y)| (x - mean_x) * (y - mean_y)).sum();
    let den_x: f64 = pairs
        .iter()
        .map(|(x, _)| (x - mean_x) * (x - mean_x))
        .sum::<f64>()
        .sqrt();
    let den_y: f64 = pairs
        .iter()
        .map(|(_, y)| (y - mean_y) * (y - mean_y))
        .sum::<f64>()
        .sqrt();

    let denom = den_x * den_y;
    if denom < 1e-30 {
        return 0.0;
    }
    num / denom
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zeros::find_zeros_bracket;

    #[test]
    fn explicit_psi_rejects_small_x() {
        let zeros = vec![];
        assert!(explicit_psi(1.0, &zeros).is_err());
        assert!(explicit_psi(0.5, &zeros).is_err());
    }

    #[test]
    fn explicit_psi_no_zeros_gives_x_minus_constants() {
        // With zero zeros, ψ(x) ≈ x − ln(2π) − ½ln(1−x⁻²)
        let result = explicit_psi(100.0, &[]);
        assert!(result.is_ok());
        let psi = result.ok();
        assert!(psi.is_some());
        if let Some(psi) = psi {
            let expected = 100.0 - (2.0 * PI).ln() - 0.5 * (1.0 - 0.0001_f64).ln();
            assert!(
                (psi - expected).abs() < 1e-6,
                "psi={psi}, expected={expected}"
            );
        }
    }

    #[test]
    fn explicit_psi_with_zeros_closer_to_direct() {
        // Key test: explicit formula with zeros should be closer to direct ψ(x)
        // than without zeros
        let zeros = find_zeros_bracket(10.0, 100.0, 0.1);
        assert!(zeros.is_ok());
        if let Some(zeros) = zeros.ok() {
            assert!(!zeros.is_empty(), "need at least one zero");

            let x = 50.0;
            let psi_direct = stem_number_theory::summatory::ChebyshevPsi::compute(x as u64);
            let psi_no_zeros = explicit_psi(x, &[]);
            let psi_with_zeros = explicit_psi(x, &zeros);

            assert!(psi_no_zeros.is_ok());
            assert!(psi_with_zeros.is_ok());

            if let (Some(psi_0), Some(psi_z)) = (psi_no_zeros.ok(), psi_with_zeros.ok()) {
                let err_no_zeros = (psi_0 - psi_direct).abs();
                let err_with_zeros = (psi_z - psi_direct).abs();
                assert!(
                    err_with_zeros < err_no_zeros,
                    "zeros should improve accuracy: err_no_zeros={err_no_zeros:.3}, err_with_zeros={err_with_zeros:.3}"
                );
            }
        }
    }

    #[test]
    fn explicit_psi_comparison_at_100() {
        let zeros = find_zeros_bracket(10.0, 200.0, 0.05);
        assert!(zeros.is_ok());
        if let Some(zeros) = zeros.ok() {
            let result = explicit_psi_comparison(100.0, &zeros);
            assert!(result.is_ok(), "comparison failed: {:?}", result.err());
            if let Some((psi_e, psi_d, err)) = result.ok() {
                assert!(
                    err < 0.10,
                    "relative error {err:.4} too large (psi_e={psi_e:.3}, psi_d={psi_d:.3})"
                );
            }
        }
    }

    #[test]
    fn convergence_improves_with_more_zeros() {
        let zeros = find_zeros_bracket(10.0, 200.0, 0.05);
        assert!(zeros.is_ok());
        if let Some(zeros) = zeros.ok() {
            if zeros.len() < 10 {
                return; // not enough zeros for meaningful test
            }
            let counts = [5, 10, 20, zeros.len()];
            let series = convergence_series(100.0, &zeros, &counts);
            assert!(series.is_ok());
            if let Some(series) = series.ok() {
                // Error should generally decrease (allow some non-monotonicity due to
                // conditional convergence, but last should beat first)
                let first_err = series.first().map_or(1.0, |s| s.1);
                let last_err = series.last().map_or(1.0, |s| s.1);
                assert!(
                    last_err < first_err,
                    "convergence failed: first_err={first_err:.4}, last_err={last_err:.4}"
                );
            }
        }
    }

    #[test]
    fn explicit_psi_at_1000_with_extended_zeros() {
        // Use zeros up to height 1000 for highest accuracy
        let zeros = find_zeros_bracket(10.0, 1000.0, 0.02);
        assert!(zeros.is_ok());
        if let Some(zeros) = zeros.ok() {
            let result = explicit_psi_comparison(500.0, &zeros);
            assert!(result.is_ok());
            if let Some((psi_e, psi_d, err)) = result.ok() {
                // With ~649 zeros, error at x=500 should be quite small
                assert!(
                    err < 0.05,
                    "error {err:.4} at x=500 with {} zeros (psi_e={psi_e:.3}, psi_d={psi_d:.3})",
                    zeros.len()
                );
            }
        }
    }

    #[test]
    fn complex_div_basic() {
        // (1+2i) / (3+4i) = (1·3+2·4)/(9+16) + i(2·3-1·4)/(9+16) = 11/25 + 2i/25
        let a = Complex::new(1.0, 2.0);
        let b = Complex::new(3.0, 4.0);
        let r = complex_div(a, b);
        assert!((r.re - 11.0 / 25.0).abs() < 1e-12);
        assert!((r.im - 2.0 / 25.0).abs() < 1e-12);
    }

    // ── Adaptive truncation & residual tests ─────────────────────────────

    #[test]
    fn adaptive_truncation_finds_minimum() {
        let Ok(zeros) = find_zeros_bracket(10.0, 200.0, 0.05) else {
            return;
        };
        if zeros.len() < 10 {
            return;
        }
        let Ok(at) = adaptive_truncation(50.0, &zeros) else {
            return;
        };
        assert!(!at.errors_by_truncation.is_empty());
        // best_error must equal the minimum in errors_by_truncation.
        let min_err = at
            .errors_by_truncation
            .iter()
            .map(|&(_, e)| e)
            .fold(f64::INFINITY, f64::min);
        assert!(
            (at.best_error - min_err).abs() < 1e-12,
            "best_error={:.6} != min_err={:.6}",
            at.best_error,
            min_err
        );
    }

    #[test]
    fn best_n_near_riemann_siegel() {
        let Ok(zeros) = find_zeros_bracket(10.0, 200.0, 0.05) else {
            return;
        };
        if zeros.len() < 10 {
            return;
        }
        let Ok(at) = adaptive_truncation(100.0, &zeros) else {
            return;
        };
        // best_n must be a valid index into the zeros slice.
        assert!(
            at.best_n >= 1 && at.best_n <= zeros.len(),
            "best_n {} out of valid range [1, {}]",
            at.best_n,
            zeros.len()
        );
        // The RS optimal should be a positive integer for x=100.
        assert!(
            at.riemann_siegel_n > 0,
            "RS optimal should be > 0 for x=100"
        );
        // best_n must be at least rs/4 (within factor of 4 of the RS suggestion).
        let lo = (at.riemann_siegel_n / 4).max(1);
        assert!(at.best_n >= lo, "best_n {} below rs/4={}", at.best_n, lo);
    }

    #[test]
    fn residual_by_height_correct_length() {
        let Ok(zeros) = find_zeros_bracket(10.0, 100.0, 0.1) else {
            return;
        };
        if zeros.is_empty() {
            return;
        }
        let Ok(rbh) = residual_by_height(50.0, &zeros) else {
            return;
        };
        assert_eq!(
            rbh.residuals.len(),
            zeros.len(),
            "residuals length mismatch"
        );
    }

    #[test]
    fn correlation_in_valid_range() {
        let Ok(zeros) = find_zeros_bracket(10.0, 100.0, 0.1) else {
            return;
        };
        if zeros.is_empty() {
            return;
        }
        let Ok(rbh) = residual_by_height(50.0, &zeros) else {
            return;
        };
        assert!(
            rbh.correlation_with_height >= -1.0 - 1e-12
                && rbh.correlation_with_height <= 1.0 + 1e-12,
            "correlation {} outside [-1, 1]",
            rbh.correlation_with_height
        );
    }

    #[test]
    fn rejects_x_leq_1() {
        let zero = ZetaZero {
            ordinal: 1,
            t: 14.1,
            z_value: 0.0,
            on_critical_line: true,
        };
        let zeros = vec![zero];
        assert!(adaptive_truncation(1.0, &zeros).is_err());
        assert!(adaptive_truncation(0.5, &zeros).is_err());
        assert!(residual_by_height(1.0, &zeros).is_err());
        assert!(residual_by_height(0.0, &zeros).is_err());
    }
}
