//! Catalan numbers — from Dudeney #279 "Barrels of Balsam"
//!
//! C(n) = C(2n,n) / (n+1) = (2n)! / ((n+1)! * n!)
//!
//! Counts: binary tree shapes, valid parenthesizations, monotone lattice paths,
//! Standard Young Tableaux of shape (n,n), valid DAG build orderings.
//!
//! Primitives: σ(order constraint) + ∂(boundary: monotonicity) + κ(comparison)

/// Compute the nth Catalan number.
///
/// Uses the recurrence C(n) = C(n-1) * 2(2n-1) / (n+1) to avoid overflow
/// as long as possible with u128.
///
/// # Examples
/// ```
/// use nexcore_combinatorics::catalan;
/// assert_eq!(catalan(0), 1);
/// assert_eq!(catalan(5), 42);  // Dudeney's answer for 10 barrels
/// assert_eq!(catalan(10), 16796);
/// ```
pub fn catalan(n: u32) -> u128 {
    if n == 0 {
        return 1;
    }
    let mut result: u128 = 1;
    for i in 1..=n {
        let i_128 = u128::from(i);
        result = result * (2 * (2 * i_128 - 1)) / (i_128 + 1);
    }
    result
}

/// First 20 Catalan numbers for quick lookup.
pub fn catalan_table() -> Vec<(u32, u128)> {
    (0..20).map(|n| (n, catalan(n))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalan_known_values() {
        // From OEIS A000108
        let expected = [1, 1, 2, 5, 14, 42, 132, 429, 1430, 4862, 16796];
        for (i, &val) in expected.iter().enumerate() {
            assert_eq!(catalan(i as u32), val, "C({i}) should be {val}");
        }
    }

    #[test]
    fn test_catalan_dudeney_279() {
        // Dudeney's puzzle: 10 barrels in 2 rows of 5
        // Answer: C(10,5)/6 = 252/6 = 42 = C(5)
        assert_eq!(catalan(5), 42);
    }

    #[test]
    fn test_catalan_small_cases() {
        // 6 barrels, 3 per row: C(3) = 5
        assert_eq!(catalan(3), 5);
    }
}
