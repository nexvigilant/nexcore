//! # Summatory Functions
//!
//! Cumulative sums of arithmetic functions over ranges.
//!
//! ## Structures
//!
//! - [`MertensFunction`] — M(n) = Σ μ(k), k=1..n
//! - [`ChebyshevTheta`] — θ(x) = Σ ln(p) for primes p ≤ x
//! - [`ChebyshevPsi`] — ψ(x) = Σ Λ(n) for n = 1..x
//!
//! ## Primitives
//!
//! - **Σ** (Sum): accumulation over range
//! - **N** (Quantity): range bounds
//! - **ν** (Frequency): prime density in Chebyshev functions

use crate::arithmetic::{mobius_mu, von_mangoldt};
use crate::primes::sieve_of_eratosthenes;

// ============================================================================
// Mertens Function
// ============================================================================

/// The Mertens function M(n) = Σ_{k=1}^{n} μ(k).
///
/// Known values:
/// - M(1) = 1, M(2) = 0, M(3) = -1, M(4) = -1, M(5) = -2, M(10) = -1
#[non_exhaustive]
pub struct MertensFunction;

impl MertensFunction {
    /// Compute M(n) = Σ_{k=1}^{n} μ(k).
    ///
    /// Returns 0 for n = 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use stem_number_theory::summatory::MertensFunction;
    ///
    /// assert_eq!(MertensFunction::compute(1), 1);
    /// assert_eq!(MertensFunction::compute(5), -2);
    /// assert_eq!(MertensFunction::compute(10), -1);
    /// ```
    pub fn compute(n: u64) -> i64 {
        (1..=n).map(|k| i64::from(mobius_mu(k))).sum()
    }
}

// ============================================================================
// Chebyshev Theta
// ============================================================================

/// The first Chebyshev function θ(x) = Σ ln(p) for primes p ≤ x.
///
/// Measures the "logarithmic density" of primes up to x.
/// By the prime number theorem, θ(x) ~ x as x → ∞.
#[non_exhaustive]
pub struct ChebyshevTheta;

impl ChebyshevTheta {
    /// Compute θ(x) = Σ_{p ≤ x, p prime} ln(p).
    ///
    /// # Examples
    ///
    /// ```
    /// use stem_number_theory::summatory::ChebyshevTheta;
    ///
    /// // θ(10) = ln(2) + ln(3) + ln(5) + ln(7)
    /// let expected = 2_f64.ln() + 3_f64.ln() + 5_f64.ln() + 7_f64.ln();
    /// let computed = ChebyshevTheta::compute(10);
    /// assert!((computed - expected).abs() < 1e-10);
    /// ```
    #[allow(
        clippy::as_conversions,
        reason = "u64 prime cast to f64 for floating-point logarithm; precision loss is acceptable for analytic number theory purposes"
    )]
    pub fn compute(x: u64) -> f64 {
        sieve_of_eratosthenes(x)
            .iter()
            .map(|&p| (p as f64).ln())
            .sum()
    }
}

// ============================================================================
// Chebyshev Psi
// ============================================================================

/// The second Chebyshev function ψ(x) = Σ_{n=1}^{x} Λ(n).
///
/// Sum of the von Mangoldt function over 1..x.
/// Counts prime powers (with logarithmic weights).
/// By PNT: ψ(x) ~ x.
#[non_exhaustive]
pub struct ChebyshevPsi;

impl ChebyshevPsi {
    /// Compute ψ(x) = Σ_{n=1}^{x} Λ(n).
    ///
    /// # Examples
    ///
    /// ```
    /// use stem_number_theory::summatory::ChebyshevPsi;
    ///
    /// // ψ(10) = sum of von_mangoldt(1..=10)
    /// // = 0 + ln2 + ln3 + ln2 + ln5 + 0 + ln7 + ln2 + ln3 + 0
    /// let psi = ChebyshevPsi::compute(10);
    /// assert!(psi > 0.0);
    /// ```
    pub fn compute(x: u64) -> f64 {
        (1..=x).map(|n| von_mangoldt(n)).sum()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mertens_known_values() {
        assert_eq!(MertensFunction::compute(0), 0);
        assert_eq!(MertensFunction::compute(1), 1);
        assert_eq!(MertensFunction::compute(2), 0);
        assert_eq!(MertensFunction::compute(3), -1);
        assert_eq!(MertensFunction::compute(4), -1);
        assert_eq!(MertensFunction::compute(5), -2);
        assert_eq!(MertensFunction::compute(10), -1);
    }

    #[test]
    fn chebyshev_theta_primes_to_10() {
        let expected = 2_f64.ln() + 3_f64.ln() + 5_f64.ln() + 7_f64.ln();
        let computed = ChebyshevTheta::compute(10);
        assert!(
            (computed - expected).abs() < 1e-10,
            "θ(10) = {computed}, expected ≈ {expected}"
        );
    }

    #[test]
    fn chebyshev_theta_zero() {
        assert_eq!(ChebyshevTheta::compute(1), 0.0);
    }

    #[test]
    fn chebyshev_psi_positive() {
        // ψ(x) > 0 for x >= 2
        assert!(ChebyshevPsi::compute(10) > 0.0);
    }

    #[test]
    fn chebyshev_psi_matches_von_mangoldt_sum() {
        // ψ(10) should equal sum of von_mangoldt(1..=10)
        let expected: f64 = (1..=10_u64).map(|n| von_mangoldt(n)).sum();
        let computed = ChebyshevPsi::compute(10);
        assert!((computed - expected).abs() < 1e-10, "ψ(10) mismatch");
    }

    #[test]
    fn chebyshev_theta_psi_relationship() {
        // ψ(x) >= θ(x) for all x (psi counts prime powers, theta only primes)
        let theta = ChebyshevTheta::compute(100);
        let psi = ChebyshevPsi::compute(100);
        assert!(psi >= theta - 1e-10, "ψ(100) should be >= θ(100)");
    }

    #[test]
    fn mertens_incremental_consistency() {
        // M(n) - M(n-1) = μ(n)
        use crate::arithmetic::mobius_mu;
        for n in 2..=20_u64 {
            let diff = MertensFunction::compute(n) - MertensFunction::compute(n - 1);
            assert_eq!(
                diff,
                i64::from(mobius_mu(n)),
                "M({n}) - M({}) != μ({n})",
                n - 1
            );
        }
    }
}
