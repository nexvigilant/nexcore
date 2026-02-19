//! Dictionary — registry of domain terms with balanced equations.
//!
//! Each term has a name, authoritative definition, source, and a
//! stoichiometrically balanced equation encoding its primitive composition.

use crate::balance::Balancer;
use crate::decomposer::Decomposer;
use crate::equation::{BalancedEquation, ReactantFormula};
use crate::error::StoichiometryError;
use crate::inventory::PrimitiveInventory;
use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Source of a term's definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefinitionSource {
    /// ICH guideline (e.g. "ICH E2A").
    IchGuideline(String),
    /// CIOMS report.
    CiomsReport(String),
    /// FDA guidance document.
    FdaGuidance(String),
    /// WHO Drug Dictionary.
    WhoDrug(String),
    /// MedDRA terminology.
    MedDRA(String),
    /// Custom / user-defined source.
    Custom(String),
}

impl fmt::Display for DefinitionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IchGuideline(s) => write!(f, "ICH: {s}"),
            Self::CiomsReport(s) => write!(f, "CIOMS: {s}"),
            Self::FdaGuidance(s) => write!(f, "FDA: {s}"),
            Self::WhoDrug(s) => write!(f, "WHO: {s}"),
            Self::MedDRA(s) => write!(f, "MedDRA: {s}"),
            Self::Custom(s) => write!(f, "Custom: {s}"),
        }
    }
}

/// A registered term with its balanced equation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermEntry {
    /// Term name (e.g. "Pharmacovigilance").
    pub name: String,
    /// The authoritative definition.
    pub definition: String,
    /// Source of the definition.
    pub source: DefinitionSource,
    /// The balanced equation encoding this term.
    pub equation: BalancedEquation,
}

/// Stop words filtered during encoding.
const STOP_WORDS: &[&str] = &[
    "the", "a", "an", "of", "and", "or", "in", "to", "for", "is", "are", "was", "were", "that",
    "with", "by", "on", "at", "from", "as", "it", "its", "be", "has", "have", "this", "which",
    "not", "but", "their", "been", "will", "can", "may", "should", "would", "could", "all", "any",
    "each", "between", "about", "into", "during", "after", "before", "through", "such", "other",
    "than", "more", "also", "only",
];

/// Dictionary of domain terms with balanced primitive equations.
#[derive(Debug, Clone)]
pub struct Dictionary {
    entries: HashMap<String, TermEntry>,
    decomposer: Decomposer,
}

impl Dictionary {
    /// Create an empty dictionary with the given decomposer.
    #[must_use]
    pub fn new(decomposer: Decomposer) -> Self {
        Self {
            entries: HashMap::new(),
            decomposer,
        }
    }

    /// Create the built-in dictionary with seed PV terms.
    ///
    /// Uses `Decomposer::with_seed()` and loads all 10 seed terms.
    #[must_use]
    pub fn builtin() -> Self {
        let decomposer = Decomposer::with_seed();
        let mut dict = Self::new(decomposer);
        dict.load_seed_terms();
        dict
    }

    /// Register a new term by encoding its definition.
    ///
    /// # Errors
    ///
    /// - `DuplicateTerm` if the name already exists.
    /// - `EmptyConcept` if name is empty.
    /// - `EmptyDefinition` if definition is empty.
    /// - `UnknownWord` if a definition word is not in the decomposer.
    pub fn register(
        &mut self,
        name: &str,
        definition: &str,
        source: DefinitionSource,
    ) -> Result<&TermEntry, StoichiometryError> {
        let key = name.to_lowercase();
        if self.entries.contains_key(&key) {
            return Err(StoichiometryError::DuplicateTerm {
                name: name.to_string(),
            });
        }
        if name.is_empty() {
            return Err(StoichiometryError::EmptyConcept);
        }
        if definition.is_empty() {
            return Err(StoichiometryError::EmptyDefinition);
        }

        let equation = self.encode_definition(name, definition)?;

        let entry = TermEntry {
            name: name.to_string(),
            definition: definition.to_string(),
            source,
            equation,
        };

        self.entries.insert(key.clone(), entry);
        // Safe: we just inserted it
        Ok(self.entries.get(&key).ok_or_else(|| {
            StoichiometryError::TermNotFound {
                name: name.to_string(),
            }
        })?)
    }

    /// Look up a term by name (case-insensitive).
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&TermEntry> {
        self.entries.get(&name.to_lowercase())
    }

    /// Reverse-lookup: find all terms whose product primitives match the given set.
    ///
    /// Returns terms where the product inventory matches the inventory built from
    /// the given primitives.
    #[must_use]
    pub fn reverse_lookup(&self, primitives: &[LexPrimitiva]) -> Vec<&TermEntry> {
        let target = PrimitiveInventory::from_primitives(primitives);
        self.entries
            .values()
            .filter(|entry| {
                let product_inv = PrimitiveInventory::from_primitives(
                    entry.equation.concept.formula.primitives(),
                );
                product_inv.is_equal(&target)
            })
            .collect()
    }

    /// All registered terms.
    #[must_use]
    pub fn all_terms(&self) -> Vec<&TermEntry> {
        self.entries.values().collect()
    }

    /// Number of registered terms.
    #[must_use]
    pub fn term_count(&self) -> usize {
        self.entries.len()
    }

    /// Access the inner decomposer.
    #[must_use]
    pub fn decomposer(&self) -> &Decomposer {
        &self.decomposer
    }

    /// Mutable access to the decomposer (for registering new words).
    pub fn decomposer_mut(&mut self) -> &mut Decomposer {
        &mut self.decomposer
    }

    /// Check if a word is a stop word.
    #[must_use]
    pub fn is_stop_word(word: &str) -> bool {
        STOP_WORDS.contains(&word.to_lowercase().as_str())
    }

    /// Encode a definition into a balanced equation using the decomposer.
    fn encode_definition(
        &self,
        concept_name: &str,
        definition: &str,
    ) -> Result<BalancedEquation, StoichiometryError> {
        let words = Self::extract_content_words(definition);

        if words.is_empty() {
            return Err(StoichiometryError::EmptyDefinition);
        }

        let mut reactants = Vec::new();
        for word in &words {
            let formula = self.decomposer.decompose(word)?;
            reactants.push(ReactantFormula {
                word: word.clone(),
                formula,
            });
        }

        Balancer::auto_balance(concept_name, definition, reactants)
    }

    /// Extract content words from a definition (lowercase, no stop words, no punctuation).
    fn extract_content_words(definition: &str) -> Vec<String> {
        definition
            .split_whitespace()
            .map(|w| {
                w.to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|w| !w.is_empty() && !Self::is_stop_word(w))
            .collect()
    }

    /// Load the 10 seed terms from `seed::seed_terms()`.
    fn load_seed_terms(&mut self) {
        let terms = crate::seed::seed_terms();
        for term in terms {
            // Seed terms should all encode cleanly — skip silently on failure.
            // The `test_builtin_has_all_seed_terms` test catches any failures.
            let _result = self.register(term.name, term.definition, term.source);
        }
    }

    /// Diagnostic: attempt to encode a definition and return the first unknown word.
    /// Used in tests to identify missing vocabulary.
    #[cfg(test)]
    fn find_unknown_words(&self, definition: &str) -> Vec<String> {
        let words = Self::extract_content_words(definition);
        words
            .into_iter()
            .filter(|w| !self.decomposer.is_known(w))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_dictionary_not_empty() {
        let dict = Dictionary::builtin();
        assert!(
            dict.term_count() > 0,
            "builtin dictionary should have terms"
        );
    }

    #[test]
    fn test_builtin_has_all_seed_terms() {
        let dict = Dictionary::builtin();
        // Diagnostic: find which seed terms fail and why
        let decomposer = Decomposer::with_seed();
        let temp_dict = Dictionary::new(decomposer);
        let terms = crate::seed::seed_terms();
        let mut all_missing: Vec<String> = Vec::new();
        for term in &terms {
            let unknown = temp_dict.find_unknown_words(term.definition);
            for w in &unknown {
                all_missing.push(format!("'{}' in '{}'", w, term.name));
            }
        }
        assert!(
            all_missing.is_empty(),
            "missing words: {}",
            all_missing.join(", ")
        );
        assert_eq!(dict.term_count(), 10, "should have all 10 seed terms");
    }

    #[test]
    fn test_lookup_pharmacovigilance() {
        let dict = Dictionary::builtin();
        let term = dict.lookup("Pharmacovigilance");
        assert!(term.is_some());
        let term = term.ok_or("missing").err();
        // Just verify it's Some — we checked above
        assert!(dict.lookup("Pharmacovigilance").is_some());
    }

    #[test]
    fn test_lookup_case_insensitive() {
        let dict = Dictionary::builtin();
        assert!(dict.lookup("pharmacovigilance").is_some());
        assert!(dict.lookup("PHARMACOVIGILANCE").is_some());
    }

    #[test]
    fn test_lookup_unknown_returns_none() {
        let dict = Dictionary::builtin();
        assert!(dict.lookup("NonexistentTerm").is_none());
    }

    #[test]
    fn test_register_new_term() -> Result<(), StoichiometryError> {
        let mut dict = Dictionary::builtin();
        let initial_count = dict.term_count();
        dict.register(
            "Drug Safety",
            "monitoring and assessment of drug adverse effects",
            DefinitionSource::Custom("test".to_string()),
        )?;
        assert_eq!(dict.term_count(), initial_count + 1);
        assert!(dict.lookup("Drug Safety").is_some());
        Ok(())
    }

    #[test]
    fn test_register_duplicate_fails() {
        let mut dict = Dictionary::builtin();
        let result = dict.register(
            "Pharmacovigilance",
            "duplicate definition",
            DefinitionSource::Custom("test".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_all_seed_terms_balanced() {
        let dict = Dictionary::builtin();
        for term in dict.all_terms() {
            assert!(
                term.equation.balance.is_balanced,
                "term '{}' is not balanced",
                term.name
            );
            assert!(
                Balancer::verify(&term.equation.balance),
                "term '{}' proof is invalid",
                term.name
            );
        }
    }

    #[test]
    fn test_definition_source_display() {
        let source = DefinitionSource::IchGuideline("ICH E2A".to_string());
        let display = format!("{source}");
        assert!(display.contains("ICH"));
        assert!(display.contains("E2A"));
    }

    #[test]
    fn test_stop_word_filtering() {
        assert!(Dictionary::is_stop_word("the"));
        assert!(Dictionary::is_stop_word("of"));
        assert!(Dictionary::is_stop_word("and"));
        assert!(!Dictionary::is_stop_word("drug"));
        assert!(!Dictionary::is_stop_word("safety"));
    }

    #[test]
    fn test_reverse_lookup() {
        let dict = Dictionary::builtin();
        // Get the primitives from Pharmacovigilance and reverse-lookup
        if let Some(pv_term) = dict.lookup("Pharmacovigilance") {
            let prims = pv_term.equation.concept.formula.primitives().to_vec();
            let matches = dict.reverse_lookup(&prims);
            assert!(!matches.is_empty());
            // Should find Pharmacovigilance itself
            assert!(matches.iter().any(|t| t.name == "Pharmacovigilance"));
        }
    }
}
