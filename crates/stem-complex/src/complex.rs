//! Core [`Complex`] number type.
//!
//! Provides a `Copy`-able complex number with full arithmetic and geometric operations.

#![allow(
    clippy::module_name_repetitions,
    reason = "ComplexField and ComplexError are the idiomatic names for these types despite sharing the 'Complex' prefix"
)]

use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

use serde::{Deserialize, Serialize};

use crate::error::ComplexError;

/// A complex number `re + im·i`.
///
/// # Examples
///
/// ```
/// use stem_complex::Complex;
///
/// let z = Complex::new(3.0, 4.0);
/// assert!((z.abs() - 5.0).abs() < 1e-12);
/// ```
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Complex {
    /// Real part.
    pub re: f64,
    /// Imaginary part.
    pub im: f64,
}

impl Complex {
    /// The additive identity `0 + 0i`.
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };

    /// The multiplicative identity `1 + 0i`.
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };

    /// Creates a new complex number from real and imaginary parts.
    #[inline]
    #[must_use]
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// Returns the imaginary unit `0 + 1i`.
    #[inline]
    #[must_use]
    pub const fn i() -> Self {
        Self { re: 0.0, im: 1.0 }
    }

    /// Returns the magnitude (modulus) |z| = sqrt(re² + im²).
    ///
    /// # Examples
    ///
    /// ```
    /// use stem_complex::Complex;
    ///
    /// let z = Complex::new(3.0, 4.0);
    /// assert!((z.abs() - 5.0).abs() < 1e-12);
    /// ```
    #[inline]
    #[must_use]
    pub fn abs(self) -> f64 {
        self.abs_sq().sqrt()
    }

    /// Returns the squared magnitude re² + im² (avoids the sqrt call).
    #[inline]
    #[must_use]
    pub fn abs_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// Returns the argument (phase angle) in radians, in `(-π, π]`.
    #[inline]
    #[must_use]
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    /// Returns the complex conjugate `re - im·i`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stem_complex::Complex;
    ///
    /// let z = Complex::new(3.0, 4.0);
    /// assert_eq!(z.conj(), Complex::new(3.0, -4.0));
    /// ```
    #[inline]
    #[must_use]
    pub const fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    /// Returns `true` if both components are exactly zero.
    ///
    /// For an approximate zero test, compare [`abs_sq`](Self::abs_sq) against an epsilon.
    #[inline]
    #[must_use]
    #[allow(
        clippy::float_cmp,
        reason = "exact comparison to 0.0 is intentional: this tests bit-exact zero, not approximate zero; callers are told to use abs_sq for approximate tests"
    )]
    pub fn is_zero(self) -> bool {
        self.re == 0.0 && self.im == 0.0
    }

    /// Constructs a complex number from polar coordinates `r·exp(iθ)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stem_complex::Complex;
    /// use std::f64::consts::PI;
    ///
    /// let z = Complex::polar(1.0, PI);
    /// assert!((z.re + 1.0).abs() < 1e-10);
    /// assert!(z.im.abs() < 1e-10);
    /// ```
    #[inline]
    #[must_use]
    pub fn polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    /// Divides `self` by `rhs`, returning an error on zero denominator.
    ///
    /// Division uses the identity `(a+bi)/(c+di) = ((ac+bd) + (bc-ad)i) / (c²+d²)`.
    ///
    /// # Errors
    ///
    /// Returns [`ComplexError::DivisionByZero`] when `rhs` has zero magnitude.
    ///
    /// # Examples
    ///
    /// ```
    /// use stem_complex::Complex;
    ///
    /// let z = Complex::new(1.0, 0.0);
    /// assert!(z.div(Complex::ZERO).is_err());
    /// ```
    pub fn div(self, rhs: Self) -> Result<Self, ComplexError> {
        let denom = rhs.abs_sq();
        #[allow(
            clippy::float_cmp,
            reason = "exact comparison to 0.0 is intentional: abs_sq returns exactly 0.0 only when both re and im are bit-exact zero, which is the only valid zero-division guard"
        )]
        if denom == 0.0 {
            return Err(ComplexError::DivisionByZero);
        }
        Ok(Self {
            re: (self.re * rhs.re + self.im * rhs.im) / denom,
            im: (self.im * rhs.re - self.re * rhs.im) / denom,
        })
    }
}

// ── Arithmetic operator impls ─────────────────────────────────────────────────

impl Add for Complex {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl Sub for Complex {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl Mul for Complex {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl Neg for Complex {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}

// ── Conversions ───────────────────────────────────────────────────────────────

impl From<f64> for Complex {
    #[inline]
    fn from(re: f64) -> Self {
        Self { re, im: 0.0 }
    }
}

impl From<(f64, f64)> for Complex {
    #[inline]
    fn from((re, im): (f64, f64)) -> Self {
        Self { re, im }
    }
}

// ── Display ───────────────────────────────────────────────────────────────────

impl fmt::Display for Complex {
    /// Formats as `"re + im·i"` or `"re - |im|·i"` for negative imaginary parts.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.im < 0.0 {
            write!(f, "{} - {}i", self.re, -self.im)
        } else {
            write!(f, "{} + {}i", self.re, self.im)
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_add() {
        let a = Complex::new(1.0, 2.0);
        let b = Complex::new(3.0, 4.0);
        assert_eq!(a + b, Complex::new(4.0, 6.0));
    }

    #[test]
    fn arithmetic_sub() {
        let a = Complex::new(5.0, 3.0);
        let b = Complex::new(2.0, 1.0);
        assert_eq!(a - b, Complex::new(3.0, 2.0));
    }

    #[test]
    fn arithmetic_mul_3_4i_times_1_2i() {
        // (3+4i)(1+2i) = 3 + 6i + 4i + 8i² = (3-8) + 10i = -5 + 10i
        let a = Complex::new(3.0, 4.0);
        let b = Complex::new(1.0, 2.0);
        assert_eq!(a * b, Complex::new(-5.0, 10.0));
    }

    #[test]
    fn arithmetic_neg() {
        let z = Complex::new(3.0, -4.0);
        assert_eq!(-z, Complex::new(-3.0, 4.0));
    }

    #[test]
    fn division_one_over_i() -> Result<(), ComplexError> {
        // 1 / i = -i
        let c = Complex::ONE.div(Complex::i())?;
        assert!(c.re.abs() < 1e-10);
        assert!((c.im + 1.0).abs() < 1e-10);
        Ok(())
    }

    #[test]
    fn division_by_zero_returns_err() {
        assert!(Complex::ONE.div(Complex::ZERO).is_err());
    }

    #[test]
    fn conjugate() {
        let z = Complex::new(3.0, -4.0);
        assert_eq!(z.conj(), Complex::new(3.0, 4.0));
    }

    #[test]
    fn magnitude() {
        let z = Complex::new(3.0, 4.0);
        assert!((z.abs() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn abs_sq_avoids_sqrt() {
        let z = Complex::new(3.0, 4.0);
        assert!((z.abs_sq() - 25.0).abs() < 1e-12);
    }

    #[test]
    fn is_zero_checks() {
        assert!(Complex::ZERO.is_zero());
        assert!(!Complex::ONE.is_zero());
    }

    #[test]
    fn polar_unit_circle() {
        use std::f64::consts::PI;
        let z = Complex::polar(1.0, PI);
        assert!((z.re + 1.0).abs() < 1e-10);
        assert!(z.im.abs() < 1e-10);
    }

    #[test]
    fn display_positive_imaginary() {
        let z = Complex::new(3.0, 4.0);
        assert_eq!(format!("{z}"), "3 + 4i");
    }

    #[test]
    fn display_negative_imaginary() {
        let z = Complex::new(3.0, -4.0);
        assert_eq!(format!("{z}"), "3 - 4i");
    }

    #[test]
    fn from_f64() {
        let z = Complex::from(5.0_f64);
        assert_eq!(z, Complex::new(5.0, 0.0));
    }

    #[test]
    fn from_tuple() {
        let z = Complex::from((3.0_f64, 4.0_f64));
        assert_eq!(z, Complex::new(3.0, 4.0));
    }

    #[test]
    fn constants() {
        assert_eq!(Complex::ZERO, Complex::new(0.0, 0.0));
        assert_eq!(Complex::ONE, Complex::new(1.0, 0.0));
        assert_eq!(Complex::i(), Complex::new(0.0, 1.0));
    }
}
