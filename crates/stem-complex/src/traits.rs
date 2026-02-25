//! Abstract traits for complex field arithmetic and analytic functions.

#![allow(
    clippy::module_name_repetitions,
    reason = "ComplexField and AnalyticFunction are idiomatic names for these traits despite the 'Complex' module prefix"
)]

use crate::complex::Complex;
use crate::error::ComplexError;

/// Abstract interface for types that behave like a complex field element.
///
/// Provides access to real/imaginary decomposition and polar coordinates.
pub trait ComplexField: Sized {
    /// Returns the real component.
    #[must_use]
    fn re(&self) -> f64;

    /// Returns the imaginary component.
    #[must_use]
    fn im(&self) -> f64;

    /// Returns the magnitude |z|.
    #[must_use]
    fn magnitude(&self) -> f64;

    /// Returns the phase angle arg(z) in radians, in `(-π, π]`.
    #[must_use]
    fn phase(&self) -> f64;
}

impl ComplexField for Complex {
    #[inline]
    fn re(&self) -> f64 {
        self.re
    }

    #[inline]
    fn im(&self) -> f64 {
        self.im
    }

    #[inline]
    fn magnitude(&self) -> f64 {
        self.abs()
    }

    #[inline]
    fn phase(&self) -> f64 {
        self.arg()
    }
}

/// A function that maps a complex number to a complex number.
///
/// Implementors represent analytic (holomorphic) functions defined on a
/// subset of the complex plane.
pub trait AnalyticFunction {
    /// Evaluates the function at `z`.
    ///
    /// # Errors
    ///
    /// Returns an error when the function is undefined at `z` (e.g., a pole).
    fn evaluate(&self, z: Complex) -> Result<Complex, ComplexError>;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_field_re_im() {
        let z = Complex::new(3.0, 4.0);
        assert!((z.re() - 3.0).abs() < 1e-12);
        assert!((z.im() - 4.0).abs() < 1e-12);
    }

    #[test]
    fn complex_field_magnitude() {
        let z = Complex::new(3.0, 4.0);
        assert!((z.magnitude() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn complex_field_phase() {
        let z = Complex::new(1.0, 0.0);
        assert!(z.phase().abs() < 1e-12);
    }

    #[test]
    fn analytic_function_object_safety() {
        // Verify the trait can be used as a Box<dyn AnalyticFunction>
        struct Identity;
        impl AnalyticFunction for Identity {
            fn evaluate(&self, z: Complex) -> Result<Complex, ComplexError> {
                Ok(z)
            }
        }

        let f: Box<dyn AnalyticFunction> = Box::new(Identity);
        let r = f.evaluate(Complex::ONE);
        assert!(r.is_ok());
    }
}
