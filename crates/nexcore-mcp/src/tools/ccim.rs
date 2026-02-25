//! CCIM MCP tool handlers (3).
//!
//! Capability Compound Interest Machine: equation, assessment, projection.
//! Grounding: ρ(Recursion) + N(Quantity) + →(Causality) + κ(Comparison).

use crate::params::ccim::{CcimAssessParams, CcimEquationParams, CcimProjectParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

fn json_result(value: serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into()),
    )])
}

fn error_result(msg: &str) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!({"success": false, "error": msg}))
            .unwrap_or_else(|_| "{}".into()),
    )])
}

/// Compute C(d) using the CCIM compound interest equation.
pub fn ccim_equation(params: CcimEquationParams) -> Result<CallToolResult, McpError> {
    let rho = match nexcore_ccim::CompoundingRatio::new(params.rho) {
        Ok(r) => r,
        Err(e) => return Ok(error_result(&e.to_string())),
    };

    match nexcore_ccim::ccim_equation(
        params.c0,
        rho,
        params.directives,
        params.t_per_directive,
        params.cumulative_w,
        params.observations,
    ) {
        Ok(measured) => Ok(json_result(json!({
            "success": true,
            "c_d": measured.value,
            "confidence": measured.confidence.value(),
            "c0": params.c0,
            "rho": params.rho,
            "directives": params.directives,
            "t_per_directive": params.t_per_directive,
            "cumulative_w": params.cumulative_w,
            "rule_of_72": if params.rho > 0.0 { 72.0 / (params.rho * 100.0) } else { f64::INFINITY }
        }))),
        Err(e) => Ok(error_result(&e.to_string())),
    }
}

/// Assess current CCIM state: NCRR, FIRE progress.
pub fn ccim_assess(params: CcimAssessParams) -> Result<CallToolResult, McpError> {
    let rho = match nexcore_ccim::CompoundingRatio::new(params.rho) {
        Ok(r) => r,
        Err(e) => return Ok(error_result(&e.to_string())),
    };

    let ncrr = params.rho - params.delta_avg;
    let fire = nexcore_ccim::assess::fire_progress(
        params.c_closing,
        params.fire_threshold,
        params.observations,
    );

    // Conservation check: C_closing = C_opening + new_tools - depreciation
    let new_tools_cu = params.c_closing - params.c_opening + (params.delta_avg * params.c_opening);
    let depreciation_cu = params.delta_avg * params.c_opening;
    let conservation = nexcore_ccim::assess::conservation_check(
        params.c_opening,
        params.c_closing,
        new_tools_cu,
        depreciation_cu,
    );

    Ok(json_result(json!({
        "success": true,
        "rho": rho.value(),
        "delta_avg": params.delta_avg,
        "ncrr": ncrr,
        "ncrr_healthy": ncrr > 0.0,
        "ncrr_target_met": ncrr > 0.20,
        "c_opening": params.c_opening,
        "c_closing": params.c_closing,
        "fire_threshold": params.fire_threshold,
        "fire_progress_pct": fire.progress_pct.value,
        "fire_progress_confidence": fire.progress_pct.confidence.value(),
        "fire_reached": fire.fire_reached,
        "conservation_holds": conservation.is_ok(),
        "conservation_delta": conservation.as_ref().map(|m| m.value).unwrap_or(f64::NAN)
    })))
}

/// Project capability trajectory over N directives.
pub fn ccim_project(params: CcimProjectParams) -> Result<CallToolResult, McpError> {
    let rho = match nexcore_ccim::CompoundingRatio::new(params.rho) {
        Ok(r) => r,
        Err(e) => return Ok(error_result(&e.to_string())),
    };

    match nexcore_ccim::trajectory_project(
        params.current_cu,
        rho,
        params.n_directives,
        params.t_per_directive,
        params.w_per_directive,
        params.fire_threshold,
        params.observations,
    ) {
        Ok(projection) => {
            let trajectory: Vec<serde_json::Value> = projection
                .trajectory
                .iter()
                .map(|p| {
                    json!({
                        "directive": p.directive,
                        "capability_units": p.capability_units
                    })
                })
                .collect();

            Ok(json_result(json!({
                "success": true,
                "current_cu": projection.current_cu,
                "fire_threshold": projection.fire_threshold,
                "directives_to_fire": projection.directives_to_fire.value,
                "directives_to_fire_confidence": projection.directives_to_fire.confidence.value(),
                "rule_of_72": projection.rule_of_72.value,
                "rule_of_72_confidence": projection.rule_of_72.confidence.value(),
                "trajectory_points": trajectory.len(),
                "trajectory": trajectory,
                "rho": params.rho,
                "t_per_directive": params.t_per_directive,
                "w_per_directive": params.w_per_directive
            })))
        }
        Err(e) => Ok(error_result(&e.to_string())),
    }
}
