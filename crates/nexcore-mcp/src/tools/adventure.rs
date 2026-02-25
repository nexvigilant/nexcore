//! Adventure HUD — session tracking for Claude Code adventures.
//!
//! Consolidated from `adventure-hud` satellite MCP server.
//! 6 tools: adventure_start, adventure_task, adventure_skill, adventure_measure,
//!          adventure_milestone, adventure_status.
//!
//! Tier: T3 (ς State + σ Sequence + μ Mapping + N Quantity)

use crate::params::{
    AdventureMeasureParams, AdventureMilestoneParams, AdventureSkillParams, AdventureStartParams,
    AdventureTaskParams,
};
use nexcore_chrono::DateTime;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;

// ============================================================================
// State
// ============================================================================

static ADVENTURE: Mutex<Option<AdventureState>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Default)]
struct AdventureState {
    session_id: String,
    name: String,
    started_at: String,
    tasks: Vec<TaskEvent>,
    skills: HashMap<String, u32>,
    measures: HashMap<String, f64>,
    milestones: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct TaskEvent {
    id: String,
    subject: String,
    status: String,
    timestamp: String,
}

fn status_icon(s: &str) -> &'static str {
    match s {
        "completed" => "\u{2713}",
        "in_progress" => "\u{27F3}",
        _ => "\u{25CB}",
    }
}

// ============================================================================
// Tools
// ============================================================================

/// Start a new adventure session.
pub fn adventure_start(params: AdventureStartParams) -> Result<CallToolResult, McpError> {
    let name = params.name;
    let now = DateTime::now();
    let state = AdventureState {
        session_id: format!("adv-{}", now.timestamp()),
        name: name.clone(),
        started_at: now.to_rfc3339(),
        ..Default::default()
    };
    let msg = format!(
        "Adventure '{}' started! ID: {}",
        state.name, state.session_id
    );
    *ADVENTURE
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))? = Some(state);
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Log a task event.
pub fn adventure_task(params: AdventureTaskParams) -> Result<CallToolResult, McpError> {
    let mut guard = ADVENTURE
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;
    let state = guard.as_mut().ok_or_else(|| {
        McpError::invalid_params("No adventure started. Use adventure_start first.", None)
    })?;

    let t = TaskEvent {
        id: params.id.clone(),
        subject: params.subject.clone(),
        status: params.status.clone(),
        timestamp: DateTime::now().to_rfc3339(),
    };
    let icon = status_icon(&t.status);
    let msg = format!("{} Task #{}: {} [{}]", icon, t.id, t.subject, t.status);
    state.tasks.push(t);
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Log a skill usage.
pub fn adventure_skill(params: AdventureSkillParams) -> Result<CallToolResult, McpError> {
    let mut guard = ADVENTURE
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;
    let state = guard.as_mut().ok_or_else(|| {
        McpError::invalid_params("No adventure started. Use adventure_start first.", None)
    })?;

    let count = state.skills.entry(params.skill.clone()).or_insert(0);
    *count += 1;
    let msg = format!("Skill /{} (x{})", params.skill, count);
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Record a numeric metric.
pub fn adventure_measure(params: AdventureMeasureParams) -> Result<CallToolResult, McpError> {
    let mut guard = ADVENTURE
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;
    let state = guard.as_mut().ok_or_else(|| {
        McpError::invalid_params("No adventure started. Use adventure_start first.", None)
    })?;

    state.measures.insert(params.name.clone(), params.value);
    let msg = format!("{} = {:.2}", params.name, params.value);
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Record a milestone.
pub fn adventure_milestone(params: AdventureMilestoneParams) -> Result<CallToolResult, McpError> {
    let mut guard = ADVENTURE
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;
    let state = guard.as_mut().ok_or_else(|| {
        McpError::invalid_params("No adventure started. Use adventure_start first.", None)
    })?;

    state.milestones.push(params.milestone.clone());
    let msg = format!("Milestone: {}", params.milestone);
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Get full adventure state.
pub fn adventure_status() -> Result<CallToolResult, McpError> {
    let guard = ADVENTURE
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;
    match guard.as_ref() {
        Some(state) => Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(state).unwrap_or_default(),
        )])),
        None => Ok(CallToolResult::success(vec![Content::text(
            "No adventure. Use adventure_start.",
        )])),
    }
}
