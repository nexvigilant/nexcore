//! # Cross-Domain Transfer Mappings
//!
//! Maps core pharmacovigilance concepts to analogous structures in Biology,
//! Cloud Infrastructure, and Economics domains.
//!
//! Transfer confidence is grounded in the T1 primitive tier system:
//! - T1 primitives transfer at 1.0 (universal)
//! - T2-P transfers at 0.9 (cross-domain primitive)
//! - T2-C transfers at 0.7 (cross-domain composite)
//! - T3 transfers at 0.4 (domain-specific)
//!
//! The confidence values in each `TransferMapping` are empirically calibrated
//! against the structural overlap between source and target types. A mapping
//! with confidence >= 0.85 indicates strong structural isomorphism; below 0.70
//! indicates metaphorical rather than structural correspondence.
//!
//! ## T1 Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|-------------|--------|
//! | Mapping operation | Mapping | μ |
//! | Confidence score | Comparison | κ |
//! | Domain boundary | Boundary | ∂ |

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// TRANSFER MAPPING TYPE
// ═══════════════════════════════════════════════════════════════════════════

/// A single cross-domain transfer mapping.
///
/// Maps a PV source type to an analogous concept in another domain,
/// with a confidence score indicating structural fidelity.
///
/// # Tier: T2-C (composed from T1: Mapping + Comparison + Boundary)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferMapping {
    /// Source type name from pv-core (e.g., "ContingencyTable")
    pub source_type: &'static str,
    /// Target domain (e.g., "biology", "cloud", "economics")
    pub domain: &'static str,
    /// Analogous concept in the target domain
    pub analog: &'static str,
    /// Transfer confidence [0.0, 1.0] — structural fidelity of the mapping
    pub confidence: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
// STATIC TRANSFER REGISTRY
// ═══════════════════════════════════════════════════════════════════════════

/// All known cross-domain transfer mappings from pv-core types.
///
/// These mappings are curated from structural analysis of PV concepts
/// against Biology, Cloud, and Economics domain models.
static TRANSFER_REGISTRY: &[TransferMapping] = &[
    // ── ContingencyTable ────────────────────────────────────────────────
    TransferMapping {
        source_type: "ContingencyTable",
        domain: "biology",
        analog: "Experimental design 2x2 (treatment vs control, outcome vs no outcome)",
        confidence: 0.95,
    },
    TransferMapping {
        source_type: "ContingencyTable",
        domain: "cloud",
        analog: "Error vs normal matrix (request outcome x service health)",
        confidence: 0.80,
    },
    TransferMapping {
        source_type: "ContingencyTable",
        domain: "economics",
        analog: "Cost-benefit matrix (investment x outcome)",
        confidence: 0.78,
    },
    // ── Icsr ────────────────────────────────────────────────────────────
    TransferMapping {
        source_type: "Icsr",
        domain: "biology",
        analog: "Clinical specimen report (patient sample with structured observations)",
        confidence: 0.88,
    },
    TransferMapping {
        source_type: "Icsr",
        domain: "cloud",
        analog: "Incident ticket (structured report of anomalous system behavior)",
        confidence: 0.85,
    },
    TransferMapping {
        source_type: "Icsr",
        domain: "economics",
        analog: "Regulatory filing (structured disclosure of material event)",
        confidence: 0.75,
    },
    // ── SafetyMargin ────────────────────────────────────────────────────
    TransferMapping {
        source_type: "SafetyMargin",
        domain: "biology",
        analog: "Therapeutic index (LD50/ED50 — distance to toxicity boundary)",
        confidence: 0.92,
    },
    TransferMapping {
        source_type: "SafetyMargin",
        domain: "cloud",
        analog: "Error budget margin (remaining SLO budget before violation)",
        confidence: 0.85,
    },
    TransferMapping {
        source_type: "SafetyMargin",
        domain: "economics",
        analog: "Margin of safety (intrinsic value vs market price distance)",
        confidence: 0.90,
    },
    // ── SafeSignalDetector ──────────────────────────────────────────────
    TransferMapping {
        source_type: "SafeSignalDetector",
        domain: "biology",
        analog: "Immune cell antigen detection (pattern recognition triggering response)",
        confidence: 0.80,
    },
    TransferMapping {
        source_type: "SafeSignalDetector",
        domain: "cloud",
        analog: "Anomaly detection system (statistical outlier identification)",
        confidence: 0.88,
    },
    TransferMapping {
        source_type: "SafeSignalDetector",
        domain: "economics",
        analog: "Market surveillance system (unusual trading pattern detection)",
        confidence: 0.82,
    },
    // ── PvCycleResult ───────────────────────────────────────────────────
    TransferMapping {
        source_type: "PvCycleResult",
        domain: "biology",
        analog: "Immune response cycle (detect antigen -> assess threat -> mount response -> resolve)",
        confidence: 0.82,
    },
    TransferMapping {
        source_type: "PvCycleResult",
        domain: "cloud",
        analog: "Incident response lifecycle (detect -> triage -> mitigate -> postmortem)",
        confidence: 0.88,
    },
    TransferMapping {
        source_type: "PvCycleResult",
        domain: "economics",
        analog: "Audit cycle (identify risk -> assess materiality -> respond -> follow-up)",
        confidence: 0.80,
    },
    // ── ThresholdRegistry ───────────────────────────────────────────────
    TransferMapping {
        source_type: "ThresholdRegistry",
        domain: "biology",
        analog: "Normal lab value ranges (reference intervals for clinical chemistry)",
        confidence: 0.90,
    },
    TransferMapping {
        source_type: "ThresholdRegistry",
        domain: "cloud",
        analog: "SLO threshold configuration (latency p99, error rate, availability targets)",
        confidence: 0.88,
    },
    TransferMapping {
        source_type: "ThresholdRegistry",
        domain: "economics",
        analog: "Regulatory capital thresholds (Basel III tier ratios)",
        confidence: 0.82,
    },
    // ── CausalityAssessment ─────────────────────────────────────────────
    TransferMapping {
        source_type: "CausalityAssessment",
        domain: "biology",
        analog: "Koch's postulates (systematic criteria for establishing causal pathogen-disease link)",
        confidence: 0.85,
    },
    TransferMapping {
        source_type: "CausalityAssessment",
        domain: "cloud",
        analog: "Root cause analysis (5-whys / fishbone determining incident causation)",
        confidence: 0.80,
    },
    TransferMapping {
        source_type: "CausalityAssessment",
        domain: "economics",
        analog: "Granger causality test (time-series causal inference for economic variables)",
        confidence: 0.75,
    },
    // ── Seriousness ─────────────────────────────────────────────────────
    TransferMapping {
        source_type: "Seriousness",
        domain: "biology",
        analog: "CTCAE grading (Common Terminology Criteria for Adverse Events, grades 1-5)",
        confidence: 0.92,
    },
    TransferMapping {
        source_type: "Seriousness",
        domain: "cloud",
        analog: "Severity classification (SEV-1 to SEV-5, incident impact grading)",
        confidence: 0.88,
    },
    TransferMapping {
        source_type: "Seriousness",
        domain: "economics",
        analog: "Material event classification (SEC materiality thresholds for disclosure)",
        confidence: 0.78,
    },
];

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════

/// Returns all registered cross-domain transfer mappings.
///
/// The registry contains curated mappings from pv-core types to
/// Biology, Cloud, and Economics analogs.
#[must_use]
pub fn transfer_mappings() -> &'static [TransferMapping] {
    TRANSFER_REGISTRY
}

/// Returns the transfer confidence for a specific source type to a target domain.
///
/// Returns `None` if no mapping exists for the given type-domain pair.
///
/// # Arguments
///
/// * `source_type` — The pv-core type name (e.g., "ContingencyTable")
/// * `domain` — The target domain (e.g., "biology", "cloud", "economics")
#[must_use]
pub fn transfer_confidence(source_type: &str, domain: &str) -> Option<f64> {
    TRANSFER_REGISTRY
        .iter()
        .find(|m| m.source_type == source_type && m.domain == domain)
        .map(|m| m.confidence)
}

/// Returns all transfer mappings for a given source type across all domains.
///
/// # Arguments
///
/// * `source_type` — The pv-core type name (e.g., "SafetyMargin")
#[must_use]
pub fn transfers_for_type(source_type: &str) -> Vec<&'static TransferMapping> {
    TRANSFER_REGISTRY
        .iter()
        .filter(|m| m.source_type == source_type)
        .collect()
}

/// Returns all transfer mappings targeting a specific domain.
///
/// # Arguments
///
/// * `domain` — The target domain (e.g., "biology")
#[must_use]
pub fn transfers_for_domain(domain: &str) -> Vec<&'static TransferMapping> {
    TRANSFER_REGISTRY
        .iter()
        .filter(|m| m.domain == domain)
        .collect()
}

/// Returns the set of unique source types that have transfer mappings.
#[must_use]
pub fn mapped_source_types() -> Vec<&'static str> {
    let mut types: Vec<&'static str> = TRANSFER_REGISTRY
        .iter()
        .map(|m| m.source_type)
        .collect();
    types.sort_unstable();
    types.dedup();
    types
}

/// Returns the set of unique target domains.
#[must_use]
pub fn target_domains() -> Vec<&'static str> {
    let mut domains: Vec<&'static str> = TRANSFER_REGISTRY
        .iter()
        .map(|m| m.domain)
        .collect();
    domains.sort_unstable();
    domains.dedup();
    domains
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_non_empty() {
        assert!(!transfer_mappings().is_empty());
    }

    #[test]
    fn all_confidences_in_valid_range() {
        for mapping in transfer_mappings() {
            assert!(
                (0.0..=1.0).contains(&mapping.confidence),
                "Confidence {} out of range for {} -> {} ({})",
                mapping.confidence,
                mapping.source_type,
                mapping.domain,
                mapping.analog
            );
        }
    }

    #[test]
    fn contingency_table_maps_to_biology() {
        let conf = transfer_confidence("ContingencyTable", "biology");
        assert_eq!(conf, Some(0.95));
    }

    #[test]
    fn safety_margin_maps_to_three_domains() {
        let mappings = transfers_for_type("SafetyMargin");
        assert_eq!(mappings.len(), 3);
        assert!(mappings.iter().any(|m| m.domain == "biology"));
        assert!(mappings.iter().any(|m| m.domain == "cloud"));
        assert!(mappings.iter().any(|m| m.domain == "economics"));
    }

    #[test]
    fn unknown_type_returns_none() {
        assert_eq!(transfer_confidence("NonexistentType", "biology"), None);
    }

    #[test]
    fn unknown_domain_returns_none() {
        assert_eq!(transfer_confidence("ContingencyTable", "astrology"), None);
    }

    #[test]
    fn transfers_for_domain_biology() {
        let bio = transfers_for_domain("biology");
        assert!(bio.len() >= 8, "Expected at least 8 biology mappings, got {}", bio.len());
        // Every biology mapping should have the "biology" domain
        for m in &bio {
            assert_eq!(m.domain, "biology");
        }
    }

    #[test]
    fn mapped_source_types_are_unique_and_sorted() {
        let types = mapped_source_types();
        for window in types.windows(2) {
            assert!(window[0] < window[1], "Not sorted: {} >= {}", window[0], window[1]);
        }
    }

    #[test]
    fn target_domains_include_all_three() {
        let domains = target_domains();
        assert!(domains.contains(&"biology"));
        assert!(domains.contains(&"cloud"));
        assert!(domains.contains(&"economics"));
    }

    #[test]
    fn seriousness_high_confidence_biology() {
        let conf = transfer_confidence("Seriousness", "biology");
        assert_eq!(conf, Some(0.92));
    }

    #[test]
    fn pv_cycle_cloud_analog() {
        let mappings = transfers_for_type("PvCycleResult");
        let cloud = mappings.iter().find(|m| m.domain == "cloud");
        assert!(cloud.is_some());
        assert!(cloud.map_or(false, |m| m.analog.contains("Incident response")));
    }

    #[test]
    fn each_source_type_has_at_least_two_domains() {
        for source in mapped_source_types() {
            let count = transfers_for_type(source).len();
            assert!(
                count >= 2,
                "Source type '{}' only maps to {} domain(s), need >= 2",
                source,
                count
            );
        }
    }
}
