//! # Signal Report
//!
//! Report generation from detection results.
//! Supports JSON summary and tabular text output.
//!
//! ## T1 Primitives: Recursion (ρ) + Mapping (μ) + Sequence (σ)
//! - **Recursion (ρ)**: Problem reduction via hierarchical aggregation (results → sections → full report).
//! - **Mapping (μ)**: Transformation of internal data models into JSON or tabular representations.
//! - **Sequence (σ)**: Ordered formatting of records within the output stream.

use crate::core::{DetectionResult, Report, Result, SignalStrength};

/// JSON summary reporter.
pub struct JsonReporter;

impl Report for JsonReporter {
    fn report(&self, results: &[DetectionResult]) -> Result<String> {
        let summary = serde_json::json!({
            "total_pairs": results.len(),
            "signals": results.iter().filter(|r| r.strength >= SignalStrength::Moderate).count(),
            "critical": results.iter().filter(|r| r.strength == SignalStrength::Critical).count(),
            "results": results.iter().map(|r| serde_json::json!({
                "drug": r.pair.drug,
                "event": r.pair.event,
                "prr": r.prr.map(|p| p.0),
                "ror": r.ror.map(|p| p.0),
                "chi_square": r.chi_square.0,
                "strength": format!("{:?}", r.strength),
                "cases": r.table.a,
            })).collect::<Vec<_>>(),
        });
        serde_json::to_string_pretty(&summary)
            .map_err(|e| crate::core::SignalError::Detection(e.to_string()))
    }
}

/// Plain-text tabular reporter.
pub struct TableReporter;

impl Report for TableReporter {
    fn report(&self, results: &[DetectionResult]) -> Result<String> {
        let mut out = String::new();
        out.push_str(&format!(
            "{:<20} {:<20} {:>8} {:>8} {:>8} {:>10}\n",
            "Drug", "Event", "PRR", "ROR", "Chi-Sq", "Strength"
        ));
        out.push_str(&"-".repeat(78));
        out.push('\n');

        for r in results {
            let prr_str = r.prr.map_or("N/A".to_owned(), |p| format!("{:.2}", p.0));
            let ror_str = r.ror.map_or("N/A".to_owned(), |p| format!("{:.2}", p.0));
            out.push_str(&format!(
                "{:<20} {:<20} {:>8} {:>8} {:>8.2} {:>10}\n",
                truncate(&r.pair.drug, 20),
                truncate(&r.pair.event, 20),
                prr_str,
                ror_str,
                r.chi_square.0,
                format!("{:?}", r.strength),
            ));
        }

        out.push_str(&format!(
            "\nTotal: {} pairs, {} signals detected\n",
            results.len(),
            results
                .iter()
                .filter(|r| r.strength >= SignalStrength::Moderate)
                .count(),
        ));
        Ok(out)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_owned()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use nexcore_chrono::DateTime;

    fn make_results() -> Vec<DetectionResult> {
        vec![DetectionResult {
            pair: DrugEventPair::new("aspirin", "bleeding"),
            table: ContingencyTable {
                a: 15,
                b: 100,
                c: 20,
                d: 10_000,
            },
            prr: Some(Prr(3.5)),
            ror: Some(Ror(7.5)),
            ic: Some(Ic(1.8)),
            ebgm: Some(Ebgm(2.5)),
            chi_square: ChiSquare(12.0),
            strength: SignalStrength::Strong,
            detected_at: DateTime::now(),
        }]
    }

    #[test]
    fn json_report_valid() {
        let reporter = JsonReporter;
        let output = reporter.report(&make_results()).unwrap();
        assert!(output.contains("aspirin"));
        assert!(output.contains("bleeding"));
    }

    #[test]
    fn table_report_has_header() {
        let reporter = TableReporter;
        let output = reporter.report(&make_results()).unwrap();
        assert!(output.contains("Drug"));
        assert!(output.contains("PRR"));
    }
}
