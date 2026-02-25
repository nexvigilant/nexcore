//! # Arithmetic Functions
//!
//! Classical number-theoretic multiplicative functions.
//!
//! ## Functions
//!
//! - **Möbius μ** — inclusion-exclusion weight
//! - **Euler φ (totient)** — count of coprimes
//! - **von Mangoldt Λ** — prime power indicator
//! - **Liouville λ** — parity of prime factors
//! - **Divisor σ_k** — power sum of divisors
//! - **ω / Ω** — distinct and total prime factor counts
//!
//! ## Primitives
//!
//! - **N** (Quantity): factor counts, divisor sums
//! - **→** (Causality): multiplicativity chain
//! - **σ** (Sequence): ordered factorization

use crate::factorize::factorize;

// ============================================================================
// Public API
// ============================================================================

/// Möbius function μ(n).
///
/// - μ(1) = 1
/// - μ(n) = (-1)^k if n is a product of k distinct primes
/// - μ(n) = 0 if n has a squared prime factor
///
/// # Examples
///
/// ```
/// use stem_number_theory::arithmetic::mobius_mu;
///
/// assert_eq!(mobius_mu(1), 1);
/// assert_eq!(mobius_mu(2), -1);
/// assert_eq!(mobius_mu(4), 0);  // 4 = 2^2, squared factor
/// assert_eq!(mobius_mu(6), 1);  // 6 = 2*3, two distinct primes
/// ```
pub fn mobius_mu(n: u64) -> i8 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    let factors = factorize(n);
    for &(_, exp) in &factors {
        if exp > 1 {
            return 0;
        }
    }
    let k = factors.len();
    if k % 2 == 0 { 1 } else { -1 }
}

/// Euler's totient function φ(n).
///
/// Count of integers in `[1, n]` that are coprime to n.
///
/// Computed via φ(n) = n × ∏(1 - 1/p) for each prime p dividing n.
///
/// # Examples
///
/// ```
/// use stem_number_theory::arithmetic::euler_totient;
///
/// assert_eq!(euler_totient(1), 1);
/// assert_eq!(euler_totient(6), 2);
/// assert_eq!(euler_totient(12), 4);
/// ```
pub fn euler_totient(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    let factors = factorize(n);
    let mut result = n;
    for &(p, _) in &factors {
        // Division before multiplication keeps result within u64; p divides result by
        // the multiplicativity of φ, so exact division is guaranteed.
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "p always divides result by multiplicativity of Euler totient; no overflow possible"
        )]
        {
            result = result / p * (p - 1);
        }
    }
    result
}

/// von Mangoldt function Λ(n).
///
/// Returns `ln(p)` if `n = p^k` for some prime `p` and integer `k >= 1`.
/// Returns `0.0` otherwise.
///
/// # Examples
///
/// ```
/// use stem_number_theory::arithmetic::von_mangoldt;
///
/// assert_eq!(von_mangoldt(1), 0.0);
/// assert!((von_mangoldt(2) - 2_f64.ln()).abs() < 1e-10);
/// assert!((von_mangoldt(4) - 2_f64.ln()).abs() < 1e-10); // 4 = 2^2
/// assert_eq!(von_mangoldt(6), 0.0);  // 6 = 2*3, not a prime power
/// ```
pub fn von_mangoldt(n: u64) -> f64 {
    if n <= 1 {
        return 0.0;
    }
    let factors = factorize(n);
    if factors.len() == 1 {
        // n = p^k for some prime p; factors is non-empty so index 0 is safe.
        #[allow(
            clippy::indexing_slicing,
            reason = "len == 1 is checked immediately above; index 0 is always present"
        )]
        let p = factors[0].0;
        // u64 → f64: precision loss is acceptable for logarithm computation.
        #[allow(
            clippy::as_conversions,
            reason = "u64 prime value cast to f64 for floating-point logarithm; precision loss is acceptable"
        )]
        let pf = p as f64;
        pf.ln()
    } else {
        0.0
    }
}

/// Liouville function λ(n).
///
/// λ(n) = (-1)^Ω(n) where Ω(n) is the total number of prime factors
/// counted with multiplicity.
///
/// # Examples
///
/// ```
/// use stem_number_theory::arithmetic::liouville_lambda;
///
/// assert_eq!(liouville_lambda(1), 1);   // Ω(1) = 0, (-1)^0 = 1
/// assert_eq!(liouville_lambda(4), 1);   // Ω(4) = 2, (-1)^2 = 1
/// assert_eq!(liouville_lambda(2), -1);  // Ω(2) = 1, (-1)^1 = -1
/// ```
pub fn liouville_lambda(n: u64) -> i8 {
    let total = big_omega(n);
    if total % 2 == 0 { 1 } else { -1 }
}

/// Divisor sigma function σ_k(n).
///
/// Sum of k-th powers of all divisors of n.
/// - σ_0(n) = number of divisors (d(n))
/// - σ_1(n) = sum of divisors
///
/// # Examples
///
/// ```
/// use stem_number_theory::arithmetic::divisor_sigma;
///
/// // Divisors of 12: 1, 2, 3, 4, 6, 12 → 6 divisors
/// assert_eq!(divisor_sigma(12, 0), 6);
/// // Sum: 1+2+3+4+6+12 = 28
/// assert_eq!(divisor_sigma(12, 1), 28);
/// ```
pub fn divisor_sigma(n: u64, k: u32) -> u64 {
    if n == 0 {
        return 0;
    }
    // σ_k is multiplicative: σ_k(p^e) = 1 + p^k + p^(2k) + ... + p^(ek)
    let factors = factorize(n);
    if factors.is_empty() {
        return 1; // n = 1, σ_k(1) = 1 for all k
    }
    let mut result: u64 = 1;
    for &(p, exp) in &factors {
        // σ_k(p^e) = sum_{i=0}^{e} p^(i*k)
        let mut term: u64 = 0;
        let mut pk: u64 = 1; // p^(i*k)
        for _ in 0..=exp {
            term = term.saturating_add(pk);
            pk = pk.saturating_mul(p.saturating_pow(k));
        }
        result = result.saturating_mul(term);
    }
    result
}

/// ω(n): count of distinct prime factors.
///
/// # Examples
///
/// ```
/// use stem_number_theory::arithmetic::omega;
///
/// assert_eq!(omega(1), 0);
/// assert_eq!(omega(12), 2); // 12 = 2^2 * 3
/// assert_eq!(omega(30), 3); // 30 = 2 * 3 * 5
/// ```
pub fn omega(n: u64) -> u32 {
    if n <= 1 {
        return 0;
    }
    // The number of distinct prime factors of a u64 is at most 15 (2*3*5*...*47 < 2^64),
    // so len() always fits in u32.
    #[allow(
        clippy::as_conversions,
        reason = "distinct prime factor count of u64 is at most 15, safely fits in u32"
    )]
    let count = factorize(n).len() as u32;
    count
}

/// Ω(n): total prime factors counted with multiplicity.
///
/// # Examples
///
/// ```
/// use stem_number_theory::arithmetic::big_omega;
///
/// assert_eq!(big_omega(1), 0);
/// assert_eq!(big_omega(12), 3); // 12 = 2^2 * 3 → 2+1 = 3
/// assert_eq!(big_omega(8), 3);  // 8  = 2^3       → 3
/// ```
pub fn big_omega(n: u64) -> u32 {
    if n <= 1 {
        return 0;
    }
    factorize(n).iter().map(|&(_, e)| e).sum()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobius_mu_known() {
        assert_eq!(mobius_mu(1), 1);
        assert_eq!(mobius_mu(2), -1);
        assert_eq!(mobius_mu(3), -1);
        assert_eq!(mobius_mu(4), 0); // 2^2
        assert_eq!(mobius_mu(5), -1);
        assert_eq!(mobius_mu(6), 1); // 2*3
        assert_eq!(mobius_mu(12), 0); // 2^2*3
    }

    #[test]
    fn euler_totient_known() {
        assert_eq!(euler_totient(1), 1);
        assert_eq!(euler_totient(2), 1);
        assert_eq!(euler_totient(6), 2);
        assert_eq!(euler_totient(12), 4);
        // For prime p: φ(p) = p-1
        assert_eq!(euler_totient(7), 6);
        assert_eq!(euler_totient(97), 96);
    }

    #[test]
    fn von_mangoldt_known() {
        assert_eq!(von_mangoldt(1), 0.0);
        assert!((von_mangoldt(2) - 2_f64.ln()).abs() < 1e-10);
        assert!((von_mangoldt(3) - 3_f64.ln()).abs() < 1e-10);
        assert!((von_mangoldt(4) - 2_f64.ln()).abs() < 1e-10); // 4 = 2^2
        assert!((von_mangoldt(8) - 2_f64.ln()).abs() < 1e-10); // 8 = 2^3
        assert_eq!(von_mangoldt(6), 0.0); // 2*3, not prime power
    }

    #[test]
    fn liouville_lambda_known() {
        assert_eq!(liouville_lambda(1), 1);
        assert_eq!(liouville_lambda(2), -1); // Ω=1
        assert_eq!(liouville_lambda(4), 1); // Ω=2
        assert_eq!(liouville_lambda(8), -1); // Ω=3
        assert_eq!(liouville_lambda(6), 1); // Ω=2 (2*3)
    }

    #[test]
    fn divisor_sigma_known() {
        // σ_0(12) = 6 divisors
        assert_eq!(divisor_sigma(12, 0), 6);
        // σ_1(12) = 1+2+3+4+6+12 = 28
        assert_eq!(divisor_sigma(12, 1), 28);
        // σ_0(1) = 1
        assert_eq!(divisor_sigma(1, 0), 1);
        // σ_1(p) = 1 + p for prime p
        assert_eq!(divisor_sigma(7, 1), 8);
    }

    #[test]
    fn omega_known() {
        assert_eq!(omega(1), 0);
        assert_eq!(omega(2), 1);
        assert_eq!(omega(12), 2); // 2^2 * 3
        assert_eq!(omega(30), 3); // 2 * 3 * 5
    }

    #[test]
    fn big_omega_known() {
        assert_eq!(big_omega(1), 0);
        assert_eq!(big_omega(2), 1);
        assert_eq!(big_omega(12), 3); // 2^2 * 3 = 2+1
        assert_eq!(big_omega(8), 3); // 2^3
        assert_eq!(big_omega(30), 3); // 2 * 3 * 5
    }
}
