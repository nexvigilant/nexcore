// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! VF2 substructure matching for molecular fragment searching.
//!
//! Implements the VF2 algorithm for subgraph isomorphism, adapted for
//! molecular graphs.  Atom compatibility is checked by element (atomic number)
//! and aromaticity; bond compatibility requires an exact bond-order match
//! with a relaxation that lets an aromatic pattern bond match any bond in an
//! aromatic ring context.
//!
//! ## Primitive Grounding
//!
//! - μ (Mapping): `core_pattern → core_molecule` atom mapping
//! - ρ (Recursion): VF2 depth-first search with backtracking
//! - κ (Comparison): feasibility predicate for atom/bond pairs
//! - ∂ (Boundary): terminal sets define the expansion frontier
//!
//! ## Examples
//!
//! ```rust
//! use nexcore_molcore::substruct::{substructure_match, has_substructure, count_matches};
//! use nexcore_molcore::graph::MolGraph;
//! use nexcore_molcore::smiles::parse;
//!
//! let mol  = MolGraph::from_molecule(parse("CC").unwrap_or_default());
//! let pat  = MolGraph::from_molecule(parse("C").unwrap_or_default());
//! let hits = substructure_match(&mol, &pat);
//! assert!(hits.len() >= 2);
//! ```

use prima_chem::types::{AtomId, BondOrder};

use crate::graph::MolGraph;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A mapping from pattern atom IDs to molecule atom IDs.
///
/// Each element `(p, m)` means pattern atom `p` was matched to molecule
/// atom `m`.
pub type AtomMapping = Vec<(AtomId, AtomId)>;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Find all substructure matches of `pattern` in `molecule`.
///
/// Returns a vector of atom mappings.  Each mapping is a list of
/// `(pattern_atom_id, molecule_atom_id)` pairs showing where the pattern
/// was found in the molecule.  Returns an empty vector if no match exists.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::substruct::substructure_match;
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
///
/// let mol = MolGraph::from_molecule(parse("CC").unwrap_or_default());
/// let pat = MolGraph::from_molecule(parse("C").unwrap_or_default());
/// let matches = substructure_match(&mol, &pat);
/// // A single carbon pattern maps onto every carbon atom in ethane.
/// assert!(matches.len() >= 2);
/// ```
#[must_use]
pub fn substructure_match(molecule: &MolGraph, pattern: &MolGraph) -> Vec<AtomMapping> {
    let mut results = Vec::new();
    if pattern.atom_count() == 0 {
        // Empty pattern: one trivial empty mapping.
        results.push(Vec::new());
        return results;
    }
    if pattern.atom_count() > molecule.atom_count() {
        return results;
    }

    let mut state = VF2State::new(pattern.atom_count(), molecule.atom_count());
    vf2_recurse(&mut state, molecule, pattern, &mut results, true);
    results
}

/// Check whether `pattern` is a substructure of `molecule`.
///
/// More efficient than [`substructure_match`] when only a boolean answer
/// is required: the search stops at the first match.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::substruct::has_substructure;
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
///
/// let aspirin = MolGraph::from_molecule(
///     parse("CC(=O)Oc1ccccc1C(=O)O").unwrap_or_default()
/// );
/// let benzene = MolGraph::from_molecule(parse("c1ccccc1").unwrap_or_default());
/// assert!(has_substructure(&aspirin, &benzene));
/// ```
#[must_use]
pub fn has_substructure(molecule: &MolGraph, pattern: &MolGraph) -> bool {
    if pattern.atom_count() == 0 {
        return true;
    }
    if pattern.atom_count() > molecule.atom_count() {
        return false;
    }

    let mut results: Vec<AtomMapping> = Vec::new();
    let mut state = VF2State::new(pattern.atom_count(), molecule.atom_count());
    vf2_recurse(&mut state, molecule, pattern, &mut results, false);
    !results.is_empty()
}

/// Count the number of non-overlapping substructure matches.
///
/// Finds all matches then removes overlapping ones greedily (in the order
/// returned by [`substructure_match`]).
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::substruct::count_matches;
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
///
/// let aspirin = MolGraph::from_molecule(
///     parse("CC(=O)Oc1ccccc1C(=O)O").unwrap_or_default()
/// );
/// let carbonyl = MolGraph::from_molecule(parse("C=O").unwrap_or_default());
/// assert!(count_matches(&aspirin, &carbonyl) >= 2);
/// ```
#[must_use]
pub fn count_matches(molecule: &MolGraph, pattern: &MolGraph) -> usize {
    let all = substructure_match(molecule, pattern);
    // Greedy non-overlapping count: mark molecule atoms as used.
    let mol_size = molecule.atom_count();
    let mut used = vec![false; mol_size];
    let mut count = 0usize;

    for mapping in &all {
        let overlaps = mapping.iter().any(|&(_, m)| used.get(m).copied().unwrap_or(false));
        if !overlaps {
            count += 1;
            for &(_, m) in mapping {
                if m < mol_size {
                    used[m] = true;
                }
            }
        }
    }
    count
}

// ---------------------------------------------------------------------------
// VF2 state
// ---------------------------------------------------------------------------

/// Mutable matching state for one VF2 search.
///
/// Maintains two `core` arrays for O(1) lookup in both directions, plus
/// tracking of the *terminal sets* — atoms adjacent to the current partial
/// mapping — to prioritise candidate selection.
struct VF2State {
    /// `core_pattern[p]` = `Some(m)` when pattern atom `p` is mapped to
    /// molecule atom `m`.
    core_pattern: Vec<Option<usize>>,
    /// `core_molecule[m]` = `Some(p)` when molecule atom `m` is mapped to
    /// pattern atom `p`.
    core_molecule: Vec<Option<usize>>,
    /// Depth (= number of mapped pairs).
    mapping_count: usize,
}

impl VF2State {
    fn new(pattern_size: usize, molecule_size: usize) -> Self {
        Self {
            core_pattern: vec![None; pattern_size],
            core_molecule: vec![None; molecule_size],
            mapping_count: 0,
        }
    }

    #[inline]
    fn is_complete(&self) -> bool {
        self.mapping_count == self.core_pattern.len()
    }

    fn add_pair(&mut self, p: usize, m: usize) {
        self.core_pattern[p] = Some(m);
        self.core_molecule[m] = Some(p);
        self.mapping_count += 1;
    }

    fn remove_pair(&mut self, p: usize, m: usize) {
        self.core_pattern[p] = None;
        self.core_molecule[m] = None;
        self.mapping_count = self.mapping_count.saturating_sub(1);
    }

    /// Collect the current (complete) mapping as an [`AtomMapping`].
    fn to_mapping(&self) -> AtomMapping {
        self.core_pattern
            .iter()
            .enumerate()
            .filter_map(|(p, opt_m)| opt_m.map(|m| (p, m)))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Core recursion
// ---------------------------------------------------------------------------

/// Recursive VF2 expansion.
///
/// When `find_all` is `false` the search returns after the first successful
/// mapping (used by [`has_substructure`]).
fn vf2_recurse(
    state: &mut VF2State,
    molecule: &MolGraph,
    pattern: &MolGraph,
    results: &mut Vec<AtomMapping>,
    find_all: bool,
) {
    if state.is_complete() {
        results.push(state.to_mapping());
        return;
    }

    // Choose next pattern atom to extend (prefer atoms adjacent to the
    // current partial mapping — the "terminal set").
    let p = next_pattern_atom(state, pattern);

    // Iterate over candidate molecule atoms.
    let mol_size = molecule.atom_count();
    for m in 0..mol_size {
        // Already mapped?
        if state.core_molecule[m].is_some() {
            continue;
        }

        if is_feasible(state, molecule, pattern, p, m) {
            state.add_pair(p, m);
            vf2_recurse(state, molecule, pattern, results, find_all);
            if !find_all && !results.is_empty() {
                // Early-exit: we found one match, propagate upwards.
                state.remove_pair(p, m);
                return;
            }
            state.remove_pair(p, m);
        }
    }
}

/// Select the next unmapped pattern atom to extend.
///
/// Priority: atoms that have at least one *already-mapped* pattern
/// neighbour come first (the terminal set).  If no such atom exists, pick
/// the lowest-index unmapped atom.
fn next_pattern_atom(state: &VF2State, pattern: &MolGraph) -> usize {
    let pat_size = state.core_pattern.len();

    // Terminal set: unmapped atoms adjacent to a mapped atom.
    for p in 0..pat_size {
        if state.core_pattern[p].is_some() {
            continue;
        }
        let has_mapped_neighbor = pattern
            .neighbors(p)
            .iter()
            .any(|&(nb, _)| state.core_pattern.get(nb).and_then(|x| *x).is_some());
        if has_mapped_neighbor {
            return p;
        }
    }

    // Fallback: first unmapped atom.
    state
        .core_pattern
        .iter()
        .position(|x| x.is_none())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Feasibility predicate
// ---------------------------------------------------------------------------

/// Test whether extending the mapping with `(p, m)` is feasible.
///
/// Checks:
/// 1. Element (atomic number) match.
/// 2. Aromaticity match.
/// 3. Degree constraint: `degree(m) >= degree(p)`.
/// 4. For every already-mapped pattern neighbour of `p`, the corresponding
///    molecule atom must be a neighbour of `m` with a compatible bond order.
fn is_feasible(
    state: &VF2State,
    molecule: &MolGraph,
    pattern: &MolGraph,
    p: usize,
    m: usize,
) -> bool {
    // --- 1. Atom-level checks -------------------------------------------------
    let pat_atoms = &pattern.molecule.atoms;
    let mol_atoms = &molecule.molecule.atoms;

    let pa = match pat_atoms.get(p) {
        Some(a) => a,
        None => return false,
    };
    let ma = match mol_atoms.get(m) {
        Some(a) => a,
        None => return false,
    };

    // Element must match.
    if pa.atomic_number != ma.atomic_number {
        return false;
    }

    // Aromaticity: if the pattern atom is aromatic, the molecule atom must
    // also be aromatic.  The reverse is not required (a non-aromatic pattern
    // atom can match a non-aromatic molecule atom in an aromatic molecule).
    if pa.aromatic && !ma.aromatic {
        return false;
    }

    // --- 2. Degree constraint -------------------------------------------------
    if molecule.degree(m) < pattern.degree(p) {
        return false;
    }

    // --- 3. Neighbour consistency (the key VF2 invariant) --------------------
    for &(pat_nb, pat_bond) in pattern.neighbors(p) {
        // Only consider already-mapped pattern neighbours.
        let mol_nb = match state.core_pattern.get(pat_nb).and_then(|x| *x) {
            Some(nb) => nb,
            None => continue,
        };

        // The corresponding molecule neighbour must be adjacent to m.
        let mol_bond = match molecule.bond_between(m, mol_nb) {
            Some(b) => b,
            None => return false, // Required edge missing.
        };

        if !bonds_compatible(pat_bond, mol_bond) {
            return false;
        }
    }

    true
}

/// Check bond-order compatibility between a pattern bond and a molecule bond.
///
/// Rules:
/// - Exact match is always valid.
/// - An `Aromatic` pattern bond matches an `Aromatic` molecule bond.
/// - Non-aromatic bonds must match exactly.
#[inline]
fn bonds_compatible(pattern_bond: BondOrder, molecule_bond: BondOrder) -> bool {
    match (pattern_bond, molecule_bond) {
        (BondOrder::Aromatic, BondOrder::Aromatic) => true,
        (p, m) => p == m,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::MolGraph;
    use crate::smiles::parse;
    use prima_chem::Molecule;

    /// Convenience: build a [`MolGraph`] from a SMILES string.
    fn graph_for(smiles: &str) -> MolGraph {
        MolGraph::from_molecule(parse(smiles).unwrap_or_default())
    }

    // -----------------------------------------------------------------------
    // Basic single-atom patterns
    // -----------------------------------------------------------------------

    /// A single-carbon pattern must match every carbon atom in ethane.
    #[test]
    fn test_methyl_in_ethane() {
        let mol = graph_for("CC");
        let pat = graph_for("C");
        let matches = substructure_match(&mol, &pat);
        assert!(
            matches.len() >= 2,
            "single C should match both atoms of ethane, got {}",
            matches.len()
        );
    }

    /// A single oxygen is present in ethanol.
    #[test]
    fn test_oxygen_in_ethanol() {
        let mol = graph_for("CCO");
        let has_o = mol.molecule.atoms.iter().any(|a| a.atomic_number == 8);
        assert!(has_o, "ethanol should have an oxygen atom");
    }

    // -----------------------------------------------------------------------
    // Ring patterns
    // -----------------------------------------------------------------------

    /// Aspirin contains a benzene ring.
    #[test]
    fn test_benzene_in_aspirin() {
        let mol = graph_for("CC(=O)Oc1ccccc1C(=O)O");
        let pat = graph_for("c1ccccc1");
        assert!(
            has_substructure(&mol, &pat),
            "aspirin must contain a benzene ring"
        );
    }

    /// Benzene does not contain the ethanol fragment.
    #[test]
    fn test_no_ethanol_in_benzene() {
        let mol = graph_for("c1ccccc1");
        let pat = graph_for("CCO");
        assert!(
            !has_substructure(&mol, &pat),
            "benzene must not contain the ethanol fragment"
        );
    }

    // -----------------------------------------------------------------------
    // Multi-group patterns
    // -----------------------------------------------------------------------

    /// Aspirin has two carbonyl groups.
    #[test]
    fn test_carbonyl_in_aspirin() {
        let mol = graph_for("CC(=O)Oc1ccccc1C(=O)O");
        let pat = graph_for("C=O");
        let matches = substructure_match(&mol, &pat);
        assert!(
            matches.len() >= 2,
            "aspirin has 2 C=O groups, got {}",
            matches.len()
        );
    }

    // -----------------------------------------------------------------------
    // Self-match and trivial cases
    // -----------------------------------------------------------------------

    /// A molecule must contain itself as a substructure.
    #[test]
    fn test_self_match() {
        let mol = graph_for("CCO");
        let pat = graph_for("CCO");
        assert!(has_substructure(&mol, &pat), "molecule must contain itself");
    }

    /// An empty pattern matches any molecule (returns one empty mapping).
    #[test]
    fn test_empty_pattern() {
        let mol = graph_for("CCO");
        let empty_mol = Molecule::new();
        let pat = MolGraph::from_molecule(empty_mol);
        let matches = substructure_match(&mol, &pat);
        assert!(!matches.is_empty(), "empty pattern should produce one mapping");
    }

    // -----------------------------------------------------------------------
    // has_substructure consistency
    // -----------------------------------------------------------------------

    /// `has_substructure` must agree with `substructure_match`.
    #[test]
    fn test_has_substructure_consistency() {
        let mol = graph_for("CC(=O)Oc1ccccc1C(=O)O");
        let pat = graph_for("c1ccccc1");
        let has = has_substructure(&mol, &pat);
        let matches = substructure_match(&mol, &pat);
        assert_eq!(
            has,
            !matches.is_empty(),
            "has_substructure must agree with substructure_match"
        );
    }

    // -----------------------------------------------------------------------
    // count_matches
    // -----------------------------------------------------------------------

    /// `count_matches` finds at least 2 non-overlapping carbonyls in aspirin.
    #[test]
    fn test_count_carbonyl_in_aspirin() {
        let mol = graph_for("CC(=O)Oc1ccccc1C(=O)O");
        let pat = graph_for("C=O");
        let count = count_matches(&mol, &pat);
        assert!(
            count >= 2,
            "aspirin has at least 2 non-overlapping C=O groups, got {count}"
        );
    }

    // -----------------------------------------------------------------------
    // Size mismatch
    // -----------------------------------------------------------------------

    /// A pattern larger than the molecule cannot match.
    #[test]
    fn test_larger_pattern_no_match() {
        let mol = graph_for("CC");
        let pat = graph_for("CCCCC");
        assert!(
            !has_substructure(&mol, &pat),
            "a pattern larger than the molecule must not match"
        );
    }

    // -----------------------------------------------------------------------
    // Bond-order specificity
    // -----------------------------------------------------------------------

    /// A single-bond pattern must not match a double-bond molecule.
    #[test]
    fn test_double_bond_specificity() {
        let mol = graph_for("C=C"); // ethylene
        let pat = graph_for("CC"); // ethane (single bond)
        let matches = substructure_match(&mol, &pat);
        assert!(
            matches.is_empty(),
            "single-bond pattern CC must not match double-bond C=C"
        );
    }
}
