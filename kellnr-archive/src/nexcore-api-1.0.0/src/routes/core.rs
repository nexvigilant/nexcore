//! nexcore Core API endpoints
//!
//! High-level PV operations unifying ingestion, analysis, and persistence.

use axum::{Json, Router, routing::post};
use nexcore_core::SignalAnalysisResult;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::ApiResult;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Ingest and analyze request
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct AnalyzeRequest {
    pub drug: String,
    pub event: String,
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
}

/// Analysis result response
#[derive(Debug, Serialize, ToSchema)]
#[allow(dead_code)]
pub struct AnalyzeResponse {
    pub result: SignalAnalysisResult,
    pub status: String,
}

// ============================================================================
// Router
// ============================================================================

#[allow(dead_code)]
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new().route("/analyze", post(analyze))
}

// ============================================================================
// Handlers
// ============================================================================

/// Run end-to-end signal analysis
#[utoipa::path(
    post,
    path = "/api/v1/core/analyze",
    tag = "core",
    request_body = AnalyzeRequest,
    responses(
        (status = 200, description = "Analysis complete", body = AnalyzeResponse)
    )
)]
#[allow(dead_code)]
pub async fn analyze(Json(req): Json<AnalyzeRequest>) -> ApiResult<AnalyzeResponse> {
    let result = SignalAnalysisResult::new(&req.drug, &req.event, req.a, req.b, req.c, req.d);

    Ok(Json(AnalyzeResponse {
        result,
        status: "success".to_string(),
    }))
}
