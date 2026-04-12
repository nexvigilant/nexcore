//! DNA-ML pipeline REST endpoints — DNA-encoded ML for PV signal detection
//!
//! Wraps `nexcore-dna-ml` for HTTP access. Gemini and other agents that can't
//! use MCP call these endpoints directly.
//!
//! ## Endpoints
//! - `POST /encode` — Encode feature vector as DNA strand
//! - `POST /similarity` — Compute DNA similarity between two feature vectors
//! - `POST /pipeline` — Full DNA-ML pipeline (features → DNA → augment → train → evaluate)

use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use nexcore_dna_ml::encode;
use nexcore_dna_ml::similarity;
use nexcore_ml_pipeline::types::{
    ContingencyTable, OutcomeBreakdown, RawPairData, ReporterBreakdown, TemporalData,
};

use super::common::ApiError;

// ─── Request/Response Types ──────────────────────────────────

/// DNA encoding request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct DnaMlEncodeRequest {
    /// Feature vector to encode as DNA.
    pub features: Vec<f64>,
    /// Per-feature minimum bounds (optional, defaults to 0.0).
    #[serde(default)]
    pub mins: Option<Vec<f64>>,
    /// Per-feature maximum bounds (optional, defaults to 1.0).
    #[serde(default)]
    pub maxs: Option<Vec<f64>>,
}

/// DNA encoding response.
#[derive(Debug, Serialize, ToSchema)]
pub struct DnaMlEncodeResponse {
    /// DNA strand as nucleotide string.
    pub strand: String,
    /// Strand length in bases.
    pub strand_length: usize,
    /// Number of features encoded.
    pub feature_count: usize,
    /// Quantized byte values.
    pub quantized_bytes: Vec<u8>,
}

/// DNA similarity request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct DnaMlSimilarityRequest {
    /// First feature vector.
    pub features_a: Vec<f64>,
    /// Second feature vector.
    pub features_b: Vec<f64>,
    /// Per-feature minimums for quantization.
    #[serde(default)]
    pub mins: Option<Vec<f64>>,
    /// Per-feature maximums for quantization.
    #[serde(default)]
    pub maxs: Option<Vec<f64>>,
}

/// DNA similarity response.
#[derive(Debug, Serialize, ToSchema)]
pub struct DnaMlSimilarityResponse {
    /// Hamming distance (0.0 = identical, 1.0 = completely different).
    pub hamming_distance: f64,
    /// GC content of strand A.
    pub gc_content_a: f64,
    /// GC content of strand B.
    pub gc_content_b: f64,
    /// GC content divergence.
    pub gc_divergence: f64,
    /// Longest common subsequence ratio.
    pub lcs_ratio: f64,
}

/// Raw FAERS entry for pipeline input.
#[derive(Debug, Deserialize, ToSchema, Clone)]
pub struct DnaMlPipelineEntry {
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

/// Full DNA-ML pipeline request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct DnaMlPipelineRequest {
    /// Labeled samples for training + evaluation.
    pub samples: Vec<DnaMlPipelineEntry>,
    /// Number of trees (default: 50).
    #[serde(default)]
    pub n_trees: Option<usize>,
    /// Maximum tree depth (default: 8).
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// Include DNA similarity features (default: true).
    #[serde(default)]
    pub use_dna_features: Option<bool>,
}

/// DNA-ML pipeline metrics.
#[derive(Debug, Serialize, ToSchema)]
pub struct DnaMlMetrics {
    /// AUC-ROC.
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

/// DNA-ML pipeline response.
#[derive(Debug, Serialize, ToSchema)]
pub struct DnaMlPipelineResponse {
    /// Number of PV features (12).
    pub pv_feature_count: usize,
    /// Number of DNA features added (0 or 5).
    pub dna_feature_count: usize,
    /// Total feature dimension.
    pub total_features: usize,
    /// Number of samples.
    pub n_samples: usize,
    /// Evaluation metrics (LOO cross-validated).
    pub metrics: DnaMlMetrics,
    /// Feature names.
    pub feature_names: Vec<String>,
}

// ─── Router ──────────────────────────────────────────────────

/// DNA-ML router. Nested under `/api/v1/dna-ml`.
pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/encode", post(encode_handler))
        .route("/similarity", post(similarity_handler))
        .route("/pipeline", post(pipeline_handler))
}

// ─── Handlers ────────────────────────────────────────────────

fn entry_to_raw(e: &DnaMlPipelineEntry) -> RawPairData {
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

/// Encode a feature vector as a DNA strand.
#[utoipa::path(
    post,
    path = "/api/v1/dna-ml/encode",
    tag = "dna-ml",
    request_body = DnaMlEncodeRequest,
    responses(
        (status = 200, description = "DNA strand encoded", body = DnaMlEncodeResponse),
        (status = 400, description = "Invalid features", body = super::common::ApiError)
    )
)]
pub async fn encode_handler(
    Json(req): Json<DnaMlEncodeRequest>,
) -> Result<Json<DnaMlEncodeResponse>, ApiError> {
    let dim = req.features.len();
    if dim == 0 {
        return Err(ApiError {
            code: "bad_request".into(),
            message: "features must not be empty".into(),
            details: None,
        });
    }

    let mins = req.mins.unwrap_or_else(|| vec![0.0; dim]);
    let maxs = req.maxs.unwrap_or_else(|| vec![1.0; dim]);

    let strand = encode::encode_features(&req.features, &mins, &maxs);
    let bases: String = strand.bases.iter().map(|n| n.as_char()).collect();
    let quantized = encode::quantize_features(&req.features, &mins, &maxs);

    Ok(Json(DnaMlEncodeResponse {
        strand: bases,
        strand_length: strand.bases.len(),
        feature_count: dim,
        quantized_bytes: quantized,
    }))
}

/// Compute DNA similarity between two encoded feature vectors.
#[utoipa::path(
    post,
    path = "/api/v1/dna-ml/similarity",
    tag = "dna-ml",
    request_body = DnaMlSimilarityRequest,
    responses(
        (status = 200, description = "Similarity computed", body = DnaMlSimilarityResponse),
        (status = 400, description = "Invalid features", body = super::common::ApiError)
    )
)]
pub async fn similarity_handler(
    Json(req): Json<DnaMlSimilarityRequest>,
) -> Result<Json<DnaMlSimilarityResponse>, ApiError> {
    let dim = req.features_a.len().max(req.features_b.len());
    if dim == 0 {
        return Err(ApiError {
            code: "bad_request".into(),
            message: "features must not be empty".into(),
            details: None,
        });
    }

    let mins = req.mins.unwrap_or_else(|| vec![0.0; dim]);
    let maxs = req.maxs.unwrap_or_else(|| vec![1.0; dim]);

    let strand_a = encode::encode_features(&req.features_a, &mins, &maxs);
    let strand_b = encode::encode_features(&req.features_b, &mins, &maxs);
    let sim = similarity::compute_similarity(&strand_a, &strand_b);

    Ok(Json(DnaMlSimilarityResponse {
        hamming_distance: sim.hamming_distance,
        gc_content_a: sim.gc_content_a,
        gc_content_b: sim.gc_content_b,
        gc_divergence: sim.gc_divergence,
        lcs_ratio: sim.lcs_ratio,
    }))
}

/// Run the full DNA-ML pipeline with LOO cross-validation.
#[utoipa::path(
    post,
    path = "/api/v1/dna-ml/pipeline",
    tag = "dna-ml",
    request_body = DnaMlPipelineRequest,
    responses(
        (status = 200, description = "Pipeline complete", body = DnaMlPipelineResponse),
        (status = 400, description = "Pipeline failed", body = super::common::ApiError)
    )
)]
pub async fn pipeline_handler(
    Json(req): Json<DnaMlPipelineRequest>,
) -> Result<Json<DnaMlPipelineResponse>, ApiError> {
    if req.samples.is_empty() {
        return Err(ApiError {
            code: "bad_request".into(),
            message: "samples must not be empty".into(),
            details: None,
        });
    }

    let raw_data: Vec<RawPairData> = req.samples.iter().map(entry_to_raw).collect();
    let labels: Vec<String> = req.samples.iter().map(|e| e.label.clone()).collect();

    let config = nexcore_dna_ml::pipeline::DnaMlConfig {
        n_trees: req.n_trees.unwrap_or(50),
        max_depth: req.max_depth.unwrap_or(8),
        use_dna_features: req.use_dna_features.unwrap_or(true),
        ..Default::default()
    };

    let result =
        nexcore_dna_ml::pipeline::run(&raw_data, &labels, &config).map_err(|e| ApiError {
            code: "bad_request".into(),
            message: format!("DNA-ML pipeline failed: {e}"),
            details: None,
        })?;

    Ok(Json(DnaMlPipelineResponse {
        pv_feature_count: result.pv_feature_count,
        dna_feature_count: result.dna_feature_count,
        total_features: result.total_features,
        n_samples: result.n_samples,
        metrics: DnaMlMetrics {
            auc: result.metrics.auc,
            precision: result.metrics.precision,
            recall: result.metrics.recall,
            f1: result.metrics.f1,
            accuracy: result.metrics.accuracy,
        },
        feature_names: result.feature_names,
    }))
}
