//! Linear extensions — combining Dudeney #279 and #253
//!
//! Count valid topological orderings of a DAG by decomposing into
//! independent subgraphs and applying multinomial interleaving.
//!
//! For k independent chains of lengths n1..nk, each with Li internal
//! linear extensions, the total count is:
//!
//! total = multinomial(n1, n2, ..., nk) × L1 × L2 × ... × Lk
//!
//! Primitives: σ(order) + ∂(boundary: dependency) + N(count) + μ(mapping: decomposition)

use crate::grid_paths::{count_interleaved_orderings, multinomial};
use serde::Serialize;

/// Description of an independent subgraph in a DAG.
#[derive(Debug, Clone, Serialize)]
pub struct SubgraphInfo {
    /// Name or identifier for this subgraph.
    pub name: String,
    /// Number of nodes in this subgraph.
    pub node_count: u32,
    /// Number of valid internal orderings (linear extensions within this subgraph).
    pub internal_orderings: u128,
}

/// Result of counting linear extensions.
#[derive(Debug, Clone, Serialize)]
pub struct LinearExtensionResult {
    /// Total number of valid topological orderings.
    pub total_orderings: u128,
    /// Multinomial component (interleaving count).
    pub multinomial_factor: u128,
    /// Product of internal orderings.
    pub internal_factor: u128,
    /// Subgraph details.
    pub subgraphs: Vec<SubgraphInfo>,
    /// Total nodes across all subgraphs.
    pub total_nodes: u32,
}

/// Count linear extensions for a DAG decomposed into independent subgraphs.
///
/// Each subgraph is described by (name, node_count, internal_orderings).
///
/// # Examples
/// ```
/// use nexcore_combinatorics::linear_extensions::count_from_subgraphs;
///
/// // Our hook DAG: 4 independent subgraphs
/// let result = count_from_subgraphs(&[
///     ("stop-chain", 6, 1),
///     ("schema-fork", 3, 2),
///     ("guardian-chain", 3, 1),
///     ("reflex-chain", 2, 1),
/// ]);
/// assert_eq!(result.total_orderings, 3_363_360);
/// ```
pub fn count_from_subgraphs(subgraphs: &[(&str, u32, u128)]) -> LinearExtensionResult {
    let lengths: Vec<u32> = subgraphs.iter().map(|s| s.1).collect();
    let internal: Vec<u128> = subgraphs.iter().map(|s| s.2).collect();

    let multinomial_factor = multinomial(&lengths);
    let internal_factor: u128 = internal.iter().product();
    let total_orderings = count_interleaved_orderings(&lengths, &internal);

    let subgraph_infos: Vec<SubgraphInfo> = subgraphs
        .iter()
        .map(|(name, count, orderings)| SubgraphInfo {
            name: (*name).to_string(),
            node_count: *count,
            internal_orderings: *orderings,
        })
        .collect();

    LinearExtensionResult {
        total_orderings,
        multinomial_factor,
        internal_factor,
        subgraphs: subgraph_infos,
        total_nodes: lengths.iter().sum(),
    }
}

/// Count linear extensions for independent chains only (no forks).
///
/// Simplified version where each subgraph is a simple chain (1 internal ordering).
///
/// # Examples
/// ```
/// use nexcore_combinatorics::count_linear_extensions_chains;
/// // Three independent chains of lengths 3, 4, 5
/// let count = count_linear_extensions_chains(&[3, 4, 5]);
/// assert_eq!(count, 27720);  // C(12,3) * C(9,4) * C(5,5) = 220 * 126 * 1
/// ```
pub fn count_linear_extensions_chains(chain_lengths: &[u32]) -> u128 {
    multinomial(chain_lengths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_dag() {
        let result = count_from_subgraphs(&[
            ("stop-chain", 6, 1),
            ("schema-fork", 3, 2),
            ("guardian-chain", 3, 1),
            ("reflex-chain", 2, 1),
        ]);
        assert_eq!(result.total_orderings, 3_363_360);
        assert_eq!(result.total_nodes, 14);
        assert_eq!(result.multinomial_factor, 1_681_680);
        assert_eq!(result.internal_factor, 2);
    }

    #[test]
    fn test_single_chain() {
        let result = count_from_subgraphs(&[("only-chain", 5, 1)]);
        assert_eq!(result.total_orderings, 1);
    }

    #[test]
    fn test_all_independent() {
        // 4 independent nodes (chains of length 1): 4! = 24 orderings
        let count = count_linear_extensions_chains(&[1, 1, 1, 1]);
        assert_eq!(count, 24);
    }

    #[test]
    fn test_two_chains() {
        // Two chains of length 2: C(4,2) = 6 interleavings
        let count = count_linear_extensions_chains(&[2, 2]);
        assert_eq!(count, 6);
    }
}
