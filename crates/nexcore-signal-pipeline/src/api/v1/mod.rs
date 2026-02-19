//! # Signal API v1
//!
//! Axum REST endpoints exposing the signal detection pipeline.

use crate::core::{ContingencyTable, SignalStrength, ThresholdConfig};
use crate::stats::{SignalMetrics, compute_all};
use axum::{
    Json, Router,
    routing::{get, post},
};

/// Request body for single signal detection.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct DetectRequest {
    /// Drug name.
    pub drug: String,
    /// Event name.
    pub event: String,
    /// Cell a: drug+ event+.
    pub a: u64,
    /// Cell b: drug+ event-.
    pub b: u64,
    /// Cell c: drug- event+.
    pub c: u64,
    /// Cell d: drug- event-.
    pub d: u64,
}

/// Response body for signal detection.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DetectResponse {
    /// Drug-event pair.
    pub drug: String,
    /// Event name.
    pub event: String,
    /// Computed metrics.
    pub metrics: SignalMetrics,
    /// Signal strength classification.
    pub signal: bool,
}

/// Build the signal API router.
pub fn router() -> Router {
    Router::new()
        .route("/detect", post(detect_handler))
        .route("/batch", post(batch_handler))
        .route("/thresholds", get(thresholds_handler))
}

async fn detect_handler(Json(req): Json<DetectRequest>) -> Json<DetectResponse> {
    let table = ContingencyTable {
        a: req.a,
        b: req.b,
        c: req.c,
        d: req.d,
    };
    let metrics = compute_all(&table);
    let signal = metrics.strength >= SignalStrength::Moderate;
    Json(DetectResponse {
        drug: req.drug,
        event: req.event,
        metrics,
        signal,
    })
}

/// Batch request body.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct BatchRequest {
    /// List of detection requests.
    pub items: Vec<DetectRequest>,
}

/// Batch response body.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BatchResponse {
    /// All results.
    pub results: Vec<DetectResponse>,
    /// Count of signals detected.
    pub signals_found: usize,
}

async fn batch_handler(Json(req): Json<BatchRequest>) -> Json<BatchResponse> {
    let results: Vec<DetectResponse> = req
        .items
        .into_iter()
        .map(|item| {
            let table = ContingencyTable {
                a: item.a,
                b: item.b,
                c: item.c,
                d: item.d,
            };
            let metrics = compute_all(&table);
            let signal = metrics.strength >= SignalStrength::Moderate;
            DetectResponse {
                drug: item.drug,
                event: item.event,
                metrics,
                signal,
            }
        })
        .collect();
    let signals_found = results.iter().filter(|r| r.signal).count();
    Json(BatchResponse {
        results,
        signals_found,
    })
}

/// Threshold configurations response.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ThresholdsResponse {
    /// Evans defaults.
    pub evans: ThresholdConfig,
    /// Strict thresholds.
    pub strict: ThresholdConfig,
    /// Sensitive thresholds.
    pub sensitive: ThresholdConfig,
}

async fn thresholds_handler() -> Json<ThresholdsResponse> {
    Json(ThresholdsResponse {
        evans: ThresholdConfig::default(),
        strict: ThresholdConfig::strict(),
        sensitive: ThresholdConfig::sensitive(),
    })
}
