use axum::routing::post;
use axum::{Json, Router};
use nexcore_core::SignalAnalysisResult;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn router() -> axum::Router<crate::ApiState> {
    Router::new().route("/analyze", post(analyze_handler))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AnalyzeRequest {
    pub drug_name: String,
    pub event_name: String,
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AnalyzeResponse {
    pub success: bool,
    pub result: SignalAnalysisResult,
}

/// Analyze a drug-event pair and produce a Signal Analysis Result
#[utoipa::path(
    post,
    path = "/api/v1/core/analyze",
    request_body = AnalyzeRequest,
    responses(
        (status = 200, description = "Analysis successful", body = AnalyzeResponse),
        (status = 400, description = "Invalid request")
    ),
    tag = "core"
)]
pub async fn analyze_handler(Json(payload): Json<AnalyzeRequest>) -> Json<AnalyzeResponse> {
    let sam = SignalAnalysisResult::new(
        &payload.drug_name,
        &payload.event_name,
        payload.a,
        payload.b,
        payload.c,
        payload.d,
    );

    Json(AnalyzeResponse {
        success: true,
        result: sam,
    })
}
