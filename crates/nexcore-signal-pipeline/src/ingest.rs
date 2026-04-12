//! # Signal Ingest
//!
//! Raw data ingestion for the signal detection pipeline.
//! Supports FAERS JSON, CSV, and custom sources.
//!
//! ## T1 Primitive: Sequence
//! Ingestion is a pure sequence operation: source → parse → emit `RawReport`s.
//!
//! ## Example
//! ```rust,no_run
//! use nexcore_signal_pipeline::ingest::JsonIngestor;
//! use nexcore_signal_pipeline::core::Ingest;
//!
//! let ingestor = JsonIngestor::from_str(r#"[{"id":"1","drugs":["aspirin"],"events":["bleeding"]}]"#);
//! let reports = ingestor.ingest().ok();
//! assert!(reports.is_some());
//! ```

use crate::core::{Ingest, RawReport, ReportSource, Result, SignalError};
use nexcore_chrono::DateTime;

/// Ingests reports from a JSON string (FAERS-like format).
pub struct JsonIngestor {
    data: String,
}

impl JsonIngestor {
    /// Create from a JSON string.
    pub fn from_str(data: &str) -> Self {
        Self {
            data: data.to_owned(),
        }
    }
}

/// JSON record format for ingestion.
#[derive(serde::Deserialize)]
struct JsonRecord {
    id: String,
    drugs: Vec<String>,
    events: Vec<String>,
    #[serde(default)]
    source: Option<String>,
}

impl Ingest for JsonIngestor {
    fn ingest(&self) -> Result<Vec<RawReport>> {
        let records: Vec<JsonRecord> =
            serde_json::from_str(&self.data).map_err(|e| SignalError::Ingestion(e.to_string()))?;
        Ok(records
            .into_iter()
            .map(|r| RawReport {
                id: r.id,
                drug_names: r.drugs,
                event_terms: r.events,
                report_date: Some(DateTime::now()),
                source: r.source.map_or(ReportSource::Faers, ReportSource::Unknown),
                metadata: serde_json::Value::Null,
            })
            .collect())
    }
}

/// Ingests reports from CSV text (drug,event per line).
pub struct CsvIngestor {
    data: String,
}

impl CsvIngestor {
    /// Create from CSV text.
    pub fn from_str(data: &str) -> Self {
        Self {
            data: data.to_owned(),
        }
    }
}

impl Ingest for CsvIngestor {
    fn ingest(&self) -> Result<Vec<RawReport>> {
        let mut reports = Vec::new();
        for (i, line) in self.data.lines().enumerate() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 2 {
                continue;
            }
            let Some(drug) = parts.first() else { continue };
            let Some(event) = parts.get(1) else { continue };
            reports.push(RawReport {
                id: format!("csv-{i}"),
                drug_names: vec![drug.trim().to_owned()],
                event_terms: vec![event.trim().to_owned()],
                report_date: Some(DateTime::now()),
                source: ReportSource::Spontaneous,
                metadata: serde_json::Value::Null,
            });
        }
        Ok(reports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_ingest() {
        let json = r#"[{"id":"1","drugs":["aspirin"],"events":["bleeding","nausea"]}]"#;
        let ingestor = JsonIngestor::from_str(json);
        let reports = ingestor.ingest().unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].drug_names, vec!["aspirin"]);
        assert_eq!(reports[0].event_terms.len(), 2);
    }

    #[test]
    fn csv_ingest() {
        let csv = "aspirin,bleeding\nibuprofen,rash\n";
        let ingestor = CsvIngestor::from_str(csv);
        let reports = ingestor.ingest().unwrap();
        assert_eq!(reports.len(), 2);
    }
}
