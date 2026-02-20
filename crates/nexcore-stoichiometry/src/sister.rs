//! Sister detection — find concepts with similar primitive compositions.
//!
//! Sister concepts share significant primitive overlap (measured by Jaccard
//! similarity). Isomers have the same primitive *set* but different dominant
//! primitive (the one with highest count).

use crate::dictionary::Dictionary;
use crate::equation::BalancedEquation;
use crate::inventory::PrimitiveInventory;
use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A sister match result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SisterMatch {
    /// Name of the matched term.
    pub name: String,
    /// Jaccard similarity coefficient (0.0 - 1.0).
    pub similarity: f64,
    /// Primitives shared between the two concepts.
    pub shared_primitives: Vec<LexPrimitiva>,
    /// Primitives unique to the query concept.
    pub unique_to_self: Vec<LexPrimitiva>,
    /// Primitives unique to the matched concept.
    pub unique_to_other: Vec<LexPrimitiva>,
    /// Whether this is an isomer (same set, different dominant).
    pub is_isomer: bool,
}

/// Compute Jaccard similarity between two primitive sets.
///
/// J(A, B) = |A ∩ B| / |A ∪ B|
///
/// Uses the *set* of unique primitives (not counts).
#[must_use]
pub fn jaccard_similarity(a: &[LexPrimitiva], b: &[LexPrimitiva]) -> f64 {
    let set_a: HashSet<LexPrimitiva> = a.iter().copied().collect();
    let set_b: HashSet<LexPrimitiva> = b.iter().copied().collect();

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f64 / union as f64
}

/// Find sister concepts in the dictionary that match above the given threshold.
///
/// Excludes the query concept itself (matched by name).
#[must_use]
pub fn find_sisters(
    equation: &BalancedEquation,
    dictionary: &Dictionary,
    threshold: f64,
) -> Vec<SisterMatch> {
    let query_prims = equation.concept.formula.primitives();
    let query_set: HashSet<LexPrimitiva> = query_prims.iter().copied().collect();
    let query_name = &equation.concept.name;

    let mut sisters = Vec::new();

    for term in dictionary.all_terms() {
        // Skip self
        if term.name.eq_ignore_ascii_case(query_name) {
            continue;
        }

        let term_prims = term.equation.concept.formula.primitives();
        let similarity = jaccard_similarity(query_prims, term_prims);

        if similarity >= threshold {
            let term_set: HashSet<LexPrimitiva> = term_prims.iter().copied().collect();

            let shared: Vec<LexPrimitiva> = query_set.intersection(&term_set).copied().collect();
            let unique_self: Vec<LexPrimitiva> = query_set.difference(&term_set).copied().collect();
            let unique_other: Vec<LexPrimitiva> =
                term_set.difference(&query_set).copied().collect();

            let is_isomer_match = is_isomer(equation, &term.equation);

            sisters.push(SisterMatch {
                name: term.name.clone(),
                similarity,
                shared_primitives: shared,
                unique_to_self: unique_self,
                unique_to_other: unique_other,
                is_isomer: is_isomer_match,
            });
        }
    }

    // Sort by similarity descending
    sisters.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    sisters
}

/// Check if two equations are isomers: same primitive *set* but different dominant.
///
/// The dominant primitive is the one with the highest count in the inventory.
#[must_use]
pub fn is_isomer(a: &BalancedEquation, b: &BalancedEquation) -> bool {
    let a_prims = a.concept.formula.primitives();
    let b_prims = b.concept.formula.primitives();

    let set_a: HashSet<LexPrimitiva> = a_prims.iter().copied().collect();
    let set_b: HashSet<LexPrimitiva> = b_prims.iter().copied().collect();

    // Same set of unique primitives
    if set_a != set_b {
        return false;
    }

    // Different dominant (highest-count primitive)
    let dom_a = dominant_primitive(a_prims);
    let dom_b = dominant_primitive(b_prims);

    dom_a != dom_b
}

/// Find the dominant primitive (highest count) in a primitive list.
fn dominant_primitive(prims: &[LexPrimitiva]) -> Option<LexPrimitiva> {
    let inv = PrimitiveInventory::from_primitives(prims);
    let ops = PrimitiveInventory::operational_primitives();

    let mut best = None;
    let mut best_count = 0u32;

    for &p in &ops {
        let c = inv.count(p);
        if c > best_count {
            best_count = c;
            best = Some(p);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance::Balancer;
    use crate::codec::StoichiometricCodec;
    use crate::equation::{BalanceProof, ConceptFormula, ReactantFormula};
    use nexcore_lex_primitiva::molecular_weight::MolecularFormula;

    fn make_equation(name: &str, prims: &[LexPrimitiva]) -> BalancedEquation {
        let formula = MolecularFormula::new(name).with_all(prims);
        let inv = PrimitiveInventory::from_primitives(prims);
        BalancedEquation {
            concept: ConceptFormula {
                name: name.to_string(),
                definition: "test".to_string(),
                formula,
            },
            reactants: vec![],
            balance: BalanceProof {
                reactant_mass: inv.total_mass(),
                product_mass: inv.total_mass(),
                delta: 0.0,
                is_balanced: true,
                reactant_inventory: inv.clone(),
                product_inventory: inv,
            },
        }
    }

    #[test]
    fn test_jaccard_identical() {
        let a = vec![LexPrimitiva::Causality, LexPrimitiva::Boundary];
        let b = vec![LexPrimitiva::Causality, LexPrimitiva::Boundary];
        let sim = jaccard_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_jaccard_partial_overlap() {
        let a = vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
        ];
        let b = vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
            LexPrimitiva::Void,
        ];
        // intersection = {Causality, Boundary} = 2, union = {Causality, Boundary, State, Void} = 4
        let sim = jaccard_similarity(&a, &b);
        assert!((sim - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_jaccard_no_overlap() {
        let a = vec![LexPrimitiva::Causality];
        let b = vec![LexPrimitiva::Void];
        let sim = jaccard_similarity(&a, &b);
        assert!(sim < 0.001);
    }

    #[test]
    fn test_exact_sister_in_dictionary() {
        let codec = StoichiometricCodec::builtin();
        // Adverse Event and SAE share many primitives
        if let Some(ae) = codec.dictionary().lookup("Adverse Event") {
            let sisters = find_sisters(&ae.equation, codec.dictionary(), 0.3);
            assert!(!sisters.is_empty(), "Adverse Event should have sisters");
        }
    }

    #[test]
    fn test_no_sister_above_threshold() {
        let codec = StoichiometricCodec::builtin();
        // Create an equation with very unusual primitives
        let eq = make_equation("Unusual", &[LexPrimitiva::Void, LexPrimitiva::Recursion]);
        let sisters = find_sisters(&eq, codec.dictionary(), 0.99);
        // At threshold 0.99, unlikely to match anything
        assert!(sisters.is_empty());
    }

    #[test]
    fn test_isomer_detection() {
        // Same set {Causality, Boundary} but different dominant
        let a = make_equation(
            "A",
            &[
                LexPrimitiva::Causality,
                LexPrimitiva::Causality,
                LexPrimitiva::Boundary,
            ],
        );
        let b = make_equation(
            "B",
            &[
                LexPrimitiva::Causality,
                LexPrimitiva::Boundary,
                LexPrimitiva::Boundary,
            ],
        );
        assert!(is_isomer(&a, &b));
    }

    #[test]
    fn test_not_isomer_different_sets() {
        let a = make_equation("A", &[LexPrimitiva::Causality]);
        let b = make_equation("B", &[LexPrimitiva::Void]);
        assert!(!is_isomer(&a, &b));
    }

    #[test]
    fn test_sister_match_excludes_self() {
        let codec = StoichiometricCodec::builtin();
        if let Some(pv) = codec.dictionary().lookup("Pharmacovigilance") {
            let sisters = find_sisters(&pv.equation, codec.dictionary(), 0.0);
            // Should not include Pharmacovigilance itself
            assert!(sisters.iter().all(|s| s.name != "Pharmacovigilance"));
        }
    }
}
