//! Concept mapping: source concept -> target domain term.
//!
//! Matches annotated concepts against profile bridges to produce
//! a mapping table. Unmapped concepts are flagged for LLM resolution.

use crate::annotation::ConceptAnnotation;
use crate::profile::DomainProfile;
use serde::{Deserialize, Serialize};

/// Method by which a concept was mapped.
///
/// Tier: T2-P | Dominant: kappa (Comparison)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MappingMethod {
    /// Deterministic bridge match from the profile.
    Bridge,
    /// LLM-assisted semantic mapping (filled by skill pipeline).
    LlmAssisted,
    /// No mapping found.
    Unmapped,
}

/// A single concept mapping: source term -> target term.
///
/// Tier: T2-C | Dominant: mu (Mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMapping {
    /// The source concept (generic term).
    pub source: String,
    /// The target domain term (empty if unmapped).
    pub target: String,
    /// Confidence in the mapping (0.0..=1.0).
    pub confidence: f64,
    /// How the mapping was determined.
    pub method: MappingMethod,
}

/// Aggregate mapping table for an entire transformation.
///
/// Tier: T2-C | Dominant: sigma (Sequence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingTable {
    /// All concept mappings.
    pub mappings: Vec<ConceptMapping>,
    /// Aggregate confidence (mean of all mapping confidences).
    pub aggregate_confidence: f64,
    /// Count of unmapped concepts needing LLM resolution.
    pub unmapped_count: usize,
}

impl MappingTable {
    /// Number of successfully mapped concepts (Bridge + LlmAssisted).
    pub fn mapped_count(&self) -> usize {
        self.mappings
            .iter()
            .filter(|m| m.method != MappingMethod::Unmapped)
            .count()
    }

    /// Get mapping for a specific source term.
    pub fn get(&self, source: &str) -> Option<&ConceptMapping> {
        let lower = source.to_lowercase();
        self.mappings.iter().find(|m| m.source == lower)
    }

    /// Replace an Unmapped entry with an LLM-assisted mapping.
    pub fn resolve_unmapped(&mut self, source: &str, target: &str, confidence: f64) {
        let lower = source.to_lowercase();
        if let Some(m) = self.mappings.iter_mut().find(|m| m.source == lower) {
            if m.method == MappingMethod::Unmapped {
                m.target = target.to_string();
                m.confidence = confidence.clamp(0.0, 1.0);
                m.method = MappingMethod::LlmAssisted;
                self.recalculate();
            }
        }
    }

    /// Recalculate aggregate stats.
    fn recalculate(&mut self) {
        self.unmapped_count = self
            .mappings
            .iter()
            .filter(|m| m.method == MappingMethod::Unmapped)
            .count();
        if self.mappings.is_empty() {
            self.aggregate_confidence = 0.0;
        } else {
            let sum: f64 = self.mappings.iter().map(|m| m.confidence).sum();
            self.aggregate_confidence = sum / self.mappings.len() as f64;
        }
    }
}

/// Build a mapping table from annotations and a target profile.
///
/// For each unique concept found in annotations, looks up the profile's
/// bridge table. Matched concepts get `MappingMethod::Bridge`; unmatched
/// get `MappingMethod::Unmapped` with confidence 0.0. When `source_domain`
/// is provided, source-specific bridges are preferred over universal ones.
pub fn build_mapping_table(
    annotations: &[ConceptAnnotation],
    profile: &DomainProfile,
    source_domain: Option<&str>,
) -> MappingTable {
    let mut seen = std::collections::HashSet::new();
    let mut mappings = Vec::new();

    for ann in annotations {
        for concept in &ann.concepts {
            if seen.contains(&concept.term) {
                continue;
            }
            seen.insert(concept.term.clone());

            let mapping = if let Some(bridge) = profile.find_bridge(&concept.term, source_domain) {
                ConceptMapping {
                    source: concept.term.clone(),
                    target: bridge.specific.clone(),
                    confidence: bridge.confidence,
                    method: MappingMethod::Bridge,
                }
            } else {
                ConceptMapping {
                    source: concept.term.clone(),
                    target: String::new(),
                    confidence: 0.0,
                    method: MappingMethod::Unmapped,
                }
            };
            mappings.push(mapping);
        }
    }

    let unmapped_count = mappings
        .iter()
        .filter(|m| m.method == MappingMethod::Unmapped)
        .count();

    let aggregate_confidence = if mappings.is_empty() {
        0.0
    } else {
        let sum: f64 = mappings.iter().map(|m| m.confidence).sum();
        sum / mappings.len() as f64
    };

    MappingTable {
        mappings,
        aggregate_confidence,
        unmapped_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::annotate;
    use crate::profile::builtin_pharmacovigilance;
    use crate::segment::segment;

    #[test]
    fn test_build_mapping_table_basic() {
        let pv = builtin_pharmacovigilance();
        let source = segment("Test", "The citizen faces danger with vigilance.");
        let annotations = annotate(&source, &pv, None);
        let table = build_mapping_table(&annotations, &pv, None);

        assert!(!table.mappings.is_empty());
        assert!(table.aggregate_confidence > 0.0);
    }

    #[test]
    fn test_mapped_concepts_have_targets() {
        let pv = builtin_pharmacovigilance();
        let source = segment("Test", "The citizen faces danger.");
        let annotations = annotate(&source, &pv, None);
        let table = build_mapping_table(&annotations, &pv, None);

        for m in &table.mappings {
            if m.method == MappingMethod::Bridge {
                assert!(!m.target.is_empty(), "{} has empty target", m.source);
                assert!(m.confidence > 0.0);
            }
        }
    }

    #[test]
    fn test_citizen_maps_to_patient() {
        let pv = builtin_pharmacovigilance();
        let source = segment("Test", "Every citizen deserves safety.");
        let annotations = annotate(&source, &pv, None);
        let table = build_mapping_table(&annotations, &pv, None);

        let citizen = table.get("citizen");
        assert!(citizen.is_some());
        let citizen = citizen.unwrap_or_else(|| panic!("citizen mapping not found"));
        assert_eq!(citizen.target, "patient");
        assert_eq!(citizen.method, MappingMethod::Bridge);
    }

    #[test]
    fn test_empty_annotations() {
        let pv = builtin_pharmacovigilance();
        let table = build_mapping_table(&[], &pv, None);
        assert!(table.mappings.is_empty());
        assert_eq!(table.aggregate_confidence, 0.0);
        assert_eq!(table.unmapped_count, 0);
    }

    #[test]
    fn test_no_duplicate_mappings() {
        let pv = builtin_pharmacovigilance();
        let source = segment("Test", "The citizen spoke.\n\nAnother citizen appeared.");
        let annotations = annotate(&source, &pv, None);
        let table = build_mapping_table(&annotations, &pv, None);

        let citizen_count = table
            .mappings
            .iter()
            .filter(|m| m.source == "citizen")
            .count();
        assert_eq!(citizen_count, 1);
    }

    #[test]
    fn test_resolve_unmapped() {
        let mut table = MappingTable {
            mappings: vec![ConceptMapping {
                source: "novelty".into(),
                target: String::new(),
                confidence: 0.0,
                method: MappingMethod::Unmapped,
            }],
            aggregate_confidence: 0.0,
            unmapped_count: 1,
        };

        table.resolve_unmapped("novelty", "emerging signal", 0.75);

        assert_eq!(table.unmapped_count, 0);
        assert_eq!(table.mappings[0].target, "emerging signal");
        assert_eq!(table.mappings[0].method, MappingMethod::LlmAssisted);
        assert!((table.aggregate_confidence - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_mapped_count() {
        let table = MappingTable {
            mappings: vec![
                ConceptMapping {
                    source: "a".into(),
                    target: "b".into(),
                    confidence: 0.8,
                    method: MappingMethod::Bridge,
                },
                ConceptMapping {
                    source: "c".into(),
                    target: String::new(),
                    confidence: 0.0,
                    method: MappingMethod::Unmapped,
                },
            ],
            aggregate_confidence: 0.4,
            unmapped_count: 1,
        };
        assert_eq!(table.mapped_count(), 1);
    }
}
