//! Complex Gamma function via the Lanczos approximation.
//!
//! Uses g=7 Lanczos coefficients with the reflection formula for Re(z) < 0.5.
//! Poles at non-positive integers are detected and return [`ComplexError::Undefined`].

use std::f64::consts::PI;

use crate::complex::Complex;
use crate::error::ComplexError;
use crate::functions;

/// sqrt(2π) — precomputed constant.
const SQRT_2PI: f64 = 2.506_628_274_631_000_5;

/// Lanczos g parameter.
const LANCZOS_G: f64 = 7.0;

/// p[0] coefficient (Lanczos series constant term).
const LANCZOS_P0: f64 = 0.999_999_999_999_809_9;

/// Remaining Lanczos coefficients paired with their k-offset (1.0..8.0).
/// Using literal f64 pairs avoids usize-to-f64 casts.
const LANCZOS_PAIRS: [(f64, f64); 8] = [
    (1.0, 676.520_368_121_885_1),
    (2.0, -1_259.139_216_722_402_8),
    (3.0, 771.323_428_777_653_1),
    (4.0, -176.615_029_162_140_6),
    (5.0, 12.507_343_278_686_905),
    (6.0, -0.138_571_095_265_720_12),
    (7.0, 9.984_369_578_019_572e-6),
    (8.0, 1.505_632_735_149_311_6e-7),
];

/// Returns `true` when `z` is a non-positive integer (a pole of Gamma).
fn is_nonpositive_integer(z: Complex) -> bool {
    z.im.abs() < 1e-10 && z.re <= 0.0 && (z.re - z.re.round()).abs() < 1e-10
}

/// Core Lanczos computation for Re(z) >= 0.5.
fn lanczos_core(z: Complex) -> Result<Complex, ComplexError> {
    let z_prime = z - Complex::ONE; // z' = z − 1

    // Ag = p[0] + Σ p[k] / (z' + k)  for k = 1..8
    let mut ag = Complex::from(LANCZOS_P0);
    for &(k, p_k) in &LANCZOS_PAIRS {
        let denom = z_prime + Complex::from(k);
        let term = Complex::from(p_k).div(denom)?;
        ag = ag + term;
    }

    // t = z' + g + 0.5
    let t = z_prime + Complex::from(LANCZOS_G + 0.5);

    // t^(z'+0.5) = exp((z'+0.5) · ln(t))
    let exponent = z_prime + Complex::from(0.5);
    let ln_t = functions::ln(t)?;
    let t_pow = functions::exp(exponent * ln_t);

    // Γ(z) = sqrt(2π) · t^(z'+0.5) · exp(−t) · Ag
    let result = Complex::from(SQRT_2PI) * t_pow * functions::exp(-t) * ag;
    Ok(result)
}

/// Computes the complex Gamma function Γ(z).
///
/// Uses the Lanczos approximation (g=7) for Re(z) ≥ 0.5 and the reflection
/// formula `Γ(z) = π / (sin(πz) · Γ(1−z))` for Re(z) < 0.5.
///
/// # Errors
///
/// Returns [`ComplexError::Undefined`] at poles (z = 0, −1, −2, …).
/// Returns [`ComplexError::DivisionByZero`] or [`ComplexError::LogOfZero`] on
/// numerical edge cases near poles.
///
/// # Examples
///
/// ```
/// use stem_complex::{Complex, gamma};
///
/// // Γ(1) = 1
/// let result = gamma::gamma(Complex::ONE);
/// assert!(result.is_ok());
/// ```
pub fn gamma(z: Complex) -> Result<Complex, ComplexError> {
    if is_nonpositive_integer(z) {
        return Err(ComplexError::Undefined(format!(
            "gamma has a pole at z = {z}"
        )));
    }

    if z.re < 0.5 {
        // Reflection formula: Γ(z) = π / (sin(πz) · Γ(1−z))
        let pi_z = Complex::from(PI) * z;
        let sin_pi_z = functions::sin(pi_z);
        let gamma_1mz = gamma(Complex::ONE - z)?;
        let denom = sin_pi_z * gamma_1mz;
        Complex::from(PI).div(denom)
    } else {
        lanczos_core(z)
    }
}

/// Computes the natural logarithm of the complex Gamma function, ln(Γ(z)).
///
/// # Errors
///
/// Propagates errors from [`gamma`] and [`functions::ln`].
///
/// # Examples
///
/// ```
/// use stem_complex::{Complex, gamma};
///
/// // ln(Γ(1)) = ln(1) = 0
/// let result = gamma::ln_gamma(Complex::ONE);
/// assert!(result.is_ok());
/// ```
pub fn ln_gamma(z: Complex) -> Result<Complex, ComplexError> {
    let g = gamma(z)?;
    functions::ln(g)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_one_equals_one() -> Result<(), ComplexError> {
        let r = gamma(Complex::ONE)?;
        assert!((r.re - 1.0).abs() < 1e-6, "Γ(1) re={}", r.re);
        assert!(r.im.abs() < 1e-6, "Γ(1) im={}", r.im);
        Ok(())
    }

    #[test]
    fn gamma_half_equals_sqrt_pi() -> Result<(), ComplexError> {
        // Γ(1/2) = √π ≈ 1.7724538509055159
        let r = gamma(Complex::from(0.5))?;
        let sqrt_pi = PI.sqrt();
        assert!(
            (r.re - sqrt_pi).abs() < 1e-6,
            "Γ(½) re={} expected={}",
            r.re,
            sqrt_pi
        );
        assert!(r.im.abs() < 1e-6, "Γ(½) im={}", r.im);
        Ok(())
    }

    #[test]
    fn gamma_factorial_4() -> Result<(), ComplexError> {
        // Γ(4) = 3! = 6
        let r = gamma(Complex::from(4.0))?;
        assert!((r.re - 6.0).abs() < 1e-4, "Γ(4) re={}", r.re);
        assert!(r.im.abs() < 1e-4);
        Ok(())
    }

    #[test]
    fn gamma_factorial_5() -> Result<(), ComplexError> {
        // Γ(5) = 4! = 24
        let r = gamma(Complex::from(5.0))?;
        assert!((r.re - 24.0).abs() < 1e-3, "Γ(5) re={}", r.re);
        assert!(r.im.abs() < 1e-3);
        Ok(())
    }

    #[test]
    fn gamma_pole_at_zero() {
        assert!(gamma(Complex::ZERO).is_err());
    }

    #[test]
    fn gamma_pole_at_neg_one() {
        assert!(gamma(Complex::from(-1.0)).is_err());
    }

    #[test]
    fn gamma_pole_at_neg_two() {
        assert!(gamma(Complex::from(-2.0)).is_err());
    }

    #[test]
    fn ln_gamma_one_is_zero() -> Result<(), ComplexError> {
        let r = ln_gamma(Complex::ONE)?;
        assert!(r.re.abs() < 1e-6, "ln_gamma(1) re={}", r.re);
        assert!(r.im.abs() < 1e-6, "ln_gamma(1) im={}", r.im);
        Ok(())
    }

    #[test]
    fn gamma_two_is_one() -> Result<(), ComplexError> {
        // Γ(2) = 1! = 1
        let r = gamma(Complex::from(2.0))?;
        assert!((r.re - 1.0).abs() < 1e-6, "Γ(2) re={}", r.re);
        assert!(r.im.abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn gamma_three_is_two() -> Result<(), ComplexError> {
        // Γ(3) = 2! = 2
        let r = gamma(Complex::from(3.0))?;
        assert!((r.re - 2.0).abs() < 1e-5, "Γ(3) re={}", r.re);
        assert!(r.im.abs() < 1e-5);
        Ok(())
    }
}
