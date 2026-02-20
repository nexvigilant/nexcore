// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Degradation product prediction under stress conditions.
//!
//! Rules applied:
//!
//! | Structural feature | Transformation | Condition |
//! |---|---|---|
//! | Ester bond (C(=O)–O–C) | Hydrolysis | Aqueous / acid / base |
//! | Amide bond (C(=O)–N) | Hydrolysis | Aqueous (slow) |

use nexcore_molcore::graph::MolGraph;
use prima_chem::types::BondOrder;

use crate::types::{Metabolite, Transformation};

/// Predict degradation products under stress conditions for `graph`.
///
/// Identifies labile bonds susceptible to hydrolysis, oxidation, or
/// photodegradation under standard ICH Q1B/Q1A stress testing protocols.
/// The engine currently covers ester hydrolysis and amide hydrolysis.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
/// use nexcore_metabolite::degradant::predict_degradants;
/// use nexcore_metabolite::types::Transformation;
///
/// // Aspirin: CC(=O)Oc1ccccc1C(=O)O
/// let mol = parse("CC(=O)Oc1ccccc1C(=O)O").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let degradants = predict_degradants(&g);
/// assert!(degradants.iter().any(|d| matches!(d.transformation, Transformation::Hydrolysis { .. })));
/// ```
#[must_use]
pub fn predict_degradants(graph: &MolGraph) -> Vec<Metabolite> {
    let mut degradants = Vec::new();

    for idx in 0..graph.atom_count() {
        let atom = match graph.molecule.atoms.get(idx) {
            Some(a) => a,
            None => continue,
        };

        // ----------------------------------------------------------------
        // Ester hydrolysis
        // Pattern: O with zero implicit H bridging two carbons, where at
        // least one of those carbons also carries a C=O (carbonyl).
        // ----------------------------------------------------------------
        if atom.atomic_number == 8 && atom.implicit_h == 0 {
            let neighbors = graph.neighbors(idx);
            if neighbors.len() == 2 {
                let both_carbon = neighbors.iter().all(|&(n, _)| {
                    graph
                        .molecule
                        .atoms
                        .get(n)
                        .is_some_and(|a| a.atomic_number == 6)
                });

                if both_carbon {
                    let has_carbonyl = neighbors.iter().any(|&(n, _)| {
                        graph.neighbors(n).iter().any(|&(nn, order)| {
                            order == BondOrder::Double
                                && graph
                                    .molecule
                                    .atoms
                                    .get(nn)
                                    .is_some_and(|a| a.atomic_number == 8)
                        })
                    });

                    if has_carbonyl {
                        degradants.push(Metabolite {
                            transformation: Transformation::Hydrolysis { site: idx },
                            site_description: format!("Ester bond at O{idx}"),
                            probability: 0.6,
                            reactive_intermediate: false,
                            enzyme: None,
                        });
                    }
                }
            }
        }

        // ----------------------------------------------------------------
        // Amide hydrolysis
        // Pattern: non-aromatic N single-bonded to a carbonyl carbon.
        // Amide hydrolysis is much slower than ester hydrolysis under
        // physiological conditions, hence lower probability (0.3).
        // ----------------------------------------------------------------
        if atom.atomic_number == 7 && !atom.aromatic {
            for &(nbr, order) in graph.neighbors(idx) {
                if order != BondOrder::Single {
                    continue;
                }
                let nbr_is_carbonyl_c = graph
                    .molecule
                    .atoms
                    .get(nbr)
                    .is_some_and(|a| a.atomic_number == 6)
                    && graph.neighbors(nbr).iter().any(|&(nn, o)| {
                        o == BondOrder::Double
                            && graph
                                .molecule
                                .atoms
                                .get(nn)
                                .is_some_and(|a| a.atomic_number == 8)
                    });

                if nbr_is_carbonyl_c {
                    degradants.push(Metabolite {
                        transformation: Transformation::Hydrolysis { site: idx },
                        site_description: format!("Amide bond at N{idx}-C{nbr}"),
                        probability: 0.3,
                        reactive_intermediate: false,
                        enzyme: None,
                    });
                }
            }
        }
    }

    degradants
}
