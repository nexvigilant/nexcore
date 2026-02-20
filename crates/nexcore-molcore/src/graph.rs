// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Molecular graph — adjacency list representation wrapping a [`Molecule`].
//!
//! [`MolGraph`] provides efficient graph-theoretic operations on top of the
//! `prima_chem::Molecule` data model.  All structural queries (BFS shortest
//! path, connected components, ring membership) are expressed through this
//! type rather than operating directly on the raw bond list.
//!
//! ## Design
//!
//! Internally the adjacency list is built once at construction time so that
//! neighbour lookups are O(degree) instead of O(bonds).  The wrapped
//! [`Molecule`] is stored verbatim so callers can still access atom/bond data
//! directly.
//!
//! ## Examples
//!
//! ```rust
//! use nexcore_molcore::graph::MolGraph;
//! use nexcore_molcore::smiles::parse;
//!
//! let mol = parse("c1ccccc1").unwrap_or_default();
//! let g = MolGraph::from_molecule(mol);
//! assert_eq!(g.atom_count(), 6);
//! assert_eq!(g.bond_count(), 6);
//! ```

use std::collections::VecDeque;

use prima_chem::Molecule;
use prima_chem::types::{AtomId, BondOrder};

/// Molecular graph with adjacency list representation.
///
/// Wraps [`prima_chem::Molecule`] with efficient graph operations including
/// BFS-based path finding and connected-component detection.
///
/// ## Primitive Grounding
///
/// - μ (Mapping): adjacency list maps atom → neighbours
/// - σ (Sequence): ordered atom/bond iteration
/// - → (Causality): directed edge from atom to neighbour
#[derive(Debug, Clone)]
pub struct MolGraph {
    /// The underlying molecular data (atoms, bonds, metadata).
    pub molecule: Molecule,
    /// `adjacency[atom_idx]` is the list of `(neighbour_id, bond_order)` pairs.
    adjacency: Vec<Vec<(AtomId, BondOrder)>>,
}

impl MolGraph {
    /// Build a [`MolGraph`] from a [`Molecule`].
    ///
    /// The adjacency list is constructed from the molecule's bond list in
    /// O(bonds) time.  The molecule is stored by value so no cloning of the
    /// original is required by the caller.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_molcore::graph::MolGraph;
    /// use nexcore_molcore::smiles::parse;
    ///
    /// let mol = parse("CCO").unwrap_or_default();
    /// let g = MolGraph::from_molecule(mol);
    /// assert_eq!(g.atom_count(), 3);
    /// ```
    #[must_use]
    pub fn from_molecule(mol: Molecule) -> Self {
        let n = mol.atoms.len();
        let mut adjacency: Vec<Vec<(AtomId, BondOrder)>> = vec![Vec::new(); n];

        for bond in &mol.bonds {
            if bond.atom1 < n && bond.atom2 < n {
                adjacency[bond.atom1].push((bond.atom2, bond.order));
                adjacency[bond.atom2].push((bond.atom1, bond.order));
            }
        }

        Self {
            molecule: mol,
            adjacency,
        }
    }

    /// Return the neighbours of `idx` together with their bond orders.
    ///
    /// Returns an empty slice if `idx` is out of range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_molcore::graph::MolGraph;
    /// use nexcore_molcore::smiles::parse;
    ///
    /// let mol = parse("CC(O)C").unwrap_or_default();
    /// let g = MolGraph::from_molecule(mol);
    /// // Atom 1 (central carbon) is bonded to atoms 0, 2, and 3.
    /// assert_eq!(g.neighbors(1).len(), 3);
    /// ```
    #[must_use]
    pub fn neighbors(&self, idx: AtomId) -> &[(AtomId, BondOrder)] {
        self.adjacency.get(idx).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Return the degree (number of bonds) of atom `idx`.
    ///
    /// Returns `0` for out-of-range indices.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_molcore::graph::MolGraph;
    /// use nexcore_molcore::smiles::parse;
    ///
    /// let mol = parse("c1ccccc1").unwrap_or_default();
    /// let g = MolGraph::from_molecule(mol);
    /// // Every benzene carbon has degree 2 in the ring (H is implicit).
    /// assert_eq!(g.degree(0), 2);
    /// ```
    #[must_use]
    pub fn degree(&self, idx: AtomId) -> usize {
        self.adjacency.get(idx).map(Vec::len).unwrap_or(0)
    }

    /// Return the number of atoms in the graph.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_molcore::graph::MolGraph;
    /// use nexcore_molcore::smiles::parse;
    ///
    /// let mol = parse("CCO").unwrap_or_default();
    /// let g = MolGraph::from_molecule(mol);
    /// assert_eq!(g.atom_count(), 3);
    /// ```
    #[must_use]
    pub fn atom_count(&self) -> usize {
        self.molecule.atom_count()
    }

    /// Return the number of bonds in the graph.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_molcore::graph::MolGraph;
    /// use nexcore_molcore::smiles::parse;
    ///
    /// let mol = parse("CCO").unwrap_or_default();
    /// let g = MolGraph::from_molecule(mol);
    /// assert_eq!(g.bond_count(), 2);
    /// ```
    #[must_use]
    pub fn bond_count(&self) -> usize {
        self.molecule.bond_count()
    }

    /// Find the shortest path between two atoms using BFS.
    ///
    /// Returns `None` if no path exists (disconnected graph), or if either
    /// atom index is out of range.  The returned vector includes both the
    /// source and target atoms.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_molcore::graph::MolGraph;
    /// use nexcore_molcore::smiles::parse;
    ///
    /// let mol = parse("CCCCC").unwrap_or_default();
    /// let g = MolGraph::from_molecule(mol);
    /// let path = g.shortest_path(0, 4).unwrap_or_default();
    /// assert_eq!(path, vec![0, 1, 2, 3, 4]);
    /// ```
    #[must_use]
    pub fn shortest_path(&self, from: AtomId, to: AtomId) -> Option<Vec<AtomId>> {
        let n = self.atom_count();
        if from >= n || to >= n {
            return None;
        }
        if from == to {
            return Some(vec![from]);
        }

        let mut visited = vec![false; n];
        let mut parent: Vec<Option<AtomId>> = vec![None; n];
        let mut queue: VecDeque<AtomId> = VecDeque::new();

        visited[from] = true;
        queue.push_back(from);

        'bfs: while let Some(current) = queue.pop_front() {
            for &(neighbour, _) in self.neighbors(current) {
                if !visited[neighbour] {
                    visited[neighbour] = true;
                    parent[neighbour] = Some(current);
                    if neighbour == to {
                        break 'bfs;
                    }
                    queue.push_back(neighbour);
                }
            }
        }

        if !visited[to] {
            return None;
        }

        // Reconstruct path by following parent pointers.
        let mut path = Vec::new();
        let mut current = to;
        loop {
            path.push(current);
            match parent[current] {
                Some(p) => current = p,
                None => break,
            }
        }
        path.reverse();
        Some(path)
    }

    /// Detect connected components using BFS.
    ///
    /// Returns a `Vec` where each inner `Vec<AtomId>` contains the atom
    /// indices belonging to one component.  Components are sorted by their
    /// lowest atom index.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_molcore::graph::MolGraph;
    /// use nexcore_molcore::smiles::parse;
    ///
    /// let mol = parse("[Na+].[Cl-]").unwrap_or_default();
    /// let g = MolGraph::from_molecule(mol);
    /// assert_eq!(g.connected_components().len(), 2);
    /// ```
    #[must_use]
    pub fn connected_components(&self) -> Vec<Vec<AtomId>> {
        let n = self.atom_count();
        let mut visited = vec![false; n];
        let mut components: Vec<Vec<AtomId>> = Vec::new();

        for start in 0..n {
            if visited[start] {
                continue;
            }
            let mut component = Vec::new();
            let mut queue: VecDeque<AtomId> = VecDeque::new();
            queue.push_back(start);
            visited[start] = true;

            while let Some(current) = queue.pop_front() {
                component.push(current);
                for &(neighbour, _) in self.neighbors(current) {
                    if !visited[neighbour] {
                        visited[neighbour] = true;
                        queue.push_back(neighbour);
                    }
                }
            }
            components.push(component);
        }

        components
    }

    /// Return the number of connected components in the graph.
    ///
    /// Convenience wrapper around [`Self::connected_components`].
    #[must_use]
    pub fn component_count(&self) -> usize {
        self.connected_components().len()
    }

    /// Test whether atom `atom` is a member of any ring.
    ///
    /// Delegates to the ring-detection module.  An atom is considered to be
    /// in a ring if it appears in at least one ring returned by
    /// [`crate::ring::find_sssr`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_molcore::graph::MolGraph;
    /// use nexcore_molcore::smiles::parse;
    ///
    /// let mol = parse("c1ccccc1").unwrap_or_default();
    /// let g = MolGraph::from_molecule(mol);
    /// assert!(g.is_in_ring(0));
    /// ```
    #[must_use]
    pub fn is_in_ring(&self, atom: AtomId) -> bool {
        let rings = crate::ring::find_sssr(self);
        rings.iter().any(|ring| ring.contains(&atom))
    }

    /// Return the [`BondOrder`] between atoms `a` and `b`, if a bond exists.
    ///
    /// Returns `None` when `a` and `b` are not bonded or when either index is
    /// out of range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_molcore::graph::MolGraph;
    /// use nexcore_molcore::smiles::parse;
    /// use prima_chem::types::BondOrder;
    ///
    /// let mol = parse("C=O").unwrap_or_default();
    /// let g = MolGraph::from_molecule(mol);
    /// assert_eq!(g.bond_between(0, 1), Some(BondOrder::Double));
    /// ```
    #[must_use]
    pub fn bond_between(&self, a: AtomId, b: AtomId) -> Option<BondOrder> {
        self.neighbors(a)
            .iter()
            .find(|&&(n, _)| n == b)
            .map(|&(_, order)| order)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smiles::parse;

    // ------------------------------------------------------------------
    // Basic construction
    // ------------------------------------------------------------------

    #[test]
    fn test_benzene_atom_and_bond_count() {
        let mol = parse("c1ccccc1").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        assert_eq!(g.atom_count(), 6);
        assert_eq!(g.bond_count(), 6);
    }

    #[test]
    fn test_benzene_all_atoms_degree_2() {
        let mol = parse("c1ccccc1").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        for atom in 0..6 {
            assert_eq!(
                g.degree(atom),
                2,
                "benzene atom {atom} should have degree 2"
            );
        }
    }

    #[test]
    fn test_ethanol_atom_and_bond_count() {
        let mol = parse("CCO").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        assert_eq!(g.atom_count(), 3);
        assert_eq!(g.bond_count(), 2);
    }

    #[test]
    fn test_ethanol_terminal_atoms_degree_1() {
        let mol = parse("CCO").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        // Atom 0 (C) → bonded only to atom 1
        assert_eq!(g.degree(0), 1);
        // Atom 2 (O) → bonded only to atom 1
        assert_eq!(g.degree(2), 1);
        // Atom 1 (C) → bonded to atoms 0 and 2
        assert_eq!(g.degree(1), 2);
    }

    // ------------------------------------------------------------------
    // Shortest path
    // ------------------------------------------------------------------

    #[test]
    fn test_shortest_path_linear_chain() {
        let mol = parse("CCCCC").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        let path = g.shortest_path(0, 4).unwrap_or_default();
        assert_eq!(path, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_shortest_path_benzene_opposite_atoms() {
        // Atoms 0 and 3 are opposite in benzene.  The shortest path going
        // around the ring should contain 4 atoms: [0,1,2,3] or [0,5,4,3].
        let mol = parse("c1ccccc1").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        let path = g.shortest_path(0, 3).unwrap_or_default();
        // Length is 4 (three intermediate atoms: two possible routes of len 4).
        assert_eq!(
            path.len(),
            4,
            "shortest path in benzene 0→3 must have 4 nodes"
        );
        assert_eq!(*path.first().unwrap_or(&usize::MAX), 0);
        assert_eq!(*path.last().unwrap_or(&usize::MAX), 3);
    }

    #[test]
    fn test_shortest_path_same_atom() {
        let mol = parse("CC").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        let path = g.shortest_path(0, 0).unwrap_or_default();
        assert_eq!(path, vec![0]);
    }

    #[test]
    fn test_shortest_path_disconnected_returns_none() {
        let mol = parse("[Na+].[Cl-]").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        assert!(g.shortest_path(0, 1).is_none());
    }

    // ------------------------------------------------------------------
    // Connected components
    // ------------------------------------------------------------------

    #[test]
    fn test_connected_components_benzene() {
        let mol = parse("c1ccccc1").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        assert_eq!(g.connected_components().len(), 1);
    }

    #[test]
    fn test_connected_components_nacl() {
        let mol = parse("[Na+].[Cl-]").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        assert_eq!(g.connected_components().len(), 2);
    }

    // ------------------------------------------------------------------
    // Neighbours
    // ------------------------------------------------------------------

    #[test]
    fn test_neighbors_central_atom_isobutane() {
        // CC(O)C: atom 1 (central C) is bonded to atoms 0, 2, and 3.
        let mol = parse("CC(O)C").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        assert_eq!(g.neighbors(1).len(), 3);
    }

    #[test]
    fn test_neighbors_out_of_range_returns_empty() {
        let mol = parse("C").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        assert_eq!(g.neighbors(99).len(), 0);
    }

    // ------------------------------------------------------------------
    // Bond between
    // ------------------------------------------------------------------

    #[test]
    fn test_bond_between_double_bond() {
        let mol = parse("C=O").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        assert_eq!(g.bond_between(0, 1), Some(BondOrder::Double));
    }

    #[test]
    fn test_bond_between_non_bonded() {
        let mol = parse("CCC").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        // Atoms 0 and 2 are not directly bonded.
        assert_eq!(g.bond_between(0, 2), None);
    }

    #[test]
    fn test_bond_between_aromatic() {
        let mol = parse("c1ccccc1").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        assert_eq!(g.bond_between(0, 1), Some(BondOrder::Aromatic));
    }
}
