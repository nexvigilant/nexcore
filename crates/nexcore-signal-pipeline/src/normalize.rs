//! # Signal Normalize
//!
//! Normalizes raw reports into standardized `NormalizedEvent`s.
//! Handles drug name standardization and `MedDRA` preferred term mapping.
//!
//! ## T1 Primitives: Mapping (μ) + Sequence (σ)
//! - **Mapping (μ)**: Pure transformation of `RawReport` fields into standardized `NormalizedEvent` attributes.
//! - **Sequence (σ)**: Iterative processing of drug/event combinations within the pipeline stage.

use crate::core::{Normalize, NormalizedEvent, RawReport, Result};
use chrono::Utc;
use nexcore_id::NexId;

/// Basic normalizer that lowercases and trims drug/event names.
pub struct BasicNormalizer;

impl Normalize for BasicNormalizer {
    fn normalize(&self, report: &RawReport) -> Result<Vec<NormalizedEvent>> {
        let mut events = Vec::new();
        for drug in &report.drug_names {
            for event in &report.event_terms {
                events.push(NormalizedEvent {
                    id: NexId::v4(),
                    drug: drug.trim().to_lowercase(),
                    event: event.trim().to_lowercase(),
                    meddra_pt: Some(event.trim().to_lowercase()),
                    meddra_soc: None,
                    report_date: report.report_date.unwrap_or_else(Utc::now),
                    source: report.source.clone(),
                });
            }
        }
        Ok(events)
    }
}

/// Normalizer with a simple synonym dictionary for drug names.
pub struct SynonymNormalizer {
    synonyms: std::collections::HashMap<String, String>,
}

impl SynonymNormalizer {
    /// Create with a synonym map (key=alias, value=canonical name).
    pub fn new(synonyms: std::collections::HashMap<String, String>) -> Self {
        Self { synonyms }
    }

    fn canonical_drug(&self, name: &str) -> String {
        let lower = name.trim().to_lowercase();
        self.synonyms.get(&lower).cloned().unwrap_or(lower)
    }
}

impl Normalize for SynonymNormalizer {
    fn normalize(&self, report: &RawReport) -> Result<Vec<NormalizedEvent>> {
        let mut events = Vec::new();
        for drug in &report.drug_names {
            for event in &report.event_terms {
                events.push(NormalizedEvent {
                    id: NexId::v4(),
                    drug: self.canonical_drug(drug),
                    event: event.trim().to_lowercase(),
                    meddra_pt: Some(event.trim().to_lowercase()),
                    meddra_soc: None,
                    report_date: report.report_date.unwrap_or_else(Utc::now),
                    source: report.source.clone(),
                });
            }
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ReportSource;

    fn test_report() -> RawReport {
        RawReport {
            id: "test-1".into(),
            drug_names: vec!["Aspirin".into(), "ASA".into()],
            event_terms: vec!["GI Bleeding".into()],
            report_date: Some(Utc::now()),
            source: ReportSource::Faers,
            metadata: serde_json::Value::Null,
        }
    }

    #[test]
    fn basic_normalize() {
        let norm = BasicNormalizer;
        let events = norm.normalize(&test_report()).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].drug, "aspirin");
        assert_eq!(events[0].event, "gi bleeding");
    }

    #[test]
    fn synonym_normalize() {
        let mut syns = std::collections::HashMap::new();
        syns.insert("asa".into(), "aspirin".into());
        let norm = SynonymNormalizer::new(syns);
        let events = norm.normalize(&test_report()).unwrap(); // INVARIANT: test
        assert!(events.iter().all(|e| e.drug == "aspirin"));
    }

    #[test]
    fn normalize_no_date() {
        let mut report = test_report();
        report.report_date = None;
        let events = BasicNormalizer.normalize(&report).unwrap(); // INVARIANT: test
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn normalize_empty_report() {
        let report = RawReport {
            id: "empty".into(),
            drug_names: vec![],
            event_terms: vec![],
            report_date: None,
            source: crate::core::ReportSource::Faers,
            metadata: serde_json::Value::Null,
        };
        let events = BasicNormalizer.normalize(&report).unwrap(); // INVARIANT: test
        assert!(events.is_empty());
    }

    #[test]
    fn test_canonical_drug_logic() {
        let mut syns = std::collections::HashMap::new();
        syns.insert("tylenol".into(), "acetaminophen".into());
        let norm = SynonymNormalizer::new(syns);
        assert_eq!(norm.canonical_drug("  TYLENOL  "), "acetaminophen");
        assert_eq!(norm.canonical_drug("Advantix"), "advantix");
    }
}
