//! Main Riemann Zeta function ζ(s).
//!
//! ## Evaluation Strategy
//!
//! | Region | Method |
//! |--------|--------|
//! | Re(s) > 1 | Dirichlet series + Euler–Maclaurin |
//! | Critical strip 0 < Re(s) ≤ 1 | Dirichlet eta relation |
//! | Re(s) ≤ 0 | Functional equation |
//! | s = 1 | Pole (error) |

use std::f64::consts::PI;

use stem_complex::Complex;
use stem_complex::{functions, gamma};

use crate::error::ZetaError;
use crate::special::{bernoulli_numbers, dirichlet_eta};

/// Computes the Riemann zeta function ζ(s).
///
/// Dispatches to the appropriate algorithm based on the region of the
/// complex plane.
///
/// # Errors
///
/// - [`ZetaError::PoleAtOne`] when s ≈ 1
/// - [`ZetaError::Complex`] on underlying arithmetic errors
/// - [`ZetaError::ConvergenceFailure`] if series do not converge
///
/// # Examples
///
/// ```
/// use nexcore_zeta::zeta::zeta;
/// use stem_complex::Complex;
/// use std::f64::consts::PI;
///
/// // ζ(2) = π²/6
/// let result = zeta(Complex::from(2.0)).unwrap();
/// let expected = PI * PI / 6.0;
/// assert!((result.re - expected).abs() < 1e-4);
/// ```
pub fn zeta(s: Complex) -> Result<Complex, ZetaError> {
    // Pole detection at s = 1
    if (s.re - 1.0).abs() < 1e-10 && s.im.abs() < 1e-10 {
        return Err(ZetaError::PoleAtOne);
    }

    // Non-positive integers: closed form via Bernoulli numbers
    // ζ(-n) = (-1)^n B_{n+1} / (n+1)
    if s.im.abs() < 1e-10 && s.re <= 0.0 {
        let rounded = s.re.round();
        if (s.re - rounded).abs() < 1e-10 {
            let n = (-rounded) as usize; // s.re = -n
            let b = bernoulli_numbers(n + 2);
            let b_nplus1 = b[n + 1];
            let sign = if n % 2 == 0 { 1.0_f64 } else { -1.0_f64 };
            return Ok(Complex::from(sign * b_nplus1 / (n as f64 + 1.0)));
        }
    }

    if s.re > 1.0 {
        // Dirichlet series with Euler–Maclaurin correction
        zeta_euler_maclaurin(s, 128)
    } else if s.re >= 0.0 {
        // Critical strip (including Re=0): use eta relation ζ(s) = η(s) / (1 - 2^{1-s})
        zeta_via_eta(s)
    } else {
        // Re(s) < 0 (non-integer): functional equation
        zeta_functional_equation(s)
    }
}

/// ζ(s) via the Dirichlet eta relation.
///
/// ζ(s) = η(s) / (1 − 2^{1−s})
///
/// Works in the critical strip where the Dirichlet series diverges.
fn zeta_via_eta(s: Complex) -> Result<Complex, ZetaError> {
    let eta_s = dirichlet_eta(s)?;

    let one_minus_s = Complex::ONE - s;
    let two_pow = functions::pow(Complex::from(2.0), one_minus_s)?;
    let denom = Complex::ONE - two_pow;

    if denom.abs_sq() < 1e-30 {
        return Err(ZetaError::ConvergenceFailure { iterations: 0 });
    }

    let result = eta_s.div(denom)?;
    Ok(result)
}

/// ζ(s) via the functional equation for Re(s) ≤ 0.
///
/// ζ(s) = 2^s · π^{s−1} · sin(πs/2) · Γ(1−s) · ζ(1−s)
fn zeta_functional_equation(s: Complex) -> Result<Complex, ZetaError> {
    let one_minus_s = Complex::ONE - s;

    // For Re(s) < 0: Re(1-s) = 1-Re(s) > 1, so Euler-Maclaurin converges.
    let zeta_1ms = zeta_euler_maclaurin(one_minus_s, 128)?;

    // 2^s
    let two_s = functions::pow(Complex::from(2.0), s)?;

    // π^{s-1}
    let pi_s_minus_1 = functions::pow(Complex::from(PI), s - Complex::ONE)?;

    // sin(πs/2)
    let sin_term = functions::sin(Complex::from(PI / 2.0) * s);

    // Γ(1-s)
    let gamma_1ms = gamma::gamma(one_minus_s)?;

    let result = two_s * pi_s_minus_1 * sin_term * gamma_1ms * zeta_1ms;
    Ok(result)
}

/// Dirichlet series Σ_{n=1}^{N} 1/n^s (partial sum).
///
/// Only converges for Re(s) > 1.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::zeta::zeta_dirichlet;
/// use stem_complex::Complex;
///
/// let result = zeta_dirichlet(Complex::from(2.0), 10000).unwrap();
/// assert!((result.re - std::f64::consts::PI.powi(2) / 6.0).abs() < 0.01);
/// ```
pub fn zeta_dirichlet(s: Complex, n_terms: usize) -> Result<Complex, ZetaError> {
    let mut sum = Complex::ZERO;
    for n in 1..=n_terms {
        let base = Complex::from(n as f64);
        let term = functions::pow(base, s)?;
        sum = sum + Complex::ONE.div(term)?;
    }
    Ok(sum)
}

/// Euler–Maclaurin summation for ζ(s) with Bernoulli correction terms.
///
/// ζ(s) ≈ Σ_{n=1}^{N-1} n^{-s} + N^{1-s}/(s-1) + N^{-s}/2
///       + Σ_{k=1}^{p} B_{2k}/(2k)! · s(s+1)…(s+2k-2) · N^{-s-2k+1}
///
/// # Examples
///
/// ```
/// use nexcore_zeta::zeta::zeta_euler_maclaurin;
/// use stem_complex::Complex;
///
/// let result = zeta_euler_maclaurin(Complex::from(2.0), 50).unwrap();
/// let expected = std::f64::consts::PI.powi(2) / 6.0;
/// assert!((result.re - expected).abs() < 1e-6);
/// ```
pub fn zeta_euler_maclaurin(s: Complex, n_terms: usize) -> Result<Complex, ZetaError> {
    let n = n_terms.max(10);
    let p = 8_usize; // number of Bernoulli correction terms

    // Partial sum: Σ_{k=1}^{N-1} k^{-s}
    let mut sum = Complex::ZERO;
    for k in 1..n {
        let base = Complex::from(k as f64);
        let pow_s = functions::pow(base, s)?;
        sum = sum + Complex::ONE.div(pow_s)?;
    }

    let n_f = Complex::from(n as f64);

    // N^{1-s} / (s-1)
    let one_minus_s = Complex::ONE - s;
    let n_1ms = functions::pow(n_f, one_minus_s)?;
    let s_minus_1 = s - Complex::ONE;
    let integral_term = n_1ms.div(s_minus_1)?;
    sum = sum + integral_term;

    // N^{-s} / 2
    let n_neg_s = functions::pow(n_f, s)?;
    let half_term = Complex::from(0.5).div(n_neg_s)?;
    sum = sum + half_term;

    // Bernoulli correction terms
    let bernoulli = bernoulli_numbers(2 * p);
    let mut rising = s; // rising factorial product

    for k in 1..=p {
        let b2k = bernoulli[2 * k];

        // Factorial: (2k)!
        let mut fact = 1.0_f64;
        for j in 1..=(2 * k) {
            fact *= j as f64;
        }

        // N^{-(s+2k-1)} = 1 / N^{s+2k-1}
        let exponent = s + Complex::from((2 * k - 1) as f64);
        let n_pow = functions::pow(n_f, exponent)?;
        let n_term = Complex::ONE.div(n_pow)?;

        let coeff = Complex::from(b2k / fact);
        let correction = coeff * rising * n_term;
        sum = sum + correction;

        // Update rising product: multiply by (s + 2k - 1)(s + 2k)
        if k < p {
            let next1 = s + Complex::from((2 * k - 1) as f64);
            let next2 = s + Complex::from((2 * k) as f64);
            rising = rising * next1 * next2;
        }
    }

    Ok(sum)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeta_2_is_pi_sq_over_6() {
        let result = zeta(Complex::from(2.0));
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            let expected = PI * PI / 6.0;
            assert!(
                (v.re - expected).abs() < 1e-4,
                "ζ(2) = {} expected {}",
                v.re,
                expected
            );
            assert!(v.im.abs() < 1e-4);
        }
    }

    #[test]
    fn zeta_4_is_pi4_over_90() {
        let result = zeta(Complex::from(4.0));
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            let expected = PI.powi(4) / 90.0;
            assert!(
                (v.re - expected).abs() < 1e-4,
                "ζ(4) = {} expected {}",
                v.re,
                expected
            );
        }
    }

    #[test]
    fn zeta_pole_at_one() {
        let result = zeta(Complex::ONE);
        assert!(result.is_err());
    }

    #[test]
    fn zeta_0_is_neg_half() {
        let result = zeta(Complex::ZERO);
        assert!(result.is_ok(), "ζ(0) failed: {:?}", result.err());
        if let Some(v) = result.ok() {
            assert!((v.re + 0.5).abs() < 1e-6, "ζ(0) = {} expected -0.5", v.re);
            assert!(v.im.abs() < 1e-10);
        }
    }

    #[test]
    fn zeta_neg1_is_neg_1_over_12() {
        let result = zeta(Complex::from(-1.0));
        assert!(result.is_ok(), "ζ(-1) failed: {:?}", result.err());
        if let Some(v) = result.ok() {
            let expected = -1.0 / 12.0;
            assert!(
                (v.re - expected).abs() < 1e-6,
                "ζ(-1) = {} expected {}",
                v.re,
                expected
            );
            assert!(v.im.abs() < 1e-10);
        }
    }

    #[test]
    fn zeta_trivial_zero_at_neg2() {
        let result = zeta(Complex::from(-2.0));
        assert!(result.is_ok(), "ζ(-2) failed: {:?}", result.err());
        if let Some(v) = result.ok() {
            assert!(
                v.re.abs() < 1e-12,
                "ζ(-2) = {} expected 0 (trivial zero)",
                v.re
            );
        }
    }

    #[test]
    fn zeta_dirichlet_converges_for_re_gt_1() {
        let result = zeta_dirichlet(Complex::from(3.0), 10000);
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            assert!((v.re - 1.2020569).abs() < 0.01, "ζ(3) = {}", v.re);
        }
    }

    #[test]
    fn zeta_euler_maclaurin_at_2() {
        let result = zeta_euler_maclaurin(Complex::from(2.0), 50);
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            let expected = PI * PI / 6.0;
            assert!((v.re - expected).abs() < 1e-6, "EM ζ(2) = {}", v.re);
        }
    }
}
