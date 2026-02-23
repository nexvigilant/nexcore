//! # Cross-Domain Transfer Mappings
//!
//! Maps signal pipeline concepts to analogs in Biology, Cloud/SRE, and Economics.
//!
//! ## T1 Primitive: Mapping (mu)
//!
//! Transfer is pure mu: a transformation from one domain's vocabulary to another.
//! Each `TransferMapping` is a single mu instance with a confidence score
//! indicating how faithfully the structural pattern transfers.
//!
//! ## Theory
//!
//! Cross-domain transfer is a core prediction of the Theory of Vigilance:
//! structures grounded in T1 primitives transfer more reliably because the
//! primitive substrate is domain-independent. A `ContingencyTable` (T2-C,
//! grounded in N + Product) transfers at 0.95 to clinical trials because
//! the 2x2 structure is identical — only the labels change.

use serde::{Deserialize, Serialize};

/// A single cross-domain transfer mapping.
///
/// Maps a source type in the signal pipeline to an analog concept
/// in another domain, with a confidence score (0.0 to 1.0).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferMapping {
    /// The signal pipeline type name (e.g., "Pipeline", "ContingencyTable").
    pub source_type: &'static str,
    /// Target domain (e.g., "Biology", "Cloud/SRE", "Economics").
    pub domain: &'static str,
    /// Analogous concept in the target domain.
    pub analog: &'static str,
    /// Transfer confidence: 0.0 (no transfer) to 1.0 (perfect isomorphism).
    pub confidence: f64,
}

impl TransferMapping {
    /// Create a new transfer mapping.
    #[must_use]
    pub const fn new(
        source_type: &'static str,
        domain: &'static str,
        analog: &'static str,
        confidence: f64,
    ) -> Self {
        Self {
            source_type,
            domain,
            analog,
            confidence,
        }
    }
}

// ---- Static mapping table ----

/// All cross-domain transfer mappings for signal pipeline types.
///
/// Each pipeline type maps to analogs in Biology, Cloud/SRE, and Economics
/// where the structural pattern meaningfully transfers.
static TRANSFER_TABLE: &[TransferMapping] = &[
    // Pipeline
    TransferMapping::new("Pipeline", "Biology", "Metabolic pathway", 0.85),
    TransferMapping::new("Pipeline", "Cloud/SRE", "CI/CD pipeline", 0.92),
    TransferMapping::new("Pipeline", "Economics", "Supply chain", 0.80),
    // ContingencyTable
    TransferMapping::new(
        "ContingencyTable",
        "Biology",
        "Clinical trial 2x2 table",
        0.95,
    ),
    TransferMapping::new("ContingencyTable", "Cloud/SRE", "Confusion matrix", 0.88),
    TransferMapping::new(
        "ContingencyTable",
        "Economics",
        "Cross-tabulation of market segments",
        0.82,
    ),
    // Alert
    TransferMapping::new("Alert", "Biology", "Cytokine storm alert", 0.82),
    TransferMapping::new("Alert", "Cloud/SRE", "PagerDuty incident", 0.92),
    TransferMapping::new("Alert", "Economics", "Market circuit breaker", 0.78),
    // DetectionResult
    TransferMapping::new("DetectionResult", "Biology", "Diagnostic test result", 0.88),
    TransferMapping::new(
        "DetectionResult",
        "Cloud/SRE",
        "Anomaly detection output",
        0.90,
    ),
    TransferMapping::new(
        "DetectionResult",
        "Economics",
        "Leading indicator signal",
        0.75,
    ),
    // SignalMetrics
    TransferMapping::new("SignalMetrics", "Biology", "Lab panel results", 0.85),
    TransferMapping::new("SignalMetrics", "Cloud/SRE", "SLI measurements", 0.88),
    TransferMapping::new(
        "SignalMetrics",
        "Economics",
        "Economic indicator dashboard",
        0.80,
    ),
    // RawReport
    TransferMapping::new("RawReport", "Biology", "Raw clinical specimen", 0.80),
    TransferMapping::new("RawReport", "Cloud/SRE", "Unprocessed log entry", 0.90),
    TransferMapping::new("RawReport", "Economics", "Raw transaction record", 0.85),
    // NormalizedEvent
    TransferMapping::new("NormalizedEvent", "Biology", "Standardized lab value", 0.85),
    TransferMapping::new("NormalizedEvent", "Cloud/SRE", "Structured log event", 0.90),
    TransferMapping::new(
        "NormalizedEvent",
        "Economics",
        "Normalized financial metric",
        0.83,
    ),
    // ThresholdConfig
    TransferMapping::new("ThresholdConfig", "Biology", "Normal lab ranges", 0.88),
    TransferMapping::new("ThresholdConfig", "Cloud/SRE", "SLO configuration", 0.92),
    TransferMapping::new(
        "ThresholdConfig",
        "Economics",
        "Regulatory capital thresholds",
        0.82,
    ),
    // DrugEventPair
    TransferMapping::new(
        "DrugEventPair",
        "Biology",
        "Genotype-phenotype association",
        0.78,
    ),
    TransferMapping::new("DrugEventPair", "Cloud/SRE", "Service-error pair", 0.85),
    TransferMapping::new(
        "DrugEventPair",
        "Economics",
        "Cause-effect variable pair",
        0.72,
    ),
];

/// Return all cross-domain transfer mappings.
#[must_use]
pub fn transfer_mappings() -> &'static [TransferMapping] {
    TRANSFER_TABLE
}

/// Compute the average transfer confidence for a given source type.
///
/// Returns `None` if no mappings exist for the source type.
#[must_use]
pub fn transfer_confidence(source_type: &str) -> Option<f64> {
    let matches: Vec<f64> = TRANSFER_TABLE
        .iter()
        .filter(|m| m.source_type == source_type)
        .map(|m| m.confidence)
        .collect();

    if matches.is_empty() {
        return None;
    }

    let sum: f64 = matches.iter().sum();
    Some(sum / matches.len() as f64)
}

/// Return all transfer mappings for a given source type.
#[must_use]
pub fn transfers_for_type(source_type: &str) -> Vec<&'static TransferMapping> {
    TRANSFER_TABLE
        .iter()
        .filter(|m| m.source_type == source_type)
        .collect()
}

/// Return all transfer mappings for a given target domain.
#[must_use]
pub fn transfers_for_domain(domain: &str) -> Vec<&'static TransferMapping> {
    TRANSFER_TABLE
        .iter()
        .filter(|m| m.domain == domain)
        .collect()
}

/// Return the highest-confidence transfer for a given source type.
#[must_use]
pub fn best_transfer(source_type: &str) -> Option<&'static TransferMapping> {
    TRANSFER_TABLE
        .iter()
        .filter(|m| m.source_type == source_type)
        .max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_table_is_not_empty() {
        let all = transfer_mappings();
        assert!(!all.is_empty());
        // 9 source types x 3 domains = 27 mappings
        assert_eq!(all.len(), 27);
    }

    #[test]
    fn every_mapping_has_positive_confidence() {
        for mapping in transfer_mappings() {
            assert!(
                mapping.confidence > 0.0 && mapping.confidence <= 1.0,
                "Invalid confidence {} for {} -> {} ({})",
                mapping.confidence,
                mapping.source_type,
                mapping.analog,
                mapping.domain,
            );
        }
    }

    #[test]
    fn transfer_confidence_pipeline() {
        let conf = transfer_confidence("Pipeline");
        assert!(conf.is_some());
        let c = conf.unwrap_or(0.0);
        // (0.85 + 0.92 + 0.80) / 3 = 0.8567
        assert!(c > 0.85 && c < 0.87, "Expected ~0.857, got {c}");
    }

    #[test]
    fn transfer_confidence_contingency_table() {
        let conf = transfer_confidence("ContingencyTable");
        assert!(conf.is_some());
        let c = conf.unwrap_or(0.0);
        // (0.95 + 0.88 + 0.82) / 3 = 0.8833
        assert!(c > 0.87 && c < 0.90, "Expected ~0.883, got {c}");
    }

    #[test]
    fn transfer_confidence_unknown_type_returns_none() {
        assert!(transfer_confidence("NonExistentType").is_none());
    }

    #[test]
    fn transfers_for_type_alert() {
        let results = transfers_for_type("Alert");
        assert_eq!(results.len(), 3);
        let domains: Vec<&str> = results.iter().map(|m| m.domain).collect();
        assert!(domains.contains(&"Biology"));
        assert!(domains.contains(&"Cloud/SRE"));
        assert!(domains.contains(&"Economics"));
    }

    #[test]
    fn transfers_for_type_unknown_returns_empty() {
        let results = transfers_for_type("FakeType");
        assert!(results.is_empty());
    }

    #[test]
    fn transfers_for_domain_biology() {
        let bio = transfers_for_domain("Biology");
        // One mapping per source type for Biology = 9
        assert_eq!(bio.len(), 9);
        for m in &bio {
            assert_eq!(m.domain, "Biology");
        }
    }

    #[test]
    fn transfers_for_domain_cloud_sre() {
        let cloud = transfers_for_domain("Cloud/SRE");
        assert_eq!(cloud.len(), 9);
    }

    #[test]
    fn best_transfer_contingency_table_is_biology() {
        let best = best_transfer("ContingencyTable");
        assert!(best.is_some());
        if let Some(b) = best {
            assert_eq!(b.domain, "Biology");
            assert_eq!(b.analog, "Clinical trial 2x2 table");
            assert!((b.confidence - 0.95).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn best_transfer_unknown_returns_none() {
        assert!(best_transfer("DoesNotExist").is_none());
    }

    #[test]
    fn all_source_types_have_three_domains() {
        let source_types = [
            "Pipeline",
            "ContingencyTable",
            "Alert",
            "DetectionResult",
            "SignalMetrics",
            "RawReport",
            "NormalizedEvent",
            "ThresholdConfig",
            "DrugEventPair",
        ];
        for st in &source_types {
            let mappings = transfers_for_type(st);
            assert_eq!(
                mappings.len(),
                3,
                "{st} should have exactly 3 domain mappings, got {}",
                mappings.len()
            );
        }
    }

    #[test]
    fn contingency_table_highest_confidence() {
        // ContingencyTable -> Biology should be the highest single mapping
        // because 2x2 tables are isomorphic across clinical trials.
        let ct_bio = transfers_for_type("ContingencyTable")
            .into_iter()
            .find(|m| m.domain == "Biology");
        assert!(ct_bio.is_some());
        if let Some(mapping) = ct_bio {
            assert!(
                mapping.confidence >= 0.95,
                "ContingencyTable -> Biology should be >= 0.95"
            );
        }
    }

    #[test]
    fn transfer_mapping_serialization() {
        let mapping = TransferMapping::new("Pipeline", "Biology", "Metabolic pathway", 0.85);
        let json = serde_json::to_string(&mapping);
        assert!(json.is_ok());
        let json_str = json.unwrap_or_default();
        // Verify JSON structure via serde_json::Value (avoids 'static lifetime)
        let value: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
        assert!(value.is_ok());
        if let Ok(v) = value {
            assert_eq!(v["source_type"], "Pipeline");
            assert_eq!(v["domain"], "Biology");
            assert!((v["confidence"].as_f64().unwrap_or(0.0) - 0.85).abs() < f64::EPSILON);
        }
    }
}
