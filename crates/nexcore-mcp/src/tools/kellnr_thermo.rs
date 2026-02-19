//! Kellnr Thermodynamic computation tools (4).
//! Consolidated from kellnr-mcp/src/thermo.rs.

use crate::params::kellnr::{
    KellnrThermoArrheniusParams, KellnrThermoBindingAffinityParams, KellnrThermoGibbsParams,
    KellnrThermoKdParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

const R: f64 = 8.314; // J/(mol*K)

fn json_result(value: serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into()),
    )])
}

/// Gibbs free energy: delta_G = delta_H - T * delta_S.
pub fn compute_thermo_gibbs(params: KellnrThermoGibbsParams) -> Result<CallToolResult, McpError> {
    let delta_g = params.delta_h - params.temperature_k * params.delta_s;
    Ok(json_result(json!({
        "success": true,
        "delta_g": delta_g,
        "delta_h": params.delta_h,
        "delta_s": params.delta_s,
        "temperature_k": params.temperature_k,
        "spontaneous": delta_g < 0.0,
        "unit": "J/mol"
    })))
}

/// Dissociation constant from Gibbs free energy: Kd = exp(delta_G / RT).
pub fn compute_thermo_kd(params: KellnrThermoKdParams) -> Result<CallToolResult, McpError> {
    let kd_val = (params.delta_g / (R * params.temperature_k)).exp();
    let ka = 1.0 / kd_val;
    Ok(json_result(json!({
        "success": true,
        "kd": kd_val,
        "ka": ka,
        "delta_g": params.delta_g,
        "temperature_k": params.temperature_k,
        "affinity": if kd_val < 1e-9 { "very high (nM)" } else if kd_val < 1e-6 { "high (μM)" } else { "moderate" }
    })))
}

/// Binding kinetics: Kd, Ka, and dissociation half-life from kon/koff.
pub fn compute_thermo_binding_affinity(
    params: KellnrThermoBindingAffinityParams,
) -> Result<CallToolResult, McpError> {
    if params.kon <= 0.0 {
        return Ok(json_result(
            json!({"success": false, "error": "kon must be > 0"}),
        ));
    }
    let kd_val = params.koff / params.kon;
    let ka = params.kon / params.koff;
    let half_life = 0.693 / params.koff;
    Ok(json_result(json!({
        "success": true,
        "kd": kd_val,
        "ka": ka,
        "kon": params.kon,
        "koff": params.koff,
        "dissociation_half_life": half_life,
        "residence_time": 1.0 / params.koff,
        "unit_kd": "M",
        "unit_half_life": "seconds"
    })))
}

/// Arrhenius temperature-dependent rate: k = A * exp(-Ea / RT).
pub fn compute_thermo_arrhenius(
    params: KellnrThermoArrheniusParams,
) -> Result<CallToolResult, McpError> {
    let k = params.pre_exponential * (-params.activation_energy / (R * params.temperature_k)).exp();
    Ok(json_result(json!({
        "success": true,
        "rate_constant": k,
        "pre_exponential": params.pre_exponential,
        "activation_energy": params.activation_energy,
        "temperature_k": params.temperature_k,
        "unit_ea": "J/mol"
    })))
}
