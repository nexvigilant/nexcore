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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPrimitive {
    pub name: String,
    pub tier: PrimitiveTier,
    pub description: String,
}

/// A concept extracted from text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedConcept {
    pub term: String,
    pub domain: Option<String>,
    pub frequency: usize,
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

    /// Extract concepts (significant terms) from text.
    pub fn extract_concepts(text: &str) -> Vec<ExtractedConcept> {
        let lower = text.to_lowercase();
        let stopwords: std::collections::HashSet<&str> = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "as", "is", "was", "are", "were", "been", "be", "have", "has", "had",
            "do", "does", "did", "will", "would", "could", "should", "may", "might", "must",
            "shall", "can", "this", "that", "these", "those", "it", "its", "they", "them", "their",
            "not", "no", "so", "if", "then",
        ]
        .into_iter()
        .collect();

        let mut freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
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
                domain: Self::classify_domain(&term),
                term,
                frequency,
            })
            .collect();

        concepts.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        concepts.truncate(50);
        concepts
    }

    /// Classify a term into a domain if recognizable.
    fn classify_domain(term: &str) -> Option<String> {
        let domains: &[(&[&str], &str)] = &[
            (
                &[
                    "signal",
                    "detection",
                    "prr",
                    "ror",
                    "ebgm",
                    "disproportionality",
                    "icsr",
                    "adverse",
                    "pharmacovigilance",
                ],
                "pv",
            ),
            (
                &[
                    "rust", "cargo", "crate", "trait", "struct", "enum", "impl", "compiler",
                ],
                "rust",
            ),
            (
                &[
                    "hook", "skill", "mcp", "tool", "session", "brain", "artifact",
                ],
                "claude-code",
            ),
            (
                &[
                    "arrhenius",
                    "michaelis",
                    "gibbs",
                    "nernst",
                    "langmuir",
                    "equilibrium",
                ],
                "chemistry",
            ),
            (
                &["conservation", "amplitude", "frequency", "inertia", "force"],
                "physics",
            ),
            (
                &["fda", "ich", "cioms", "ema", "regulatory", "guideline"],
                "regulatory",
            ),
        ];

        for (keywords, domain) in domains {
            if keywords.contains(&term) {
                return Some(domain.to_string());
            }
        }
        None
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
        assert_eq!(
            ConceptExtractor::classify_domain("signal"),
            Some("pv".to_string())
        );
        assert_eq!(ConceptExtractor::classify_domain("unknown"), None);
    }
}
