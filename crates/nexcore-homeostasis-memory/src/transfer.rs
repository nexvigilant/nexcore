//! # Cross-Domain Transfer Mappings
//!
//! Maps homeostasis memory concepts to analogous constructs in other domains.
//!
//! ## Domains
//!
//! - **PV** (Pharmacovigilance): Drug safety signal management
//! - **Biology**: Immune system memory
//! - **Cloud**: Incident management and runbooks
//!
//! ## T1 Grounding
//!
//! Transfer mappings themselves ground to: μ (Mapping) + κ (Comparison).

use serde::{Deserialize, Serialize};

/// A mapping between a homeostasis memory concept and its analog in another domain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferMapping {
    /// The source type name from this crate.
    pub source_type: &'static str,
    /// The target domain (e.g., "PV", "Biology", "Cloud").
    pub domain: &'static str,
    /// The analogous concept in the target domain.
    pub analog: &'static str,
    /// Confidence in the structural similarity (0.0-1.0).
    pub confidence: f64,
}

/// Returns all cross-domain transfer mappings for this crate.
#[must_use]
pub fn transfer_mappings() -> Vec<TransferMapping> {
    vec![
        // IncidentSignature transfers
        TransferMapping {
            source_type: "IncidentSignature",
            domain: "PV",
            analog: "Signal detection pattern",
            confidence: 0.85,
        },
        TransferMapping {
            source_type: "IncidentSignature",
            domain: "Biology",
            analog: "Antigen epitope",
            confidence: 0.75,
        },
        TransferMapping {
            source_type: "IncidentSignature",
            domain: "Cloud",
            analog: "Error signature",
            confidence: 0.90,
        },
        // Playbook transfers
        TransferMapping {
            source_type: "Playbook",
            domain: "PV",
            analog: "Signal investigation SOP",
            confidence: 0.80,
        },
        TransferMapping {
            source_type: "Playbook",
            domain: "Biology",
            analog: "Immune response cascade",
            confidence: 0.70,
        },
        TransferMapping {
            source_type: "Playbook",
            domain: "Cloud",
            analog: "Runbook",
            confidence: 0.95,
        },
        // MemoryStore transfers
        TransferMapping {
            source_type: "MemoryStore",
            domain: "PV",
            analog: "Case database / safety database",
            confidence: 0.85,
        },
        TransferMapping {
            source_type: "MemoryStore",
            domain: "Biology",
            analog: "Memory B-cells / T-cells",
            confidence: 0.75,
        },
        TransferMapping {
            source_type: "MemoryStore",
            domain: "Cloud",
            analog: "Incident database",
            confidence: 0.90,
        },
        // SimilarIncident transfers
        TransferMapping {
            source_type: "SimilarIncident",
            domain: "PV",
            analog: "Historical case matching",
            confidence: 0.80,
        },
        TransferMapping {
            source_type: "SimilarIncident",
            domain: "Biology",
            analog: "Cross-reactive immunity",
            confidence: 0.65,
        },
        TransferMapping {
            source_type: "SimilarIncident",
            domain: "Cloud",
            analog: "Similar incident lookup",
            confidence: 0.90,
        },
    ]
}

/// Returns all transfer mappings for a given source type.
///
/// Returns an empty vec if no mappings exist for the type.
#[must_use]
pub fn transfers_for_type(source_type: &str) -> Vec<TransferMapping> {
    transfer_mappings()
        .into_iter()
        .filter(|m| m.source_type == source_type)
        .collect()
}

/// Returns the transfer confidence for a specific source type and domain.
///
/// Returns `None` if no mapping exists for the given combination.
#[must_use]
pub fn transfer_confidence(source_type: &str, domain: &str) -> Option<f64> {
    transfer_mappings()
        .into_iter()
        .find(|m| m.source_type == source_type && m.domain == domain)
        .map(|m| m.confidence)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn total_mappings_count() {
        let all = transfer_mappings();
        // 4 source types * 3 domains = 12 mappings
        assert_eq!(all.len(), 12);
    }

    #[test]
    fn all_confidences_in_range() {
        for m in transfer_mappings() {
            assert!(
                (0.0..=1.0).contains(&m.confidence),
                "Out of range: {} -> {} = {}",
                m.source_type,
                m.domain,
                m.confidence,
            );
        }
    }

    #[test]
    fn transfers_for_incident_signature() {
        let mappings = transfers_for_type("IncidentSignature");
        assert_eq!(mappings.len(), 3);
        let domains: Vec<&str> = mappings.iter().map(|m| m.domain).collect();
        assert!(domains.contains(&"PV"));
        assert!(domains.contains(&"Biology"));
        assert!(domains.contains(&"Cloud"));
    }

    #[test]
    fn transfers_for_playbook() {
        let mappings = transfers_for_type("Playbook");
        assert_eq!(mappings.len(), 3);
        // Cloud runbook should have highest confidence
        let cloud = mappings.iter().find(|m| m.domain == "Cloud");
        assert!(cloud.is_some());
        let cloud = cloud.map(|m| m.confidence).unwrap_or(0.0);
        assert!(cloud >= 0.90);
    }

    #[test]
    fn transfers_for_memory_store() {
        let mappings = transfers_for_type("MemoryStore");
        assert_eq!(mappings.len(), 3);
    }

    #[test]
    fn transfers_for_similar_incident() {
        let mappings = transfers_for_type("SimilarIncident");
        assert_eq!(mappings.len(), 3);
    }

    #[test]
    fn transfers_for_unknown_type() {
        let mappings = transfers_for_type("NonExistent");
        assert!(mappings.is_empty());
    }

    #[test]
    fn transfer_confidence_lookup() {
        let conf = transfer_confidence("Playbook", "Cloud");
        assert_eq!(conf, Some(0.95));
    }

    #[test]
    fn transfer_confidence_missing() {
        let conf = transfer_confidence("Playbook", "Finance");
        assert!(conf.is_none());
    }

    #[test]
    fn all_domains_covered() {
        let domains: std::collections::HashSet<&str> =
            transfer_mappings().iter().map(|m| m.domain).collect();
        assert!(domains.contains("PV"));
        assert!(domains.contains("Biology"));
        assert!(domains.contains("Cloud"));
        assert_eq!(domains.len(), 3);
    }

    #[test]
    fn all_source_types_covered() {
        let types: std::collections::HashSet<&str> =
            transfer_mappings().iter().map(|m| m.source_type).collect();
        assert!(types.contains("IncidentSignature"));
        assert!(types.contains("Playbook"));
        assert!(types.contains("MemoryStore"));
        assert!(types.contains("SimilarIncident"));
        assert_eq!(types.len(), 4);
    }

    #[test]
    fn serde_round_trip() {
        for m in transfer_mappings() {
            let json = serde_json::to_string(&m);
            assert!(json.is_ok(), "Serialization failed for {}", m.source_type);
        }
    }
}
