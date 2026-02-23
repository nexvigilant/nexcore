//! Complex elementary functions.
//!
//! All functions operate on [`Complex`](crate::Complex) numbers.
//! Functions that can fail (e.g., logarithm of zero) return [`Result`].

use crate::complex::Complex;
use crate::error::ComplexError;

/// Computes the complex exponential `e^z = e^(re) · (cos(im) + i·sin(im))`.
///
/// # Examples
///
/// ```
/// use stem_complex::{Complex, functions};
///
/// let result = functions::exp(Complex::ZERO);
/// assert!((result.re - 1.0).abs() < 1e-12);
/// assert!(result.im.abs() < 1e-12);
/// ```
#[must_use]
pub fn exp(z: Complex) -> Complex {
    let e_re = z.re.exp();
    Complex::new(e_re * z.im.cos(), e_re * z.im.sin())
}

/// Computes the principal complex logarithm `ln(z) = ln|z| + i·arg(z)`.
///
/// # Errors
///
/// Returns [`ComplexError::LogOfZero`] when `z` is exactly zero.
///
/// # Examples
///
/// ```
/// use stem_complex::{Complex, functions};
///
/// let result = functions::ln(Complex::ONE);
/// assert!(result.is_ok());
/// ```
pub fn ln(z: Complex) -> Result<Complex, ComplexError> {
    if z.is_zero() {
        return Err(ComplexError::LogOfZero);
    }
    Ok(Complex::new(z.abs().ln(), z.arg()))
}

/// Computes `z^w = exp(w · ln(z))`.
///
/// # Errors
///
/// Returns an error when `z` is zero (logarithm undefined).
///
/// # Examples
///
/// ```
/// use stem_complex::{Complex, functions};
///
/// // 1^anything = 1
/// let result = functions::pow(Complex::ONE, Complex::new(2.0, 1.0));
/// assert!(result.is_ok());
/// ```
pub fn pow(z: Complex, w: Complex) -> Result<Complex, ComplexError> {
    let ln_z = ln(z)?;
    Ok(exp(w * ln_z))
}

/// Computes the principal square root of `z`.
///
/// Uses the polar form: `sqrt(z) = sqrt(|z|) · exp(i·arg(z)/2)`.
///
/// # Examples
///
/// ```
/// use stem_complex::{Complex, functions};
///
/// let result = functions::sqrt(Complex::new(4.0, 0.0));
/// assert!((result.re - 2.0).abs() < 1e-12);
/// assert!(result.im.abs() < 1e-12);
/// ```
#[must_use]
pub fn sqrt(z: Complex) -> Complex {
    if z.is_zero() {
        return Complex::ZERO;
    }
    Complex::polar(z.abs().sqrt(), z.arg() / 2.0)
}

/// Computes the complex sine via the exponential form:
/// `sin(z) = (exp(iz) - exp(-iz)) / (2i)`.
///
/// # Examples
///
/// ```
/// use stem_complex::{Complex, functions};
/// use std::f64::consts::PI;
///
/// // sin(0) = 0
/// let result = functions::sin(Complex::ZERO);
/// assert!(result.re.abs() < 1e-12);
/// assert!(result.im.abs() < 1e-12);
/// ```
#[must_use]
pub fn sin(z: Complex) -> Complex {
    let iz = Complex::new(-z.im, z.re);
    let exp_iz = exp(iz);
    let exp_neg_iz = exp(-iz);
    let diff = exp_iz - exp_neg_iz;
    // diff / (2i) — multiply by the scalar -i/2: (re, im) → (im/2, -re/2)
    Complex::new(diff.im / 2.0, -diff.re / 2.0)
}

/// Computes the complex cosine via the exponential form:
/// `cos(z) = (exp(iz) + exp(-iz)) / 2`.
///
/// # Examples
///
/// ```
/// use stem_complex::{Complex, functions};
///
/// // cos(0) = 1
/// let result = functions::cos(Complex::ZERO);
/// assert!((result.re - 1.0).abs() < 1e-12);
/// assert!(result.im.abs() < 1e-12);
/// ```
#[must_use]
pub fn cos(z: Complex) -> Complex {
    let iz = Complex::new(-z.im, z.re);
    let exp_iz = exp(iz);
    let exp_neg_iz = exp(-iz);
    let sum = exp_iz + exp_neg_iz;
    Complex::new(sum.re / 2.0, sum.im / 2.0)
}

/// Computes the complex hyperbolic sine `sinh(z) = (exp(z) - exp(-z)) / 2`.
///
/// # Examples
///
/// ```
/// use stem_complex::{Complex, functions};
///
/// // sinh(0) = 0
/// let result = functions::sinh(Complex::ZERO);
/// assert!(result.re.abs() < 1e-12);
/// assert!(result.im.abs() < 1e-12);
/// ```
#[must_use]
pub fn sinh(z: Complex) -> Complex {
    let diff = exp(z) - exp(-z);
    Complex::new(diff.re / 2.0, diff.im / 2.0)
}

/// Computes the complex hyperbolic cosine `cosh(z) = (exp(z) + exp(-z)) / 2`.
///
/// # Examples
///
/// ```
/// use stem_complex::{Complex, functions};
///
/// // cosh(0) = 1
/// let result = functions::cosh(Complex::ZERO);
/// assert!((result.re - 1.0).abs() < 1e-12);
/// assert!(result.im.abs() < 1e-12);
/// ```
#[must_use]
pub fn cosh(z: Complex) -> Complex {
    let sum = exp(z) + exp(-z);
    Complex::new(sum.re / 2.0, sum.im / 2.0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{E, PI};

    #[test]
    fn exp_of_zero_is_one() {
        let r = exp(Complex::ZERO);
        assert!((r.re - 1.0).abs() < 1e-12);
        assert!(r.im.abs() < 1e-12);
    }

    #[test]
    fn euler_identity_exp_i_pi() {
        // exp(i·π) = -1
        let r = exp(Complex::new(0.0, PI));
        assert!((r.re + 1.0).abs() < 1e-12);
        assert!(r.im.abs() < 1e-12);
    }

    #[test]
    fn ln_of_one_is_zero() -> Result<(), ComplexError> {
        let r = ln(Complex::ONE)?;
        assert!(r.re.abs() < 1e-12);
        assert!(r.im.abs() < 1e-12);
        Ok(())
    }

    #[test]
    fn ln_of_e_is_one() -> Result<(), ComplexError> {
        let r = ln(Complex::from(E))?;
        assert!((r.re - 1.0).abs() < 1e-12);
        assert!(r.im.abs() < 1e-12);
        Ok(())
    }

    #[test]
    fn ln_of_zero_returns_err() {
        assert!(ln(Complex::ZERO).is_err());
    }

    #[test]
    fn sqrt_of_four() {
        let r = sqrt(Complex::new(4.0, 0.0));
        assert!((r.re - 2.0).abs() < 1e-12);
        assert!(r.im.abs() < 1e-12);
    }

    #[test]
    fn sqrt_of_negative_one_is_i() {
        // sqrt(-1) = i
        let r = sqrt(Complex::new(-1.0, 0.0));
        assert!(r.re.abs() < 1e-12);
        assert!((r.im - 1.0).abs() < 1e-12);
    }

    #[test]
    fn sqrt_of_zero() {
        assert_eq!(sqrt(Complex::ZERO), Complex::ZERO);
    }

    #[test]
    fn sin_of_zero() {
        let r = sin(Complex::ZERO);
        assert!(r.re.abs() < 1e-12);
        assert!(r.im.abs() < 1e-12);
    }

    #[test]
    fn cos_of_zero() {
        let r = cos(Complex::ZERO);
        assert!((r.re - 1.0).abs() < 1e-12);
        assert!(r.im.abs() < 1e-12);
    }

    #[test]
    fn pythagorean_identity_sin_sq_plus_cos_sq() {
        // sin²(z) + cos²(z) = 1 for z = 1 + 0.5i
        let z = Complex::new(1.0, 0.5);
        let sin_z = sin(z);
        let cos_z = cos(z);
        let sum = sin_z * sin_z + cos_z * cos_z;
        assert!((sum.re - 1.0).abs() < 1e-10);
        assert!(sum.im.abs() < 1e-10);
    }

    #[test]
    fn sinh_of_zero() {
        let r = sinh(Complex::ZERO);
        assert!(r.re.abs() < 1e-12);
        assert!(r.im.abs() < 1e-12);
    }

    #[test]
    fn cosh_of_zero_is_one() {
        let r = cosh(Complex::ZERO);
        assert!((r.re - 1.0).abs() < 1e-12);
        assert!(r.im.abs() < 1e-12);
    }

    #[test]
    fn cosh_sq_minus_sinh_sq_is_one() {
        // cosh²(z) - sinh²(z) = 1 for z = 0.5 + 0.3i
        let z = Complex::new(0.5, 0.3);
        let c = cosh(z);
        let s = sinh(z);
        let diff = c * c - s * s;
        assert!((diff.re - 1.0).abs() < 1e-10);
        assert!(diff.im.abs() < 1e-10);
    }

    #[test]
    fn pow_one_to_anything_is_one() -> Result<(), ComplexError> {
        let result = pow(Complex::ONE, Complex::new(3.0, 2.0))?;
        assert!((result.re - 1.0).abs() < 1e-10);
        assert!(result.im.abs() < 1e-10);
        Ok(())
    }

    #[test]
    fn pow_zero_base_returns_err() {
        assert!(pow(Complex::ZERO, Complex::ONE).is_err());
    }
}
