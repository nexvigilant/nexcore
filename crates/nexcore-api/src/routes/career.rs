//! Career Transitions REST endpoint
//!
//! Dispatches to `career_transitions` MCP tool via in-process bridge.
//! GET /api/v1/career/transitions

use axum::{Json, Router, extract::{Query, State}, routing::get};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::ApiError;
use crate::mcp_bridge;

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
    State(state): State<crate::ApiState>,
    Query(params): Query<CareerTransitionsQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut mcp_params = serde_json::Map::new();

    if let Some(ref roles_csv) = params.roles {
        let roles: Vec<String> = roles_csv.split(',').map(|s| s.trim().to_string()).collect();
        mcp_params.insert(
            "roles".to_string(),
            serde_json::to_value(roles).unwrap_or_default(),
        );
    }
    if let Some(threshold) = params.threshold {
        mcp_params.insert("threshold".to_string(), serde_json::json!(threshold));
    }
    if let Some(include_salary) = params.include_salary {
        mcp_params.insert(
            "include_salary".to_string(),
            serde_json::json!(include_salary),
        );
    }

    let result = mcp_bridge::call_tool("career_transitions", serde_json::Value::Object(mcp_params), &state.mcp_server)
        .await
        .map_err(|e| ApiError::new("MCP_ERROR", format!("career_transitions failed: {e}")))?;

    // Extract text content from MCP result
    let payload = extract_mcp_text(&result)?;
    Ok(Json(payload))
}

/// Extract the text content from an MCP CallToolResult and parse as JSON.
fn extract_mcp_text(result: &serde_json::Value) -> Result<serde_json::Value, ApiError> {
    let text = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| ApiError::new("PARSE_ERROR", "No text content in MCP result"))?;

    serde_json::from_str(text)
        .map_err(|e| ApiError::new("PARSE_ERROR", format!("Invalid JSON in MCP result: {e}")))
}
