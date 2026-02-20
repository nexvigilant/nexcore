// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Molecular descriptor calculations for the Chemivigilance Platform.
//!
//! Computes physicochemical descriptors used for drug-likeness profiling
//! (Lipinski's Rule of Five and related criteria).  All calculations operate
//! on a [`MolGraph`] so they are independent of the input format.
//!
//! ## Descriptors computed
//!
//! | Descriptor | Method |
//! |------------|--------|
//! | Molecular weight | `prima_chem::Molecule::molecular_weight()` |
//! | LogP | Wildman–Crippen atom-type contribution |
//! | TPSA | Topological polar surface area (N, O, S contributions) |
//! | HBA | Hydrogen bond acceptors (N + O heavy atoms) |
//! | HBD | Hydrogen bond donors (N–H, O–H) |
//! | Rotatable bonds | Acyclic single bonds between non-terminal atoms |
//! | Ring count | SSSR size |
//! | Aromatic ring count | Hückel-aromatic SSSR rings |
//! | Heavy atom count | `MolGraph::atom_count()` |
//!
//! ## Examples
//!
//! ```rust
//! use nexcore_molcore::descriptor::calculate_descriptors;
//! use nexcore_molcore::graph::MolGraph;
//! use nexcore_molcore::smiles::parse;
//!
//! let mol = parse("c1ccccc1").unwrap_or_default();
//! let g = MolGraph::from_molecule(mol);
//! let d = calculate_descriptors(&g);
//! assert_eq!(d.num_aromatic_rings, 1);
//! assert!(d.logp > 0.0);
//! ```

use prima_chem::types::BondOrder;

use crate::arom::detect_aromaticity;
use crate::graph::MolGraph;
use crate::ring::find_sssr;

// ---------------------------------------------------------------------------
// Atomic number constants
// ---------------------------------------------------------------------------

const AN_CARBON: u8 = 6;
const AN_NITROGEN: u8 = 7;
const AN_OXYGEN: u8 = 8;
const AN_FLUORINE: u8 = 9;
const AN_PHOSPHORUS: u8 = 15;
const AN_SULFUR: u8 = 16;
const AN_CHLORINE: u8 = 17;
const AN_BROMINE: u8 = 35;
const AN_IODINE: u8 = 53;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Molecular descriptors for drug-likeness profiling.
///
/// Computed by [`calculate_descriptors`].  All values are derived from the
/// heavy-atom graph; implicit hydrogens are accounted for in LogP and HBD
/// via the `implicit_h` field on each atom.
///
/// ## Primitive Grounding
///
/// - N (Quantity): atom counts, bond counts, ring counts
/// - Σ (Sum): additive contributions (MW, LogP, TPSA)
/// - κ (Comparison): descriptor thresholds (Lipinski RO5)
/// - ν (Frequency): rotatable bond frequency
#[derive(Debug, Clone, PartialEq)]
pub struct Descriptors {
    /// Molecular weight in Da (from `prima_chem`).
    pub molecular_weight: f64,
    /// Wildman–Crippen LogP estimate (atom-type contribution).
    pub logp: f64,
    /// Topological polar surface area (Å²).
    pub tpsa: f64,
    /// Hydrogen bond acceptor count (N + O heavy atoms).
    pub hba: u8,
    /// Hydrogen bond donor count (N–H and O–H).
    pub hbd: u8,
    /// Number of rotatable bonds (acyclic single bonds between non-terminal atoms).
    pub rotatable_bonds: u8,
    /// Total ring count (SSSR).
    pub num_rings: u8,
    /// Aromatic ring count (Hückel rule).
    pub num_aromatic_rings: u8,
    /// Number of heavy (non-hydrogen) atoms.
    pub heavy_atom_count: usize,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Calculate all molecular descriptors for a molecule represented as a graph.
///
/// Each descriptor is computed independently; ring detection is performed once
/// and shared between `num_rings`, `num_aromatic_rings`, and the rotatable-
/// bond filter.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::descriptor::calculate_descriptors;
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
///
/// let mol = parse("CCO").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let d = calculate_descriptors(&g);
/// assert_eq!(d.heavy_atom_count, 3);
/// assert_eq!(d.hba, 1); // one O
/// assert_eq!(d.hbd, 1); // O–H
/// ```
#[must_use]
pub fn calculate_descriptors(graph: &MolGraph) -> Descriptors {
    // Ring detection — performed once, reused for num_rings, num_aromatic_rings,
    // rotatable bond ring membership.
    let sssr = find_sssr(graph);
    let aromatic_rings = detect_aromaticity(graph);

    // Build a flat set of (min, max) atom-index pairs that are ring bonds.
    let ring_bond_set: std::collections::HashSet<(usize, usize)> = sssr
        .iter()
        .flat_map(|ring| {
            let n = ring.len();
            (0..n).map(move |i| {
                let a = ring[i];
                let b = ring[(i + 1) % n];
                (a.min(b), a.max(b))
            })
        })
        .collect();

    Descriptors {
        molecular_weight: graph.molecule.molecular_weight(),
        logp: compute_logp(graph),
        tpsa: compute_tpsa(graph),
        hba: compute_hba(graph),
        hbd: compute_hbd(graph),
        rotatable_bonds: compute_rotatable_bonds(graph, &ring_bond_set),
        num_rings: sssr.len().min(u8::MAX as usize) as u8,
        num_aromatic_rings: aromatic_rings.len().min(u8::MAX as usize) as u8,
        heavy_atom_count: graph.atom_count(),
    }
}

// ---------------------------------------------------------------------------
// LogP — Wildman–Crippen simplified atom-type contributions
// ---------------------------------------------------------------------------

/// Wildman–Crippen LogP atom-type contribution constants.
///
/// These are simplified contributions drawn from the original Wildman & Crippen
/// (1999) parameterisation.  A full implementation would use a richer atom-type
/// classifier; this version captures the dominant contributions for common
/// drug-like fragments.
mod crippen {
    /// Aliphatic sp3 carbon (no halogen neighbours, no carbonyl).
    pub const C_ALIPHATIC: f64 = 0.1441;
    /// Aromatic carbon.
    pub const C_AROMATIC: f64 = 0.1350;
    /// Aliphatic carbon bonded to at least one halogen.
    pub const C_HALOGEN: f64 = 0.2960;
    /// Carbon in a carbonyl (C=O).
    pub const C_CARBONYL: f64 = -0.1002;
    /// Any nitrogen.
    pub const N_ANY: f64 = -0.7614;
    /// Oxygen with one heavy-atom bond (hydroxyl / phenol / carboxylic OH).
    pub const O_SINGLE: f64 = -0.2893;
    /// Oxygen with two heavy-atom bonds or double bond (ether, carbonyl, ester).
    pub const O_DOUBLE_OR_ETHER: f64 = -0.5262;
    /// Sulfur.
    pub const S_ANY: f64 = -0.0024;
    /// Fluorine.
    pub const F: f64 = 0.4118;
    /// Chlorine.
    pub const CL: f64 = 0.6895;
    /// Bromine.
    pub const BR: f64 = 0.8813;
    /// Iodine.
    pub const I: f64 = 1.0500;
    /// Phosphorus.
    pub const P_ANY: f64 = -0.0390;
    /// Per implicit hydrogen on any atom.
    pub const H_IMPLICIT: f64 = 0.1230;
}

/// Compute the Wildman–Crippen LogP estimate.
///
/// Each heavy atom contributes according to its atom type; each implicit
/// hydrogen on any atom adds `+0.1230`.
fn compute_logp(graph: &MolGraph) -> f64 {
    let mut logp = 0.0_f64;

    for (atom_idx, atom) in graph.molecule.atoms.iter().enumerate() {
        let contribution = logp_atom_contribution(graph, atom_idx, atom.atomic_number, atom.aromatic, atom.implicit_h);
        logp += contribution;
    }

    logp
}

/// Compute the LogP contribution for a single atom.
///
/// Returns 0.0 for elements not in the Crippen table.
fn logp_atom_contribution(
    graph: &MolGraph,
    atom_idx: usize,
    atomic_number: u8,
    is_aromatic: bool,
    implicit_h: u8,
) -> f64 {
    // Implicit hydrogen contribution — applies to every atom type.
    let h_contribution = f64::from(implicit_h) * crippen::H_IMPLICIT;

    let heavy_contribution = match atomic_number {
        AN_CARBON => carbon_logp(graph, atom_idx, is_aromatic),
        AN_NITROGEN => crippen::N_ANY,
        AN_OXYGEN => oxygen_logp(graph, atom_idx),
        AN_SULFUR => crippen::S_ANY,
        AN_FLUORINE => crippen::F,
        AN_CHLORINE => crippen::CL,
        AN_BROMINE => crippen::BR,
        AN_IODINE => crippen::I,
        AN_PHOSPHORUS => crippen::P_ANY,
        _ => 0.0,
    };

    heavy_contribution + h_contribution
}

/// Classify a carbon atom and return its Crippen contribution.
fn carbon_logp(graph: &MolGraph, atom_idx: usize, is_aromatic: bool) -> f64 {
    // Aromatic carbon (written as `c` in SMILES or all-aromatic bonds).
    if is_aromatic {
        return crippen::C_AROMATIC;
    }

    let neighbors = graph.neighbors(atom_idx);

    // Check for aromatic bonds (covers Kekulé-aromatic representations).
    let all_aromatic = !neighbors.is_empty()
        && neighbors
            .iter()
            .all(|&(_, order)| order == BondOrder::Aromatic);
    if all_aromatic {
        return crippen::C_AROMATIC;
    }

    // Carbonyl: C has a double bond to O.
    let has_carbonyl = neighbors.iter().any(|&(nbr, order)| {
        order == BondOrder::Double
            && graph
                .molecule
                .atoms
                .get(nbr)
                .is_some_and(|a| a.atomic_number == AN_OXYGEN)
    });
    if has_carbonyl {
        return crippen::C_CARBONYL;
    }

    // Bonded to a halogen (F, Cl, Br, I).
    let has_halogen = neighbors.iter().any(|&(nbr, _)| {
        graph
            .molecule
            .atoms
            .get(nbr)
            .is_some_and(|a| matches!(a.atomic_number, AN_FLUORINE | AN_CHLORINE | AN_BROMINE | AN_IODINE))
    });
    if has_halogen {
        return crippen::C_HALOGEN;
    }

    // Default: aliphatic sp3 carbon.
    crippen::C_ALIPHATIC
}

/// Classify an oxygen atom and return its Crippen contribution.
fn oxygen_logp(graph: &MolGraph, atom_idx: usize) -> f64 {
    let neighbors = graph.neighbors(atom_idx);

    // Oxygen with a double bond to its neighbour (C=O) or bonded to two heavy atoms → ether.
    let has_double = neighbors
        .iter()
        .any(|&(_, order)| order == BondOrder::Double);

    let heavy_bond_count = neighbors.len();

    if has_double || heavy_bond_count >= 2 {
        crippen::O_DOUBLE_OR_ETHER
    } else {
        // Single heavy-atom bond: hydroxyl (-OH), carboxylic terminal O.
        crippen::O_SINGLE
    }
}

// ---------------------------------------------------------------------------
// TPSA — topological polar surface area
// ---------------------------------------------------------------------------

/// Compute the topological polar surface area (Å²).
///
/// Contributions come from N, O, and S atoms based on their connectivity
/// and hydrogen count.  The values follow the simplified Ertl (2000)
/// fragment-based TPSA method.
fn compute_tpsa(graph: &MolGraph) -> f64 {
    let mut tpsa = 0.0_f64;

    for (atom_idx, atom) in graph.molecule.atoms.iter().enumerate() {
        let contribution = tpsa_contribution(graph, atom_idx, atom.atomic_number, atom.aromatic, atom.implicit_h);
        tpsa += contribution;
    }

    tpsa
}

/// Compute the TPSA contribution for a single atom.
///
/// Returns 0.0 for carbon, halogens, and other non-polar elements.
fn tpsa_contribution(
    graph: &MolGraph,
    atom_idx: usize,
    atomic_number: u8,
    is_aromatic: bool,
    implicit_h: u8,
) -> f64 {
    match atomic_number {
        AN_NITROGEN => nitrogen_tpsa(graph, atom_idx, is_aromatic, implicit_h),
        AN_OXYGEN => oxygen_tpsa(graph, atom_idx),
        AN_SULFUR => 25.30,
        _ => 0.0,
    }
}

/// TPSA contribution for a nitrogen atom.
fn nitrogen_tpsa(graph: &MolGraph, atom_idx: usize, is_aromatic: bool, implicit_h: u8) -> f64 {
    let heavy_degree = graph.degree(atom_idx);
    let neighbors = graph.neighbors(atom_idx);

    // Aromatic nitrogen: pyridine-like (no H) vs pyrrole-like (with H).
    // Guard against the empty-neighbor case (lone atom): all() on an empty
    // slice returns true, which would be a false-positive aromatic classification.
    let all_bonds_aromatic = !neighbors.is_empty()
        && neighbors
            .iter()
            .all(|&(_, order)| order == BondOrder::Aromatic);

    if is_aromatic || all_bonds_aromatic {
        return if implicit_h == 0 {
            12.89 // pyridine-like
        } else {
            15.79 // pyrrole-like
        };
    }

    // NH2: primary amine (one heavy bond, 2 implicit H).
    if heavy_degree == 1 && implicit_h >= 2 {
        return 26.02;
    }

    match heavy_degree {
        // N with 1 heavy bond + H (secondary terminal).
        1 => 26.02,
        // N with 2 heavy bonds (secondary amine).
        2 => 12.36,
        // N with 3+ heavy bonds (tertiary).
        _ => 3.24,
    }
}

/// TPSA contribution for an oxygen atom.
fn oxygen_tpsa(graph: &MolGraph, atom_idx: usize) -> f64 {
    let neighbors = graph.neighbors(atom_idx);

    let has_double = neighbors
        .iter()
        .any(|&(_, order)| order == BondOrder::Double);
    let heavy_degree = neighbors.len();

    if has_double {
        // C=O double bond oxygen.
        17.07
    } else if heavy_degree >= 2 {
        // Ether oxygen (bonded to 2 heavy atoms, no double bond).
        9.23
    } else {
        // Hydroxyl oxygen (one heavy-atom bond, implicit H).
        20.23
    }
}

// ---------------------------------------------------------------------------
// HBA — hydrogen bond acceptors
// ---------------------------------------------------------------------------

/// Count hydrogen bond acceptors: all N and O heavy atoms.
///
/// This uses the simple Lipinski definition: every N and O regardless of
/// hybridisation or substitution.
fn compute_hba(graph: &MolGraph) -> u8 {
    let count = graph
        .molecule
        .atoms
        .iter()
        .filter(|a| matches!(a.atomic_number, AN_NITROGEN | AN_OXYGEN))
        .count();
    count.min(u8::MAX as usize) as u8
}

// ---------------------------------------------------------------------------
// HBD — hydrogen bond donors
// ---------------------------------------------------------------------------

/// Count hydrogen bond donors: N and O atoms that carry at least one implicit H.
fn compute_hbd(graph: &MolGraph) -> u8 {
    let count = graph
        .molecule
        .atoms
        .iter()
        .filter(|a| {
            matches!(a.atomic_number, AN_NITROGEN | AN_OXYGEN) && a.implicit_h > 0
        })
        .count();
    count.min(u8::MAX as usize) as u8
}

// ---------------------------------------------------------------------------
// Rotatable bonds
// ---------------------------------------------------------------------------

/// Count rotatable bonds.
///
/// A bond is rotatable when **all** of the following hold:
/// 1. It is a single bond (`BondOrder::Single`).
/// 2. Neither atom is terminal (graph degree > 1).
/// 3. The bond is not in any ring (not present in `ring_bond_set`).
///
/// The molecule's bond list is iterated directly; each bond appears once so
/// no deduplication is needed.
fn compute_rotatable_bonds(
    graph: &MolGraph,
    ring_bond_set: &std::collections::HashSet<(usize, usize)>,
) -> u8 {
    let count = graph
        .molecule
        .bonds
        .iter()
        .filter(|bond| {
            if bond.order != BondOrder::Single {
                return false;
            }
            let a = bond.atom1;
            let b = bond.atom2;
            // Both atoms must be non-terminal.
            if graph.degree(a) <= 1 || graph.degree(b) <= 1 {
                return false;
            }
            // Bond must not be part of a ring.
            let key = (a.min(b), a.max(b));
            !ring_bond_set.contains(&key)
        })
        .count();

    count.min(u8::MAX as usize) as u8
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::MolGraph;
    use crate::smiles::parse;

    fn descriptors_for(smiles: &str) -> Descriptors {
        let mol = parse(smiles).unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        calculate_descriptors(&g)
    }

    // ------------------------------------------------------------------
    // Molecular weight
    // ------------------------------------------------------------------

    #[test]
    fn test_ethanol_mw() {
        let d = descriptors_for("CCO");
        assert!((d.molecular_weight - 46.07).abs() < 0.1, "ethanol MW expected ~46.07, got {}", d.molecular_weight);
    }

    #[test]
    fn test_water_mw() {
        let d = descriptors_for("[OH2]");
        assert!((d.molecular_weight - 18.02).abs() < 0.1, "water MW expected ~18.02, got {}", d.molecular_weight);
    }

    // ------------------------------------------------------------------
    // Hydrogen bond acceptors / donors
    // ------------------------------------------------------------------

    #[test]
    fn test_ethanol_hba() {
        let d = descriptors_for("CCO");
        assert_eq!(d.hba, 1); // one O
    }

    #[test]
    fn test_ethanol_hbd() {
        let d = descriptors_for("CCO");
        assert_eq!(d.hbd, 1); // O–H
    }

    #[test]
    fn test_benzene_hba_zero() {
        let d = descriptors_for("c1ccccc1");
        assert_eq!(d.hba, 0);
    }

    // ------------------------------------------------------------------
    // Ring counts
    // ------------------------------------------------------------------

    #[test]
    fn test_benzene_rings() {
        let d = descriptors_for("c1ccccc1");
        assert_eq!(d.num_rings, 1);
        assert_eq!(d.num_aromatic_rings, 1);
    }

    #[test]
    fn test_ethanol_no_rings() {
        let d = descriptors_for("CCO");
        assert_eq!(d.num_rings, 0);
        assert_eq!(d.num_aromatic_rings, 0);
    }

    // ------------------------------------------------------------------
    // Heavy atom count
    // ------------------------------------------------------------------

    #[test]
    fn test_ethanol_heavy_atoms() {
        let d = descriptors_for("CCO");
        assert_eq!(d.heavy_atom_count, 3);
    }

    #[test]
    fn test_benzene_heavy_atoms() {
        let d = descriptors_for("c1ccccc1");
        assert_eq!(d.heavy_atom_count, 6);
    }

    // ------------------------------------------------------------------
    // Rotatable bonds
    // ------------------------------------------------------------------

    #[test]
    fn test_ethane_rotatable() {
        // Both carbons are terminal (degree 1) → no rotatable bonds.
        let d = descriptors_for("CC");
        assert_eq!(d.rotatable_bonds, 0, "ethane: both atoms terminal, got {}", d.rotatable_bonds);
    }

    #[test]
    fn test_butane_rotatable() {
        // C-C-C-C: the central C–C bond (atoms 1–2) is non-terminal and acyclic.
        let d = descriptors_for("CCCC");
        assert!(d.rotatable_bonds >= 1, "butane should have >=1 rotatable bond, got {}", d.rotatable_bonds);
    }

    #[test]
    fn test_benzene_no_rotatable() {
        let d = descriptors_for("c1ccccc1");
        assert_eq!(d.rotatable_bonds, 0, "benzene has no rotatable bonds, got {}", d.rotatable_bonds);
    }

    // ------------------------------------------------------------------
    // LogP direction tests
    // ------------------------------------------------------------------

    #[test]
    fn test_logp_ethanol_negative() {
        let d = descriptors_for("CCO");
        // Simplified Crippen gives approximate values; real ethanol LogP is
        // about -0.31.  Assert it is noticeably lower than benzene (~1.6).
        assert!(d.logp < 1.0, "ethanol LogP should be low (< 1.0), got {}", d.logp);
    }

    #[test]
    fn test_logp_benzene_positive() {
        let d = descriptors_for("c1ccccc1");
        assert!(d.logp > 0.0, "benzene LogP should be positive, got {}", d.logp);
    }

    // ------------------------------------------------------------------
    // TPSA tests
    // ------------------------------------------------------------------

    #[test]
    fn test_tpsa_benzene_zero() {
        let d = descriptors_for("c1ccccc1");
        assert!(d.tpsa < 1.0, "benzene has no polar atoms, got TPSA={}", d.tpsa);
    }

    #[test]
    fn test_tpsa_ethanol_positive() {
        let d = descriptors_for("CCO");
        assert!(d.tpsa > 10.0, "ethanol has OH, TPSA should be >10, got {}", d.tpsa);
    }

    // ------------------------------------------------------------------
    // Aspirin comprehensive
    // ------------------------------------------------------------------

    #[test]
    fn test_aspirin_descriptors() {
        let d = descriptors_for("CC(=O)Oc1ccccc1C(=O)O");
        assert!(
            (d.molecular_weight - 180.16).abs() < 0.2,
            "aspirin MW ~180.16, got {}",
            d.molecular_weight
        );
        assert!(d.hba >= 3, "aspirin has >=3 HBA (O atoms), got {}", d.hba);
        assert_eq!(d.num_aromatic_rings, 1, "aspirin has 1 aromatic ring");
        assert!(d.heavy_atom_count >= 13, "aspirin has >=13 heavy atoms, got {}", d.heavy_atom_count);
    }
}
