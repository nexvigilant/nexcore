//! Graph Layout REST endpoint
//!
//! Dispatches to `graph_layout_converge` MCP tool via in-process bridge.
//! POST /api/v1/graph/converge (POST because input can be large)

use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::ApiError;
use crate::mcp_bridge;

// ── Request ──────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct GraphLayoutRequest {
    /// Nodes to position
    pub nodes: Vec<LayoutNodeInput>,
    /// Edges between nodes
    pub edges: Vec<LayoutEdgeInput>,
    /// 2 or 3 dimensions (default 3)
    #[serde(default)]
    pub dimensions: Option<u8>,
    /// Max iterations (default 500)
    #[serde(default)]
    pub iterations: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LayoutNodeInput {
    pub id: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub value: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LayoutEdgeInput {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub weight: Option<f64>,
}

// ── Response ─────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct GraphLayoutResponse {
    pub positions: Vec<serde_json::Value>,
    pub iterations_run: u32,
    pub converged: bool,
    pub total_energy: f64,
}

// ── Router ───────────────────────────────────

pub fn router() -> Router<crate::ApiState> {
    Router::new().route("/converge", post(converge_layout))
}

// ── Handler ──────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/graph/converge",
    tag = "graph",
    request_body = GraphLayoutRequest,
    responses(
        (status = 200, description = "Converged layout positions", body = GraphLayoutResponse),
        (status = 400, description = "Invalid input", body = ApiError),
        (status = 502, description = "MCP dispatch error", body = ApiError)
    )
)]
async fn converge_layout(
    State(state): State<crate::ApiState>,
    Json(body): Json<GraphLayoutRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if body.nodes.is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "At least one node is required",
        ));
    }

    // Forward as-is to MCP tool (param struct matches)
    let mcp_params = serde_json::json!({
        "nodes": body.nodes,
        "edges": body.edges,
        "dimensions": body.dimensions,
        "iterations": body.iterations,
    });

    let result = mcp_bridge::call_tool("graph_layout_converge", mcp_params, &state.mcp_server)
        .await
        .map_err(|e| ApiError::new("MCP_ERROR", format!("graph_layout_converge failed: {e}")))?;

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
