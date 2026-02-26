//! Concept and primitive extraction from raw text.
//!
//! Expanded from `nexcore-mcp/src/tools/lessons.rs:96-166`.

use serde::{Deserialize, Serialize};

/// Primitive tier classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PrimitiveTier {
    T1,
    T2P,
    T2C,
    T3,
}

impl std::fmt::Display for PrimitiveTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::T1 => write!(f, "T1"),
            Self::T2P => write!(f, "T2-P"),
            Self::T2C => write!(f, "T2-C"),
            Self::T3 => write!(f, "T3"),
        }
    }
}

/// An extracted primitive from text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedPrimitive {
    pub name: String,
    pub tier: PrimitiveTier,
    pub description: String,
}

/// A concept extracted from text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedConcept {
    pub term: String,
    pub domain: Option<String>,
    pub frequency: usize,
}

/// Configurable domain classifier with keyword-to-domain mappings.
///
/// The default configuration covers 6 domains: pv, rust, claude-code, chemistry,
/// physics, and regulatory. Extend with [`DomainClassifier::add_domain`] for
/// project-specific vocabularies.
///
/// ```
/// use nexcore_knowledge_engine::extraction::DomainClassifier;
///
/// let mut classifier = DomainClassifier::new();
/// classifier.add_domain("genomics", &["dna", "rna", "gene"]);
///
/// assert_eq!(classifier.classify("signal"), Some("pv".to_string()));
/// assert_eq!(classifier.classify("gene"), Some("genomics".to_string()));
/// assert_eq!(classifier.classify("unknown"), None);
/// ```
pub struct DomainClassifier {
    domains: Vec<(Vec<String>, String)>,
}

impl Default for DomainClassifier {
    fn default() -> Self {
        let domains = vec![
            (
                vec![
                    "signal",
                    "detection",
                    "prr",
                    "ror",
                    "ebgm",
                    "disproportionality",
                    "icsr",
                    "adverse",
                    "pharmacovigilance",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                "pv".to_string(),
            ),
            (
                vec![
                    "rust", "cargo", "crate", "trait", "struct", "enum", "impl", "compiler",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                "rust".to_string(),
            ),
            (
                vec![
                    "hook", "skill", "mcp", "tool", "session", "brain", "artifact",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                "claude-code".to_string(),
            ),
            (
                vec![
                    "arrhenius",
                    "michaelis",
                    "gibbs",
                    "nernst",
                    "langmuir",
                    "equilibrium",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                "chemistry".to_string(),
            ),
            (
                vec!["conservation", "amplitude", "frequency", "inertia", "force"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                "physics".to_string(),
            ),
            (
                vec!["fda", "ich", "cioms", "ema", "regulatory", "guideline"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                "regulatory".to_string(),
            ),
        ];
        Self { domains }
    }
}

impl DomainClassifier {
    /// Create a classifier with the default 6-domain vocabulary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a custom domain with its keyword set.
    pub fn add_domain(&mut self, domain: &str, keywords: &[&str]) {
        self.domains.push((
            keywords.iter().map(|k| k.to_string()).collect(),
            domain.to_string(),
        ));
    }

    /// Classify a term into a domain if recognizable.
    pub fn classify(&self, term: &str) -> Option<String> {
        for (keywords, domain) in &self.domains {
            if keywords.iter().any(|kw| kw == term) {
                return Some(domain.clone());
            }
        }
        None
    }
}

/// Concept extractor with keyword heuristics.
pub struct ConceptExtractor;

impl ConceptExtractor {
    /// Extract primitives from text using keyword heuristics.
    pub fn extract_primitives(text: &str) -> Vec<ExtractedPrimitive> {
        let lower = text.to_lowercase();
        let mut prims = Vec::new();

        // T1 primitives
        let t1_patterns: &[(&[&str], &str, &str)] = &[
            (
                &["cause", "effect", "trigger", "result in", "leads to"],
                "Causality (→)",
                "Cause-effect relationship",
            ),
            (
                &["count", "number", "quantity", "amount", "total"],
                "Quantity (N)",
                "Numerical magnitude",
            ),
            (
                &["create", "new", "instantiate", "construct", "exist"],
                "Existence (∃)",
                "Instantiation",
            ),
            (
                &["compare", "match", "equal", "differ", "versus"],
                "Comparison (κ)",
                "Predicate matching",
            ),
            (
                &["state", "mutate", "update", "status", "context"],
                "State (ς)",
                "Encapsulated context",
            ),
            (
                &["map", "transform", "convert", "translate"],
                "Mapping (μ)",
                "Transformation A→B",
            ),
            (
                &["sequence", "iterate", "loop", "order", "list"],
                "Sequence (σ)",
                "Ordered succession",
            ),
            (
                &["recursive", "tree", "traverse", "nested", "self-reference"],
                "Recursion (ρ)",
                "Self-reference",
            ),
            (
                &["none", "null", "empty", "absent", "void"],
                "Void (∅)",
                "Meaningful absence",
            ),
            (
                &["boundary", "limit", "guard", "validate", "error"],
                "Boundary (∂)",
                "Limits and transitions",
            ),
            (
                &["frequency", "rate", "count per", "polling", "interval"],
                "Frequency (ν)",
                "Rate of occurrence",
            ),
            (
                &["path", "location", "position", "index", "url"],
                "Location (λ)",
                "Positional context",
            ),
            (
                &["persist", "store", "save", "database", "log"],
                "Persistence (π)",
                "Continuity through time",
            ),
            (
                &["irreversible", "consume", "drop", "hash", "one-way"],
                "Irreversibility (∝)",
                "One-way transition",
            ),
            (
                &["enum", "variant", "either", "one of", "sum type"],
                "Sum (Σ)",
                "Exclusive disjunction",
            ),
            (
                &["struct", "tuple", "combine", "product", "zip"],
                "Product (×)",
                "Conjunctive combination",
            ),
        ];

        for (keywords, name, desc) in t1_patterns {
            if keywords.iter().any(|kw| lower.contains(kw)) {
                prims.push(ExtractedPrimitive {
                    name: name.to_string(),
                    tier: PrimitiveTier::T1,
                    description: desc.to_string(),
                });
            }
        }

        // T2-P patterns
        let t2p_patterns: &[(&[&str], &str, &str)] = &[
            (
                &["threshold", "limit", "timeout", "ceiling"],
                "Threshold",
                "Limit enforcement",
            ),
            (
                &["transform", "convert", "parse", "encode"],
                "Transform",
                "Data transformation",
            ),
            (
                &["decision", "gate", "allow", "block", "exit code"],
                "DecisionGate",
                "Hook decision pattern",
            ),
            (
                &["signal", "detect", "alert", "anomaly"],
                "Signal",
                "Detection pattern",
            ),
        ];

        for (keywords, name, desc) in t2p_patterns {
            if keywords.iter().any(|kw| lower.contains(kw)) {
                prims.push(ExtractedPrimitive {
                    name: name.to_string(),
                    tier: PrimitiveTier::T2P,
                    description: desc.to_string(),
                });
            }
        }

        // T2-C patterns
        let t2c_patterns: &[(&[&str], &str, &str)] = &[
            (
                &["pipeline", "chain", "compose", "workflow"],
                "Pipeline",
                "Composable pipeline",
            ),
            (
                &["feedback", "loop", "homeostasis", "control"],
                "FeedbackLoop",
                "Control loop",
            ),
        ];

        for (keywords, name, desc) in t2c_patterns {
            if keywords.iter().any(|kw| lower.contains(kw)) {
                prims.push(ExtractedPrimitive {
                    name: name.to_string(),
                    tier: PrimitiveTier::T2C,
                    description: desc.to_string(),
                });
            }
        }

        // T3 domain-specific
        let t3_patterns: &[(&[&str], &str, &str)] = &[
            (
                &["pretooluse", "posttooluse", "tool lifecycle"],
                "ToolInterceptor",
                "Tool lifecycle interception",
            ),
            (
                &["sessionstart", "session end", "session lifecycle"],
                "SessionLifecycle",
                "Session boundary",
            ),
            (
                &["icsr", "adverse event", "pharmacovigilance"],
                "PVReport",
                "PV domain concept",
            ),
            (
                &["meddra", "soc", "preferred term"],
                "MedDRACoding",
                "Medical coding",
            ),
        ];

        for (keywords, name, desc) in t3_patterns {
            if keywords.iter().any(|kw| lower.contains(kw)) {
                prims.push(ExtractedPrimitive {
                    name: name.to_string(),
                    tier: PrimitiveTier::T3,
                    description: desc.to_string(),
                });
            }
        }

        prims
    }

    /// Extract concepts (significant terms) from text using the default domain classifier.
    pub fn extract_concepts(text: &str) -> Vec<ExtractedConcept> {
        Self::extract_concepts_with(&DomainClassifier::default(), text)
    }

    /// Extract concepts using a custom domain classifier.
    pub fn extract_concepts_with(
        classifier: &DomainClassifier,
        text: &str,
    ) -> Vec<ExtractedConcept> {
        let lower = text.to_lowercase();
        let stopwords: std::collections::BTreeSet<&str> =
            crate::scoring::STOPWORDS.iter().copied().collect();

        let mut freq: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for word in lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
        {
            if !stopwords.contains(word) {
                *freq.entry(word.to_string()).or_default() += 1;
            }
        }

        let mut concepts: Vec<_> = freq
            .into_iter()
            .filter(|(_, count)| *count >= 1)
            .map(|(term, frequency)| ExtractedConcept {
                domain: classifier.classify(&term),
                term,
                frequency,
            })
            .collect();

        concepts.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        concepts.truncate(50);
        concepts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_t1_primitives() {
        let text = "The pipeline transforms data through a sequence of steps, storing state.";
        let prims = ConceptExtractor::extract_primitives(text);
        let names: Vec<_> = prims.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Mapping (μ)"));
        assert!(names.contains(&"Sequence (σ)"));
        assert!(names.contains(&"State (ς)"));
    }

    #[test]
    fn extract_concepts() {
        let text = "Signal detection uses PRR and ROR for pharmacovigilance safety analysis.";
        let concepts = ConceptExtractor::extract_concepts(text);
        assert!(!concepts.is_empty());
    }

    #[test]
    fn classify_pv_domain() {
        let classifier = DomainClassifier::default();
        assert_eq!(classifier.classify("signal"), Some("pv".to_string()));
        assert_eq!(classifier.classify("unknown"), None);
    }

    #[test]
    fn custom_domain_classifier() {
        let mut classifier = DomainClassifier::new();
        classifier.add_domain("genomics", &["dna", "rna", "gene", "mutation"]);
        assert_eq!(classifier.classify("gene"), Some("genomics".to_string()));
        // Default domains still work
        assert_eq!(classifier.classify("signal"), Some("pv".to_string()));
    }

    #[test]
    fn extract_concepts_with_custom_classifier() {
        let mut classifier = DomainClassifier::new();
        classifier.add_domain("genomics", &["gene", "mutation"]);
        let text = "The gene mutation was detected in the patient sample.";
        let concepts = ConceptExtractor::extract_concepts_with(&classifier, text);
        let gene = concepts.iter().find(|c| c.term == "gene");
        assert!(gene.is_some());
        assert_eq!(gene.map(|c| c.domain.as_deref()), Some(Some("genomics")));
    }
}
