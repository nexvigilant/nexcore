//! Cycle decomposition — from Dudeney #238 "Arranging the Jampots"
//!
//! Any permutation decomposes into disjoint cycles.
//! Minimum transpositions to sort = n - number_of_cycles.
//!
//! Dudeney's insight: 24 jampots, 7 cycles → 24 - 7 = 17 swaps (not 22).
//!
//! Primitives: σ(sequence) + μ(mapping: swap) + N(quantity: cycle count)

use serde::Serialize;

/// A single cycle in the decomposition.
#[derive(Debug, Clone, Serialize)]
pub struct Cycle {
    /// Elements in this cycle, in order of traversal.
    pub elements: Vec<usize>,
    /// Length of the cycle.
    pub length: usize,
}

/// Result of cycle decomposition.
#[derive(Debug, Clone, Serialize)]
pub struct CycleDecompositionResult {
    /// All cycles found.
    pub cycles: Vec<Cycle>,
    /// Total number of cycles (including fixed points).
    pub num_cycles: usize,
    /// Number of fixed points (cycles of length 1).
    pub fixed_points: usize,
    /// Minimum transpositions to sort: n - num_cycles.
    pub min_transpositions: usize,
    /// Size of the permutation.
    pub n: usize,
}

/// Decompose a permutation into disjoint cycles.
///
/// Input: a permutation as a slice where `perm[i]` is the value at position i.
/// Values must be 0-indexed: a permutation of {0, 1, ..., n-1}.
///
/// # Examples
/// ```
/// use nexcore_combinatorics::cycle_decomposition;
/// // Identity: [0, 1, 2] → 3 fixed points, 0 swaps needed
/// let r = cycle_decomposition(&[0, 1, 2]);
/// assert_eq!(r.num_cycles, 3);
/// assert_eq!(r.min_transpositions, 0);
///
/// // Rotation: [1, 2, 0] → 1 cycle of length 3, 2 swaps needed
/// let r = cycle_decomposition(&[1, 2, 0]);
/// assert_eq!(r.num_cycles, 1);
/// assert_eq!(r.min_transpositions, 2);
/// ```
pub fn cycle_decomposition(perm: &[usize]) -> CycleDecompositionResult {
    let n = perm.len();
    let mut visited = vec![false; n];
    let mut cycles = Vec::new();

    for start in 0..n {
        if visited[start] {
            continue;
        }
        let mut cycle_elements = Vec::new();
        let mut current = start;

        loop {
            visited[current] = true;
            cycle_elements.push(current);
            current = perm[current];
            if current == start {
                break;
            }
        }

        let length = cycle_elements.len();
        cycles.push(Cycle {
            elements: cycle_elements,
            length,
        });
    }

    let num_cycles = cycles.len();
    let fixed_points = cycles.iter().filter(|c| c.length == 1).count();

    CycleDecompositionResult {
        cycles,
        num_cycles,
        fixed_points,
        min_transpositions: n - num_cycles,
        n,
    }
}

/// Compute minimum transpositions to sort a permutation.
///
/// This is a convenience wrapper: min_swaps = n - cycles.
///
/// # Examples
/// ```
/// use nexcore_combinatorics::min_transpositions;
/// assert_eq!(min_transpositions(&[0, 1, 2, 3]), 0);  // Already sorted
/// assert_eq!(min_transpositions(&[1, 0, 3, 2]), 2);   // Two 2-cycles
/// assert_eq!(min_transpositions(&[3, 2, 1, 0]), 2);   // Two 2-cycles
/// ```
pub fn min_transpositions(perm: &[usize]) -> usize {
    cycle_decomposition(perm).min_transpositions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let r = cycle_decomposition(&[0, 1, 2, 3, 4]);
        assert_eq!(r.num_cycles, 5);
        assert_eq!(r.fixed_points, 5);
        assert_eq!(r.min_transpositions, 0);
    }

    #[test]
    fn test_full_rotation() {
        // (0 1 2 3 4) → one 5-cycle → 4 swaps
        let r = cycle_decomposition(&[1, 2, 3, 4, 0]);
        assert_eq!(r.num_cycles, 1);
        assert_eq!(r.min_transpositions, 4);
    }

    #[test]
    fn test_two_swaps() {
        // (0 1)(2 3) → two 2-cycles → 2 swaps
        let r = cycle_decomposition(&[1, 0, 3, 2]);
        assert_eq!(r.num_cycles, 2);
        assert_eq!(r.min_transpositions, 2);
    }

    #[test]
    fn test_dudeney_238_principle() {
        // Dudeney: 24 jampots, 7 cycles → 17 swaps
        // We can't reproduce his exact permutation without the puzzle diagram,
        // but we can verify the principle: n - cycles = min_swaps
        // Create a permutation with exactly 7 cycles of lengths summing to 24
        // Cycles: (0)(1)(2,3)(4,5,6)(7,8,9,10)(11,12,13,14,15)(16..23)
        let mut perm = vec![0usize; 24];
        // Fixed: 0, 1
        perm[0] = 0;
        perm[1] = 1;
        // 2-cycle: 2→3→2
        perm[2] = 3;
        perm[3] = 2;
        // 3-cycle: 4→5→6→4
        perm[4] = 5;
        perm[5] = 6;
        perm[6] = 4;
        // 4-cycle: 7→8→9→10→7
        perm[7] = 8;
        perm[8] = 9;
        perm[9] = 10;
        perm[10] = 7;
        // 5-cycle: 11→12→13→14→15→11
        perm[11] = 12;
        perm[12] = 13;
        perm[13] = 14;
        perm[14] = 15;
        perm[15] = 11;
        // 8-cycle: 16→17→18→19→20→21→22→23→16
        for i in 16..23 {
            perm[i] = i + 1;
        }
        perm[23] = 16;

        let r = cycle_decomposition(&perm);
        assert_eq!(r.n, 24);
        assert_eq!(r.num_cycles, 7); // 2 fixed + 2-cycle + 3-cycle + 4-cycle + 5-cycle + 8-cycle = 7
        assert_eq!(r.min_transpositions, 17); // 24 - 7 = 17, Dudeney's answer
    }
}
