//! Workflow Intelligence MCP tools — map workflows, identify gaps, improve live.
//!
//! Tools:
//! - `workflow_map`: DAG of tool transitions across recent sessions
//! - `workflow_gaps`: Gap analysis with health score and recommendations
//! - `workflow_bottlenecks`: Find error-prone tools and inefficient patterns
//! - `workflow_live`: Live session intel with similar workflows and suggestions
//! - `workflow_suggest`: Targeted improvement recommendations

use nexcore_workflow_intel::{analysis, db};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::workflow_intel::{
    WorkflowGapsParams, WorkflowLiveParams, WorkflowMapParams, WorkflowSuggestParams,
};

/// Open brain.db read-only or return MCP error.
fn brain_conn() -> Result<rusqlite::Connection, McpError> {
    let path = db::default_brain_path();
    db::open_brain_db(&path).map_err(|e| McpError::internal_error(e.to_string(), None))
}

/// `workflow_map` — Build a DAG of tool transitions across recent sessions.
///
/// Shows how tools chain together, which tools are most used, and
/// the breakdown of builtin vs MCP vs other tool categories.
pub fn workflow_map(params: WorkflowMapParams) -> Result<CallToolResult, McpError> {
    let conn = brain_conn()?;
    let days = params.days.unwrap_or(30);

    let map = analysis::build_workflow_map(&conn, days)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let transitions_json: Vec<serde_json::Value> = map
        .transitions
        .iter()
        .map(|t| {
            json!({
                "from": t.from,
                "to": t.to,
                "count": t.count,
                "pct": format!("{:.1}%", t.pct)
            })
        })
        .collect();

    let result = json!({
        "window": map.window,
        "sessions_analyzed": map.sessions_analyzed,
        "total_events": map.total_events,
        "top_tools": map.top_tools.iter().map(|(name, count)| json!({"tool": name, "calls": count})).collect::<Vec<_>>(),
        "category_breakdown": map.category_breakdown.iter().map(|(cat, count)| json!({"category": cat, "calls": count})).collect::<Vec<_>>(),
        "transitions": transitions_json
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

/// `workflow_gaps` — Analyze workflows for gaps and inefficiencies.
///
/// Identifies missing automations, error-prone tools, underused capabilities,
/// and returns a health score (0.0–1.0) with prioritized recommendations.
pub fn workflow_gaps(params: WorkflowGapsParams) -> Result<CallToolResult, McpError> {
    let conn = brain_conn()?;
    let days = params.days.unwrap_or(30);

    let analysis = analysis::analyze_gaps(&conn, days)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let gaps_json: Vec<serde_json::Value> = analysis
        .gaps
        .iter()
        .map(|g| {
            json!({
                "type": g.gap_type,
                "description": g.description,
                "severity": g.severity,
                "evidence": g.evidence,
                "suggestion": g.suggestion
            })
        })
        .collect();

    let result = json!({
        "window": analysis.window,
        "health_score": format!("{:.2}", analysis.health_score),
        "gap_count": analysis.gaps.len(),
        "gaps": gaps_json,
        "stats": {
            "sessions": analysis.stats.sessions,
            "sessions_with_errors": analysis.stats.sessions_with_errors,
            "avg_tools_per_session": format!("{:.1}", analysis.stats.avg_tools_per_session),
            "mcp_usage_pct": format!("{:.1}%", analysis.stats.mcp_usage_pct),
            "skill_invocation_rate": format!("{:.0}%", analysis.stats.skill_invocation_rate * 100.0),
            "full_verdict_rate": format!("{:.0}%", analysis.stats.full_verdict_rate * 100.0)
        }
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

/// `workflow_bottlenecks` — Find workflow bottlenecks.
///
/// Identifies high-failure tools, repeated file reads, excessive bash usage,
/// and other patterns that slow down workflows.
pub fn workflow_bottlenecks(params: WorkflowGapsParams) -> Result<CallToolResult, McpError> {
    let conn = brain_conn()?;
    let days = params.days.unwrap_or(30);

    let bottlenecks = analysis::find_bottlenecks(&conn, days)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let bottlenecks_json: Vec<serde_json::Value> = bottlenecks
        .iter()
        .map(|b| {
            json!({
                "name": b.name,
                "type": b.bottleneck_type,
                "impact": format!("{:.2}", b.impact),
                "evidence": b.evidence,
                "recommendation": b.recommendation
            })
        })
        .collect();

    let result = json!({
        "bottleneck_count": bottlenecks.len(),
        "bottlenecks": bottlenecks_json
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

/// `workflow_live` — Live session intelligence.
///
/// Given the current session's tool sequence, finds similar past workflows,
/// suggests next tools, and surfaces warnings about current patterns.
pub fn workflow_live(params: WorkflowLiveParams) -> Result<CallToolResult, McpError> {
    let conn = brain_conn()?;

    let intel = analysis::live_intel(&conn, &params.current_tools)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let similar_json: Vec<serde_json::Value> = intel
        .similar_workflows
        .iter()
        .map(|s| {
            json!({
                "session_id": s.session_id,
                "description": s.description,
                "verdict": s.verdict,
                "similarity": format!("{:.2}", s.similarity)
            })
        })
        .collect();

    let result = json!({
        "current_sequence_length": intel.current_sequence.len(),
        "similar_workflows": similar_json,
        "suggested_next": intel.suggested_next.iter().map(|(tool, conf)| json!({"tool": tool, "confidence": format!("{:.2}", conf)})).collect::<Vec<_>>(),
        "warnings": intel.warnings
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

/// `workflow_suggest` — Targeted improvement recommendations.
///
/// Combines gap analysis and bottleneck detection to produce a prioritized
/// list of workflow improvements with expected impact.
pub fn workflow_suggest(params: WorkflowSuggestParams) -> Result<CallToolResult, McpError> {
    let conn = brain_conn()?;
    let days = params.days.unwrap_or(30);

    let gap_analysis = analysis::analyze_gaps(&conn, days)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    let bottlenecks = analysis::find_bottlenecks(&conn, days)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Merge suggestions from both sources, dedup by description
    let mut suggestions: Vec<serde_json::Value> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for gap in &gap_analysis.gaps {
        if seen.insert(gap.suggestion.clone()) {
            suggestions.push(json!({
                "priority": gap.severity,
                "source": "gap_analysis",
                "issue": gap.description,
                "action": gap.suggestion,
                "evidence": gap.evidence
            }));
        }
    }

    for bottleneck in &bottlenecks {
        if seen.insert(bottleneck.recommendation.clone()) {
            let priority = if bottleneck.impact > 0.3 { 4 } else { 2 };
            suggestions.push(json!({
                "priority": priority,
                "source": "bottleneck",
                "issue": bottleneck.name,
                "action": bottleneck.recommendation,
                "evidence": bottleneck.evidence
            }));
        }
    }

    // Sort by priority descending
    suggestions.sort_by(|a, b| {
        let pa = a["priority"].as_u64().unwrap_or(0);
        let pb = b["priority"].as_u64().unwrap_or(0);
        pb.cmp(&pa)
    });

    let result = json!({
        "health_score": format!("{:.2}", gap_analysis.health_score),
        "suggestion_count": suggestions.len(),
        "suggestions": suggestions,
        "focus_area": if gap_analysis.health_score < 0.5 {
            "Critical — multiple workflow gaps need immediate attention"
        } else if gap_analysis.health_score < 0.75 {
            "Moderate — targeted improvements will have high impact"
        } else {
            "Healthy — fine-tuning for marginal gains"
        }
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}
