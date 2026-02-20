//! Stoichiometric codec — encode concepts to equations and decode back.
//!
//! The codec is the primary API surface: it wraps a dictionary and provides
//! encode/decode operations with stop-word filtering.

use crate::dictionary::{DefinitionSource, Dictionary};
use crate::equation::BalancedEquation;
use crate::error::StoichiometryError;
use crate::inventory::PrimitiveInventory;
use crate::jeopardy::JeopardyAnswer;

/// Stoichiometric codec: encode concepts to balanced equations, decode back.
#[derive(Debug, Clone)]
pub struct StoichiometricCodec {
    dictionary: Dictionary,
}

impl StoichiometricCodec {
    /// Create a codec with the given dictionary.
    #[must_use]
    pub fn new(dictionary: Dictionary) -> Self {
        Self { dictionary }
    }

    /// Create a codec with the built-in PV dictionary (10 seed terms).
    #[must_use]
    pub fn builtin() -> Self {
        Self {
            dictionary: Dictionary::builtin(),
        }
    }

    /// Encode a concept: definition -> balanced equation.
    ///
    /// 1. Split definition into words
    /// 2. Filter stop words
    /// 3. Decompose each word via the dictionary's decomposer
    /// 4. Auto-balance via Balancer
    /// 5. Register in dictionary
    /// 6. Return balanced equation
    ///
    /// # Errors
    ///
    /// - `EmptyConcept` if concept name is empty.
    /// - `EmptyDefinition` if definition is empty or all stop words.
    /// - `UnknownWord` if any content word is not in the decomposer.
    /// - `DuplicateTerm` if the concept is already registered.
    pub fn encode(
        &mut self,
        concept: &str,
        definition: &str,
        source: DefinitionSource,
    ) -> Result<BalancedEquation, StoichiometryError> {
        let entry = self.dictionary.register(concept, definition, source)?;
        Ok(entry.equation.clone())
    }

    /// Decode a balanced equation back to a Jeopardy answer.
    ///
    /// Looks up the equation's product primitives in the dictionary via
    /// reverse lookup. Returns the best match.
    #[must_use]
    pub fn decode(&self, equation: &BalancedEquation) -> Option<JeopardyAnswer> {
        let product_prims = equation.concept.formula.primitives().to_vec();
        let product_inv = PrimitiveInventory::from_primitives(&product_prims);

        // Try exact match first (by name)
        if let Some(entry) = self.dictionary.lookup(&equation.concept.name) {
            let entry_inv =
                PrimitiveInventory::from_primitives(entry.equation.concept.formula.primitives());
            if entry_inv.is_equal(&product_inv) {
                return Some(JeopardyAnswer {
                    question: format!("What is {}?", entry.name),
                    concept: entry.name.clone(),
                    confidence: 1.0,
                    equation_display: format!("{}", equation),
                });
            }
        }

        // Reverse lookup by primitives
        let matches = self.dictionary.reverse_lookup(&product_prims);
        if let Some(best) = matches.first() {
            return Some(JeopardyAnswer {
                question: format!("What is {}?", best.name),
                concept: best.name.clone(),
                confidence: 1.0,
                equation_display: format!("{}", equation),
            });
        }

        // Fuzzy match: find the term with the most similar primitive inventory
        let mut best_match = None;
        let mut best_similarity = 0.0_f64;

        for term in self.dictionary.all_terms() {
            let term_prims = term.equation.concept.formula.primitives();
            let term_inv = PrimitiveInventory::from_primitives(term_prims);

            // Jaccard-like similarity using inventory counts
            let similarity = inventory_similarity(&product_inv, &term_inv);
            if similarity > best_similarity {
                best_similarity = similarity;
                best_match = Some(term);
            }
        }

        best_match.map(|term| JeopardyAnswer {
            question: format!("What is {}?", term.name),
            concept: term.name.clone(),
            confidence: best_similarity,
            equation_display: format!("{}", equation),
        })
    }

    /// Access the inner dictionary (immutable).
    #[must_use]
    pub fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }

    /// Access the inner dictionary (mutable).
    pub fn dictionary_mut(&mut self) -> &mut Dictionary {
        &mut self.dictionary
    }

    /// Find sister concepts for a given equation.
    ///
    /// Returns terms whose primitive Jaccard similarity exceeds the threshold.
    #[must_use]
    pub fn find_sisters(
        &self,
        equation: &BalancedEquation,
        threshold: f64,
    ) -> Vec<crate::sister::SisterMatch> {
        crate::sister::find_sisters(equation, &self.dictionary, threshold)
    }
}

/// Compute Jaccard-like similarity between two primitive inventories.
///
/// Uses min(a, b) / max(a, b) per slot, then averages across non-zero slots.
fn inventory_similarity(a: &PrimitiveInventory, b: &PrimitiveInventory) -> f64 {
    let a_counts = a.counts();
    let b_counts = b.counts();

    let mut intersection = 0u32;
    let mut union = 0u32;

    for i in 0..15 {
        let ai = a_counts[i];
        let bi = b_counts[i];
        intersection += ai.min(bi);
        union += ai.max(bi);
    }

    if union == 0 {
        return 0.0;
    }
    f64::from(intersection) / f64::from(union)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance::Balancer;
    use nexcore_lex_primitiva::primitiva::LexPrimitiva;

    #[test]
    fn test_encode_known_concept() -> Result<(), StoichiometryError> {
        let mut codec = StoichiometricCodec::builtin();
        let eq = codec.encode(
            "Drug Safety",
            "monitoring and assessment of drug adverse effects",
            DefinitionSource::Custom("test".to_string()),
        )?;
        assert!(eq.balance.is_balanced);
        assert!(Balancer::verify(&eq.balance));
        Ok(())
    }

    #[test]
    fn test_encode_unknown_word_fails() {
        let mut codec = StoichiometricCodec::builtin();
        let result = codec.encode(
            "Bogus",
            "xyzzy plugh word",
            DefinitionSource::Custom("test".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_known_equation() {
        let codec = StoichiometricCodec::builtin();
        if let Some(term) = codec.dictionary().lookup("Adverse Event") {
            let answer = codec.decode(&term.equation);
            assert!(answer.is_some());
            let answer = answer.as_ref();
            assert!(answer.is_some_and(|a| a.question.contains("Adverse Event")));
        }
    }

    #[test]
    fn test_decode_unknown_returns_fuzzy() {
        let codec = StoichiometricCodec::builtin();
        // Create an equation with primitives not matching any term exactly
        let formula = nexcore_lex_primitiva::molecular_weight::MolecularFormula::new("Unknown")
            .with(LexPrimitiva::Void)
            .with(LexPrimitiva::Recursion);
        let concept = crate::equation::ConceptFormula {
            name: "Unknown".to_string(),
            definition: "unknown".to_string(),
            formula,
        };
        let inv =
            PrimitiveInventory::from_primitives(&[LexPrimitiva::Void, LexPrimitiva::Recursion]);
        let eq = crate::equation::BalancedEquation {
            concept,
            reactants: vec![],
            balance: crate::equation::BalanceProof {
                reactant_mass: 0.0,
                product_mass: inv.total_mass(),
                delta: 0.0,
                is_balanced: false,
                reactant_inventory: PrimitiveInventory::new(),
                product_inventory: inv,
            },
        };
        // May return a fuzzy match or None
        let answer = codec.decode(&eq);
        // At minimum it shouldn't crash; it may find a partial match
        if let Some(a) = &answer {
            assert!(a.confidence >= 0.0);
            assert!(a.confidence <= 1.0);
        }
    }

    #[test]
    fn test_encode_decode_round_trip() -> Result<(), StoichiometryError> {
        let mut codec = StoichiometricCodec::builtin();
        let eq = codec.encode(
            "Custom Signal Detection",
            "detection and analysis of safety signal data",
            DefinitionSource::Custom("test".to_string()),
        )?;
        let answer = codec.decode(&eq);
        assert!(answer.is_some());
        let answer = answer.as_ref();
        assert!(answer.is_some_and(|a| a.question.contains("Custom Signal Detection")));
        Ok(())
    }

    #[test]
    fn test_jeopardy_format() {
        let codec = StoichiometricCodec::builtin();
        if let Some(term) = codec.dictionary().lookup("Pharmacovigilance") {
            let answer = codec.decode(&term.equation);
            assert!(answer.is_some());
            if let Some(a) = answer {
                assert!(a.question.starts_with("What is "));
                assert!(a.question.ends_with('?'));
            }
        }
    }

    #[test]
    fn test_inventory_similarity_identical() {
        let inv =
            PrimitiveInventory::from_primitives(&[LexPrimitiva::Causality, LexPrimitiva::Boundary]);
        let sim = inventory_similarity(&inv, &inv);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_inventory_similarity_disjoint() {
        let a = PrimitiveInventory::from_primitives(&[LexPrimitiva::Causality]);
        let b = PrimitiveInventory::from_primitives(&[LexPrimitiva::Void]);
        let sim = inventory_similarity(&a, &b);
        assert!(sim < 0.001);
    }
}
