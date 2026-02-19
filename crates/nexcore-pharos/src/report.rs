//! Surveillance report generation.
//!
//! Primitive composition: Σ(Sum) + π(Persistence) + σ(Sequence)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single signal entry in the surveillance report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEntry {
    pub drug: String,
    pub event: String,
    pub case_count: u64,
    pub prr: f64,
    pub prr_lower_ci: f64,
    pub ror: f64,
    pub ror_lower_ci: f64,
    pub ic: f64,
    pub ic025: f64,
    pub ebgm: f64,
    pub eb05: f64,
    pub algorithms_flagged: u32,
    pub threat_level: String,
}

/// Complete surveillance report from a PHAROS run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveillanceReport {
    /// Unique run identifier.
    pub run_id: String,

    /// Timestamp of the run.
    pub timestamp: DateTime<Utc>,

    /// FAERS data directory processed.
    pub faers_dir: String,

    /// Total drug-event pairs evaluated.
    pub total_pairs: usize,

    /// Total signals detected (before threshold filtering).
    pub raw_signals: usize,

    /// Signals that passed threshold filtering.
    pub actionable_signals: usize,

    /// Signals injected into Guardian.
    pub guardian_injections: usize,

    /// Cytokine events emitted.
    pub cytokines_emitted: usize,

    /// Top signals sorted by EBGM descending.
    pub top_signals: Vec<SignalEntry>,

    /// Pipeline duration in milliseconds.
    pub duration_ms: u128,

    /// Threshold configuration used.
    pub thresholds_used: String,
}

impl SurveillanceReport {
    /// Create a new empty report with a generated run ID.
    pub fn new(faers_dir: &str) -> Self {
        Self {
            run_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            faers_dir: faers_dir.to_string(),
            total_pairs: 0,
            raw_signals: 0,
            actionable_signals: 0,
            guardian_injections: 0,
            cytokines_emitted: 0,
            top_signals: Vec::new(),
            duration_ms: 0,
            thresholds_used: "default".to_string(),
        }
    }

    /// Persist the report as JSON to the output directory.
    pub fn save(&self, output_dir: &std::path::Path) -> anyhow::Result<()> {
        std::fs::create_dir_all(output_dir)?;
        let filename = format!("pharos-report-{}.json", self.run_id);
        let path = output_dir.join(filename);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        tracing::info!(path = %path.display(), signals = self.actionable_signals, "Report saved");
        Ok(())
    }

    /// Format a one-line summary for logging.
    pub fn summary(&self) -> String {
        format!(
            "PHAROS run {} | {} pairs → {} raw → {} actionable | {}ms",
            &self.run_id[..8],
            self.total_pairs,
            self.raw_signals,
            self.actionable_signals,
            self.duration_ms,
        )
    }
}
