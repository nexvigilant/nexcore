//! Balancer — conservation law engine for primitive equations.
//!
//! Verifies that the sum of reactant primitives equals the product primitives,
//! and provides auto-balancing by unioning all reactant primitives into the product.

use crate::equation::{BalanceProof, BalancedEquation, ConceptFormula, ReactantFormula};
use crate::error::StoichiometryError;
use crate::inventory::PrimitiveInventory;
use nexcore_lex_primitiva::molecular_weight::MolecularFormula;

/// Tolerance for floating-point mass comparison (daltons).
const MASS_TOLERANCE: f64 = 0.001;

/// Stoichiometric balancer — enforces primitive conservation.
pub struct Balancer;

impl Balancer {
    /// Check if an equation's reactant and product primitive counts match.
    #[must_use]
    pub fn is_balanced(eq: &BalancedEquation) -> bool {
        let reactant_inv = Self::reactant_inventory(&eq.reactants);
        let product_inv = PrimitiveInventory::from_primitives(eq.concept.formula.primitives());
        reactant_inv.is_equal(&product_inv)
    }

    /// Compute per-primitive deficit: reactant_count - product_count.
    /// Positive = excess reactants. Negative = missing reactants.
    #[must_use]
    pub fn deficit(eq: &BalancedEquation) -> [i32; 15] {
        let reactant_inv = Self::reactant_inventory(&eq.reactants);
        let product_inv = PrimitiveInventory::from_primitives(eq.concept.formula.primitives());
        reactant_inv.deficit(&product_inv)
    }

    /// Create a `BalanceProof` from reactants and product.
    #[must_use]
    pub fn prove(reactants: &[ReactantFormula], concept: &ConceptFormula) -> BalanceProof {
        let reactant_inv = Self::reactant_inventory(reactants);
        let product_inv = PrimitiveInventory::from_primitives(concept.formula.primitives());
        let reactant_mass = reactant_inv.total_mass();
        let product_mass = product_inv.total_mass();
        let delta = reactant_mass - product_mass;
        let is_balanced = delta.abs() < MASS_TOLERANCE && reactant_inv.is_equal(&product_inv);

        BalanceProof {
            reactant_mass,
            product_mass,
            delta,
            is_balanced,
            reactant_inventory: reactant_inv,
            product_inventory: product_inv,
        }
    }

    /// Verify an existing proof's arithmetic is correct.
    #[must_use]
    pub fn verify(proof: &BalanceProof) -> bool {
        let expected_delta = proof.reactant_mass - proof.product_mass;
        let delta_matches = (proof.delta - expected_delta).abs() < MASS_TOLERANCE;
        let inventories_match = proof.reactant_inventory.is_equal(&proof.product_inventory);
        let mass_matches = proof.delta.abs() < MASS_TOLERANCE;

        delta_matches && inventories_match && mass_matches && proof.is_balanced
    }

    /// Auto-balance: union all reactant primitives into a product formula.
    /// This is the "forward" balancer — it CREATES the product from reactants.
    ///
    /// # Errors
    ///
    /// Returns `StoichiometryError::EmptyConcept` if the concept name is empty.
    /// Returns `StoichiometryError::EmptyDefinition` if the definition is empty.
    pub fn auto_balance(
        concept_name: &str,
        definition: &str,
        reactants: Vec<ReactantFormula>,
    ) -> Result<BalancedEquation, StoichiometryError> {
        if concept_name.is_empty() {
            return Err(StoichiometryError::EmptyConcept);
        }
        if definition.is_empty() {
            return Err(StoichiometryError::EmptyDefinition);
        }

        // Union all reactant primitives into the product
        let mut all_primitives = Vec::new();
        for r in &reactants {
            all_primitives.extend_from_slice(r.formula.primitives());
        }

        let product_formula = MolecularFormula::new(concept_name).with_all(&all_primitives);

        let concept = ConceptFormula {
            name: concept_name.to_string(),
            definition: definition.to_string(),
            formula: product_formula,
        };

        let balance = Self::prove(&reactants, &concept);

        Ok(BalancedEquation {
            concept,
            reactants,
            balance,
        })
    }

    /// Compute the combined inventory of all reactant formulas.
    #[must_use]
    fn reactant_inventory(reactants: &[ReactantFormula]) -> PrimitiveInventory {
        let mut inv = PrimitiveInventory::new();
        for r in reactants {
            for &p in r.formula.primitives() {
                inv.add(p);
            }
        }
        inv
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::primitiva::LexPrimitiva;

    fn make_formula(name: &str, prims: &[LexPrimitiva]) -> MolecularFormula {
        MolecularFormula::new(name).with_all(prims)
    }

    fn make_reactant(word: &str, prims: &[LexPrimitiva]) -> ReactantFormula {
        ReactantFormula {
            word: word.to_string(),
            formula: make_formula(word, prims),
        }
    }

    #[test]
    fn test_balanced_equation_passes() {
        let r1 = make_reactant("drug", &[LexPrimitiva::Existence, LexPrimitiva::State]);
        let r2 = make_reactant(
            "safety",
            &[LexPrimitiva::Boundary, LexPrimitiva::Comparison],
        );
        let all_prims = vec![
            LexPrimitiva::Existence,
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
        ];
        let concept = ConceptFormula {
            name: "DrugSafety".to_string(),
            definition: "drug safety".to_string(),
            formula: make_formula("DrugSafety", &all_prims),
        };
        let balance = Balancer::prove(&[r1.clone(), r2.clone()], &concept);
        let eq = BalancedEquation {
            concept,
            reactants: vec![r1, r2],
            balance,
        };
        assert!(Balancer::is_balanced(&eq));
    }

    #[test]
    fn test_unbalanced_equation_fails() {
        let r1 = make_reactant("drug", &[LexPrimitiva::Existence, LexPrimitiva::State]);
        // Product has extra primitive not in reactants
        let concept = ConceptFormula {
            name: "Wrong".to_string(),
            definition: "wrong".to_string(),
            formula: make_formula(
                "Wrong",
                &[
                    LexPrimitiva::Existence,
                    LexPrimitiva::State,
                    LexPrimitiva::Causality, // extra — not in reactants
                ],
            ),
        };
        let balance = Balancer::prove(&[r1.clone()], &concept);
        let eq = BalancedEquation {
            concept,
            reactants: vec![r1],
            balance,
        };
        assert!(!Balancer::is_balanced(&eq));
    }

    #[test]
    fn test_deficit_identifies_missing() {
        let r1 = make_reactant("drug", &[LexPrimitiva::Existence, LexPrimitiva::State]);
        let concept = ConceptFormula {
            name: "Extra".to_string(),
            definition: "extra".to_string(),
            formula: make_formula(
                "Extra",
                &[
                    LexPrimitiva::Existence,
                    LexPrimitiva::State,
                    LexPrimitiva::Causality,
                ],
            ),
        };
        let balance = Balancer::prove(&[r1.clone()], &concept);
        let eq = BalancedEquation {
            concept,
            reactants: vec![r1],
            balance,
        };
        let deficit = Balancer::deficit(&eq);
        // Causality index is 9. Reactants have 0, product has 1 => deficit = -1
        assert_eq!(deficit[9], -1);
    }

    #[test]
    fn test_verify_proof_valid() {
        let r1 = make_reactant("a", &[LexPrimitiva::Quantity]);
        let concept = ConceptFormula {
            name: "A".to_string(),
            definition: "a".to_string(),
            formula: make_formula("A", &[LexPrimitiva::Quantity]),
        };
        let proof = Balancer::prove(&[r1], &concept);
        assert!(proof.is_balanced);
        assert!(Balancer::verify(&proof));
    }

    #[test]
    fn test_verify_proof_invalid() {
        // Manually construct an invalid proof
        let inv_a = PrimitiveInventory::from_primitives(&[LexPrimitiva::Causality]);
        let inv_b = PrimitiveInventory::from_primitives(&[LexPrimitiva::Boundary]);
        let proof = BalanceProof {
            reactant_mass: 5.0,
            product_mass: 3.0,
            delta: 2.0,
            is_balanced: false,
            reactant_inventory: inv_a,
            product_inventory: inv_b,
        };
        assert!(!Balancer::verify(&proof));
    }

    #[test]
    fn test_auto_balance_creates_balanced_equation() -> Result<(), StoichiometryError> {
        let r1 = make_reactant("drug", &[LexPrimitiva::Existence, LexPrimitiva::State]);
        let r2 = make_reactant(
            "safety",
            &[LexPrimitiva::Boundary, LexPrimitiva::Comparison],
        );
        let eq = Balancer::auto_balance("DrugSafety", "drug safety", vec![r1, r2])?;
        assert!(eq.balance.is_balanced);
        assert!(Balancer::is_balanced(&eq));
        Ok(())
    }

    #[test]
    fn test_auto_balance_empty_concept_fails() {
        let r1 = make_reactant("a", &[LexPrimitiva::Quantity]);
        let result = Balancer::auto_balance("", "some def", vec![r1]);
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_balance_empty_definition_fails() {
        let r1 = make_reactant("a", &[LexPrimitiva::Quantity]);
        let result = Balancer::auto_balance("Concept", "", vec![r1]);
        assert!(result.is_err());
    }
}
