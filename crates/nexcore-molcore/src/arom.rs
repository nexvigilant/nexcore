// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Aromaticity detection using Hückel's rule.
//!
//! A ring is aromatic when it is planar, fully conjugated, and contains
//! `4n + 2` π electrons for some non-negative integer `n` (2, 6, 10, 14 …).
//!
//! ## Pi-electron counting rules
//!
//! | Atom / context | π electrons contributed |
//! |----------------|------------------------|
//! | Carbon with a double bond to a ring neighbour | 1 |
//! | Carbon marked aromatic in the SMILES (`c`) | 1 |
//! | Nitrogen with two ring bonds (pyridine-like) | 1 |
//! | Nitrogen with two ring bonds + implicit H (pyrrole-like) | 2 |
//! | Oxygen in ring (furan-like) | 2 |
//! | Sulfur in ring (thiophene-like) | 2 |
//! | Aromatic bond in ring | 1 per bond (for atom counting purposes this is
//! |                       | handled via the aromatic flag on the atom) |
//!
//! ## Examples
//!
//! ```rust
//! use nexcore_molcore::graph::MolGraph;
//! use nexcore_molcore::arom::detect_aromaticity;
//! use nexcore_molcore::smiles::parse;
//!
//! let mol = parse("c1ccccc1").unwrap_or_default();
//! let g = MolGraph::from_molecule(mol);
//! let rings = detect_aromaticity(&g);
//! assert_eq!(rings.len(), 1);
//! assert_eq!(rings[0].pi_electrons, 6);
//! ```

use prima_chem::types::BondOrder;

use crate::graph::MolGraph;
use crate::ring::find_sssr;

// Atomic numbers of interest.
const AN_CARBON: u8 = 6;
const AN_NITROGEN: u8 = 7;
const AN_OXYGEN: u8 = 8;
const AN_SULFUR: u8 = 16;

/// A ring that satisfies Hückel's aromaticity criterion.
///
/// ## Primitive Grounding
///
/// - ν (Frequency): 4n+2 recurring pattern of pi electrons
/// - Σ (Sum): total pi electron count
/// - σ (Sequence): ordered list of ring atoms
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AromaticRing {
    /// Atom IDs of the ring members (in traversal order).
    pub atoms: Vec<usize>,
    /// Total number of π electrons contributing to the aromatic system.
    pub pi_electrons: usize,
}

/// Find all aromatic rings in the molecular graph.
///
/// Detects aromaticity by:
/// 1. Finding all rings in the SSSR.
/// 2. Counting π electrons for each ring using the rules above.
/// 3. Returning only those rings where `pi_electrons % 4 == 2` (i.e. 2, 6, 10 …).
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::arom::detect_aromaticity;
/// use nexcore_molcore::smiles::parse;
///
/// let mol = parse("C1CCCCC1").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let rings = detect_aromaticity(&g);
/// assert!(rings.is_empty(), "cyclohexane is not aromatic");
/// ```
#[must_use]
pub fn detect_aromaticity(graph: &MolGraph) -> Vec<AromaticRing> {
    let rings = find_sssr(graph);
    let mut aromatic_rings = Vec::new();

    for ring in &rings {
        if let Some(pi) = count_pi_electrons(graph, ring) {
            if is_huckel(pi) {
                aromatic_rings.push(AromaticRing {
                    atoms: ring.clone(),
                    pi_electrons: pi,
                });
            }
        }
    }

    aromatic_rings
}

/// Test whether a specific ring (given as a slice of atom IDs) is aromatic.
///
/// Returns `true` if the ring satisfies Hückel's `4n+2` rule.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::arom::is_aromatic_ring;
/// use nexcore_molcore::smiles::parse;
///
/// let mol = parse("c1ccccc1").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let ring: Vec<usize> = (0..6).collect();
/// assert!(is_aromatic_ring(&g, &ring));
/// ```
#[must_use]
pub fn is_aromatic_ring(graph: &MolGraph, ring: &[usize]) -> bool {
    match count_pi_electrons(graph, ring) {
        Some(pi) => is_huckel(pi),
        None => false,
    }
}

/// Apply Hückel's rule: 4n + 2 for n >= 0.
///
/// Valid counts: 2, 6, 10, 14, 18, …
#[must_use]
fn is_huckel(pi: usize) -> bool {
    if pi < 2 {
        return false;
    }
    // pi = 4n + 2  ↔  (pi - 2) % 4 == 0
    (pi - 2) % 4 == 0
}

/// Collect the set of atom IDs that form the ring as a `HashSet` for fast
/// membership testing.
fn ring_set(ring: &[usize]) -> std::collections::HashSet<usize> {
    ring.iter().copied().collect()
}

/// Count the π electrons contributed by each atom in `ring`.
///
/// Returns `None` if the ring contains an atom whose π contribution cannot be
/// determined (e.g. an unknown element).  A return of `Some(0)` means the
/// ring has no π electrons and will fail the Hückel test.
fn count_pi_electrons(graph: &MolGraph, ring: &[usize]) -> Option<usize> {
    let ring_members = ring_set(ring);
    let mut total_pi: usize = 0;

    for &atom_id in ring {
        let atom = graph.molecule.atoms.get(atom_id)?;
        let pi = pi_from_atom(graph, atom_id, atom.atomic_number, atom.aromatic, atom.implicit_h, &ring_members);
        total_pi += pi;
    }

    Some(total_pi)
}

/// Compute the π-electron contribution of a single ring atom.
///
/// The decision tree follows the standard Hückel donor rules:
///
/// - Aromatic carbon (`c`): 1
/// - Carbon with a ring double bond: 1
/// - Pyridine nitrogen (2 ring bonds, no H): 1
/// - Pyrrole nitrogen (2 ring bonds, 1 H): 2
/// - Furan oxygen: 2
/// - Thiophene sulfur: 2
/// - All other atoms: 0
fn pi_from_atom(
    graph: &MolGraph,
    atom_id: usize,
    atomic_number: u8,
    is_aromatic: bool,
    implicit_h: u8,
    ring_members: &std::collections::HashSet<usize>,
) -> usize {
    // Collect the bond orders to ring neighbours.
    let ring_bonds: Vec<BondOrder> = graph
        .neighbors(atom_id)
        .iter()
        .filter(|&&(n, _)| ring_members.contains(&n))
        .map(|&(_, order)| order)
        .collect();

    let has_ring_double = ring_bonds.contains(&BondOrder::Double);
    let all_aromatic = ring_bonds
        .iter()
        .all(|&o| o == BondOrder::Aromatic);

    match atomic_number {
        AN_CARBON => {
            if is_aromatic || all_aromatic {
                // Aromatic carbon from SMILES notation → 1 π electron.
                1
            } else if has_ring_double {
                // Kekulé representation double bond → 1 π electron.
                1
            } else {
                0
            }
        }
        AN_NITROGEN => {
            // Pyridine-like: N is sp² hybridised, p orbital into the ring.
            // This happens when N has a double bond to a ring neighbour or
            // is written as lowercase `n` (aromatic).
            if has_ring_double || (is_aromatic && implicit_h == 0) {
                1
            } else {
                // Pyrrole-like: lone pair donated to the ring.
                2
            }
        }
        AN_OXYGEN | AN_SULFUR => {
            // Furan / thiophene: lone pair donated.
            2
        }
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smiles::parse;
    use crate::graph::MolGraph;

    fn aromatic_rings_for(smiles: &str) -> Vec<AromaticRing> {
        let mol = parse(smiles).unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        detect_aromaticity(&g)
    }

    // ------------------------------------------------------------------
    // Hückel rule helper
    // ------------------------------------------------------------------

    #[test]
    fn test_huckel_valid_counts() {
        assert!(is_huckel(2));
        assert!(is_huckel(6));
        assert!(is_huckel(10));
        assert!(is_huckel(14));
    }

    #[test]
    fn test_huckel_invalid_counts() {
        assert!(!is_huckel(0));
        assert!(!is_huckel(1));
        assert!(!is_huckel(4));
        assert!(!is_huckel(8));
    }

    // ------------------------------------------------------------------
    // Benzene (c1ccccc1)
    // ------------------------------------------------------------------

    #[test]
    fn test_benzene_is_aromatic() {
        let rings = aromatic_rings_for("c1ccccc1");
        assert_eq!(rings.len(), 1, "benzene must have exactly one aromatic ring");
    }

    #[test]
    fn test_benzene_pi_electron_count() {
        let rings = aromatic_rings_for("c1ccccc1");
        assert_eq!(rings[0].pi_electrons, 6);
    }

    #[test]
    fn test_benzene_ring_size() {
        let rings = aromatic_rings_for("c1ccccc1");
        assert_eq!(rings[0].atoms.len(), 6);
    }

    // ------------------------------------------------------------------
    // Pyridine (c1ccncc1)
    // ------------------------------------------------------------------

    #[test]
    fn test_pyridine_is_aromatic() {
        // Pyridine: nitrogen contributes 1 p-orbital electron (like C in Kekulé).
        let rings = aromatic_rings_for("c1ccncc1");
        assert_eq!(rings.len(), 1, "pyridine must be aromatic");
    }

    #[test]
    fn test_pyridine_pi_electrons() {
        let rings = aromatic_rings_for("c1ccncc1");
        assert_eq!(rings[0].pi_electrons, 6);
    }

    // ------------------------------------------------------------------
    // Cyclohexane (C1CCCCC1) — not aromatic
    // ------------------------------------------------------------------

    #[test]
    fn test_cyclohexane_not_aromatic() {
        let rings = aromatic_rings_for("C1CCCCC1");
        assert!(rings.is_empty(), "cyclohexane must not be aromatic");
    }

    // ------------------------------------------------------------------
    // Furan (c1ccoc1) — O contributes 2 lone-pair electrons
    // ------------------------------------------------------------------

    #[test]
    fn test_furan_is_aromatic() {
        let rings = aromatic_rings_for("c1ccoc1");
        assert_eq!(rings.len(), 1, "furan must be aromatic");
    }

    #[test]
    fn test_furan_pi_electrons() {
        // 4 aromatic C × 1 + 1 O × 2 = 6
        let rings = aromatic_rings_for("c1ccoc1");
        assert_eq!(rings[0].pi_electrons, 6);
    }

    // ------------------------------------------------------------------
    // Ethanol (CCO) — no aromatic rings
    // ------------------------------------------------------------------

    #[test]
    fn test_ethanol_no_aromatic_rings() {
        let rings = aromatic_rings_for("CCO");
        assert!(rings.is_empty(), "ethanol must have no aromatic rings");
    }

    // ------------------------------------------------------------------
    // Thiophene (c1ccsc1) — S contributes 2 lone-pair electrons
    // ------------------------------------------------------------------

    #[test]
    fn test_thiophene_is_aromatic() {
        let rings = aromatic_rings_for("c1ccsc1");
        assert_eq!(rings.len(), 1, "thiophene must be aromatic");
    }

    #[test]
    fn test_thiophene_pi_electrons() {
        // 4 aromatic C × 1 + 1 S × 2 = 6
        let rings = aromatic_rings_for("c1ccsc1");
        assert_eq!(rings[0].pi_electrons, 6);
    }

    // ------------------------------------------------------------------
    // is_aromatic_ring helper
    // ------------------------------------------------------------------

    #[test]
    fn test_is_aromatic_ring_benzene() {
        let mol = parse("c1ccccc1").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        let ring: Vec<usize> = (0..6).collect();
        assert!(is_aromatic_ring(&g, &ring));
    }

    #[test]
    fn test_is_aromatic_ring_empty_returns_false() {
        let mol = parse("CCO").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        assert!(!is_aromatic_ring(&g, &[]));
    }
}
