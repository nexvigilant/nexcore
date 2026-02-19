//! Grid path counting — from Dudeney #253 "A Bank Holiday Puzzle"
//!
//! Count paths in an m×n grid going only right or down.
//! Answer: C(m+n, m) = (m+n)! / (m! * n!)
//!
//! Dudeney's solution: 12 towns by 5, so m=11, n=4.
//! C(15,4) = 1,365 routes.
//!
//! Primitives: σ(sequence: path steps) + N(quantity: count) + ∂(boundary: direction constraint)

/// Count the number of monotone lattice paths from (0,0) to (m,n).
///
/// Each path consists of m right-steps and n down-steps.
/// The count is the binomial coefficient C(m+n, m).
///
/// Uses iterative computation to avoid overflow as long as possible.
///
/// # Examples
/// ```
/// use nexcore_combinatorics::grid_paths;
/// assert_eq!(grid_paths(11, 4), 1365);  // Dudeney's answer
/// assert_eq!(grid_paths(3, 3), 20);      // 3×3 grid
/// assert_eq!(grid_paths(1, 1), 2);       // Simple: right-down or down-right
/// ```
pub fn grid_paths(m: u32, n: u32) -> u128 {
    binomial(m + n, m.min(n))
}

/// Compute binomial coefficient C(n, k) = n! / (k! * (n-k)!)
///
/// Uses the multiplicative formula to avoid computing large factorials.
pub fn binomial(n: u32, k: u32) -> u128 {
    if k > n {
        return 0;
    }
    if k == 0 || k == n {
        return 1;
    }
    // Use smaller k for efficiency
    let k = k.min(n - k);
    let mut result: u128 = 1;
    for i in 0..k {
        result = result * u128::from(n - i) / u128::from(i + 1);
    }
    result
}

/// Multinomial coefficient: (n1+n2+...+nk)! / (n1! * n2! * ... * nk!)
///
/// Counts the number of ways to interleave k independent sequences
/// of lengths n1, n2, ..., nk. This is the generalization of grid_paths
/// to k dimensions.
///
/// # Examples
/// ```
/// use nexcore_combinatorics::grid_paths::multinomial;
/// // Two sequences of length 2 and 3: C(5,2) = 10
/// assert_eq!(multinomial(&[2, 3]), 10);
/// // Four sequences of lengths 6,3,3,2: our hook DAG interleaving
/// assert_eq!(multinomial(&[6, 3, 3, 2]), 1681680);
/// ```
pub fn multinomial(lengths: &[u32]) -> u128 {
    let total: u32 = lengths.iter().sum();
    let mut result: u128 = 1;
    let mut remaining = total;

    for &len in lengths.iter().take(lengths.len().saturating_sub(1)) {
        result *= binomial(remaining, len);
        remaining -= len;
    }

    result
}

/// Count valid execution orderings for independent subgraphs.
///
/// Given a set of independent subgraphs, each with a chain length and
/// a count of internal valid orderings, compute the total number of
/// valid execution orderings.
///
/// total = multinomial(lengths) × product(internal_orderings)
///
/// # Examples
/// ```
/// use nexcore_combinatorics::grid_paths::count_interleaved_orderings;
/// // Our hook DAG: 4 subgraphs with lengths [6,3,3,2] and internal orderings [1,2,1,1]
/// assert_eq!(count_interleaved_orderings(&[6,3,3,2], &[1,2,1,1]), 3363360);
/// ```
pub fn count_interleaved_orderings(lengths: &[u32], internal_orderings: &[u128]) -> u128 {
    let m = multinomial(lengths);
    let product: u128 = internal_orderings.iter().product();
    m * product
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_paths_dudeney_253() {
        // Dudeney: 12×5 grid (m=11, n=4), answer is 1365
        assert_eq!(grid_paths(11, 4), 1365);
    }

    #[test]
    fn test_binomial_small() {
        assert_eq!(binomial(0, 0), 1);
        assert_eq!(binomial(5, 0), 1);
        assert_eq!(binomial(5, 5), 1);
        assert_eq!(binomial(5, 2), 10);
        assert_eq!(binomial(10, 3), 120);
    }

    #[test]
    fn test_multinomial_two_sequences() {
        // multinomial([a,b]) = C(a+b, a)
        assert_eq!(multinomial(&[3, 4]), binomial(7, 3));
    }

    #[test]
    fn test_hook_dag_orderings() {
        // Our actual hook DAG: 4 independent subgraphs
        // Lengths: [6, 3, 3, 2], Internal orderings: [1, 2, 1, 1]
        let total = count_interleaved_orderings(&[6, 3, 3, 2], &[1, 2, 1, 1]);
        assert_eq!(total, 3_363_360);
    }

    #[test]
    fn test_grid_paths_symmetric() {
        // C(m+n, m) = C(m+n, n)
        assert_eq!(grid_paths(3, 7), grid_paths(7, 3));
    }
}
