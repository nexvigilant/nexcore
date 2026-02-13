//! Concept annotation: mark concepts per paragraph.
//!
//! Scans each paragraph for vocabulary terms from a domain profile,
//! producing per-paragraph concept occurrence records.

use crate::profile::DomainProfile;
use crate::segment::{Paragraph, SourceText};
use serde::{Deserialize, Serialize};

/// A single concept occurrence within a paragraph.
///
/// Tier: T2-P | Dominant: mu (Mapping)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConceptOccurrence {
    /// The concept term found.
    pub term: String,
    /// Whether this term exists in the target profile's bridge table.
    pub has_bridge: bool,
}

/// Annotations for a single paragraph.
///
/// Tier: T2-C | Dominant: mu (Mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptAnnotation {
    /// Index of the source paragraph.
    pub paragraph_index: usize,
    /// Concepts found in this paragraph.
    pub concepts: Vec<ConceptOccurrence>,
}

impl ConceptAnnotation {
    /// Number of concepts with bridges available.
    pub fn bridged_count(&self) -> usize {
        self.concepts.iter().filter(|c| c.has_bridge).count()
    }

    /// Number of concepts without bridges (need LLM resolution).
    pub fn unbridged_count(&self) -> usize {
        self.concepts.iter().filter(|c| !c.has_bridge).count()
    }

    /// Coverage ratio: bridged / total (0.0 if no concepts).
    pub fn coverage(&self) -> f64 {
        if self.concepts.is_empty() {
            return 0.0;
        }
        self.bridged_count() as f64 / self.concepts.len() as f64
    }
}

/// Annotate all paragraphs in a source text against a profile.
///
/// For each paragraph, scans for vocabulary terms using case-insensitive
/// word boundary matching. Each found term is checked against the
/// profile's bridge table. When `source_domain` is provided, source-specific
/// bridges are preferred over universal ones.
pub fn annotate(
    source: &SourceText,
    profile: &DomainProfile,
    source_domain: Option<&str>,
) -> Vec<ConceptAnnotation> {
    source
        .paragraphs
        .iter()
        .map(|para| annotate_paragraph(para, profile, source_domain))
        .collect()
}

/// Annotate a single paragraph.
fn annotate_paragraph(
    para: &Paragraph,
    profile: &DomainProfile,
    source_domain: Option<&str>,
) -> ConceptAnnotation {
    let lower_text = para.text.to_lowercase();
    let mut concepts = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Check each bridge's generic term against the paragraph
    for bridge in &profile.bridges {
        if !seen.contains(&bridge.generic) && contains_term(&lower_text, &bridge.generic) {
            seen.insert(bridge.generic.clone());
            concepts.push(ConceptOccurrence {
                term: bridge.generic.clone(),
                has_bridge: true,
            });
        }
    }

    // Check vocabulary terms that aren't already bridge generics
    for vocab_term in &profile.vocabulary {
        let lower_term = vocab_term.to_lowercase();
        if !seen.contains(&lower_term) && contains_term(&lower_text, &lower_term) {
            seen.insert(lower_term.clone());
            concepts.push(ConceptOccurrence {
                term: lower_term,
                has_bridge: profile.find_bridge(vocab_term, source_domain).is_some(),
            });
        }
    }

    ConceptAnnotation {
        paragraph_index: para.index,
        concepts,
    }
}

/// Check if text contains a term (case-insensitive, word-boundary-aware).
///
/// Uses simple boundary checking: the character before and after the match
/// must be non-alphanumeric (or start/end of string).
fn contains_term(text: &str, term: &str) -> bool {
    if term.is_empty() {
        return false;
    }
    let text_bytes = text.as_bytes();
    let term_bytes = term.as_bytes();
    let term_len = term_bytes.len();

    if text_bytes.len() < term_len {
        return false;
    }

    for i in 0..=(text_bytes.len() - term_len) {
        if text[i..i + term_len] == *term {
            // Check left boundary
            let left_ok = i == 0 || !text_bytes[i - 1].is_ascii_alphanumeric();
            // Check right boundary
            let right_ok = i + term_len == text_bytes.len()
                || !text_bytes[i + term_len].is_ascii_alphanumeric();
            if left_ok && right_ok {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::builtin_pharmacovigilance;
    use crate::segment::segment;

    #[test]
    fn test_contains_term_basic() {
        assert!(contains_term("the citizen spoke", "citizen"));
        assert!(contains_term("citizen spoke", "citizen"));
        assert!(contains_term("the citizen", "citizen"));
        assert!(contains_term("citizen", "citizen"));
    }

    #[test]
    fn test_contains_term_word_boundary() {
        // "citizens" should NOT match "citizen" at word boundary
        assert!(!contains_term("citizens spoke", "citizen"));
    }

    #[test]
    fn test_contains_term_empty() {
        assert!(!contains_term("text", ""));
        assert!(!contains_term("", "term"));
    }

    #[test]
    fn test_annotate_finds_bridge_terms() {
        let pv = builtin_pharmacovigilance();
        let source = segment(
            "Test",
            "The citizen must exercise vigilance against danger.",
        );
        let annotations = annotate(&source, &pv, None);
        assert_eq!(annotations.len(), 1);
        let ann = &annotations[0];
        let terms: Vec<&str> = ann.concepts.iter().map(|c| c.term.as_str()).collect();
        assert!(terms.contains(&"citizen"));
        assert!(terms.contains(&"vigilance"));
        assert!(terms.contains(&"danger"));
    }

    #[test]
    fn test_annotate_marks_bridge_status() {
        let pv = builtin_pharmacovigilance();
        let source = segment("Test", "The citizen faces danger.");
        let annotations = annotate(&source, &pv, None);
        let ann = &annotations[0];
        for concept in &ann.concepts {
            // "citizen" and "danger" both have bridges in PV profile
            assert!(concept.has_bridge, "{} should have bridge", concept.term);
        }
    }

    #[test]
    fn test_annotate_empty_text() {
        let pv = builtin_pharmacovigilance();
        let source = segment("Empty", "");
        let annotations = annotate(&source, &pv, None);
        assert!(annotations.is_empty());
    }

    #[test]
    fn test_annotate_no_matches() {
        let pv = builtin_pharmacovigilance();
        let source = segment("No Match", "The quick brown fox jumps over the lazy dog.");
        let annotations = annotate(&source, &pv, None);
        assert_eq!(annotations.len(), 1);
        assert!(annotations[0].concepts.is_empty());
    }

    #[test]
    fn test_coverage_calculation() {
        let ann = ConceptAnnotation {
            paragraph_index: 0,
            concepts: vec![
                ConceptOccurrence {
                    term: "a".into(),
                    has_bridge: true,
                },
                ConceptOccurrence {
                    term: "b".into(),
                    has_bridge: true,
                },
                ConceptOccurrence {
                    term: "c".into(),
                    has_bridge: false,
                },
            ],
        };
        let coverage = ann.coverage();
        assert!((coverage - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_coverage_empty() {
        let ann = ConceptAnnotation {
            paragraph_index: 0,
            concepts: vec![],
        };
        assert!((ann.coverage() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_no_duplicate_concepts() {
        let pv = builtin_pharmacovigilance();
        // "danger" appears once — should only produce one concept
        let source = segment("Test", "Danger and more danger everywhere.");
        let annotations = annotate(&source, &pv, None);
        let danger_count = annotations[0]
            .concepts
            .iter()
            .filter(|c| c.term == "danger")
            .count();
        assert_eq!(danger_count, 1);
    }
}
