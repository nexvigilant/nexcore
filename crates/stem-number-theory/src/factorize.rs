//! # Integer Factorization
//!
//! Efficient factorization combining trial division for small factors
//! and Pollard's rho algorithm (with Brent's improvement) for large composites.
//!
//! ## Algorithms
//!
//! - **Trial division** — O(√n) for small factors up to ~10^6
//! - **Pollard's rho (Brent)** — O(n^(1/4)) expected for large factors
//! - **factorize** — combined dispatcher, returns sorted (prime, exponent) pairs
//!
//! ## Primitives
//!
//! - **N** (Quantity): factor multiplicity
//! - **ρ** (Recursion): Pollard cycle detection
//! - **σ** (Sequence): sorted output ordering

use crate::NumberTheoryError;
use crate::primes::is_prime_miller_rabin;

// ============================================================================
// Internal helpers
// ============================================================================

/// Euclidean GCD.
#[inline]
fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Modular multiplication using Russian peasant to avoid overflow.
#[inline]
fn mul_mod(mut a: u128, mut b: u128, m: u128) -> u128 {
    let mut result: u128 = 0;
    a %= m;
    while b > 0 {
        if b % 2 == 1 {
            result = (result + a) % m;
        }
        a = (a * 2) % m;
        b /= 2;
    }
    result
}

// ============================================================================
// Public API
// ============================================================================

/// Factor `n` by trial division.
///
/// Returns `(prime, exponent)` pairs sorted by prime. Efficient for numbers
/// with small prime factors.
///
/// # Examples
///
/// ```
/// use stem_number_theory::factorize::trial_division;
///
/// assert_eq!(trial_division(12), vec![(2, 2), (3, 1)]);
/// assert_eq!(trial_division(1), vec![]);
/// ```
pub fn trial_division(mut n: u64) -> Vec<(u64, u32)> {
    if n <= 1 {
        return vec![];
    }
    let mut factors = Vec::new();

    // Trial divide by 2
    if n % 2 == 0 {
        let mut exp = 0_u32;
        while n % 2 == 0 {
            exp += 1;
            n /= 2;
        }
        factors.push((2u64, exp));
    }

    // Trial divide by odd numbers up to sqrt(n)
    let mut d = 3_u64;
    while d * d <= n {
        if n % d == 0 {
            let mut exp = 0_u32;
            while n % d == 0 {
                exp += 1;
                n /= d;
            }
            factors.push((d, exp));
        }
        d += 2;
    }

    if n > 1 {
        factors.push((n, 1));
    }
    factors
}

/// Pollard's rho algorithm (Brent's improvement) to find a non-trivial factor.
///
/// Uses the deterministic polynomial `f(x) = x^2 + c mod n` with constant
/// `c = 1`. Falls back to `c = 2` if the first attempt stalls.
///
/// # Errors
///
/// Returns [`NumberTheoryError::FactorizationFailed`] if no factor is found
/// (e.g. `n` is prime or `n <= 1`).
///
/// # Examples
///
/// ```
/// use stem_number_theory::factorize::pollard_rho;
///
/// if let Ok(f) = pollard_rho(15) {
///     assert!(f == 3 || f == 5);
/// }
/// ```
#[allow(clippy::many_single_char_names)]
pub fn pollard_rho(n: u64) -> Result<u64, NumberTheoryError> {
    if n <= 1 {
        return Err(NumberTheoryError::NonPositive(n));
    }
    if n % 2 == 0 {
        return Ok(2);
    }
    if is_prime_miller_rabin(n) {
        return Err(NumberTheoryError::FactorizationFailed(n));
    }

    let n128 = n as u128;

    // Try a few values of c
    for c in [1u128, 2, 3, 5, 7] {
        let mut x = 2u128;
        let mut y = 2u128;
        let mut r = 1u128;
        let mut q = 1u128;
        let mut d = 1u64;

        // Brent's cycle detection
        loop {
            let x_saved = x;
            for _ in 0..r {
                x = (mul_mod(x, x, n128) + c) % n128;
            }

            let mut k = 0u128;
            while k < r && d == 1 {
                y = x_saved;
                let step = r.min(r - k);
                for _ in 0..step {
                    y = (mul_mod(y, y, n128) + c) % n128;
                    let diff = (x as u64).abs_diff(y as u64);
                    q = mul_mod(q, diff as u128, n128);
                }
                d = gcd(q as u64, n);
                k += step;
            }

            r *= 2;

            if d != 1 {
                break;
            }
        }

        if d != n && d != 1 {
            return Ok(d);
        }

        // Slow fallback if Brent failed: Floyd's cycle detection
        let mut x2 = 2u128;
        let mut y2 = 2u128;
        let mut d2 = 1u64;
        while d2 == 1 {
            x2 = (mul_mod(x2, x2, n128) + c) % n128;
            y2 = (mul_mod(y2, y2, n128) + c) % n128;
            y2 = (mul_mod(y2, y2, n128) + c) % n128;
            let diff = (x2 as u64).abs_diff(y2 as u64);
            d2 = gcd(diff, n);
        }
        if d2 != n && d2 != 1 {
            return Ok(d2);
        }
    }

    Err(NumberTheoryError::FactorizationFailed(n))
}

/// Fully factor `n` into sorted `(prime, exponent)` pairs.
///
/// Combines trial division for small factors with Pollard's rho for large
/// composites. Returns an empty vec for `n <= 1`.
///
/// # Examples
///
/// ```
/// use stem_number_theory::factorize::factorize;
///
/// assert_eq!(factorize(1), vec![]);
/// assert_eq!(factorize(12), vec![(2, 2), (3, 1)]);
/// assert_eq!(factorize(7), vec![(7, 1)]);
/// ```
pub fn factorize(n: u64) -> Vec<(u64, u32)> {
    if n <= 1 {
        return vec![];
    }

    let mut pending = vec![n];
    let mut prime_factors: Vec<u64> = Vec::new();

    while let Some(m) = pending.pop() {
        if m <= 1 {
            continue;
        }
        if is_prime_miller_rabin(m) {
            prime_factors.push(m);
            continue;
        }

        // Try small trial division first (faster for small factors)
        let mut found = false;
        for &p in &[2u64, 3, 5, 7, 11, 13, 17, 19, 23] {
            if m % p == 0 {
                prime_factors.push(p);
                pending.push(m / p);
                found = true;
                break;
            }
        }
        if found {
            continue;
        }

        // Pollard's rho for large composites
        match pollard_rho(m) {
            Ok(d) => {
                pending.push(d);
                pending.push(m / d);
            }
            Err(_) => {
                // Last resort: full trial division
                let td = trial_division(m);
                for (p, e) in td {
                    for _ in 0..e {
                        prime_factors.push(p);
                    }
                }
            }
        }
    }

    // Collect into (prime, exponent) pairs
    prime_factors.sort_unstable();
    let mut result: Vec<(u64, u32)> = Vec::new();
    for p in prime_factors {
        if let Some(last) = result.last_mut() {
            if last.0 == p {
                last.1 += 1;
                continue;
            }
        }
        result.push((p, 1));
    }
    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trial_division_basic() {
        assert_eq!(trial_division(1), vec![]);
        assert_eq!(trial_division(2), vec![(2, 1)]);
        assert_eq!(trial_division(12), vec![(2, 2), (3, 1)]);
        assert_eq!(trial_division(360), vec![(2, 3), (3, 2), (5, 1)]);
    }

    #[test]
    fn trial_division_prime() {
        assert_eq!(trial_division(97), vec![(97, 1)]);
    }

    #[test]
    fn factorize_small() {
        assert_eq!(factorize(1), vec![]);
        assert_eq!(factorize(2), vec![(2, 1)]);
        assert_eq!(factorize(12), vec![(2, 2), (3, 1)]);
        assert_eq!(factorize(7), vec![(7, 1)]);
    }

    #[test]
    fn factorize_semiprime() {
        // 15 = 3 * 5
        assert_eq!(factorize(15), vec![(3, 1), (5, 1)]);
        // 77 = 7 * 11
        assert_eq!(factorize(77), vec![(7, 1), (11, 1)]);
    }

    #[test]
    fn factorize_large_semiprime() {
        // 8051 = 83 * 97
        let f = factorize(8051);
        assert_eq!(f, vec![(83, 1), (97, 1)]);
    }

    #[test]
    fn factorize_prime_powers() {
        assert_eq!(factorize(64), vec![(2, 6)]);
        assert_eq!(factorize(81), vec![(3, 4)]);
    }

    #[test]
    fn pollard_rho_finds_factor() {
        let f = pollard_rho(15);
        let d = f.unwrap();
        assert!(d == 3 || d == 5);
    }

    #[test]
    fn pollard_rho_prime_errors() {
        assert!(pollard_rho(7).is_err());
        assert!(pollard_rho(97).is_err());
    }

    #[test]
    fn factorize_consistency() {
        // Product of all (prime^exp) should equal original
        for n in [2u64, 12, 360, 1001, 9999, 99_991] {
            let factors = factorize(n);
            let product: u64 = factors.iter().map(|&(p, e)| p.pow(e)).product();
            assert_eq!(product, n, "factorize({n}) product mismatch");
        }
    }
}
