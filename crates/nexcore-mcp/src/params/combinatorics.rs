//! Combinatorics MCP tool parameters.
//!
//! Typed parameter structs for Catalan numbers, derangements, cycle
//! decomposition, Josephus problem, grid paths, and linear extensions.

use schemars::JsonSchema;
use serde::Deserialize;

/// Compute the nth Catalan number C(n).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CombCatalanParams {
    /// Which Catalan number to compute (0-indexed).
    pub n: u32,
}

/// Get the first 20 Catalan numbers as a lookup table.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CombCatalanTableParams {}

/// Decompose a permutation into disjoint cycles.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CombCycleDecompositionParams {
    /// 0-indexed permutation array (e.g. [1, 0, 3, 2]).
    pub permutation: Vec<usize>,
}

/// Compute minimum transpositions to sort a permutation.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CombMinTranspositionsParams {
    /// 0-indexed permutation array.
    pub permutation: Vec<usize>,
}

/// Compute D(n), the number of derangements of n elements.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CombDerangementParams {
    /// Number of elements.
    pub n: u32,
}

/// Compute the probability D(n)/n! that a random permutation is a derangement.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CombDerangementProbabilityParams {
    /// Number of elements.
    pub n: u32,
}

/// Count monotone lattice paths from (0,0) to (m,n) — binomial C(m+n, m).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CombGridPathsParams {
    /// Grid width (right-steps).
    pub m: u32,
    /// Grid height (down-steps).
    pub n: u32,
}

/// Compute the binomial coefficient C(n, k).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CombBinomialParams {
    /// Total elements.
    pub n: u32,
    /// Elements chosen.
    pub k: u32,
}

/// Compute the multinomial coefficient (sum of lengths)! / product(length_i!).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CombMultinomialParams {
    /// Lengths of independent sequences.
    pub lengths: Vec<u32>,
}

/// Compute the Josephus survivor position (0-indexed).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CombJosephusParams {
    /// Number of people in the circle.
    pub n: u32,
    /// Count step (every kth person eliminated).
    pub k: u32,
}

/// Compute the full Josephus elimination order.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CombEliminationOrderParams {
    /// Number of people in the circle.
    pub n: u32,
    /// Count step.
    pub k: u32,
}

/// Count linear extensions for independent chains (multinomial interleaving).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CombLinearExtensionsParams {
    /// Lengths of independent chains.
    pub chain_lengths: Vec<u32>,
}
