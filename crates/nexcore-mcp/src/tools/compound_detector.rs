//! Compound Growth Detector MCP tool.
//!
//! Accepts time-series BasisSnapshots, runs phase and bottleneck detection,
//! returns a structured DetectionResult as JSON.

use crate::params::{CompoundDetectorParams, CompoundDetectorSnapshot};
use nexcore_lex_primitiva::compound::{BasisSnapshot, CompoundTracker};
use nexcore_lex_primitiva::compound_detector::CompoundDetector;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Convert a param snapshot to a domain snapshot.
fn to_basis_snapshot(s: &CompoundDetectorSnapshot) -> BasisSnapshot {
    BasisSnapshot {
        session: s.session.clone(),
        t1_count: s.t1_count,
        t2_p_count: s.t2_p_count,
        t2_c_count: s.t2_c_count,
        t3_count: s.t3_count,
        reused: s.reused,
        total_needed: s.total_needed,
    }
}

/// Detect compound growth phase and bottleneck from time-series snapshots.
pub fn compound_detect(params: CompoundDetectorParams) -> Result<CallToolResult, McpError> {
    if params.snapshots.is_empty() {
        return Err(McpError::invalid_params(
            "At least 1 snapshot is required",
            None,
        ));
    }

    let mut tracker = CompoundTracker::new();
    for snap in &params.snapshots {
        tracker.record(to_basis_snapshot(snap));
    }

    let result = CompoundDetector::detect(&tracker);

    let output = json!({
        "phase": result.phase,
        "bottleneck": result.bottleneck,
        "current_velocity": format!("{:.4}", result.current_velocity),
        "latest_growth_rate": result.latest_growth_rate.map(|r| format!("{:.4}", r)),
        "avg_growth_rate": format!("{:.4}", result.avg_growth_rate),
        "recommendation": result.recommendation,
        "snapshot_count": result.snapshot_count,
        "component_analysis": result.component_analysis.as_ref().map(|ca| json!({
            "basis_value": format!("{:.2}", ca.basis_value),
            "efficiency_value": format!("{:.4}", ca.efficiency_value),
            "reuse_value": format!("{:.4}", ca.reuse_value),
            "basis_contribution_pct": format!("{:.1}%", ca.basis_contribution_pct),
            "efficiency_contribution_pct": format!("{:.1}%", ca.efficiency_contribution_pct),
            "reuse_contribution_pct": format!("{:.1}%", ca.reuse_contribution_pct),
            "weakest_component": ca.weakest_component,
        })),
        "velocity_history": tracker.velocity_history().iter()
            .map(|v| format!("{:.4}", v))
            .collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
