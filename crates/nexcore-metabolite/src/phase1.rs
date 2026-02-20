// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Phase I metabolite prediction (CYP-mediated oxidations).
//!
//! Rules applied:
//!
//! | Structural feature | Transformation | Enzyme |
//! |---|---|---|
//! | Aromatic C bearing implicit H | Hydroxylation | CYP1A2/CYP2D6 |
//! | N bonded to CH₃ | N-dealkylation | CYP3A4 |
//! | O bonded to CH₃ | O-dealkylation | CYP2D6 |
//! | Aromatic C=C bond | Epoxidation (reactive) | CYP1A2 |

use nexcore_molcore::graph::MolGraph;
use prima_chem::types::BondOrder;

use crate::types::{Metabolite, Transformation};

/// Predict Phase I metabolic transformations for `graph`.
///
/// Scans every atom and bond for CYP-susceptible structural features and
/// returns a list of predicted metabolites ordered by discovery.  Probability
/// estimates are rule-based constants derived from published metabolic soft-
/// spot heuristics.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
/// use nexcore_metabolite::phase1::predict_phase1;
/// use nexcore_metabolite::types::Transformation;
///
/// let mol = parse("c1ccccc1").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let metabolites = predict_phase1(&g);
/// assert!(metabolites.iter().any(|m| matches!(m.transformation, Transformation::Hydroxylation { .. })));
/// ```
#[must_use]
pub fn predict_phase1(graph: &MolGraph) -> Vec<Metabolite> {
    let mut metabolites = Vec::new();

    for idx in 0..graph.atom_count() {
        let atom = match graph.molecule.atoms.get(idx) {
            Some(a) => a,
            None => continue,
        };

        // ----------------------------------------------------------------
        // Aromatic hydroxylation
        // CYP1A2 and CYP2D6 preferentially hydroxylate aromatic rings at
        // positions bearing an available hydrogen.
        // ----------------------------------------------------------------
        if atom.aromatic && atom.atomic_number == 6 && atom.implicit_h > 0 {
            metabolites.push(Metabolite {
                transformation: Transformation::Hydroxylation { site: idx },
                site_description: format!("Aromatic C at position {idx}"),
                probability: 0.4,
                reactive_intermediate: false,
                enzyme: Some("CYP1A2/CYP2D6".to_string()),
            });
        }

        // ----------------------------------------------------------------
        // N-dealkylation
        // CYP3A4 cleaves N-methyl (and other N-alkyl) groups. Detected as
        // a nitrogen bonded to a carbon carrying three implicit hydrogens.
        // ----------------------------------------------------------------
        if atom.atomic_number == 7 {
            for &(nbr, order) in graph.neighbors(idx) {
                if (order == BondOrder::Single || order == BondOrder::Aromatic)
                    && graph
                        .molecule
                        .atoms
                        .get(nbr)
                        .is_some_and(|a| a.atomic_number == 6 && a.implicit_h >= 3)
                {
                    metabolites.push(Metabolite {
                        transformation: Transformation::NDealkylation { site: nbr },
                        site_description: format!("N-methyl at C{nbr}"),
                        probability: 0.6,
                        reactive_intermediate: false,
                        enzyme: Some("CYP3A4".to_string()),
                    });
                }
            }
        }

        // ----------------------------------------------------------------
        // O-dealkylation
        // CYP2D6 cleaves O-methyl ethers. Detected as an oxygen (single-
        // bonded) to a carbon carrying three implicit hydrogens.
        // ----------------------------------------------------------------
        if atom.atomic_number == 8 {
            for &(nbr, order) in graph.neighbors(idx) {
                if order == BondOrder::Single
                    && graph
                        .molecule
                        .atoms
                        .get(nbr)
                        .is_some_and(|a| a.atomic_number == 6 && a.implicit_h >= 3)
                {
                    metabolites.push(Metabolite {
                        transformation: Transformation::ODealkylation { site: nbr },
                        site_description: format!("O-methyl at C{nbr}"),
                        probability: 0.5,
                        reactive_intermediate: false,
                        enzyme: Some("CYP2D6".to_string()),
                    });
                }
            }
        }
    }

    // ----------------------------------------------------------------
    // Aromatic epoxidation
    // CYP1A2 can epoxidise aromatic bonds, producing reactive arene
    // oxides (genotoxic intermediates).  Flag only the first qualifying
    // bond to avoid duplicate entries per ring system.
    // ----------------------------------------------------------------
    for bond in &graph.molecule.bonds {
        let a = match graph.molecule.atoms.get(bond.atom1) {
            Some(a) => a,
            None => continue,
        };
        let b = match graph.molecule.atoms.get(bond.atom2) {
            Some(b) => b,
            None => continue,
        };

        if a.aromatic
            && b.aromatic
            && a.atomic_number == 6
            && b.atomic_number == 6
            && bond.atom1 < bond.atom2
        {
            metabolites.push(Metabolite {
                transformation: Transformation::Epoxidation {
                    site1: bond.atom1,
                    site2: bond.atom2,
                },
                site_description: format!("Aromatic bond C{}-C{}", bond.atom1, bond.atom2),
                probability: 0.2,
                reactive_intermediate: true,
                enzyme: Some("CYP1A2".to_string()),
            });
            // One epoxidation site per molecule is sufficient for the
            // first-pass rule engine.
            break;
        }
    }

    metabolites
}
