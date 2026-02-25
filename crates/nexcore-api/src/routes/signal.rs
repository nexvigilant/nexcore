//! Signal detection pipeline endpoints (signal-* crates)
//!
//! Wraps `signal-stats` and `signal-core` for the nexcore-api router.
//! Coexists with `/api/v1/pv/*` (which uses `nexcore-vigilance` directly).
//!
//! ## Endpoints
//! - `POST /detect` — Single drug-event signal detection
//! - `POST /batch` — Batch detection on multiple pairs
//! - `GET  /thresholds` — Threshold configurations (Evans/Strict/Sensitive)

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use signal::core::{ContingencyTable, SignalStrength, ThresholdConfig};
use signal::stats::{SignalMetrics, compute_all};

use super::common::ApiError;

// ─── Request/Response Types ──────────────────────────────────

/// Signal detection request for a drug-event pair.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SignalDetectRequest {
    /// Drug name
    pub drug: String,
    /// Event name (MedDRA PT preferred)
    pub event: String,
    /// Cell a: drug+ event+
    pub a: u64,
    /// Cell b: drug+ event-
    pub b: u64,
    /// Cell c: drug- event+
    pub c: u64,
    /// Cell d: drug- event-
    pub d: u64,
}

/// Signal detection response with all metrics.
#[derive(Debug, Serialize, ToSchema)]
pub struct SignalDetectResponse {
    /// Drug name
    pub drug: String,
    /// Event name
    pub event: String,
    /// PRR value (None if denominator zero)
    pub prr: Option<f64>,
    /// ROR value (None if denominator zero)
    pub ror: Option<f64>,
    /// Information Component
    pub ic: f64,
    /// Empirical Bayesian Geometric Mean
    pub ebgm: f64,
    /// Chi-square statistic
    pub chi_square: f64,
    /// Signal strength classification
    pub strength: String,
    /// Signal detected (Moderate or above)
    pub signal: bool,
}

/// Batch detection request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SignalBatchRequest {
    /// List of drug-event pairs to analyze
    pub items: Vec<SignalDetectRequest>,
}

/// Batch detection response.
#[derive(Debug, Serialize, ToSchema)]
pub struct SignalBatchResponse {
    /// Individual results
    pub results: Vec<SignalDetectResponse>,
    /// Count of signals detected
    pub signals_found: usize,
}

/// Threshold configurations response.
#[derive(Debug, Serialize, ToSchema)]
pub struct SignalThresholdsResponse {
    /// Evans default thresholds
    pub evans: ThresholdSummary,
    /// Strict thresholds (fewer false positives)
    pub strict: ThresholdSummary,
    /// Sensitive thresholds (fewer false negatives)
    pub sensitive: ThresholdSummary,
}

/// Threshold summary for API response.
#[derive(Debug, Serialize, ToSchema)]
pub struct ThresholdSummary {
    /// Minimum PRR
    pub prr_min: f64,
    /// Minimum chi-square
    pub chi_square_min: f64,
    /// Minimum case count
    pub case_count_min: u64,
    /// Minimum ROR lower CI
    pub ror_lower_ci_min: f64,
    /// Minimum IC025
    pub ic025_min: f64,
    /// Minimum EB05
    pub eb05_min: f64,
}

// ─── Router ──────────────────────────────────────────────────

/// Signal pipeline router. Nested under `/api/v1/signal`.
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/detect", post(detect))
        .route("/batch", post(batch))
        .route("/thresholds", get(thresholds))
}

// ─── Handlers ────────────────────────────────────────────────

fn metrics_to_response(
    drug: String,
    event: String,
    metrics: &SignalMetrics,
) -> SignalDetectResponse {
    let signal = metrics.strength >= SignalStrength::Moderate;
    SignalDetectResponse {
        drug,
        event,
        prr: metrics.prr.map(|p| p.0),
        ror: metrics.ror.map(|r| r.0),
        ic: metrics.ic.0,
        ebgm: metrics.ebgm.0,
        chi_square: metrics.chi_square.0,
        strength: format!("{:?}", metrics.strength),
        signal,
    }
}

/// Detect signal for a single drug-event pair.
#[utoipa::path(
    post,
    path = "/api/v1/signal/detect",
    tag = "signal",
    request_body = SignalDetectRequest,
    responses(
        (status = 200, description = "Signal detection complete", body = SignalDetectResponse),
        (status = 400, description = "Invalid input", body = super::common::ApiError)
    )
)]
pub async fn detect(
    Json(req): Json<SignalDetectRequest>,
) -> Result<Json<SignalDetectResponse>, ApiError> {
    let table = ContingencyTable::new(req.a, req.b, req.c, req.d);
    let metrics = compute_all(&table);
    Ok(Json(metrics_to_response(req.drug, req.event, &metrics)))
}

/// Batch detect signals for multiple drug-event pairs.
#[utoipa::path(
    post,
    path = "/api/v1/signal/batch",
    tag = "signal",
    request_body = SignalBatchRequest,
    responses(
        (status = 200, description = "Batch detection complete", body = SignalBatchResponse)
    )
)]
pub async fn batch(Json(req): Json<SignalBatchRequest>) -> Json<SignalBatchResponse> {
    let results: Vec<SignalDetectResponse> = req
        .items
        .into_iter()
        .map(|item| {
            let table = ContingencyTable::new(item.a, item.b, item.c, item.d);
            let metrics = compute_all(&table);
            metrics_to_response(item.drug, item.event, &metrics)
        })
        .collect();
    let signals_found = results.iter().filter(|r| r.signal).count();
    Json(SignalBatchResponse {
        results,
        signals_found,
    })
}

fn config_to_summary(cfg: &ThresholdConfig) -> ThresholdSummary {
    ThresholdSummary {
        prr_min: cfg.prr_min,
        chi_square_min: cfg.chi_square_min,
        case_count_min: cfg.case_count_min,
        ror_lower_ci_min: cfg.ror_lower_ci_min,
        ic025_min: cfg.ic025_min,
        eb05_min: cfg.eb05_min,
    }
}

/// Get threshold configurations (Evans, Strict, Sensitive).
#[utoipa::path(
    get,
    path = "/api/v1/signal/thresholds",
    tag = "signal",
    responses(
        (status = 200, description = "Threshold configurations", body = SignalThresholdsResponse)
    )
)]
pub async fn thresholds() -> Json<SignalThresholdsResponse> {
    Json(SignalThresholdsResponse {
        evans: config_to_summary(&ThresholdConfig::default()),
        strict: config_to_summary(&ThresholdConfig::strict()),
        sensitive: config_to_summary(&ThresholdConfig::sensitive()),
    })
}
