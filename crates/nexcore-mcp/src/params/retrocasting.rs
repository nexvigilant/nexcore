//! Retrocasting MCP tool parameters.
//!
//! Typed parameter structs for retrospective signal-to-structure analysis,
//! structural clustering, alert correlation, and ML training data generation.

use schemars::JsonSchema;
use serde::Deserialize;

/// Compute structural similarity between two SMILES strings.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RetroStructuralSimilarityParams {
    /// First SMILES string.
    pub smiles_a: String,
    /// Second SMILES string.
    pub smiles_b: String,
}

/// Check if a pharmacovigilance signal meets standard significance thresholds.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RetroSignalSignificanceParams {
    /// Proportional Reporting Ratio.
    pub prr: f64,
    /// Reporting Odds Ratio.
    pub ror: f64,
    /// Number of FAERS cases.
    pub case_count: u64,
    /// ROR lower 95% confidence interval (optional).
    pub ror_lci: Option<f64>,
    /// PRR chi-squared statistic (optional).
    pub prr_chi_sq: Option<f64>,
}

/// Cluster structured signals by structural similarity.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RetroClusterSignalsParams {
    /// Array of structured signal records as JSON.
    /// Each must have: drug (string), event (string), prr (f64), ror (f64),
    /// case_count (u64), and optionally smiles (string).
    pub signals_json: String,
    /// Tanimoto similarity threshold [0.0, 1.0]. Default: 0.7.
    pub threshold: Option<f64>,
}

/// Correlate structural clusters with adverse event patterns to find alert candidates.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RetroCorrelateAlertsParams {
    /// Cluster results JSON (from retro_cluster_signals).
    pub clusters_json: String,
    /// Structured signals JSON (same format as retro_cluster_signals input).
    pub signals_json: String,
    /// Minimum confidence threshold [0.0, 1.0]. Default: 0.5.
    pub min_confidence: Option<f64>,
}

/// Extract ML feature vector from a SMILES string (160-dim).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RetroExtractFeaturesParams {
    /// SMILES string to extract features from.
    pub smiles: String,
}

/// Compute dataset statistics for a training dataset.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RetroDatasetStatsParams {
    /// Training dataset JSON (from retro_generate_training).
    pub dataset_json: String,
}
