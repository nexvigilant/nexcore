// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Primary prediction API.
//!
//! The two public entry points are:
//! - [`predict_metabolites`] — takes an already-parsed [`MolGraph`].
//! - [`predict_from_smiles`] — parses a SMILES string then calls
//!   [`predict_metabolites`].

use nexcore_molcore::graph::MolGraph;
use nexcore_molcore::smiles::parse;

use crate::degradant::predict_degradants;
use crate::error::{MetaboliteError, MetaboliteResult};
use crate::phase1::predict_phase1;
use crate::phase2::predict_phase2;
use crate::types::MetaboliteTree;

/// Predict the full metabolite tree for a pre-parsed molecular graph.
///
/// This is the low-level entry point.  Use [`predict_from_smiles`] when
/// starting from a SMILES string.
///
/// The returned [`MetaboliteTree`] partitions predictions into:
/// - `phase1` — all CYP-mediated Phase I transformations
/// - `phase2` — all conjugation Phase II transformations
/// - `reactive_intermediates` — Phase I entries flagged as reactive
/// - `degradants` — stress-condition degradation products
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
/// use nexcore_metabolite::predict::predict_metabolites;
///
/// let mol = parse("CCO").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let tree = predict_metabolites(&g, "CCO");
/// assert_eq!(tree.parent_smiles, "CCO");
/// ```
#[must_use]
pub fn predict_metabolites(graph: &MolGraph, parent_smiles: &str) -> MetaboliteTree {
    let phase1 = predict_phase1(graph);
    let phase2 = predict_phase2(graph);
    let degradants = predict_degradants(graph);

    // Reactive intermediates are the subset of Phase I metabolites where
    // `reactive_intermediate` is set — e.g. arene oxides.
    let reactive_intermediates = phase1
        .iter()
        .filter(|m| m.reactive_intermediate)
        .cloned()
        .collect();

    MetaboliteTree {
        parent_smiles: parent_smiles.to_string(),
        phase1,
        phase2,
        reactive_intermediates,
        degradants,
    }
}

/// Parse `smiles` and predict the full metabolite tree.
///
/// # Errors
///
/// Returns [`MetaboliteError::SmilesParse`] if `smiles` is not valid
/// OpenSMILES syntax.
///
/// # Examples
///
/// ```rust
/// use nexcore_metabolite::predict_from_smiles;
///
/// let tree = predict_from_smiles("CCO").unwrap_or_default();
/// assert_eq!(tree.parent_smiles, "CCO");
///
/// let err = predict_from_smiles("INVALID$$");
/// assert!(err.is_err());
/// ```
pub fn predict_from_smiles(smiles: &str) -> MetaboliteResult<MetaboliteTree> {
    let mol = parse(smiles).map_err(|e| MetaboliteError::SmilesParse(e.to_string()))?;
    let graph = MolGraph::from_molecule(mol);
    Ok(predict_metabolites(&graph, smiles))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Transformation;

    #[test]
    fn test_ethanol_metabolites() {
        let tree = predict_from_smiles("CCO").unwrap_or_default();
        // Ethanol has an OH group → must have Phase II metabolites (glucuronidation).
        assert!(
            !tree.phase2.is_empty(),
            "ethanol should have Phase II metabolites"
        );
    }

    #[test]
    fn test_benzene_hydroxylation() {
        let tree = predict_from_smiles("c1ccccc1").unwrap_or_default();
        // Benzene has aromatic C with H → Phase I hydroxylation.
        assert!(
            !tree.phase1.is_empty(),
            "benzene should have Phase I metabolites"
        );
        let has_hydroxylation = tree
            .phase1
            .iter()
            .any(|m| matches!(m.transformation, Transformation::Hydroxylation { .. }));
        assert!(has_hydroxylation, "benzene should predict hydroxylation");
    }

    #[test]
    fn test_aspirin_ester_hydrolysis() {
        let tree = predict_from_smiles("CC(=O)Oc1ccccc1C(=O)O").unwrap_or_default();
        // Aspirin has an ester bond → should predict hydrolysis as a degradant.
        assert!(
            !tree.degradants.is_empty(),
            "aspirin should have degradation products"
        );
        let has_hydrolysis = tree
            .degradants
            .iter()
            .any(|m| matches!(m.transformation, Transformation::Hydrolysis { .. }));
        assert!(has_hydrolysis, "aspirin should predict ester hydrolysis");
    }

    #[test]
    fn test_caffeine_n_dealkylation() {
        // Caffeine has three N-methyl groups → N-dealkylation expected.
        let tree = predict_from_smiles("Cn1cnc2c1c(=O)n(c(=O)n2C)C").unwrap_or_default();
        let has_ndealk = tree
            .phase1
            .iter()
            .any(|m| matches!(m.transformation, Transformation::NDealkylation { .. }));
        assert!(has_ndealk, "caffeine should predict N-dealkylation");
    }

    #[test]
    fn test_reactive_intermediates_from_phase1() {
        let tree = predict_from_smiles("c1ccccc1").unwrap_or_default();
        // Benzene epoxidation produces a reactive arene oxide intermediate.
        let has_epoxide = tree.phase1.iter().any(|m| m.reactive_intermediate);
        assert!(
            has_epoxide,
            "benzene should have a reactive epoxide intermediate"
        );
        assert!(
            !tree.reactive_intermediates.is_empty(),
            "reactive_intermediates should be populated from phase1"
        );
    }

    #[test]
    fn test_methane_no_phase2_or_degradants() {
        let tree = predict_from_smiles("C").unwrap_or_default();
        // Methane: single aliphatic carbon, no heteroatoms, no labile bonds.
        assert!(tree.phase2.is_empty(), "methane has no Phase II sites");
        assert!(tree.degradants.is_empty(), "methane has no degradant sites");
    }

    #[test]
    fn test_invalid_smiles_returns_error() {
        let result = predict_from_smiles("INVALID$$");
        assert!(result.is_err(), "invalid SMILES must return Err");
    }

    #[test]
    fn test_parent_smiles_stored() {
        let tree = predict_from_smiles("CCO").unwrap_or_default();
        assert_eq!(tree.parent_smiles, "CCO");
    }

    #[test]
    fn test_probabilities_bounded() {
        let tree = predict_from_smiles("CC(=O)Oc1ccccc1C(=O)O").unwrap_or_default();
        let all = tree
            .phase1
            .iter()
            .chain(tree.phase2.iter())
            .chain(tree.degradants.iter());
        for m in all {
            assert!(
                (0.0..=1.0).contains(&m.probability),
                "probability out of [0,1]: {}",
                m.probability
            );
        }
    }

    #[test]
    fn test_reactive_intermediates_subset_of_phase1() {
        // Every reactive intermediate must also appear in phase1.
        let tree = predict_from_smiles("c1ccccc1").unwrap_or_default();
        for ri in &tree.reactive_intermediates {
            assert!(
                tree.phase1
                    .iter()
                    .any(|m| m.reactive_intermediate && m.transformation == ri.transformation),
                "reactive intermediate not found in phase1: {:?}",
                ri.transformation
            );
        }
    }

    #[test]
    fn test_codeine_o_dealkylation() {
        // Codeine SMILES — has O-methyl group → O-dealkylation to morphine.
        let tree =
            predict_from_smiles("COc1ccc2c(c1)C[C@@H]3N(CC[C@]24CCO[C@H]3O4)C").unwrap_or_default();
        let has_odealk = tree
            .phase1
            .iter()
            .any(|m| matches!(m.transformation, Transformation::ODealkylation { .. }));
        assert!(has_odealk, "codeine should predict O-dealkylation");
    }
}
