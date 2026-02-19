//! Decomposer — maps words to primitive formulas.
//!
//! Each known word is associated with a `Vec<LexPrimitiva>` that captures the
//! word's essential primitive composition. The seed vocabulary covers the
//! fundamental PV/regulatory domain words.

use crate::error::StoichiometryError;
use nexcore_lex_primitiva::molecular_weight::MolecularFormula;
use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use std::collections::HashMap;

/// Decomposes words into primitive formulas.
#[derive(Debug, Clone)]
pub struct Decomposer {
    known_words: HashMap<String, Vec<LexPrimitiva>>,
}

impl Decomposer {
    /// Create an empty decomposer with no known words.
    #[must_use]
    pub fn new() -> Self {
        Self {
            known_words: HashMap::new(),
        }
    }

    /// Create a decomposer pre-loaded with the seed PV vocabulary.
    ///
    /// The seed covers ~80 fundamental words that appear in authoritative
    /// PV definitions (ICH, WHO, CIOMS).
    #[must_use]
    pub fn with_seed() -> Self {
        let mut d = Self::new();
        d.load_seed();
        d
    }

    /// Register a word with its primitive decomposition (lowercased).
    pub fn register(&mut self, word: &str, primitives: Vec<LexPrimitiva>) {
        self.known_words.insert(word.to_lowercase(), primitives);
    }

    /// Decompose a word into a `MolecularFormula`.
    ///
    /// Lookup is case-insensitive.
    ///
    /// # Errors
    ///
    /// Returns `StoichiometryError::UnknownWord` if the word is not registered.
    pub fn decompose(&self, word: &str) -> Result<MolecularFormula, StoichiometryError> {
        let key = word.to_lowercase();
        let prims = self
            .known_words
            .get(&key)
            .ok_or_else(|| StoichiometryError::UnknownWord {
                word: word.to_string(),
            })?;

        let formula = MolecularFormula::new(word).with_all(prims);
        Ok(formula)
    }

    /// Check if a word is in the known vocabulary.
    #[must_use]
    pub fn is_known(&self, word: &str) -> bool {
        self.known_words.contains_key(&word.to_lowercase())
    }

    /// Number of known words in the vocabulary.
    #[must_use]
    pub fn known_word_count(&self) -> usize {
        self.known_words.len()
    }

    /// Get the decomposer's vocabulary (for inspection).
    #[must_use]
    pub fn vocabulary(&self) -> &HashMap<String, Vec<LexPrimitiva>> {
        &self.known_words
    }

    /// Load the seed PV vocabulary.
    ///
    /// These ~80 words cover all content words appearing in the 10 seed term
    /// definitions (ICH E2A, E2B, E2C, E2E, WHO). Each word is mapped to 2-4
    /// primitives that capture its essential meaning.
    fn load_seed(&mut self) {
        use LexPrimitiva::*;

        // Core PV domain words
        self.register("drug", vec![Existence, State, Boundary]);
        self.register("medicinal", vec![Existence, State, Boundary]);
        self.register("product", vec![Existence, State, Mapping]);
        self.register("pharmaceutical", vec![Existence, State, Boundary, Mapping]);
        self.register("medicine", vec![Existence, State, Boundary]);
        self.register("safety", vec![Boundary, Comparison, Irreversibility]);
        self.register("monitoring", vec![Frequency, Sequence, Persistence]);
        self.register("adverse", vec![Boundary, Irreversibility, Causality]);
        self.register("event", vec![Existence, Causality, Frequency]);
        self.register("reaction", vec![Causality, Mapping, Boundary]);
        self.register("signal", vec![Frequency, Comparison, Quantity, Boundary]);
        self.register("risk", vec![Boundary, Quantity, Causality]);
        self.register("benefit", vec![Quantity, Comparison, Existence]);
        self.register("harm", vec![Irreversibility, Boundary, Causality]);
        self.register("exposure", vec![Boundary, Frequency, Existence]);

        // Reporting and documentation
        self.register("report", vec![Persistence, Sequence, Existence]);
        self.register("reporting", vec![Persistence, Sequence, Existence, Frequency]);
        self.register("case", vec![Existence, Boundary, Persistence]);
        self.register("individual", vec![Existence, Boundary, Quantity]);
        self.register("submission", vec![Sequence, Persistence, Boundary]);
        self.register("notification", vec![Causality, Sequence, Persistence]);
        self.register("periodic", vec![Frequency, Sequence, Persistence]);
        self.register("update", vec![State, Sequence, Persistence]);
        self.register("record", vec![Persistence, Existence, Sequence]);
        self.register("documentation", vec![Persistence, Sequence, Mapping]);
        self.register("format", vec![Mapping, Boundary, Sequence]);

        // Assessment and analysis
        self.register("assessment", vec![Comparison, Quantity, Boundary]);
        self.register("evaluation", vec![Comparison, Quantity, Mapping]);
        self.register("analysis", vec![Mapping, Quantity, Comparison]);
        self.register("detection", vec![Frequency, Comparison, Boundary]);
        self.register("identification", vec![Existence, Comparison, Boundary]);
        self.register("surveillance", vec![Frequency, Persistence, Boundary, Comparison]);
        self.register("investigation", vec![Causality, Comparison, Sequence]);
        self.register("review", vec![Comparison, Sequence, Mapping]);
        self.register("determination", vec![Causality, Comparison, Boundary]);
        self.register("causality", vec![Causality, Comparison, Sequence]);
        self.register("relationship", vec![Causality, Mapping, Comparison]);
        self.register("causal", vec![Causality, Mapping, Comparison]);

        // Severity and classification
        self.register("serious", vec![Irreversibility, Boundary, Quantity]);
        self.register("severity", vec![Quantity, Boundary, Irreversibility]);
        self.register("unexpected", vec![Void, Boundary, Comparison]);
        self.register("labeled", vec![Persistence, Boundary, Mapping]);
        self.register("expedited", vec![Sequence, Frequency, Irreversibility]);
        self.register("undesirable", vec![Boundary, Irreversibility, Comparison]);
        self.register("noxious", vec![Irreversibility, Boundary, Causality]);

        // Population and frequency
        self.register("population", vec![Quantity, Existence, Boundary]);
        self.register("frequency", vec![Frequency, Quantity, Comparison]);
        self.register("outcome", vec![Causality, Existence, Irreversibility]);
        self.register("incidence", vec![Frequency, Quantity, Existence]);
        self.register("rate", vec![Frequency, Quantity, Sequence]);

        // Management and process
        self.register("management", vec![State, Sequence, Causality, Boundary]);
        self.register("process", vec![Sequence, Mapping, Causality]);
        self.register("system", vec![Mapping, Boundary, Sequence, Persistence]);
        self.register("control", vec![Boundary, State, Causality]);
        self.register("measure", vec![Quantity, Comparison, Boundary]);
        self.register("prevention", vec![Boundary, Causality, Irreversibility]);
        self.register("minimization", vec![Quantity, Boundary, Comparison]);
        self.register("plan", vec![Sequence, Causality, Persistence]);
        self.register("strategy", vec![Sequence, Causality, Mapping]);
        self.register("activities", vec![Sequence, Causality, Existence]);
        self.register("action", vec![Causality, Sequence, Existence]);
        self.register("response", vec![Causality, Mapping, Sequence]);

        // Regulatory and standards
        self.register("standard", vec![Boundary, Comparison, Persistence]);
        self.register("guideline", vec![Sequence, Boundary, Persistence]);
        self.register("regulatory", vec![Boundary, Persistence, Causality]);
        self.register("compliance", vec![Boundary, Comparison, Persistence]);
        self.register("quality", vec![Comparison, Boundary, Quantity]);
        self.register("authority", vec![Boundary, Causality, Persistence]);
        self.register("authorities", vec![Boundary, Causality, Persistence]);
        self.register("requirements", vec![Boundary, Existence, Persistence]);
        self.register("regulations", vec![Boundary, Persistence, Sequence]);

        // Data and evidence
        self.register("data", vec![Persistence, Quantity, Existence]);
        self.register("evidence", vec![Existence, Comparison, Persistence]);
        self.register("information", vec![Persistence, Mapping, Existence]);
        self.register("collection", vec![Quantity, Sequence, Persistence]);
        self.register("source", vec![Existence, Location, Causality]);
        self.register("sources", vec![Existence, Location, Causality]);
        self.register("aggregate", vec![Quantity, Sum, Mapping]);
        self.register("cumulative", vec![Quantity, Sequence, Sum]);
        self.register("summary", vec![Mapping, Quantity, Comparison]);

        // Medical/clinical terms needed for seed definitions
        self.register("patient", vec![Existence, Boundary, State]);
        self.register("patients", vec![Existence, Boundary, State]);
        self.register("medical", vec![Existence, State, Boundary]);
        self.register("clinical", vec![Existence, Comparison, State]);
        self.register("treatment", vec![Mapping, Causality, State]);
        self.register("dose", vec![Quantity, Boundary, Causality]);
        self.register("doses", vec![Quantity, Boundary, Causality]);
        self.register("health", vec![State, Existence, Boundary]);
        self.register("care", vec![State, Causality, Existence]);
        self.register("practice", vec![Sequence, Mapping, Persistence]);
        self.register("condition", vec![State, Boundary, Existence]);
        self.register("diagnosis", vec![Comparison, Existence, Mapping]);
        self.register("therapy", vec![Mapping, Causality, State]);
        self.register("disease", vec![State, Boundary, Irreversibility]);
        self.register("experience", vec![Existence, Sequence, Persistence]);

        // Terms specific to seed definitions
        self.register("science", vec![Mapping, Comparison, Existence]);
        self.register("relating", vec![Mapping, Causality, Comparison]);
        self.register("related", vec![Mapping, Causality, Comparison]);
        self.register("associated", vec![Causality, Mapping, Comparison]);
        self.register("use", vec![Mapping, Causality, Existence]);
        self.register("used", vec![Mapping, Causality, Existence]);
        self.register("using", vec![Mapping, Causality, Existence]);
        self.register("prevent", vec![Boundary, Causality, Irreversibility]);
        self.register("preventing", vec![Boundary, Causality, Irreversibility]);
        self.register("protect", vec![Boundary, Irreversibility, State]);
        self.register("protection", vec![Boundary, Irreversibility, State]);
        self.register("public", vec![Quantity, Existence, Boundary]);
        self.register("new", vec![Existence, State, Void]);
        self.register("possible", vec![Existence, Comparison, Void]);
        self.register("known", vec![Persistence, Comparison, Existence]);
        self.register("potential", vec![Existence, Comparison, Causality]);
        self.register("effects", vec![Causality, Mapping, Existence]);
        self.register("change", vec![State, Mapping, Causality]);
        self.register("changes", vec![State, Mapping, Causality]);

        // Specific to ICH definitions
        self.register("unfavorable", vec![Boundary, Irreversibility, Comparison]);
        self.register("unintended", vec![Void, Causality, Boundary]);
        self.register("occurrence", vec![Existence, Frequency, Causality]);
        self.register("necessarily", vec![Causality, Boundary, Comparison]);
        self.register("administered", vec![Mapping, Causality, Sequence]);
        self.register("following", vec![Sequence, Causality, Existence]);
        self.register("results", vec![Causality, Existence, Mapping]);
        self.register("death", vec![Irreversibility, Existence, Boundary]);
        self.register("life", vec![Existence, Sequence, Persistence]);
        self.register("threatening", vec![Irreversibility, Boundary, Causality]);
        self.register("hospitalization", vec![Location, State, Boundary]);
        self.register("inpatient", vec![Location, State, Boundary]);
        self.register("disability", vec![Irreversibility, State, Boundary]);
        self.register("incapacity", vec![Irreversibility, State, Boundary]);
        self.register("congenital", vec![Existence, Irreversibility, Boundary]);
        self.register("anomaly", vec![Void, Existence, Boundary]);
        self.register("birth", vec![Existence, Causality, Irreversibility]);
        self.register("defect", vec![Boundary, Void, Irreversibility]);
        self.register("medically", vec![Existence, State, Boundary]);
        self.register("important", vec![Quantity, Comparison, Causality]);
        self.register("requires", vec![Boundary, Existence, Causality]);
        self.register("intervention", vec![Causality, Mapping, Boundary]);
        self.register("prolonged", vec![Sequence, Persistence, Quantity]);
        self.register("existing", vec![Existence, Persistence, State]);
        self.register("significant", vec![Quantity, Comparison, Boundary]);

        // Hypothesis / signal detection
        self.register("hypothesis", vec![Causality, Comparison, Void]);
        self.register("generating", vec![Causality, Existence, Mapping]);
        self.register("generated", vec![Causality, Existence, Mapping]);
        self.register("strengthened", vec![Quantity, Comparison, Causality]);
        self.register("quantified", vec![Quantity, Mapping, Comparison]);
        self.register("disproportionate", vec![Quantity, Comparison, Boundary]);
        self.register("disproportionality", vec![Quantity, Comparison, Boundary]);

        // Benefit-risk specific
        self.register("balance", vec![Comparison, Quantity, Boundary]);
        self.register("favorable", vec![Comparison, Boundary, Existence]);
        self.register("comparative", vec![Comparison, Mapping, Quantity]);
        self.register("overall", vec![Sum, Quantity, Comparison]);
        self.register("weigh", vec![Comparison, Quantity, Causality]);
        self.register("weighing", vec![Comparison, Quantity, Causality]);
        self.register("evaluation", vec![Comparison, Quantity, Mapping]);

        // ICSR specific
        self.register("structured", vec![Mapping, Boundary, Sequence]);
        self.register("standardized", vec![Boundary, Comparison, Mapping]);
        self.register("electronic", vec![Mapping, Sequence, Persistence]);
        self.register("transmission", vec![Sequence, Mapping, Persistence]);
        self.register("transmitting", vec![Sequence, Mapping, Persistence]);
        self.register("describing", vec![Mapping, Existence, Persistence]);
        self.register("suspected", vec![Comparison, Causality, Void]);
        self.register("one", vec![Quantity, Existence]);
        self.register("more", vec![Quantity, Comparison]);

        // PSUR specific
        self.register("interval", vec![Sequence, Boundary, Frequency]);
        self.register("worldwide", vec![Location, Quantity, Boundary]);
        self.register("comprehensive", vec![Quantity, Mapping, Boundary]);
        self.register("profile", vec![Mapping, Persistence, Boundary]);

        // RMP specific
        self.register("pharmacovigilance", vec![Frequency, Boundary, State, Comparison, Persistence, Causality]);
        self.register("detailed", vec![Mapping, Quantity, Comparison]);
        self.register("description", vec![Mapping, Persistence, Existence]);
        self.register("characterize", vec![Mapping, Comparison, Boundary]);
        self.register("characterizing", vec![Mapping, Comparison, Boundary]);
        self.register("minimize", vec![Quantity, Boundary, Comparison]);
        self.register("risks", vec![Boundary, Quantity, Causality]);
        self.register("outlining", vec![Mapping, Sequence, Boundary]);
        self.register("proposed", vec![Causality, Sequence, Existence]);
        self.register("studies", vec![Comparison, Sequence, Persistence]);
        self.register("effectiveness", vec![Causality, Comparison, Quantity]);
        self.register("measures", vec![Quantity, Comparison, Boundary]);
        self.register("those", vec![Mapping, Existence, Comparison]);
        self.register("document", vec![Persistence, Mapping, Existence]);

        // Verb forms needed by seed definitions
        self.register("arising", vec![Existence, Causality, Sequence]);
        self.register("determining", vec![Causality, Comparison, Boundary]);
        self.register("exists", vec![Existence, Persistence, Comparison]);
        self.register("reactions", vec![Causality, Mapping, Boundary]);

        // Additional connecting words appearing in definitions
        self.register("association", vec![Causality, Mapping, Comparison]);
        self.register("regarded", vec![Comparison, Mapping, Persistence]);
        self.register("whether", vec![Comparison, Void, Causality]);
        self.register("may", vec![Existence, Comparison, Void]);
        self.register("single", vec![Quantity, Existence, Boundary]);
        self.register("multiple", vec![Quantity, Sequence, Existence]);
        self.register("consistent", vec![Comparison, Persistence, Boundary]);
        self.register("type", vec![Mapping, Boundary, Comparison]);
        self.register("including", vec![Sum, Mapping, Existence]);
        self.register("defined", vec![Boundary, Mapping, Persistence]);
    }
}

impl Default for Decomposer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompose_known_word() -> Result<(), StoichiometryError> {
        let d = Decomposer::with_seed();
        let formula = d.decompose("safety")?;
        assert_eq!(formula.name(), "safety");
        assert!(!formula.primitives().is_empty());
        // safety -> [Boundary, Comparison, Irreversibility]
        assert_eq!(formula.primitives().len(), 3);
        Ok(())
    }

    #[test]
    fn test_decompose_unknown_word_fails() {
        let d = Decomposer::with_seed();
        let result = d.decompose("xyzzy");
        assert!(result.is_err());
    }

    #[test]
    fn test_register_custom_word() -> Result<(), StoichiometryError> {
        let mut d = Decomposer::new();
        d.register("myword", vec![LexPrimitiva::Causality, LexPrimitiva::Boundary]);
        let formula = d.decompose("myword")?;
        assert_eq!(formula.primitives().len(), 2);
        Ok(())
    }

    #[test]
    fn test_decompose_case_insensitive() -> Result<(), StoichiometryError> {
        let d = Decomposer::with_seed();
        let lower = d.decompose("safety")?;
        let upper = d.decompose("Safety")?;
        let all_caps = d.decompose("SAFETY")?;
        assert_eq!(lower.primitives(), upper.primitives());
        assert_eq!(lower.primitives(), all_caps.primitives());
        Ok(())
    }

    #[test]
    fn test_formula_has_correct_name() -> Result<(), StoichiometryError> {
        let d = Decomposer::with_seed();
        let formula = d.decompose("drug")?;
        assert_eq!(formula.name(), "drug");
        Ok(())
    }

    #[test]
    fn test_is_known() {
        let d = Decomposer::with_seed();
        assert!(d.is_known("drug"));
        assert!(d.is_known("Drug")); // case insensitive
        assert!(!d.is_known("xyzzy"));
    }

    #[test]
    fn test_seed_has_sufficient_vocabulary() {
        let d = Decomposer::with_seed();
        // Must have at least 50 words for the seed definitions
        assert!(
            d.known_word_count() >= 50,
            "seed vocabulary too small: {}",
            d.known_word_count()
        );
    }
}
