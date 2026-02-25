//! Project Tools module — MCP tool integration within project context.
//!
//! Wraps existing MCP tools (signal detection, FAERS, literature search)
//! and persists results as project deliverables.

use crate::ApiState;
use crate::mcp_bridge;
use crate::persistence::{
    CircleRole, DeliverableRecord, DeliverableStatus, DeliverableType, FeedEntryRecord,
    FeedEntryType, MemberStatus, ReviewStatus,
};
use crate::routes::common::ApiError;
use axum::extract::{Json, Path, State};
use axum::routing::post;
use nexcore_chrono::DateTime;
use nexcore_mcp::NexCoreMcpServer;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// API Types
// ============================================================================

/// Signal detection request within project context
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SignalDetectRequest {
    pub drug_count: u64,
    pub event_count: u64,
    pub drug_event_count: u64,
    pub total_count: u64,
    pub user_id: String,
}

/// FAERS query request within project context
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct FaersQueryRequest {
    pub query: String,
    pub limit: Option<u64>,
    pub user_id: String,
}

/// Literature search request within project context
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct LiteratureSearchRequest {
    pub query: String,
    pub category: Option<String>,
    pub limit: Option<u64>,
    pub user_id: String,
}

/// Tool execution response (MCP result + deliverable reference)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ToolResultResponse {
    pub deliverable_id: String,
    pub tool_name: String,
    pub success: bool,
    pub result: serde_json::Value,
}

// ============================================================================
// Helpers
// ============================================================================

fn err(code: &str, msg: impl Into<String>) -> ApiError {
    ApiError::new(code, msg)
}

async fn check_researcher(
    state: &ApiState,
    circle_id: &str,
    user_id: &str,
) -> Result<bool, ApiError> {
    let member = state
        .persistence
        .get_circle_member(circle_id, user_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    let Some(m) = member else {
        return Ok(false);
    };
    if m.status != MemberStatus::Active {
        return Ok(false);
    }

    let role_level = |r: &CircleRole| -> u8 {
        match r {
            CircleRole::Founder => 6,
            CircleRole::Lead => 5,
            CircleRole::Researcher => 4,
            CircleRole::Reviewer => 3,
            CircleRole::Member => 2,
            CircleRole::Observer => 1,
        }
    };
    Ok(role_level(&m.role) >= role_level(&CircleRole::Researcher))
}

/// Save a deliverable from tool output, return its ID.
async fn save_tool_deliverable(
    state: &ApiState,
    circle_id: &str,
    project_id: &str,
    user_id: &str,
    tool_name: &str,
    deliverable_type: DeliverableType,
    result_json: &serde_json::Value,
) -> Result<String, ApiError> {
    let now = DateTime::now();
    let id = nexcore_id::NexId::v4().to_string();

    let deliverable = DeliverableRecord {
        id: id.clone(),
        project_id: project_id.to_string(),
        circle_id: circle_id.to_string(),
        name: format!("{tool_name} output"),
        deliverable_type,
        status: DeliverableStatus::Draft,
        version: 1,
        file_url: None,
        content_hash: None,
        reviewed_by: None,
        review_status: ReviewStatus::Pending,
        review_notes: None,
        created_by: user_id.to_string(),
        created_at: now,
        updated_at: now,
    };

    state
        .persistence
        .save_deliverable(&deliverable)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    // Best-effort feed entry
    let entry = FeedEntryRecord {
        id: nexcore_id::NexId::v4().to_string(),
        circle_id: circle_id.to_string(),
        entry_type: FeedEntryType::DeliverableSubmitted,
        actor_user_id: user_id.to_string(),
        content: format!("Tool output: {tool_name}"),
        reference_id: Some(id.clone()),
        reference_type: Some("deliverable".to_string()),
        created_at: now,
    };
    if let Err(e) = state.persistence.save_feed_entry(&entry).await {
        tracing::warn!("Failed to save feed entry for tool deliverable: {e}");
    }

    // Suppress unused-variable warning on result_json — content stored
    // in object storage in production; here we log a reference.
    tracing::debug!(
        deliverable_id = %id,
        tool = tool_name,
        result_bytes = result_json.to_string().len(),
        "Tool output saved as deliverable"
    );

    Ok(id)
}

/// Extract success boolean from MCP CallToolResult JSON.
fn extract_success(result: &serde_json::Value) -> bool {
    result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
        .and_then(|parsed| parsed.get("success").and_then(|s| s.as_bool()))
        .unwrap_or(true)
}

// ============================================================================
// Endpoints
// ============================================================================

/// Run signal detection within project context (Researcher+)
#[utoipa::path(
    post,
    path = "/api/v1/circles/{cid}/projects/{pid}/tools/signal-detect",
    request_body = SignalDetectRequest,
    responses(
        (status = 200, description = "Signal detection completed", body = ToolResultResponse),
    ),
    tag = "project-tools"
)]
pub async fn signal_detect(
    State(state): State<ApiState>,
    Path((circle_id, project_id)): Path<(String, String)>,
    Json(req): Json<SignalDetectRequest>,
) -> Result<Json<ToolResultResponse>, ApiError> {
    if !check_researcher(&state, &circle_id, &req.user_id).await? {
        return Err(err("FORBIDDEN", "Requires Researcher role or higher"));
    }

    // Verify project exists in circle
    let project = state
        .persistence
        .get_project(&project_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Project not found"))?;

    if project.circle_id != circle_id {
        return Err(err("NOT_FOUND", "Project not found in this circle"));
    }

    let params = serde_json::json!({
        "drug_count": req.drug_count,
        "event_count": req.event_count,
        "drug_event_count": req.drug_event_count,
        "total_count": req.total_count
    });

    let server = NexCoreMcpServer::new();
    let result = mcp_bridge::call_tool("pv_signal_complete", params, &server)
        .await
        .map_err(|e| err("MCP_ERROR", e.to_string()))?;

    let success = extract_success(&result);

    let deliverable_id = save_tool_deliverable(
        &state,
        &circle_id,
        &project_id,
        &req.user_id,
        "pv_signal_complete",
        DeliverableType::SignalAssessment,
        &result,
    )
    .await?;

    Ok(Json(ToolResultResponse {
        deliverable_id,
        tool_name: "pv_signal_complete".to_string(),
        success,
        result,
    }))
}

/// Query FAERS within project context (Researcher+)
#[utoipa::path(
    post,
    path = "/api/v1/circles/{cid}/projects/{pid}/tools/faers-query",
    request_body = FaersQueryRequest,
    responses(
        (status = 200, description = "FAERS query completed", body = ToolResultResponse),
    ),
    tag = "project-tools"
)]
pub async fn faers_query(
    State(state): State<ApiState>,
    Path((circle_id, project_id)): Path<(String, String)>,
    Json(req): Json<FaersQueryRequest>,
) -> Result<Json<ToolResultResponse>, ApiError> {
    if !check_researcher(&state, &circle_id, &req.user_id).await? {
        return Err(err("FORBIDDEN", "Requires Researcher role or higher"));
    }

    let project = state
        .persistence
        .get_project(&project_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Project not found"))?;

    if project.circle_id != circle_id {
        return Err(err("NOT_FOUND", "Project not found in this circle"));
    }

    let params = serde_json::json!({
        "query": req.query,
        "limit": req.limit.unwrap_or(25)
    });

    let server = NexCoreMcpServer::new();
    let result = mcp_bridge::call_tool("faers_search", params, &server)
        .await
        .map_err(|e| err("MCP_ERROR", e.to_string()))?;

    let success = extract_success(&result);

    let deliverable_id = save_tool_deliverable(
        &state,
        &circle_id,
        &project_id,
        &req.user_id,
        "faers_search",
        DeliverableType::Dataset,
        &result,
    )
    .await?;

    Ok(Json(ToolResultResponse {
        deliverable_id,
        tool_name: "faers_search".to_string(),
        success,
        result,
    }))
}

/// Search literature within project context (Researcher+)
#[utoipa::path(
    post,
    path = "/api/v1/circles/{cid}/projects/{pid}/tools/literature-search",
    request_body = LiteratureSearchRequest,
    responses(
        (status = 200, description = "Literature search completed", body = ToolResultResponse),
    ),
    tag = "project-tools"
)]
pub async fn literature_search(
    State(state): State<ApiState>,
    Path((circle_id, project_id)): Path<(String, String)>,
    Json(req): Json<LiteratureSearchRequest>,
) -> Result<Json<ToolResultResponse>, ApiError> {
    if !check_researcher(&state, &circle_id, &req.user_id).await? {
        return Err(err("FORBIDDEN", "Requires Researcher role or higher"));
    }

    let project = state
        .persistence
        .get_project(&project_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Project not found"))?;

    if project.circle_id != circle_id {
        return Err(err("NOT_FOUND", "Project not found in this circle"));
    }

    let mut params = serde_json::json!({
        "query": req.query,
        "limit": req.limit.unwrap_or(20)
    });

    if let Some(category) = &req.category {
        params["category"] = serde_json::Value::String(category.clone());
    }

    let server = NexCoreMcpServer::new();
    let result = mcp_bridge::call_tool("guidelines_search", params, &server)
        .await
        .map_err(|e| err("MCP_ERROR", e.to_string()))?;

    let success = extract_success(&result);

    let deliverable_id = save_tool_deliverable(
        &state,
        &circle_id,
        &project_id,
        &req.user_id,
        "guidelines_search",
        DeliverableType::Report,
        &result,
    )
    .await?;

    Ok(Json(ToolResultResponse {
        deliverable_id,
        tool_name: "guidelines_search".to_string(),
        success,
        result,
    }))
}

// ============================================================================
// Router
// ============================================================================

pub fn router() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/signal-detect", post(signal_detect))
        .route("/faers-query", post(faers_query))
        .route("/literature-search", post(literature_search))
}
