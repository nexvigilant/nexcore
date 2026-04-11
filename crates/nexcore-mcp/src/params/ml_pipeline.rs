//! Parameter structs for ML pipeline MCP tools.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for `ml_feature_extract`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MlFeatureExtractParams {
    /// Drug name (generic).
    pub drug: String,
    /// Event name (MedDRA preferred term).
    pub event: String,
    /// Contingency table cell: cases with both drug and event.
    pub a: u64,
    /// Contingency table cell: cases with drug but not event.
    pub b: u64,
    /// Contingency table cell: cases with event but not drug.
    pub c: u64,
    /// Contingency table cell: cases with neither.
    pub d: u64,
    /// HCP reporter count.
    #[serde(default)]
    pub hcp_reports: Option<u64>,
    /// Consumer reporter count.
    #[serde(default)]
    pub consumer_reports: Option<u64>,
    /// Total serious outcome count.
    #[serde(default)]
    pub serious_count: Option<u64>,
    /// Death count.
    #[serde(default)]
    pub death_count: Option<u64>,
    /// Hospitalization count.
    #[serde(default)]
    pub hospitalization_count: Option<u64>,
    /// Median time-to-onset in days.
    #[serde(default)]
    pub median_tto_days: Option<f64>,
    /// Reporting velocity (cases per quarter).
    #[serde(default)]
    pub velocity: Option<f64>,
}

/// Parameters for `ml_train`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MlTrainParams {
    /// Training samples: each entry is { drug, event, features: [f64; 12], label: "signal"|"noise" }.
    pub samples: Vec<MlSampleInput>,
    /// Number of trees (default: 100).
    #[serde(default)]
    pub n_trees: Option<usize>,
    /// Maximum tree depth (default: 10).
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// Random seed for reproducibility.
    #[serde(default)]
    pub seed: Option<u64>,
}

/// A single training sample.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MlSampleInput {
    /// Drug name.
    pub drug: String,
    /// Event name.
    pub event: String,
    /// 12-element feature vector.
    pub features: Vec<f64>,
    /// Label: "signal" or "noise".
    pub label: String,
}

/// Parameters for `ml_predict`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MlPredictParams {
    /// Model ID to use for prediction (from ml_train).
    pub model_id: String,
    /// Feature vectors to predict. Each is a 12-element array.
    pub samples: Vec<MlPredictSample>,
}

/// A sample to predict on.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MlPredictSample {
    /// Drug name.
    pub drug: String,
    /// Event name.
    pub event: String,
    /// 12-element feature vector.
    pub features: Vec<f64>,
}

/// Parameters for `ml_evaluate`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MlEvaluateParams {
    /// Model ID to evaluate.
    pub model_id: String,
    /// Test samples with labels.
    pub test_samples: Vec<MlSampleInput>,
}

/// Parameters for `ml_pipeline_run`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MlPipelineRunParams {
    /// Raw FAERS-like data entries.
    pub data: Vec<MlRawEntry>,
    /// Labels: list of { drug, event, label }.
    pub labels: Vec<MlLabelEntry>,
    /// Number of trees (default: 100).
    #[serde(default)]
    pub n_trees: Option<usize>,
    /// Maximum tree depth (default: 10).
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// Train/test split ratio (default: 0.8).
    #[serde(default)]
    pub train_ratio: Option<f64>,
}

/// A raw FAERS-like data entry for pipeline input.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MlRawEntry {
    /// Drug name.
    pub drug: String,
    /// Event name.
    pub event: String,
    /// Contingency: a (drug+event).
    pub a: u64,
    /// Contingency: b (drug, no event).
    pub b: u64,
    /// Contingency: c (event, no drug).
    pub c: u64,
    /// Contingency: d (neither).
    pub d: u64,
    /// HCP reports.
    #[serde(default)]
    pub hcp_reports: Option<u64>,
    /// Consumer reports.
    #[serde(default)]
    pub consumer_reports: Option<u64>,
    /// Serious outcomes.
    #[serde(default)]
    pub serious_count: Option<u64>,
    /// Deaths.
    #[serde(default)]
    pub death_count: Option<u64>,
    /// Hospitalizations.
    #[serde(default)]
    pub hospitalization_count: Option<u64>,
    /// Median time-to-onset (days).
    #[serde(default)]
    pub median_tto_days: Option<f64>,
    /// Reporting velocity.
    #[serde(default)]
    pub velocity: Option<f64>,
}

/// A label entry for pipeline input.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MlLabelEntry {
    /// Drug name.
    pub drug: String,
    /// Event name.
    pub event: String,
    /// Label: "signal" or "noise".
    pub label: String,
}
