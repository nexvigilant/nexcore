//! Dirichlet L-functions L(s, χ).
//!
//! Provides Dirichlet character construction and L-series evaluation.

use serde::{Deserialize, Serialize};

use stem_complex::Complex;
use stem_complex::functions;

use crate::error::ZetaError;

/// A Dirichlet character modulo q.
///
/// Stores χ(n) for n = 0, 1, …, q−1 as complex values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirichletCharacter {
    /// The modulus of the character.
    pub modulus: u64,
    /// Values χ(0), χ(1), …, χ(q−1).
    pub values: Vec<Complex>,
}

impl DirichletCharacter {
    /// Construct the principal character χ₀ mod q.
    ///
    /// χ₀(n) = 1 if gcd(n, q) = 1, else 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_zeta::l_functions::DirichletCharacter;
    ///
    /// let chi = DirichletCharacter::principal(6);
    /// assert_eq!(chi.modulus, 6);
    /// // χ₀(1) = 1, χ₀(2) = 0 (gcd(2,6)=2), χ₀(5) = 1
    /// ```
    #[must_use]
    pub fn principal(q: u64) -> Self {
        let mut values = Vec::with_capacity(q as usize);
        for n in 0..q {
            if gcd(n, q) == 1 {
                values.push(Complex::ONE);
            } else {
                values.push(Complex::ZERO);
            }
        }
        Self { modulus: q, values }
    }

    /// Evaluate χ(n) for any non-negative n.
    ///
    /// Uses periodicity: χ(n) = χ(n mod q).
    #[must_use]
    pub fn evaluate(&self, n: u64) -> Complex {
        if self.modulus == 0 {
            return Complex::ZERO;
        }
        let idx = (n % self.modulus) as usize;
        if idx < self.values.len() {
            self.values[idx]
        } else {
            Complex::ZERO
        }
    }

    /// Check if this is the principal character.
    #[must_use]
    pub fn is_principal(&self) -> bool {
        for n in 0..self.modulus {
            let expected = if gcd(n, self.modulus) == 1 {
                Complex::ONE
            } else {
                Complex::ZERO
            };
            let actual = self.evaluate(n);
            if (actual.re - expected.re).abs() > 1e-10 || (actual.im - expected.im).abs() > 1e-10 {
                return false;
            }
        }
        true
    }
}

/// Compute the Dirichlet L-function L(s, χ) via partial Dirichlet series.
///
/// L(s, χ) = Σ_{n=1}^{N} χ(n) / n^s
///
/// # Errors
///
/// Returns [`ZetaError::Complex`] on arithmetic errors.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::l_functions::{DirichletCharacter, dirichlet_l};
/// use stem_complex::Complex;
///
/// let chi = DirichletCharacter::principal(1); // trivial character = all 1s
/// let result = dirichlet_l(Complex::from(2.0), &chi, 1000).unwrap();
/// // L(2, χ₀ mod 1) = ζ(2) ≈ π²/6
/// assert!((result.re - std::f64::consts::PI.powi(2) / 6.0).abs() < 0.1);
/// ```
pub fn dirichlet_l(
    s: Complex,
    chi: &DirichletCharacter,
    n_terms: usize,
) -> Result<Complex, ZetaError> {
    let mut sum = Complex::ZERO;
    for n in 1..=n_terms {
        let chi_n = chi.evaluate(n as u64);
        // Skip zero terms (saves computation)
        if chi_n.is_zero() {
            continue;
        }
        let base = Complex::from(n as f64);
        let n_s = functions::pow(base, s)?;
        let term = chi_n.div(n_s)?;
        sum = sum + term;
    }
    Ok(sum)
}

/// Binary GCD (Stein's algorithm).
fn gcd(mut a: u64, mut b: u64) -> u64 {
    if a == 0 {
        return b;
    }
    if b == 0 {
        return a;
    }
    let shift = (a | b).trailing_zeros();
    a >>= a.trailing_zeros();
    loop {
        b >>= b.trailing_zeros();
        if a > b {
            core::mem::swap(&mut a, &mut b);
        }
        b -= a;
        if b == 0 {
            return a << shift;
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn principal_character_mod_6() {
        let chi = DirichletCharacter::principal(6);
        assert_eq!(chi.modulus, 6);
        // gcd(0,6)=6, gcd(1,6)=1, gcd(2,6)=2, gcd(3,6)=3, gcd(4,6)=2, gcd(5,6)=1
        assert!(chi.evaluate(0).is_zero());
        assert!((chi.evaluate(1).re - 1.0).abs() < 1e-12);
        assert!(chi.evaluate(2).is_zero());
        assert!(chi.evaluate(3).is_zero());
        assert!(chi.evaluate(4).is_zero());
        assert!((chi.evaluate(5).re - 1.0).abs() < 1e-12);
    }

    #[test]
    fn principal_character_periodicity() {
        let chi = DirichletCharacter::principal(5);
        // χ(6) = χ(1) = 1
        assert!((chi.evaluate(6).re - chi.evaluate(1).re).abs() < 1e-12);
        // χ(10) = χ(0) = 0
        assert!(chi.evaluate(10).is_zero());
    }

    #[test]
    fn is_principal_check() {
        let chi = DirichletCharacter::principal(6);
        assert!(chi.is_principal());
    }

    #[test]
    fn l_trivial_character_approximates_zeta() {
        // χ₀ mod 1 sends every integer to 1, so L(s, χ₀) = ζ(s)
        let chi = DirichletCharacter::principal(1);
        let result = dirichlet_l(Complex::from(2.0), &chi, 5000);
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            let expected = PI * PI / 6.0;
            assert!((v.re - expected).abs() < 0.01, "L(2, χ₀) = {}", v.re);
        }
    }

    #[test]
    fn l_principal_mod_6_euler_product() {
        // L(s, χ₀ mod q) = ζ(s) · Π_{p|q} (1 - p^{-s})
        // For q=6, primes dividing 6 are 2 and 3
        // L(2, χ₀ mod 6) = ζ(2) · (1 - 2^{-2}) · (1 - 3^{-2})
        //                 = (π²/6) · (3/4) · (8/9)
        let chi = DirichletCharacter::principal(6);
        let result = dirichlet_l(Complex::from(2.0), &chi, 5000);
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            let expected = (PI * PI / 6.0) * (3.0 / 4.0) * (8.0 / 9.0);
            assert!(
                (v.re - expected).abs() < 0.05,
                "L(2, χ₀ mod 6) = {} expected {}",
                v.re,
                expected
            );
        }
    }

    #[test]
    fn gcd_known() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(7, 5), 1);
        assert_eq!(gcd(0, 5), 5);
        assert_eq!(gcd(6, 0), 6);
    }
}
