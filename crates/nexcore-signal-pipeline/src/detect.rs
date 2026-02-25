//! # Signal Detect
//!
//! Detection orchestration: builds contingency tables from normalized events
//! and runs statistical analysis. Sliding window support for temporal detection.

use crate::core::{
    ChiSquare, ContingencyTable, Detect, DetectionResult, DrugEventPair, Ebgm, Ic, NormalizedEvent,
    Prr, Result, Ror, SignalStrength,
};
use nexcore_chrono::DateTime;
use std::collections::BTreeMap;

/// Builds contingency tables from normalized events and computes metrics.
#[non_exhaustive]
pub struct TableDetector;

impl TableDetector {
    /// Build contingency tables for all drug-event pairs in the dataset.
    #[allow(
        clippy::as_conversions,
        reason = "usize->u64 cast for dataset length; dataset size never exceeds u64::MAX"
    )]
    pub fn build_tables(events: &[NormalizedEvent]) -> BTreeMap<DrugEventPair, ContingencyTable> {
        let total = events.len() as u64;
        let mut drug_event_counts: BTreeMap<(String, String), u64> = BTreeMap::new();
        let mut drug_counts: BTreeMap<String, u64> = BTreeMap::new();
        let mut event_counts: BTreeMap<String, u64> = BTreeMap::new();

        for ev in events {
            let de_count = drug_event_counts
                .entry((ev.drug.clone(), ev.event.clone()))
                .or_default();
            *de_count = de_count.saturating_add(1);
            let d_count = drug_counts.entry(ev.drug.clone()).or_default();
            *d_count = d_count.saturating_add(1);
            let e_count = event_counts.entry(ev.event.clone()).or_default();
            *e_count = e_count.saturating_add(1);
        }

        let mut tables = BTreeMap::new();
        for ((drug, event), a) in &drug_event_counts {
            let drug_total = drug_counts.get(drug).copied().unwrap_or(0);
            let event_total = event_counts.get(event).copied().unwrap_or(0);
            let b = drug_total.saturating_sub(*a);
            let c = event_total.saturating_sub(*a);
            let d = total.saturating_sub(a.saturating_add(b).saturating_add(c));
            tables.insert(
                DrugEventPair::new(drug, event),
                ContingencyTable { a: *a, b, c, d },
            );
        }
        tables
    }
}

impl Detect for TableDetector {
    #[allow(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "u64->f64 cast is intentional for statistical computation; saturating ops prevent overflow"
    )]
    fn detect(&self, events: &[NormalizedEvent]) -> Result<Vec<DetectionResult>> {
        let tables = Self::build_tables(events);
        let mut results = Vec::with_capacity(tables.len());

        for (pair, table) in tables {
            let prr = table.prr().map(Prr);
            let ror = table.ror().map(Ror);
            let n = table.total() as f64;
            let expected = if n > 0.0 {
                table.a.saturating_add(table.b) as f64 * table.a.saturating_add(table.c) as f64 / n
            } else {
                0.0
            };
            let ic = if expected > 0.0 {
                Some(Ic((table.a as f64 / expected).log2()))
            } else {
                None
            };
            let ebgm = Some(Ebgm((table.a as f64 + 0.5) / (expected + 0.5)));
            let chi_sq = ChiSquare(table.chi_square());
            let strength = prr.map_or(SignalStrength::None, |p| SignalStrength::from_prr(p.0));

            results.push(DetectionResult {
                pair,
                table,
                prr,
                ror,
                ic,
                ebgm,
                chi_square: chi_sq,
                strength,
                detected_at: DateTime::now(),
            });
        }
        results.sort_by(|a, b| b.strength.cmp(&a.strength));
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{NormalizedEvent, ReportSource};
    use nexcore_id::NexId;

    fn add_evs(evs: &mut Vec<NormalizedEvent>, drug: &str, event: &str, count: usize) {
        for _ in 0..count {
            evs.push(NormalizedEvent {
                id: NexId::v4(),
                drug: drug.into(),
                event: event.into(),
                meddra_pt: None,
                meddra_soc: None,
                report_date: DateTime::now(),
                source: ReportSource::Faers,
            });
        }
    }

    fn make_events() -> Vec<NormalizedEvent> {
        let mut evs = Vec::new();
        add_evs(&mut evs, "aspirin", "bleeding", 100);
        add_evs(&mut evs, "aspirin", "headache", 10);
        add_evs(&mut evs, "ibuprofen", "bleeding", 10);
        add_evs(&mut evs, "ibuprofen", "headache", 1000);
        evs
    }

    #[test]
    fn detect_builds_tables() {
        let tables = TableDetector::build_tables(&make_events());
        assert!(tables.contains_key(&DrugEventPair::new("aspirin", "bleeding")));
    }

    #[test]
    fn detect_returns_results() {
        let results = TableDetector.detect(&make_events()).unwrap(); // INVARIANT: test
        let res = results
            .iter()
            .find(|r| r.pair.drug == "aspirin" && r.pair.event == "bleeding")
            .unwrap(); // INVARIANT: test
        assert!(res.prr.unwrap().0 > 1.0); // INVARIANT: test
    }

    #[test]
    fn detect_empty_events() {
        assert!(TableDetector.detect(&[]).unwrap().is_empty()); // INVARIANT: test
    }

    #[test]
    fn test_building_table_values() {
        let evs = vec![
            NormalizedEvent {
                id: NexId::v4(),
                drug: "A".into(),
                event: "X".into(),
                meddra_pt: None,
                meddra_soc: None,
                report_date: DateTime::now(),
                source: ReportSource::Spontaneous,
            },
            NormalizedEvent {
                id: NexId::v4(),
                drug: "B".into(),
                event: "Y".into(),
                meddra_pt: None,
                meddra_soc: None,
                report_date: DateTime::now(),
                source: ReportSource::Spontaneous,
            },
        ];
        let t = TableDetector::build_tables(&evs);
        let ta = t.get(&DrugEventPair::new("A", "X")).unwrap(); // INVARIANT: test
        assert_eq!(ta.a, 1);
        assert_eq!(ta.d, 1);
    }
}
