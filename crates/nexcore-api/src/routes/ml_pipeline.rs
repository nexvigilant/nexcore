//! ML pipeline REST endpoints — autonomous PV signal detection
//!
//! Wraps `nexcore-ml-pipeline` (random forest trained on FAERS features)
//! for the nexcore-api router. HTTP surface for agents that can't use MCP.
//!
//! ## Endpoints
//! - `POST /extract` — Extract 12 PV features from contingency table
//! - `POST /pipeline` — Full end-to-end pipeline (ingest → train → evaluate → predict)

use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use nexcore_ml_pipeline::prelude::{
    ContingencyTable, Dataset, ForestConfig, OutcomeBreakdown, PipelineConfig, RandomForest,
    RawPairData, ReporterBreakdown, Sample, TemporalData, extract_features, feature_names,
    pipeline,
};

use super::common::ApiError;

// ─── Request/Response Types ──────────────────────────────────

/// Feature extraction request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct MlFeatureExtractRequest {
    /// Drug name (generic).
    pub drug: String,
    /// Event name (MedDRA preferred term).
    pub event: String,
    /// Contingency cell a: cases with both drug and event.
    pub a: u64,
    /// Contingency cell b: cases with drug but not event.
    pub b: u64,
    /// Contingency cell c: cases with event but not drug.
    pub c: u64,
    /// Contingency cell d: cases with neither.
    pub d: u64,
    /// HCP reporter count.
    #[serde(default)]
    pub hcp_reports: Option<u64>,
    /// Consumer reporter count.
    #[serde(default)]
    pub consumer_reports: Option<u64>,
    /// Serious outcome count.
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

/// Feature extraction response with 12-element vector.
#[derive(Debug, Serialize, ToSchema)]
pub struct MlFeatureExtractResponse {
    /// Drug name.
    pub drug: String,
    /// Event name.
    pub event: String,
    /// 12-element feature vector.
    pub features: Vec<f64>,
    /// Feature names (aligned with vector).
    pub feature_names: Vec<String>,
}

/// Raw FAERS-like entry for pipeline input.
#[derive(Debug, Deserialize, ToSchema, Clone)]
pub struct MlPipelineEntry {
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
    /// Label: "signal" or "noise".
    pub label: String,
    /// HCP reports (optional).
    #[serde(default)]
    pub hcp_reports: Option<u64>,
    /// Consumer reports (optional).
    #[serde(default)]
    pub consumer_reports: Option<u64>,
    /// Serious outcome count (optional).
    #[serde(default)]
    pub serious_count: Option<u64>,
    /// Death count (optional).
    #[serde(default)]
    pub death_count: Option<u64>,
    /// Hospitalization count (optional).
    #[serde(default)]
    pub hospitalization_count: Option<u64>,
    /// Median time-to-onset in days (optional).
    #[serde(default)]
    pub median_tto_days: Option<f64>,
    /// Reporting velocity (optional).
    #[serde(default)]
    pub velocity: Option<f64>,
}

/// Full pipeline request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct MlPipelineRequest {
    /// Labeled samples for training + evaluation.
    pub samples: Vec<MlPipelineEntry>,
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

/// Simplified metrics for API response.
#[derive(Debug, Serialize, ToSchema)]
pub struct MlMetrics {
    /// Area under the ROC curve.
    pub auc: f64,
    /// Precision.
    pub precision: f64,
    /// Recall.
    pub recall: f64,
    /// F1 score.
    pub f1: f64,
    /// Accuracy.
    pub accuracy: f64,
}

/// Prediction for a test sample.
#[derive(Debug, Serialize, ToSchema)]
pub struct MlPredictionResult {
    /// Drug name.
    pub drug: String,
    /// Event name.
    pub event: String,
    /// Predicted class: "signal" or "noise".
    pub prediction: String,
    /// Signal probability (0.0 - 1.0).
    pub signal_probability: f64,
}

/// Full pipeline response.
#[derive(Debug, Serialize, ToSchema)]
pub struct MlPipelineResponse {
    /// Model version identifier.
    pub model_version: String,
    /// Number of trees trained.
    pub n_trees: usize,
    /// Training sample count.
    pub n_train_samples: usize,
    /// Test sample count.
    pub n_test_samples: usize,
    /// Training metrics.
    pub train_metrics: MlMetrics,
    /// Test metrics (held-out).
    pub test_metrics: MlMetrics,
    /// Predictions on the test set.
    pub test_predictions: Vec<MlPredictionResult>,
}

// ─── Router ──────────────────────────────────────────────────

/// ML pipeline router. Nested under `/api/v1/ml`.
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/extract", post(extract))
        .route("/pipeline", post(run_pipeline))
}

// ─── Handlers ────────────────────────────────────────────────

fn entry_to_raw(e: &MlPipelineEntry) -> RawPairData {
    RawPairData {
        contingency: ContingencyTable {
            drug: e.drug.clone(),
            event: e.event.clone(),
            a: e.a,
            b: e.b,
            c: e.c,
            d: e.d,
        },
        reporters: ReporterBreakdown {
            hcp: e.hcp_reports.unwrap_or(0),
            consumer: e.consumer_reports.unwrap_or(0),
            other: 0,
        },
        outcomes: OutcomeBreakdown {
            total: e.a,
            serious: e.serious_count.unwrap_or(0),
            death: e.death_count.unwrap_or(0),
            hospitalization: e.hospitalization_count.unwrap_or(0),
        },
        temporal: TemporalData {
            median_tto_days: e.median_tto_days,
            velocity: e.velocity.unwrap_or(0.0),
        },
    }
}

/// Extract 12-element PV feature vector from FAERS contingency data.
#[utoipa::path(
    post,
    path = "/api/v1/ml/extract",
    tag = "ml",
    request_body = MlFeatureExtractRequest,
    responses(
        (status = 200, description = "Features extracted", body = MlFeatureExtractResponse),
        (status = 400, description = "Invalid contingency table", body = super::common::ApiError)
    )
)]
pub async fn extract(
    Json(req): Json<MlFeatureExtractRequest>,
) -> Result<Json<MlFeatureExtractResponse>, ApiError> {
    let raw = RawPairData {
        contingency: ContingencyTable {
            drug: req.drug.clone(),
            event: req.event.clone(),
            a: req.a,
            b: req.b,
            c: req.c,
            d: req.d,
        },
        reporters: ReporterBreakdown {
            hcp: req.hcp_reports.unwrap_or(0),
            consumer: req.consumer_reports.unwrap_or(0),
            other: 0,
        },
        outcomes: OutcomeBreakdown {
            total: req.a,
            serious: req.serious_count.unwrap_or(0),
            death: req.death_count.unwrap_or(0),
            hospitalization: req.hospitalization_count.unwrap_or(0),
        },
        temporal: TemporalData {
            median_tto_days: req.median_tto_days,
            velocity: req.velocity.unwrap_or(0.0),
        },
    };

    let sample = extract_features(&raw).map_err(|e| ApiError {
        code: "bad_request".to_string(),
        message: format!("Feature extraction failed: {e}"),
        details: None,
    })?;

    Ok(Json(MlFeatureExtractResponse {
        drug: sample.drug,
        event: sample.event,
        features: sample.features,
        feature_names: feature_names(),
    }))
}

/// Run the full autonomous ML pipeline: ingest → train → evaluate → predict.
#[utoipa::path(
    post,
    path = "/api/v1/ml/pipeline",
    tag = "ml",
    request_body = MlPipelineRequest,
    responses(
        (status = 200, description = "Pipeline complete", body = MlPipelineResponse),
        (status = 400, description = "Pipeline failed", body = super::common::ApiError)
    )
)]
pub async fn run_pipeline(
    Json(req): Json<MlPipelineRequest>,
) -> Result<Json<MlPipelineResponse>, ApiError> {
    let raw_data: Vec<RawPairData> = req.samples.iter().map(entry_to_raw).collect();
    let labels: Vec<(String, String, String)> = req
        .samples
        .iter()
        .map(|e| (e.drug.clone(), e.event.clone(), e.label.clone()))
        .collect();

    let config = PipelineConfig {
        forest: ForestConfig {
            n_trees: req.n_trees.unwrap_or(100),
            max_depth: req.max_depth.or(Some(10)),
            seed: 42,
            ..ForestConfig::default()
        },
        train_ratio: req.train_ratio.unwrap_or(0.8),
        ..PipelineConfig::default()
    };

    let result = pipeline::run(&raw_data, &labels, config).map_err(|e| ApiError {
        code: "bad_request".to_string(),
        message: format!("Pipeline failed: {e}"),
        details: None,
    })?;

    let test_predictions: Vec<MlPredictionResult> = result
        .test_predictions
        .into_iter()
        .map(|p| MlPredictionResult {
            drug: p.drug,
            event: p.event,
            prediction: p.prediction,
            signal_probability: p.signal_probability,
        })
        .collect();

    Ok(Json(MlPipelineResponse {
        model_version: result.model_version,
        n_trees: result.n_trees,
        n_train_samples: result.n_train_samples,
        n_test_samples: result.n_test_samples,
        train_metrics: MlMetrics {
            auc: result.train_metrics.auc,
            precision: result.train_metrics.precision,
            recall: result.train_metrics.recall,
            f1: result.train_metrics.f1,
            accuracy: result.train_metrics.accuracy,
        },
        test_metrics: MlMetrics {
            auc: result.test_metrics.auc,
            precision: result.test_metrics.precision,
            recall: result.test_metrics.recall,
            f1: result.test_metrics.f1,
            accuracy: result.test_metrics.accuracy,
        },
        test_predictions,
    }))
}

// Suppress unused warnings for types that may be used in future endpoints
#[allow(dead_code)]
fn _unused_dataset_rf_sample() -> (Dataset, RandomForest, Sample) {
    unimplemented!()
}
