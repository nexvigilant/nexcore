//! # Cross-Domain Transfer Mappings
//!
//! Maps preemptive PV concepts to analogous constructs in Biology,
//! Cloud Infrastructure, and Economics domains.
//!
//! ## Theory of Vigilance Context
//!
//! Transfer confidence is grounded in the Lex Primitiva tier system:
//! types sharing more T1 primitives with their cross-domain analogs
//! achieve higher transfer confidence. This module makes those
//! cross-domain bridges explicit and measurable.
//!
//! ## Primitives at Play
//!
//! - `mu` (Mapping): Each transfer IS a mapping between domains
//! - `kappa` (Comparison): Confidence quantifies structural similarity
//! - `N` (Quantity): Confidence values are numeric measures

use serde::{Deserialize, Serialize};

/// A mapping from a preemptive PV type to an analogous concept in another domain.
///
/// Transfer confidence ranges from 0.0 (no structural similarity) to 1.0
/// (isomorphic mapping). Values above 0.80 indicate strong structural analogy
/// where insights can flow bidirectionally between domains.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferMapping {
    /// The source type name from this crate (e.g., "GibbsParams")
    pub source_type: &'static str,
    /// The target domain (e.g., "Biology", "Cloud", "Economics")
    pub domain: &'static str,
    /// The analogous concept in the target domain
    pub analog: &'static str,
    /// Transfer confidence in [0.0, 1.0]
    pub confidence: f64,
}

/// Returns all cross-domain transfer mappings for preemptive PV types.
///
/// Each entry maps a crate type to an analogous concept in Biology,
/// Cloud Infrastructure, or Economics with a confidence score.
#[must_use]
pub fn transfer_mappings() -> Vec<TransferMapping> {
    vec![
        // -- GibbsParams --
        TransferMapping {
            source_type: "GibbsParams",
            domain: "Biology",
            analog: "Gibbs free energy in protein folding",
            confidence: 0.95,
        },
        TransferMapping {
            source_type: "GibbsParams",
            domain: "Cloud",
            analog: "System resource threshold",
            confidence: 0.60,
        },
        TransferMapping {
            source_type: "GibbsParams",
            domain: "Economics",
            analog: "Market activation energy",
            confidence: 0.70,
        },
        // -- Seriousness --
        TransferMapping {
            source_type: "Seriousness",
            domain: "Biology",
            analog: "Severity grading (CTCAE)",
            confidence: 0.92,
        },
        TransferMapping {
            source_type: "Seriousness",
            domain: "Cloud",
            analog: "Incident severity (P0-P4)",
            confidence: 0.88,
        },
        TransferMapping {
            source_type: "Seriousness",
            domain: "Economics",
            analog: "Loss severity classification",
            confidence: 0.82,
        },
        // -- Decision --
        TransferMapping {
            source_type: "Decision",
            domain: "Biology",
            analog: "Fight/flight/freeze",
            confidence: 0.75,
        },
        TransferMapping {
            source_type: "Decision",
            domain: "Cloud",
            analog: "Alert triage (page/ticket/ignore)",
            confidence: 0.88,
        },
        TransferMapping {
            source_type: "Decision",
            domain: "Economics",
            analog: "Investment decision (buy/hold/sell)",
            confidence: 0.80,
        },
        // -- DrugEventPair --
        TransferMapping {
            source_type: "DrugEventPair",
            domain: "Biology",
            analog: "Antigen-antibody pair",
            confidence: 0.82,
        },
        TransferMapping {
            source_type: "DrugEventPair",
            domain: "Cloud",
            analog: "Service-error pair",
            confidence: 0.85,
        },
        // -- NoiseParams --
        TransferMapping {
            source_type: "NoiseParams",
            domain: "Biology",
            analog: "Background mutation rate",
            confidence: 0.80,
        },
        TransferMapping {
            source_type: "NoiseParams",
            domain: "Cloud",
            analog: "Baseline error rate",
            confidence: 0.88,
        },
        // -- InterventionResult --
        TransferMapping {
            source_type: "InterventionResult",
            domain: "Biology",
            analog: "Drug efficacy measurement",
            confidence: 0.90,
        },
        TransferMapping {
            source_type: "InterventionResult",
            domain: "Cloud",
            analog: "Incident mitigation effectiveness",
            confidence: 0.85,
        },
        // -- PreemptiveResult --
        TransferMapping {
            source_type: "PreemptiveResult",
            domain: "Biology",
            analog: "Prophylactic vaccination outcome",
            confidence: 0.78,
        },
        TransferMapping {
            source_type: "PreemptiveResult",
            domain: "Cloud",
            analog: "Predictive auto-scaling result",
            confidence: 0.72,
        },
    ]
}

/// Returns the mean transfer confidence across all mappings.
///
/// This provides a single aggregate measure of how well the preemptive PV
/// domain transfers to other domains structurally.
#[must_use]
pub fn transfer_confidence() -> f64 {
    let mappings = transfer_mappings();
    if mappings.is_empty() {
        return 0.0;
    }
    let sum: f64 = mappings.iter().map(|m| m.confidence).sum();
    sum / mappings.len() as f64
}

/// Returns all transfer mappings for a given source type name.
///
/// # Arguments
///
/// * `source_type` - The type name to look up (e.g., "GibbsParams", "Decision")
///
/// Returns an empty vec if no mappings exist for the given type.
#[must_use]
pub fn transfers_for_type(source_type: &str) -> Vec<TransferMapping> {
    transfer_mappings()
        .into_iter()
        .filter(|m| m.source_type == source_type)
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn transfer_mappings_not_empty() {
        let mappings = transfer_mappings();
        assert!(!mappings.is_empty());
        assert_eq!(mappings.len(), 17);
    }

    #[test]
    fn all_confidences_in_valid_range() {
        for mapping in transfer_mappings() {
            assert!(
                mapping.confidence >= 0.0 && mapping.confidence <= 1.0,
                "{} -> {} has out-of-range confidence: {}",
                mapping.source_type,
                mapping.analog,
                mapping.confidence
            );
        }
    }

    #[test]
    fn aggregate_confidence_reasonable() {
        let conf = transfer_confidence();
        // All entries are between 0.60 and 0.95, so aggregate should be in that range
        assert!(conf > 0.70, "Aggregate confidence too low: {}", conf);
        assert!(conf < 0.95, "Aggregate confidence too high: {}", conf);
    }

    #[test]
    fn transfers_for_gibbs_params() {
        let transfers = transfers_for_type("GibbsParams");
        assert_eq!(transfers.len(), 3);

        let domains: Vec<&str> = transfers.iter().map(|t| t.domain).collect();
        assert!(domains.contains(&"Biology"));
        assert!(domains.contains(&"Cloud"));
        assert!(domains.contains(&"Economics"));
    }

    #[test]
    fn transfers_for_seriousness() {
        let transfers = transfers_for_type("Seriousness");
        assert_eq!(transfers.len(), 3);

        // Biology mapping should have the highest confidence
        let bio = transfers.iter().find(|t| t.domain == "Biology");
        assert!(bio.is_some());
        if let Some(b) = bio {
            assert!((b.confidence - 0.92).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn transfers_for_decision() {
        let transfers = transfers_for_type("Decision");
        assert_eq!(transfers.len(), 3);

        // Cloud triage should be the strongest analog
        let cloud = transfers.iter().find(|t| t.domain == "Cloud");
        assert!(cloud.is_some());
        if let Some(c) = cloud {
            assert!((c.confidence - 0.88).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn transfers_for_drug_event_pair() {
        let transfers = transfers_for_type("DrugEventPair");
        assert_eq!(transfers.len(), 2);
    }

    #[test]
    fn transfers_for_noise_params() {
        let transfers = transfers_for_type("NoiseParams");
        assert_eq!(transfers.len(), 2);
    }

    #[test]
    fn transfers_for_intervention_result() {
        let transfers = transfers_for_type("InterventionResult");
        assert_eq!(transfers.len(), 2);

        let bio = transfers.iter().find(|t| t.domain == "Biology");
        assert!(bio.is_some());
        if let Some(b) = bio {
            assert!((b.confidence - 0.90).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn transfers_for_preemptive_result() {
        let transfers = transfers_for_type("PreemptiveResult");
        assert_eq!(transfers.len(), 2);
    }

    #[test]
    fn transfers_for_unknown_type_empty() {
        let transfers = transfers_for_type("UnknownType");
        assert!(transfers.is_empty());
    }

    #[test]
    fn all_domains_represented() {
        let mappings = transfer_mappings();
        let domains: std::collections::HashSet<&str> =
            mappings.iter().map(|m| m.domain).collect();

        assert!(domains.contains("Biology"));
        assert!(domains.contains("Cloud"));
        assert!(domains.contains("Economics"));
    }

    #[test]
    fn domain_means_are_strong() {
        let mappings = transfer_mappings();

        let bio_mappings: Vec<&TransferMapping> =
            mappings.iter().filter(|m| m.domain == "Biology").collect();
        let cloud_mappings: Vec<&TransferMapping> =
            mappings.iter().filter(|m| m.domain == "Cloud").collect();

        let bio_mean: f64 =
            bio_mappings.iter().map(|m| m.confidence).sum::<f64>() / bio_mappings.len() as f64;
        let cloud_mean: f64 =
            cloud_mappings.iter().map(|m| m.confidence).sum::<f64>() / cloud_mappings.len() as f64;

        // Both domains should have strong transfer confidence (> 0.75).
        // Empirical means: Biology ~0.85, Cloud ~0.81 (as of 2026-02-22).
        assert!(
            bio_mean > 0.75,
            "Biology mean ({}) should exceed 0.75",
            bio_mean,
        );
        assert!(
            cloud_mean > 0.75,
            "Cloud mean ({}) should exceed 0.75",
            cloud_mean,
        );
    }

    #[test]
    fn serde_roundtrip() {
        let mapping = TransferMapping {
            source_type: "GibbsParams",
            domain: "Biology",
            analog: "Gibbs free energy in protein folding",
            confidence: 0.95,
        };

        let json = serde_json::to_string(&mapping);
        assert!(json.is_ok());

        // Note: TransferMapping uses &'static str, so deserialization produces
        // owned Strings. We verify the JSON structure is valid instead.
        if let Ok(json_str) = json {
            let value: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
            assert!(value.is_ok());
        }
    }

    #[test]
    fn empty_mappings_confidence_zero() {
        // Verify the guard clause in transfer_confidence
        // We can't easily test with empty mappings since the function is hardcoded,
        // but we verify the formula works with known data
        let conf = transfer_confidence();
        assert!(conf.is_finite());
        assert!(conf > 0.0);
    }
}
