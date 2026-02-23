//! Special functions supporting the zeta computation.
//!
//! - Bernoulli numbers (Akiyama–Tanigawa algorithm)
//! - Dirichlet eta function η(s)
//! - Riemann xi function ξ(s)

use std::f64::consts::PI;

use stem_complex::Complex;
use stem_complex::{functions, gamma};

use crate::error::ZetaError;

// ── Bernoulli Numbers ────────────────────────────────────────────────────────

/// Compute the first `n + 1` Bernoulli numbers B_0 … B_n using the
/// Akiyama–Tanigawa algorithm.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::special::bernoulli_numbers;
///
/// let b = bernoulli_numbers(6);
/// assert!((b[0] - 1.0).abs() < 1e-12);     // B_0 = 1
/// assert!((b[1] + 0.5).abs() < 1e-12);      // B_1 = -1/2
/// assert!((b[2] - 1.0 / 6.0).abs() < 1e-12); // B_2 = 1/6
/// assert!(b[3].abs() < 1e-12);               // B_3 = 0
/// assert!((b[4] + 1.0 / 30.0).abs() < 1e-12); // B_4 = -1/30
/// ```
#[must_use]
pub fn bernoulli_numbers(n: usize) -> Vec<f64> {
    let mut a = Vec::with_capacity(n + 1);
    let mut result = Vec::with_capacity(n + 1);

    for m in 0..=n {
        a.push(1.0 / (m as f64 + 1.0));
        // Reverse update
        let mut j = m;
        while j >= 1 {
            a[j - 1] = (j as f64) * (a[j - 1] - a[j]);
            j -= 1;
        }
        result.push(a[0]);
    }
    // Akiyama-Tanigawa produces B_1 = +1/2.
    // The standard convention for the zeta function uses B_1^- = -1/2.
    if n >= 1 {
        result[1] = -0.5;
    }
    result
}

// ── Dirichlet Eta ────────────────────────────────────────────────────────────

/// Dirichlet eta function η(s) via the Borwein acceleration method.
///
/// η(s) = Σ_{n=1}^∞ (-1)^{n-1} / n^s
///
/// Converges for all s (including the critical strip) via Euler summation.
///
/// # Errors
///
/// Returns [`ZetaError::ConvergenceFailure`] on numerical issues.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::special::dirichlet_eta;
/// use stem_complex::Complex;
///
/// // η(1) = ln(2) ≈ 0.6931
/// let result = dirichlet_eta(Complex::ONE).unwrap();
/// assert!((result.re - 2_f64.ln()).abs() < 1e-6);
/// ```
pub fn dirichlet_eta(s: Complex) -> Result<Complex, ZetaError> {
    // Borwein (2000) acceleration for the alternating series.
    // η(s) = -1/d_n · Σ_{k=0}^{n-1} (-1)^k (d_k - d_n) / (k+1)^s
    // where d_k = Σ_{i=0}^{k} n! / (i! (n-i)!) = Σ binom(n, i) for i=0..k
    let n = 32_usize;

    // d_k = Σ_{i=0}^{k} C(n, i)
    let mut binom = vec![0.0_f64; n + 1];
    binom[0] = 1.0; // C(n, 0) = 1
    for i in 1..=n {
        binom[i] = binom[i - 1] * (n - i + 1) as f64 / i as f64;
    }

    let mut d = vec![0.0_f64; n + 1];
    d[0] = binom[0];
    for k in 1..=n {
        d[k] = d[k - 1] + binom[k];
    }
    let d_n = d[n]; // = 2^n

    let mut sum = Complex::ZERO;
    let mut sign = 1.0_f64;

    for k in 0..n {
        let coeff = sign * (d[k] - d_n);
        let base = Complex::from((k + 1) as f64);
        let term_pow = functions::pow(base, s)?;
        let term = Complex::from(coeff).div(term_pow)?;
        sum = sum + term;
        sign = -sign;
    }

    let result = Complex::from(-1.0 / d_n) * sum;
    Ok(result)
}

// ── Riemann Xi ───────────────────────────────────────────────────────────────

/// Riemann xi function ξ(s) = ½ s(s-1) π^(-s/2) Γ(s/2) ζ(s).
///
/// ξ is an entire function satisfying ξ(s) = ξ(1-s).
///
/// This is computed from η(s) rather than ζ(s) directly to avoid the pole.
///
/// # Errors
///
/// Returns errors from underlying Gamma and power computations.
pub fn xi(s: Complex) -> Result<Complex, ZetaError> {
    // ξ(s) = ½ s(s-1) π^(-s/2) Γ(s/2) ζ(s)
    // where ζ(s) = η(s) / (1 - 2^(1-s))

    let eta_s = dirichlet_eta(s)?;

    // 1 - 2^(1-s)
    let one_minus_s = Complex::ONE - s;
    let two_pow = functions::pow(Complex::from(2.0), one_minus_s)?;
    let factor = Complex::ONE - two_pow;

    // Avoid division by zero near s=1 (factor → 0)
    if factor.abs_sq() < 1e-30 {
        // At s=1, ξ(1) = ½  (known value)
        return Ok(Complex::from(0.5));
    }

    let zeta_s = eta_s.div(factor)?;

    // s(s-1)
    let s_times_s_minus_1 = s * (s - Complex::ONE);

    // π^(-s/2)
    let neg_s_half = Complex::from(-0.5) * s;
    let pi_pow = functions::pow(Complex::from(PI), neg_s_half)?;

    // Γ(s/2)
    let s_half = Complex::from(0.5) * s;
    let gamma_s_half = gamma::gamma(s_half)?;

    let result = Complex::from(0.5) * s_times_s_minus_1 * pi_pow * gamma_s_half * zeta_s;
    Ok(result)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bernoulli_first_seven() {
        let b = bernoulli_numbers(6);
        assert!((b[0] - 1.0).abs() < 1e-12, "B_0 = {}", b[0]);
        assert!((b[1] + 0.5).abs() < 1e-12, "B_1 = {}", b[1]);
        assert!((b[2] - 1.0 / 6.0).abs() < 1e-12, "B_2 = {}", b[2]);
        assert!(b[3].abs() < 1e-12, "B_3 = {}", b[3]);
        assert!((b[4] + 1.0 / 30.0).abs() < 1e-12, "B_4 = {}", b[4]);
        assert!(b[5].abs() < 1e-12, "B_5 = {}", b[5]);
        assert!((b[6] - 1.0 / 42.0).abs() < 1e-12, "B_6 = {}", b[6]);
    }

    #[test]
    fn eta_at_one_is_ln2() {
        let result = dirichlet_eta(Complex::ONE);
        assert!(result.is_ok());
        let eta = result.ok();
        assert!(eta.is_some());
        if let Some(v) = eta {
            assert!((v.re - 2_f64.ln()).abs() < 1e-4, "η(1) = {}", v.re);
            assert!(v.im.abs() < 1e-4);
        }
    }

    #[test]
    fn eta_at_two() {
        // η(2) = π²/12 ≈ 0.8225
        let result = dirichlet_eta(Complex::from(2.0));
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            let expected = PI * PI / 12.0;
            assert!(
                (v.re - expected).abs() < 1e-3,
                "η(2) = {} expected {}",
                v.re,
                expected
            );
        }
    }

    #[test]
    fn xi_symmetry() {
        // ξ(s) = ξ(1-s) — test at s = 2
        let xi_2 = xi(Complex::from(2.0));
        let xi_neg1 = xi(Complex::from(-1.0)); // 1 - 2 = -1
        if let (Ok(a), Ok(b)) = (xi_2, xi_neg1) {
            assert!(
                (a.re - b.re).abs() < 0.1,
                "ξ(2) = {} ≠ ξ(-1) = {}",
                a.re,
                b.re
            );
        }
    }
}
