//! Suit Compute MCP tools — flight state, Exo MCU, SoC, redundancy voting.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use suit_compute::flight::FlightState;
use suit_compute::safety::VoteResult;

use crate::params::suit_compute::{ComputeFlightStateParams, ComputeTmrVoteParams};

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

/// Get current flight state snapshot.
pub fn compute_flight_state(_p: ComputeFlightStateParams) -> Result<CallToolResult, McpError> {
    // In a real implementation this would query the MCU bridge.
    let state = FlightState::Nominal;
    ok_json(json!({
        "state": format!("{:?}", state),
        "status": "online",
        "latency_ms": 1.2
    }))
}

/// Simulate a Triple-Modular Redundancy (TMR) vote.
pub fn compute_tmr_vote(p: ComputeTmrVoteParams) -> Result<CallToolResult, McpError> {
    let tol = p.tolerance.unwrap_or(0.01);
    let ab = (p.sensor_a - p.sensor_b).abs() <= tol;
    let bc = (p.sensor_b - p.sensor_c).abs() <= tol;
    let ac = (p.sensor_a - p.sensor_c).abs() <= tol;

    let result = if ab && bc {
        VoteResult::Unanimous(p.sensor_a)
    } else if ab {
        VoteResult::Majority(p.sensor_a)
    } else if bc {
        VoteResult::Majority(p.sensor_b)
    } else if ac {
        VoteResult::Majority(p.sensor_a)
    } else {
        VoteResult::Divergent
    };

    let result_str = match result {
        VoteResult::Unanimous(_) => "Unanimous",
        VoteResult::Majority(_) => "Majority",
        VoteResult::Divergent => "Divergent",
    };

    let resolved_value = match result {
        VoteResult::Unanimous(v) => Some(v),
        VoteResult::Majority(v) => Some(v),
        VoteResult::Divergent => None,
    };

    ok_json(json!({
        "vote_result": result_str,
        "resolved_value": resolved_value,
        "tolerance_used": tol
    }))
}
