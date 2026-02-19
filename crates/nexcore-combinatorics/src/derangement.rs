//! Derangements — from Dudeney #267 "The Wrong Hats"
//!
//! D(n) = (n-1) * (D(n-1) + D(n-2))
//!
//! A derangement is a permutation where NO element is in its original position.
//! D(n)/n! converges to 1/e ≈ 0.3679 as n grows.
//!
//! Primitives: σ(permutation) + κ(comparison: position ≠ original) + N(count)

/// Compute D(n), the number of derangements of n elements.
///
/// Uses Dudeney's recurrence: D(n) = (n-1) * (D(n-1) + D(n-2))
/// with D(0) = 1, D(1) = 0.
///
/// # Examples
/// ```
/// use nexcore_combinatorics::derangement;
/// assert_eq!(derangement(0), 1);
/// assert_eq!(derangement(1), 0);
/// assert_eq!(derangement(8), 14833);  // Dudeney's answer
/// ```
pub fn derangement(n: u32) -> u128 {
    match n {
        0 => 1,
        1 => 0,
        _ => {
            let mut prev2: u128 = 1; // D(0)
            let mut prev1: u128 = 0; // D(1)
            for i in 2..=n {
                let current = u128::from(i - 1) * (prev1 + prev2);
                prev2 = prev1;
                prev1 = current;
            }
            prev1
        }
    }
}

/// Compute the probability that a random permutation is a derangement.
///
/// D(n)/n! → 1/e ≈ 0.36788 as n → ∞
///
/// # Examples
/// ```
/// use nexcore_combinatorics::derangement_probability;
/// let p = derangement_probability(8);
/// assert!((p - 0.36788).abs() < 0.001);
/// ```
pub fn derangement_probability(n: u32) -> f64 {
    if n == 0 {
        return 1.0;
    }
    let d = derangement(n) as f64;
    let mut factorial: f64 = 1.0;
    for i in 1..=n {
        factorial *= f64::from(i);
    }
    d / factorial
}

/// Dudeney's alternative formula: multiply by n, add 1 if n even, subtract 1 if n odd.
///
/// D(n) = n * D(n-1) + (-1)^n
///
/// Returns the same result but demonstrates Dudeney's insight about the
/// alternating add/subtract pattern.
pub fn derangement_alternating(n: u32) -> i128 {
    match n {
        0 => 1,
        1 => 0,
        _ => {
            let mut prev: i128 = 0; // D(1)
            for i in 2..=n {
                let sign: i128 = if i % 2 == 0 { 1 } else { -1 };
                prev = i128::from(i) * prev + sign;
            }
            prev
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derangement_dudeney_267() {
        // Dudeney's table from the solution
        let expected: [(u32, u128); 9] = [
            (0, 1),
            (1, 0),
            (2, 1),
            (3, 2),
            (4, 9),
            (5, 44),
            (6, 265),
            (7, 1854),
            (8, 14833),
        ];
        for (n, d) in expected {
            assert_eq!(derangement(n), d, "D({n}) should be {d}");
        }
    }

    #[test]
    fn test_derangement_alternating_matches() {
        for n in 0..15 {
            assert_eq!(
                derangement(n) as i128,
                derangement_alternating(n),
                "Both formulas should agree for n={n}"
            );
        }
    }

    #[test]
    fn test_derangement_probability_converges() {
        let inv_e = 1.0_f64 / std::f64::consts::E;
        for n in 5..15 {
            let p = derangement_probability(n);
            assert!(
                (p - inv_e).abs() < 0.01,
                "D({n})/n! = {p} should be close to 1/e = {inv_e}"
            );
        }
    }
}
