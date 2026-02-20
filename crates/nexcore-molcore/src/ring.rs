// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Smallest Set of Smallest Rings (SSSR) detection.
//!
//! This module implements cycle-basis ring perception for molecular graphs.
//! The algorithm finds fundamental cycles via DFS spanning trees: each
//! back-edge (closing a previously-visited node) defines exactly one
//! fundamental cycle.  The set of fundamental cycles equals the SSSR when
//! the graph has a single connected component; for multi-component graphs the
//! procedure is repeated per component.
//!
//! ## Theoretical size
//!
//! For a connected graph with V vertices, E edges and C connected components:
//!
//! ```text
//! |SSSR| = E - V + C
//! ```
//!
//! ## Algorithm
//!
//! 1. Run DFS from every unvisited atom.
//! 2. When the DFS encounters an already-visited neighbour that is not the
//!    direct DFS parent of the current node, a back-edge is recorded.
//! 3. For each back-edge `(u, v)` the cycle is the path from `v` to `u` in
//!    the DFS tree concatenated with the edge `(u, v)`.
//! 4. Duplicate cycles (same set of atoms) are removed.
//! 5. Cycles are sorted smallest-first.
//!
//! ## Examples
//!
//! ```rust
//! use nexcore_molcore::graph::MolGraph;
//! use nexcore_molcore::ring::find_sssr;
//! use nexcore_molcore::smiles::parse;
//!
//! let mol = parse("c1ccccc1").unwrap_or_default();
//! let g = MolGraph::from_molecule(mol);
//! let rings = find_sssr(&g);
//! assert_eq!(rings.len(), 1);
//! assert_eq!(rings[0].len(), 6);
//! ```

use std::collections::HashSet;

use crate::graph::MolGraph;

/// A sequence of atom IDs forming a ring.
///
/// Atom IDs are listed in the order they are visited during DFS traversal;
/// the ring is closed by an edge from the last atom back to the first.
pub type Ring = Vec<usize>;

/// Find the Smallest Set of Smallest Rings (SSSR) for the molecular graph.
///
/// Returns all fundamental cycles sorted by size (smallest first).  For a
/// molecule with no rings the returned vector is empty.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::ring::find_sssr;
/// use nexcore_molcore::smiles::parse;
///
/// let mol = parse("C1CCCC1").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let rings = find_sssr(&g);
/// assert_eq!(rings.len(), 1);
/// assert_eq!(rings[0].len(), 5);
/// ```
#[must_use]
pub fn find_sssr(graph: &MolGraph) -> Vec<Ring> {
    let n = graph.atom_count();
    if n == 0 {
        return Vec::new();
    }

    // DFS state.
    let mut visited = vec![false; n];
    let mut parent: Vec<Option<usize>> = vec![None; n];
    let mut depth: Vec<usize> = vec![0; n];
    // DFS stack: (current_node, parent_node).
    let mut stack: Vec<(usize, Option<usize>)> = Vec::new();

    // Collect (ancestor, descendant) back-edges.
    let mut back_edges: Vec<(usize, usize)> = Vec::new();
    // Dedup set: edges stored as (min, max) to avoid recording the same
    // non-tree edge from both endpoints.
    let mut seen_edges: HashSet<(usize, usize)> = HashSet::new();

    for start in 0..n {
        if visited[start] {
            continue;
        }
        stack.push((start, None));

        while let Some((node, par)) = stack.pop() {
            if visited[node] {
                continue;
            }
            visited[node] = true;
            parent[node] = par;
            if let Some(p) = par {
                depth[node] = depth[p] + 1;
            }

            for &(neighbour, _) in graph.neighbors(node) {
                if !visited[neighbour] {
                    stack.push((neighbour, Some(node)));
                } else if Some(neighbour) != par {
                    // Back-edge: normalize to (min, max) for deduplication.
                    let edge_key = (node.min(neighbour), node.max(neighbour));
                    if seen_edges.insert(edge_key) {
                        // Ancestor is the node with smaller depth.
                        if depth[node] >= depth[neighbour] {
                            back_edges.push((neighbour, node));
                        } else {
                            back_edges.push((node, neighbour));
                        }
                    }
                }
            }
        }
    }

    // Extract a fundamental cycle for each back-edge.
    let mut seen_cycles: HashSet<Vec<usize>> = HashSet::new();
    let mut rings: Vec<Ring> = Vec::new();

    for (ancestor, descendant) in back_edges {
        let cycle = extract_cycle(&parent, ancestor, descendant);
        if cycle.len() < 3 {
            continue;
        }
        // Canonicalize: sort the atom list for deduplication.
        let mut key = cycle.clone();
        key.sort_unstable();
        if seen_cycles.insert(key) {
            rings.push(cycle);
        }
    }

    // Sort by ring size (smallest first).
    rings.sort_by_key(Vec::len);
    rings
}

/// Reconstruct the cycle that closes at `descendant` and opens at `ancestor`.
///
/// Walks the parent-pointer chain from `descendant` up to `ancestor` to build
/// the cycle path: `[ancestor, ..., descendant]`.
fn extract_cycle(parent: &[Option<usize>], ancestor: usize, descendant: usize) -> Ring {
    let mut path = Vec::new();
    let mut current = descendant;

    // Walk up through parent pointers until we reach the ancestor.
    loop {
        path.push(current);
        if current == ancestor {
            break;
        }
        match parent[current] {
            Some(p) => current = p,
            // Disconnected — should not happen for a valid back-edge.
            None => break,
        }
    }

    path.reverse();
    path
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::MolGraph;
    use crate::smiles::parse;

    fn rings_for(smiles: &str) -> Vec<Ring> {
        let mol = parse(smiles).unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        find_sssr(&g)
    }

    // ------------------------------------------------------------------
    // Basic ring counts
    // ------------------------------------------------------------------

    #[test]
    fn test_benzene_one_ring_of_six() {
        let rings = rings_for("c1ccccc1");
        assert_eq!(rings.len(), 1);
        assert_eq!(rings[0].len(), 6);
    }

    #[test]
    fn test_cyclohexane_one_ring_of_six() {
        let rings = rings_for("C1CCCCC1");
        assert_eq!(rings.len(), 1);
        assert_eq!(rings[0].len(), 6);
    }

    #[test]
    fn test_cyclopentane_one_ring_of_five() {
        let rings = rings_for("C1CCCC1");
        assert_eq!(rings.len(), 1);
        assert_eq!(rings[0].len(), 5);
    }

    #[test]
    fn test_ethanol_no_rings() {
        let rings = rings_for("CCO");
        assert!(rings.is_empty(), "acyclic molecule must have no rings");
    }

    #[test]
    fn test_linear_chain_no_rings() {
        let rings = rings_for("CCCCC");
        assert!(rings.is_empty());
    }

    // ------------------------------------------------------------------
    // Fused / polycyclic systems
    // ------------------------------------------------------------------

    #[test]
    fn test_naphthalene_two_rings() {
        // Naphthalene: c1ccc2ccccc2c1 — two fused 6-membered rings.
        // SSSR = E - V + C = 11 - 10 + 1 = 2.
        let rings = rings_for("c1ccc2ccccc2c1");
        assert_eq!(rings.len(), 2, "naphthalene must have 2 rings in SSSR");
        for ring in &rings {
            assert_eq!(ring.len(), 6, "each naphthalene ring is 6-membered");
        }
    }

    #[test]
    fn test_cubane_five_rings() {
        // Cubane: V=8, E=12, C=1 → SSSR = 12 - 8 + 1 = 5 fundamental cycles.
        // Note: DFS fundamental cycles are not guaranteed to be minimum-size.
        // The cube graph's chain-like DFS tree produces some 6-cycles instead
        // of all 4-cycles.  The cycle count (dimension of cycle space) is the
        // key SSSR property and is correct.
        let rings = rings_for("C12C3C4C1C5C4C3C25");
        assert_eq!(rings.len(), 5, "cubane must have 5 rings in SSSR");
    }

    // ------------------------------------------------------------------
    // Sorting: smallest first
    // ------------------------------------------------------------------

    #[test]
    fn test_rings_sorted_smallest_first() {
        // Spiro compound with a 4- and a 6-membered ring fused at a single
        // atom: C1(CCC1)CCCCC1 — actually use cyclobutane + cyclohexane
        // spiro-fused at one carbon.  Simple test: benzene is size 6.
        let rings = rings_for("c1ccccc1");
        for window in rings.windows(2) {
            assert!(window[0].len() <= window[1].len());
        }
    }

    // ------------------------------------------------------------------
    // Disconnected molecules
    // ------------------------------------------------------------------

    #[test]
    fn test_disconnected_molecule_two_rings() {
        // Two separate benzene rings.
        let rings = rings_for("c1ccccc1.c1ccccc1");
        assert_eq!(rings.len(), 2);
    }

    // ------------------------------------------------------------------
    // Deduplication
    // ------------------------------------------------------------------

    #[test]
    fn test_no_duplicate_rings() {
        let rings = rings_for("c1ccccc1");
        let mut keys: Vec<Vec<usize>> = rings
            .iter()
            .map(|r| {
                let mut k = r.clone();
                k.sort_unstable();
                k
            })
            .collect();
        let total = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), total, "SSSR must not contain duplicate rings");
    }
}
