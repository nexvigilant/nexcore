//! Equation types — the structural units of stoichiometric balancing.

use crate::inventory::PrimitiveInventory;
use nexcore_lex_primitiva::molecular_weight::MolecularFormula;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A single reactant (definition word) with its primitive formula.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactantFormula {
    /// The word from the definition.
    pub word: String,
    /// The molecular formula for this word.
    pub formula: MolecularFormula,
}

/// The product side — a concept with its name, definition, and formula.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptFormula {
    /// Concept name (e.g. "Pharmacovigilance").
    pub name: String,
    /// The definition text.
    pub definition: String,
    /// The molecular formula for the concept (union of reactant primitives).
    pub formula: MolecularFormula,
}

/// Proof that an equation balances — primitive counts match on both sides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceProof {
    /// Total mass of all reactants in daltons (bits).
    pub reactant_mass: f64,
    /// Total mass of the product in daltons (bits).
    pub product_mass: f64,
    /// Difference: reactant_mass - product_mass.
    pub delta: f64,
    /// Whether the equation balances (delta == 0 within tolerance).
    pub is_balanced: bool,
    /// Primitive inventory of the reactant side.
    pub reactant_inventory: PrimitiveInventory,
    /// Primitive inventory of the product side.
    pub product_inventory: PrimitiveInventory,
}

/// A stoichiometrically balanced equation: reactants -> product with proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalancedEquation {
    /// The concept (product side).
    pub concept: ConceptFormula,
    /// The reactants (definition words).
    pub reactants: Vec<ReactantFormula>,
    /// The balance proof.
    pub balance: BalanceProof,
}

impl fmt::Display for BalancedEquation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Chemical notation: "word1"[symbols] + "word2"[symbols] -> "Concept"[symbols]
        let reactant_parts: Vec<String> = self
            .reactants
            .iter()
            .map(|r| format!("\"{}\"[{}]", r.word, r.formula.formula_string()))
            .collect();
        let reactants_str = reactant_parts.join(" + ");
        write!(
            f,
            "{} \u{2192} \"{}\"[{}]",
            reactants_str,
            self.concept.name,
            self.concept.formula.formula_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::primitiva::LexPrimitiva;

    fn make_formula(name: &str, prims: &[LexPrimitiva]) -> MolecularFormula {
        let mut f = MolecularFormula::new(name);
        for &p in prims {
            f = f.with(p);
        }
        f
    }

    #[test]
    fn test_reactant_formula_creation() {
        let formula = make_formula("drug", &[
            LexPrimitiva::Existence,
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
        ]);
        let rf = ReactantFormula {
            word: "drug".to_string(),
            formula,
        };
        assert_eq!(rf.word, "drug");
        assert_eq!(rf.formula.primitives().len(), 3);
    }

    #[test]
    fn test_concept_formula_creation() {
        let formula = make_formula("TestConcept", &[
            LexPrimitiva::Existence,
            LexPrimitiva::Causality,
        ]);
        let cf = ConceptFormula {
            name: "TestConcept".to_string(),
            definition: "a test concept".to_string(),
            formula,
        };
        assert_eq!(cf.name, "TestConcept");
        assert_eq!(cf.formula.primitives().len(), 2);
    }

    #[test]
    fn test_balanced_equation_display() {
        let r1 = ReactantFormula {
            word: "drug".to_string(),
            formula: make_formula("drug", &[
                LexPrimitiva::Existence,
                LexPrimitiva::State,
                LexPrimitiva::Boundary,
            ]),
        };
        let r2 = ReactantFormula {
            word: "safety".to_string(),
            formula: make_formula("safety", &[
                LexPrimitiva::Boundary,
                LexPrimitiva::Comparison,
                LexPrimitiva::Irreversibility,
            ]),
        };

        let all_prims = vec![
            LexPrimitiva::Existence,
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::Irreversibility,
        ];
        let product_inv = PrimitiveInventory::from_primitives(&all_prims);
        let reactant_inv = product_inv.clone();

        let eq = BalancedEquation {
            concept: ConceptFormula {
                name: "PV".to_string(),
                definition: "drug safety".to_string(),
                formula: make_formula("PV", &all_prims),
            },
            reactants: vec![r1, r2],
            balance: BalanceProof {
                reactant_mass: 0.0,
                product_mass: 0.0,
                delta: 0.0,
                is_balanced: true,
                reactant_inventory: reactant_inv,
                product_inventory: product_inv,
            },
        };

        let display = format!("{eq}");
        assert!(display.contains("drug"));
        assert!(display.contains("safety"));
        assert!(display.contains("PV"));
        assert!(display.contains("\u{2192}")); // arrow →
    }
}
