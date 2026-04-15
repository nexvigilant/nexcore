//! PHAROS configuration.
//!
//! Primitive composition: ς(State) + ∂(Boundary) + N(Quantity)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::thresholds::SignalThresholds;

/// Top-level PHAROS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PharosConfig {
    /// Path to FAERS quarterly ASCII data directory.
    pub faers_dir: PathBuf,

    /// Output directory for Parquet files and reports.
    pub output_dir: PathBuf,

    /// Minimum case count to include a drug-event pair.
    pub min_cases: i64,

    /// Include all drug roles (not just primary suspect).
    pub include_all_roles: bool,

    /// Signal detection thresholds.
    pub thresholds: SignalThresholds,

    /// Qdrant endpoint for signal embedding.
    pub qdrant_url: String,

    /// Qdrant collection name for PHAROS signals.
    pub qdrant_collection: String,

    /// Whether to emit cytokine events on signal detection.
    pub emit_cytokines: bool,

    /// Whether to inject signals into Guardian homeostasis loop.
    pub inject_guardian: bool,

    /// Maximum number of top signals to include in the report.
    pub top_n_report: usize,
}

impl Default for PharosConfig {
    fn default() -> Self {
        Self {
            faers_dir: PathBuf::from("./data/faers"),
            output_dir: PathBuf::from("./output/pharos"),
            min_cases: 3,
            include_all_roles: false,
            thresholds: SignalThresholds::default(),
            qdrant_url: "http://localhost:6333".to_string(),
            qdrant_collection: "pharos_signals".to_string(),
            emit_cytokines: true,
            inject_guardian: true,
            top_n_report: 50,
        }
    }
}

impl PharosConfig {
    /// Load config from a TOML file, falling back to defaults.
    pub fn from_file(path: &std::path::Path) -> nexcore_error::Result<Self> {
        if path.exists() {
            let content =
                std::fs::read_to_string(path).map_err(|e| nexcore_error::nexerror!("{e}"))?;
            let config: Self =
                toml::from_str(&content).map_err(|e| nexcore_error::nexerror!("{e}"))?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Validate that required paths exist.
    pub fn validate(&self) -> nexcore_error::Result<()> {
        if !self.faers_dir.exists() {
            nexcore_error::bail!(
                "FAERS data directory does not exist: {}",
                self.faers_dir.display()
            );
        }
        Ok(())
    }
}
