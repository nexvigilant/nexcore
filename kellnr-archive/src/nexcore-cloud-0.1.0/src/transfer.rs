//! # Cross-Domain Transfer Confidence
//!
//! Maps cloud computing concepts to other domains (PV, Biology, Economics)
//! with calibrated transfer confidence scores.

use serde::{Deserialize, Serialize};

/// A cross-domain transfer mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferMapping {
    /// Cloud type name
    pub cloud_type: &'static str,
    /// Target domain
    pub domain: &'static str,
    /// Analogous concept in target domain
    pub analog: &'static str,
    /// Transfer confidence (0.0-1.0)
    pub confidence: f64,
}

/// All cross-domain transfer mappings for cloud primitives.
pub fn transfer_mappings() -> Vec<TransferMapping> {
    vec![
        // T1: Identity
        TransferMapping {
            cloud_type: "Identity",
            domain: "PV",
            analog: "Case ID / Patient ID",
            confidence: 0.95,
        },
        TransferMapping {
            cloud_type: "Identity",
            domain: "Biology",
            analog: "Genome / DNA fingerprint",
            confidence: 0.90,
        },
        TransferMapping {
            cloud_type: "Identity",
            domain: "Economics",
            analog: "Account number / SSN",
            confidence: 0.92,
        },
        // T1: Threshold
        TransferMapping {
            cloud_type: "Threshold",
            domain: "PV",
            analog: "Signal detection threshold (PRR >= 2.0)",
            confidence: 0.95,
        },
        TransferMapping {
            cloud_type: "Threshold",
            domain: "Biology",
            analog: "Action potential threshold",
            confidence: 0.90,
        },
        TransferMapping {
            cloud_type: "Threshold",
            domain: "Economics",
            analog: "Price floor / ceiling",
            confidence: 0.88,
        },
        // T1: FeedbackLoop
        TransferMapping {
            cloud_type: "FeedbackLoop",
            domain: "PV",
            analog: "Signal-action-outcome loop",
            confidence: 0.88,
        },
        TransferMapping {
            cloud_type: "FeedbackLoop",
            domain: "Biology",
            analog: "Homeostasis (thermoregulation)",
            confidence: 0.95,
        },
        TransferMapping {
            cloud_type: "FeedbackLoop",
            domain: "Economics",
            analog: "Supply-demand equilibrium",
            confidence: 0.85,
        },
        // T1: Idempotency
        TransferMapping {
            cloud_type: "Idempotency",
            domain: "PV",
            analog: "Duplicate report detection",
            confidence: 0.90,
        },
        TransferMapping {
            cloud_type: "Idempotency",
            domain: "Biology",
            analog: "Vaccine booster plateau",
            confidence: 0.75,
        },
        TransferMapping {
            cloud_type: "Idempotency",
            domain: "Economics",
            analog: "Exactly-once payment processing",
            confidence: 0.92,
        },
        // T1: Immutability
        TransferMapping {
            cloud_type: "Immutability",
            domain: "PV",
            analog: "Locked regulatory submission",
            confidence: 0.92,
        },
        TransferMapping {
            cloud_type: "Immutability",
            domain: "Biology",
            analog: "Fossilized DNA / amber",
            confidence: 0.70,
        },
        TransferMapping {
            cloud_type: "Immutability",
            domain: "Economics",
            analog: "Executed contract / ledger entry",
            confidence: 0.88,
        },
        // T1: Convergence
        TransferMapping {
            cloud_type: "Convergence",
            domain: "PV",
            analog: "Consensus causality assessment",
            confidence: 0.85,
        },
        TransferMapping {
            cloud_type: "Convergence",
            domain: "Biology",
            analog: "Tissue homeostasis",
            confidence: 0.82,
        },
        TransferMapping {
            cloud_type: "Convergence",
            domain: "Economics",
            analog: "Market price discovery",
            confidence: 0.80,
        },
        // T2-P: Compute
        TransferMapping {
            cloud_type: "Compute",
            domain: "PV",
            analog: "Case processing capacity",
            confidence: 0.85,
        },
        TransferMapping {
            cloud_type: "Compute",
            domain: "Biology",
            analog: "Metabolic rate / ATP production",
            confidence: 0.88,
        },
        TransferMapping {
            cloud_type: "Compute",
            domain: "Economics",
            analog: "Labor capacity / productivity",
            confidence: 0.82,
        },
        // T2-P: Storage
        TransferMapping {
            cloud_type: "Storage",
            domain: "PV",
            analog: "Case database / FAERS",
            confidence: 0.90,
        },
        TransferMapping {
            cloud_type: "Storage",
            domain: "Biology",
            analog: "DNA / memory engrams",
            confidence: 0.85,
        },
        TransferMapping {
            cloud_type: "Storage",
            domain: "Economics",
            analog: "Warehouse / inventory",
            confidence: 0.82,
        },
        // T2-P: ResourcePool
        TransferMapping {
            cloud_type: "ResourcePool",
            domain: "PV",
            analog: "Reviewer pool",
            confidence: 0.88,
        },
        TransferMapping {
            cloud_type: "ResourcePool",
            domain: "Biology",
            analog: "Blood supply / stem cells",
            confidence: 0.85,
        },
        TransferMapping {
            cloud_type: "ResourcePool",
            domain: "Economics",
            analog: "Capital pool / labor market",
            confidence: 0.90,
        },
        // T2-P: Queue
        TransferMapping {
            cloud_type: "Queue",
            domain: "PV",
            analog: "Case processing backlog",
            confidence: 0.92,
        },
        TransferMapping {
            cloud_type: "Queue",
            domain: "Biology",
            analog: "Synaptic vesicle pool",
            confidence: 0.78,
        },
        TransferMapping {
            cloud_type: "Queue",
            domain: "Economics",
            analog: "Order book / wait list",
            confidence: 0.88,
        },
        // T2-P: HealthCheck
        TransferMapping {
            cloud_type: "HealthCheck",
            domain: "PV",
            analog: "System uptime monitoring",
            confidence: 0.85,
        },
        TransferMapping {
            cloud_type: "HealthCheck",
            domain: "Biology",
            analog: "Pulse / vital signs",
            confidence: 0.90,
        },
        TransferMapping {
            cloud_type: "HealthCheck",
            domain: "Economics",
            analog: "Financial audit / stress test",
            confidence: 0.82,
        },
        // T2-P: Lease
        TransferMapping {
            cloud_type: "Lease",
            domain: "PV",
            analog: "Case lock / processing window",
            confidence: 0.82,
        },
        TransferMapping {
            cloud_type: "Lease",
            domain: "Biology",
            analog: "Receptor binding duration",
            confidence: 0.78,
        },
        TransferMapping {
            cloud_type: "Lease",
            domain: "Economics",
            analog: "Rental agreement / option contract",
            confidence: 0.92,
        },
        // T2-P: Encryption
        TransferMapping {
            cloud_type: "Encryption",
            domain: "PV",
            analog: "Data anonymization / pseudonymization",
            confidence: 0.80,
        },
        TransferMapping {
            cloud_type: "Encryption",
            domain: "Biology",
            analog: "Protein folding / genetic code",
            confidence: 0.65,
        },
        TransferMapping {
            cloud_type: "Encryption",
            domain: "Economics",
            analog: "Information asymmetry / trade secrets",
            confidence: 0.72,
        },
    ]
}

/// Lookup transfer confidence for a specific cloud type to a target domain.
pub fn transfer_confidence(cloud_type: &str, domain: &str) -> Option<f64> {
    transfer_mappings()
        .iter()
        .find(|m| m.cloud_type == cloud_type && m.domain == domain)
        .map(|m| m.confidence)
}

/// Get all transfer mappings for a specific cloud type.
pub fn transfers_for_type(cloud_type: &str) -> Vec<TransferMapping> {
    transfer_mappings()
        .into_iter()
        .filter(|m| m.cloud_type == cloud_type)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_confidences_in_range() {
        for mapping in transfer_mappings() {
            assert!(
                mapping.confidence >= 0.0 && mapping.confidence <= 1.0,
                "{} -> {} confidence {} out of range",
                mapping.cloud_type,
                mapping.domain,
                mapping.confidence
            );
        }
    }

    #[test]
    fn test_lookup_existing() {
        let conf = transfer_confidence("Identity", "PV");
        assert!(conf.is_some());
        assert!((conf.unwrap_or(0.0) - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lookup_missing() {
        let conf = transfer_confidence("NonExistent", "PV");
        assert!(conf.is_none());
    }

    #[test]
    fn test_transfers_for_type() {
        let transfers = transfers_for_type("Identity");
        assert_eq!(transfers.len(), 3);
        assert!(transfers.iter().all(|t| t.cloud_type == "Identity"));
    }

    #[test]
    fn test_transfers_for_unknown_type() {
        let transfers = transfers_for_type("Unknown");
        assert!(transfers.is_empty());
    }

    #[test]
    fn test_three_domains_per_t1() {
        let t1_types = [
            "Identity",
            "Threshold",
            "FeedbackLoop",
            "Idempotency",
            "Immutability",
            "Convergence",
        ];
        for type_name in &t1_types {
            let transfers = transfers_for_type(type_name);
            assert_eq!(
                transfers.len(),
                3,
                "{} should have 3 domain mappings, got {}",
                type_name,
                transfers.len()
            );
        }
    }

    #[test]
    fn test_serde_transfer_mapping() {
        let mapping = TransferMapping {
            cloud_type: "Compute",
            domain: "PV",
            analog: "Case processing",
            confidence: 0.85,
        };
        let json = serde_json::to_string(&mapping).unwrap_or_default();
        assert!(!json.is_empty());
        assert!(json.contains("Compute"));
    }
}
