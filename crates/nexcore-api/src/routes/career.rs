//! Career Transitions REST endpoint
//!
//! Returns career role-transition graph data.
//! GET /api/v1/career/transitions

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::ApiError;

// ── Request ──────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct CareerTransitionsQuery {
    /// Comma-separated role IDs (default: all)
    #[serde(default)]
    pub roles: Option<String>,
    /// Min similarity to include edge (default 0.15)
    #[serde(default)]
    pub threshold: Option<f64>,
    /// Include value-mining salary data
    #[serde(default)]
    pub include_salary: Option<bool>,
}

// ── Response ─────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct CareerTransitionsResponse {
    pub nodes: Vec<serde_json::Value>,
    pub edges: Vec<serde_json::Value>,
    pub similarity_matrix_size: u32,
}

// ── Router ───────────────────────────────────

pub fn router() -> Router<crate::ApiState> {
    Router::new().route("/transitions", get(transitions))
}

// ── Handler ──────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/career/transitions",
    tag = "career",
    params(
        ("roles" = Option<String>, Query, description = "Comma-separated role IDs"),
        ("threshold" = Option<f64>, Query, description = "Min similarity (default 0.15)"),
        ("include_salary" = Option<bool>, Query, description = "Include salary data")
    ),
    responses(
        (status = 200, description = "Career transition graph", body = CareerTransitionsResponse),
        (status = 502, description = "MCP dispatch error", body = ApiError)
    )
)]
async fn transitions(
    State(_state): State<crate::ApiState>,
    Query(params): Query<CareerTransitionsQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Build the parameter payload that will be forwarded to the career_transitions tool.
    // Currently returns a seed graph; wire to the MCP tool once the in-process bridge
    // is available in this crate.
    let threshold = params.threshold.unwrap_or(0.15);
    let include_salary = params.include_salary.unwrap_or(false);

    let payload = serde_json::json!({
        "nodes": [],
        "edges": [],
        "similarity_matrix_size": 0,
        "threshold": threshold,
        "include_salary": include_salary,
        "status": "stub — career_transitions MCP tool not yet bridged in nexcore-api"
    });

    Ok(Json(payload))
}
