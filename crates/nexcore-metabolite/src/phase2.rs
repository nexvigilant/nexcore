// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Phase II metabolite prediction (conjugation reactions).
//!
//! Rules applied:
//!
//! | Structural feature | Transformation | Enzyme |
//! |---|---|---|
//! | OH (O with implicit H) | Glucuronidation | UGT |
//! | OH (O with implicit H) | Sulfation | SULT |
//! | NH/NH₂ (non-aromatic N with implicit H) | Acetylation | NAT |
//! | SH (S with implicit H) | Glutathione conjugation | GST |

use nexcore_molcore::graph::MolGraph;

use crate::types::{Metabolite, Transformation};

/// Predict Phase II conjugation metabolites for `graph`.
///
/// Identifies heteroatoms available for conjugation — hydroxyl groups,
/// primary/secondary amines, and free thiols — and returns the expected
/// Phase II products with rule-based probability estimates.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
/// use nexcore_metabolite::phase2::predict_phase2;
/// use nexcore_metabolite::types::Transformation;
///
/// let mol = parse("CCO").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let metabolites = predict_phase2(&g);
/// assert!(metabolites.iter().any(|m| matches!(m.transformation, Transformation::Glucuronidation { .. })));
/// ```
#[must_use]
pub fn predict_phase2(graph: &MolGraph) -> Vec<Metabolite> {
    let mut metabolites = Vec::new();

    for idx in 0..graph.atom_count() {
        let atom = match graph.molecule.atoms.get(idx) {
            Some(a) => a,
            None => continue,
        };

        // ----------------------------------------------------------------
        // Hydroxyl groups (O with at least one implicit H)
        // Both UGT (glucuronidation) and SULT (sulfation) compete for the
        // same hydroxyl acceptor site.
        // ----------------------------------------------------------------
        if atom.atomic_number == 8 && atom.implicit_h > 0 {
            metabolites.push(Metabolite {
                transformation: Transformation::Glucuronidation { site: idx },
                site_description: format!("Hydroxyl O at position {idx}"),
                probability: 0.5,
                reactive_intermediate: false,
                enzyme: Some("UGT".to_string()),
            });
            metabolites.push(Metabolite {
                transformation: Transformation::Sulfation { site: idx },
                site_description: format!("Hydroxyl O at position {idx}"),
                probability: 0.3,
                reactive_intermediate: false,
                enzyme: Some("SULT".to_string()),
            });
        }

        // ----------------------------------------------------------------
        // Amine groups (non-aromatic N with at least one implicit H)
        // NAT acetylates primary and secondary aliphatic amines.
        // Aromatic amines are excluded here; they are addressed by Phase I
        // hydroxylation first.
        // ----------------------------------------------------------------
        if atom.atomic_number == 7 && atom.implicit_h > 0 && !atom.aromatic {
            metabolites.push(Metabolite {
                transformation: Transformation::Acetylation { site: idx },
                site_description: format!("Amine N at position {idx}"),
                probability: 0.4,
                reactive_intermediate: false,
                enzyme: Some("NAT".to_string()),
            });
        }

        // ----------------------------------------------------------------
        // Thiol groups (S with at least one implicit H)
        // GST conjugates free thiols with glutathione.
        // ----------------------------------------------------------------
        if atom.atomic_number == 16 && atom.implicit_h > 0 {
            metabolites.push(Metabolite {
                transformation: Transformation::GlutathioneConjugation { site: idx },
                site_description: format!("Thiol S at position {idx}"),
                probability: 0.6,
                reactive_intermediate: false,
                enzyme: Some("GST".to_string()),
            });
        }
    }

    metabolites
}
