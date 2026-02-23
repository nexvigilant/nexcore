//! # Prime Numbers
//!
//! Classical prime generation, testing, and counting algorithms.
//!
//! ## Algorithms
//!
//! - **Sieve of Eratosthenes** — O(n log log n) prime generation
//! - **Miller-Rabin** — Deterministic primality test for u64 using 12 witnesses
//! - **Segmented sieve** — Memory-efficient primes in arbitrary ranges
//!
//! ## Primitives
//!
//! - **N** (Quantity): prime counting, n-th prime indexing
//! - **σ** (Sequence): sieve elimination ordering
//! - **ρ** (Recursion): segmented sieve builds on base sieve

use crate::NumberTheoryError;

// ============================================================================
// Internal helpers
// ============================================================================

/// Modular multiplication using Russian peasant algorithm to avoid u128 overflow.
///
/// Computes `(a * b) mod m` safely for large values near u128::MAX.
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

/// Modular exponentiation: `base^exp mod modulus`.
///
/// Uses u128 to avoid overflow; falls back to `mul_mod` for large intermediates.
#[inline]
fn mod_pow(mut base: u128, mut exp: u128, modulus: u128) -> u128 {
    if modulus == 1 {
        return 0;
    }
    let mut result: u128 = 1;
    base %= modulus;
    while exp > 0 {
        if exp % 2 == 1 {
            result = result
                .checked_mul(base)
                .map_or_else(|| mul_mod(result, base, modulus), |v| v % modulus);
        }
        exp /= 2;
        base = base
            .checked_mul(base)
            .map_or_else(|| mul_mod(base, base, modulus), |v| v % modulus);
    }
    result
}

/// Miller-Rabin witness test for a single witness `a` against `n`.
///
/// Writes `n-1 = 2^s * d` and checks the sequence `a^d, a^(2d), ..., a^(n-1)`.
#[allow(clippy::many_single_char_names)]
#[inline]
fn miller_rabin_witness(n: u128, d: u128, s: u32, a: u128) -> bool {
    let mut x = mod_pow(a % n, d, n);
    if x == 1 || x == n - 1 {
        return true;
    }
    for _ in 0..s - 1 {
        x = mul_mod(x, x, n);
        if x == n - 1 {
            return true;
        }
    }
    false
}

// ============================================================================
// Public API
// ============================================================================

/// Generate all primes up to `limit` (inclusive) using the Sieve of Eratosthenes.
///
/// Returns an empty vec if `limit < 2`.
///
/// # Examples
///
/// ```
/// use stem_number_theory::primes::sieve_of_eratosthenes;
///
/// assert_eq!(sieve_of_eratosthenes(10), vec![2, 3, 5, 7]);
/// assert_eq!(sieve_of_eratosthenes(1), vec![]);
/// ```
pub fn sieve_of_eratosthenes(limit: u64) -> Vec<u64> {
    if limit < 2 {
        return vec![];
    }
    let n = limit as usize;
    let mut is_composite = vec![false; n + 1];
    let mut primes = Vec::new();

    let mut i = 2_usize;
    while i * i <= n {
        if !is_composite[i] {
            let mut j = i * i;
            while j <= n {
                is_composite[j] = true;
                j += i;
            }
        }
        i += 1;
    }
    for p in 2..=n {
        if !is_composite[p] {
            primes.push(p as u64);
        }
    }
    primes
}

/// Deterministic Miller-Rabin primality test for all `u64` values.
///
/// Uses the witness set `{2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37}`,
/// which is sufficient for all n < 3,317,044,064,679,887,385,961,981
/// (covers the full u64 range).
///
/// # Examples
///
/// ```
/// use stem_number_theory::primes::is_prime_miller_rabin;
///
/// assert!(is_prime_miller_rabin(2));
/// assert!(is_prime_miller_rabin(7919));
/// assert!(!is_prime_miller_rabin(9));
/// assert!(!is_prime_miller_rabin(1));
/// ```
#[allow(clippy::many_single_char_names)]
pub fn is_prime_miller_rabin(n: u64) -> bool {
    match n {
        0 | 1 => return false,
        2 | 3 => return true,
        _ if n % 2 == 0 => return false,
        _ => {}
    }

    let n128 = n as u128;

    // Write n-1 = 2^s * d
    let mut d = n - 1;
    let mut s = 0_u32;
    while d % 2 == 0 {
        d /= 2;
        s += 1;
    }
    let d128 = d as u128;

    // Witnesses sufficient for full u64 range
    const WITNESSES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    for &w in &WITNESSES {
        if w >= n {
            continue;
        }
        if !miller_rabin_witness(n128, d128, s, w as u128) {
            return false;
        }
    }
    true
}

/// Count of primes ≤ x (the prime-counting function π(x)).
///
/// Uses a sieve for moderate x. For x = 0 or 1, returns 0.
///
/// # Examples
///
/// ```
/// use stem_number_theory::primes::prime_counting;
///
/// assert_eq!(prime_counting(10), 4);
/// assert_eq!(prime_counting(100), 25);
/// ```
pub fn prime_counting(x: u64) -> u64 {
    sieve_of_eratosthenes(x).len() as u64
}

/// Return the n-th prime (1-indexed: `nth_prime(1) == 2`).
///
/// # Errors
///
/// Returns [`NumberTheoryError::NonPositive`] if `n == 0`.
///
/// # Examples
///
/// ```
/// use stem_number_theory::primes::nth_prime;
///
/// assert_eq!(nth_prime(1).ok(), Some(2));
/// assert_eq!(nth_prime(25).ok(), Some(97));
/// ```
pub fn nth_prime(n: u64) -> Result<u64, NumberTheoryError> {
    if n == 0 {
        return Err(NumberTheoryError::NonPositive(0));
    }

    // Upper bound estimate via prime number theorem: p_n ~ n * ln(n * ln(n))
    // We use a safe over-estimate then sieve.
    let limit = if n < 6 {
        15
    } else {
        let nf = n as f64;
        let ln_n = nf.ln();
        let ln_ln_n = ln_n.ln().max(1.0);
        // p_n < n * (ln(n) + ln(ln(n))) for n >= 6
        let est = nf * (ln_n + ln_ln_n);
        (est as u64).max(15) + 10
    };

    let primes = sieve_of_eratosthenes(limit);
    if primes.len() >= n as usize {
        // Safety: we checked len >= n, and n >= 1
        Ok(primes[n as usize - 1])
    } else {
        // Fallback: grow the sieve if estimate was too tight
        let extended = sieve_of_eratosthenes(limit * 2 + 100);
        if extended.len() >= n as usize {
            Ok(extended[n as usize - 1])
        } else {
            Err(NumberTheoryError::FactorizationFailed(n))
        }
    }
}

/// Generate all primes in the range `[lo, hi]` using a segmented sieve.
///
/// Handles the case where `lo <= 1` by starting from 2.
///
/// # Examples
///
/// ```
/// use stem_number_theory::primes::segmented_sieve;
///
/// assert_eq!(segmented_sieve(10, 30), vec![11, 13, 17, 19, 23, 29]);
/// ```
pub fn segmented_sieve(lo: u64, hi: u64) -> Vec<u64> {
    if hi < 2 || lo > hi {
        return vec![];
    }
    let lo = lo.max(2);

    // Sieve small primes up to sqrt(hi)
    let sqrt_hi = (hi as f64).sqrt() as u64 + 1;
    let small_primes = sieve_of_eratosthenes(sqrt_hi);

    let len = (hi - lo + 1) as usize;
    let mut is_composite = vec![false; len];

    for p in &small_primes {
        let p = *p;
        // First multiple of p in [lo, hi]
        let start = if p * p >= lo {
            p * p
        } else {
            let rem = lo % p;
            if rem == 0 { lo } else { lo + p - rem }
        };
        let mut j = start;
        while j <= hi {
            if j != p {
                is_composite[(j - lo) as usize] = true;
            }
            j += p;
        }
    }

    let mut result = Vec::new();
    for i in 0..len {
        if !is_composite[i] {
            result.push(lo + i as u64);
        }
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
    fn sieve_basic() {
        assert_eq!(
            sieve_of_eratosthenes(30),
            vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]
        );
    }

    #[test]
    fn sieve_edge_cases() {
        assert_eq!(sieve_of_eratosthenes(0), Vec::<u64>::new());
        assert_eq!(sieve_of_eratosthenes(1), Vec::<u64>::new());
        assert_eq!(sieve_of_eratosthenes(2), vec![2u64]);
    }

    #[test]
    fn miller_rabin_known_primes() {
        let primes = [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 97, 7919, 999_983];
        for p in primes {
            assert!(is_prime_miller_rabin(p), "{p} should be prime");
        }
    }

    #[test]
    fn miller_rabin_known_composites() {
        let composites = [0u64, 1, 4, 6, 8, 9, 12, 100, 561, 1729];
        for c in composites {
            assert!(!is_prime_miller_rabin(c), "{c} should be composite");
        }
    }

    #[test]
    fn miller_rabin_large_prime() {
        // Known large prime
        assert!(is_prime_miller_rabin(15_485_863));
        // Known Carmichael number (composite)
        assert!(!is_prime_miller_rabin(8_911));
    }

    #[test]
    fn prime_counting_known() {
        assert_eq!(prime_counting(10), 4);
        assert_eq!(prime_counting(100), 25);
        assert_eq!(prime_counting(1000), 168);
    }

    #[test]
    fn nth_prime_known() {
        assert_eq!(nth_prime(1).unwrap(), 2);
        assert_eq!(nth_prime(2).unwrap(), 3);
        assert_eq!(nth_prime(25).unwrap(), 97);
    }

    #[test]
    fn nth_prime_zero_error() {
        assert!(matches!(
            nth_prime(0),
            Err(NumberTheoryError::NonPositive(0))
        ));
    }

    #[test]
    fn segmented_sieve_basic() {
        assert_eq!(segmented_sieve(10, 30), vec![11, 13, 17, 19, 23, 29]);
    }

    #[test]
    fn segmented_sieve_from_start() {
        assert_eq!(segmented_sieve(2, 10), vec![2, 3, 5, 7]);
    }

    #[test]
    fn segmented_sieve_single() {
        assert_eq!(segmented_sieve(7, 7), vec![7u64]);
        assert_eq!(segmented_sieve(8, 8), Vec::<u64>::new());
    }
}
