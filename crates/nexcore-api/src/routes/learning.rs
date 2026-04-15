//! Learning DAG REST endpoint
//!
//! Dispatches to `learning_dag_resolve` MCP tool via in-process bridge.
//! GET /api/v1/learning/dag

#[cfg(not(feature = "mcp-bridge"))]
use crate::mcp_bridge::NexCoreMcpServer;
use axum::{Json, Router, extract::Query, routing::get};
#[cfg(feature = "mcp-bridge")]
use nexcore_mcp::NexCoreMcpServer;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::ApiError;
use crate::mcp_bridge;

// ── Request ──────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct LearningDagQuery {
    /// Capability Pathway ID
    pub pathway_id: String,
    /// User ID for personalized completion state
    #[serde(default)]
    pub user_id: Option<String>,
}

// ── Response ─────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct LearningDagResponse {
    pub pathway: String,
    pub nodes: Vec<serde_json::Value>,
    pub edges: Vec<serde_json::Value>,
    pub total_completion: f64,
    pub unlocked_count: u32,
}

// ── Router ───────────────────────────────────

pub fn router() -> Router<crate::ApiState> {
    Router::new().route("/dag", get(resolve_dag))
}

// ── Handler ──────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/learning/dag",
    tag = "learning",
    params(
        ("pathway_id" = String, Query, description = "Capability Pathway ID"),
        ("user_id" = Option<String>, Query, description = "User ID for completion state")
    ),
    responses(
        (status = 200, description = "Resolved learning DAG", body = LearningDagResponse),
        (status = 400, description = "Missing pathway_id", body = ApiError),
        (status = 502, description = "MCP dispatch error", body = ApiError)
    )
)]
async fn resolve_dag(
    Query(params): Query<LearningDagQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if params.pathway_id.trim().is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "pathway_id parameter is required",
        ));
    }

    let mut mcp_params = serde_json::Map::new();
    mcp_params.insert(
        "pathway_id".to_string(),
        serde_json::json!(params.pathway_id),
    );
    if let Some(ref user_id) = params.user_id {
        mcp_params.insert("user_id".to_string(), serde_json::json!(user_id));
    }

    let server = NexCoreMcpServer::new();
    let result = mcp_bridge::call_tool(
        "learning_dag_resolve",
        serde_json::Value::Object(mcp_params),
        &server,
    )
    .await
    .map_err(|e| ApiError::new("MCP_ERROR", format!("learning_dag_resolve failed: {e}")))?;

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
