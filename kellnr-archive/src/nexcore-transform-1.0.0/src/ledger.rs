//! Transfer ledger: formatted mapping report with summary stats.
//!
//! The ledger is the human-readable artifact summarizing all
//! concept mappings, their methods, and confidence levels.

use crate::mapping::{MappingMethod, MappingTable};
use serde::{Deserialize, Serialize};

/// A single row in the transfer ledger.
///
/// Tier: T2-P | Dominant: mu (Mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Source concept.
    pub source: String,
    /// Target domain term.
    pub target: String,
    /// Confidence (0.0..=1.0).
    pub confidence: f64,
    /// Mapping method label.
    pub method: String,
}

/// The complete transfer ledger.
///
/// Tier: T2-C | Dominant: sigma (Sequence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferLedger {
    /// All ledger entries in order.
    pub entries: Vec<LedgerEntry>,
    /// Summary statistics.
    pub summary: LedgerSummary,
}

/// Summary statistics for the ledger.
///
/// Tier: T2-P | Dominant: N (Quantity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSummary {
    /// Total concepts.
    pub total: usize,
    /// Bridged (deterministic) mappings.
    pub bridged: usize,
    /// LLM-assisted mappings.
    pub llm_assisted: usize,
    /// Unmapped concepts.
    pub unmapped: usize,
    /// Aggregate confidence.
    pub aggregate_confidence: f64,
}

/// Build a transfer ledger from a mapping table.
pub fn build_ledger(table: &MappingTable) -> TransferLedger {
    let entries: Vec<LedgerEntry> = table
        .mappings
        .iter()
        .map(|m| LedgerEntry {
            source: m.source.clone(),
            target: if m.target.is_empty() {
                "(unmapped)".to_string()
            } else {
                m.target.clone()
            },
            confidence: m.confidence,
            method: match m.method {
                MappingMethod::Bridge => "Bridge".to_string(),
                MappingMethod::LlmAssisted => "LLM".to_string(),
                MappingMethod::Unmapped => "Unmapped".to_string(),
            },
        })
        .collect();

    let bridged = table
        .mappings
        .iter()
        .filter(|m| m.method == MappingMethod::Bridge)
        .count();
    let llm_assisted = table
        .mappings
        .iter()
        .filter(|m| m.method == MappingMethod::LlmAssisted)
        .count();

    let summary = LedgerSummary {
        total: table.mappings.len(),
        bridged,
        llm_assisted,
        unmapped: table.unmapped_count,
        aggregate_confidence: table.aggregate_confidence,
    };

    TransferLedger { entries, summary }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::annotate;
    use crate::mapping::build_mapping_table;
    use crate::profile::builtin_pharmacovigilance;
    use crate::segment::segment;

    #[test]
    fn test_build_ledger_basic() {
        let pv = builtin_pharmacovigilance();
        let source = segment("Test", "The citizen faces danger.");
        let annotations = annotate(&source, &pv, None);
        let table = build_mapping_table(&annotations, &pv, None);
        let ledger = build_ledger(&table);

        assert!(!ledger.entries.is_empty());
        assert_eq!(ledger.summary.total, ledger.entries.len());
    }

    #[test]
    fn test_ledger_unmapped_label() {
        let table = MappingTable {
            mappings: vec![crate::mapping::ConceptMapping {
                source: "novelty".into(),
                target: String::new(),
                confidence: 0.0,
                method: MappingMethod::Unmapped,
            }],
            aggregate_confidence: 0.0,
            unmapped_count: 1,
        };
        let ledger = build_ledger(&table);
        assert_eq!(ledger.entries[0].target, "(unmapped)");
        assert_eq!(ledger.entries[0].method, "Unmapped");
        assert_eq!(ledger.summary.unmapped, 1);
    }

    #[test]
    fn test_ledger_summary_counts() {
        let pv = builtin_pharmacovigilance();
        let source = segment("Test", "The citizen faces danger with vigilance.");
        let annotations = annotate(&source, &pv, None);
        let table = build_mapping_table(&annotations, &pv, None);
        let ledger = build_ledger(&table);

        assert_eq!(
            ledger.summary.bridged + ledger.summary.llm_assisted + ledger.summary.unmapped,
            ledger.summary.total
        );
    }

    #[test]
    fn test_empty_ledger() {
        let table = MappingTable {
            mappings: vec![],
            aggregate_confidence: 0.0,
            unmapped_count: 0,
        };
        let ledger = build_ledger(&table);
        assert!(ledger.entries.is_empty());
        assert_eq!(ledger.summary.total, 0);
    }
}
