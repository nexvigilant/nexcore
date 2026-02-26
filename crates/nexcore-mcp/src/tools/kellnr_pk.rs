//! Kellnr Pharmacokinetic computation tools (6).
//! Consolidated from kellnr-mcp/src/pk.rs.

use crate::params::kellnr::{
    KellnrPkAucParams, KellnrPkClearanceParams, KellnrPkIonizationParams,
    KellnrPkMichaelisMentenParams, KellnrPkSteadyStateParams, KellnrPkVolumeDistributionParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

fn json_result(value: serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into()),
    )])
}

/// AUC via trapezoidal rule (linear or log-linear).
pub fn compute_pk_auc(params: KellnrPkAucParams) -> Result<CallToolResult, McpError> {
    let times = &params.times;
    let conc = &params.concentrations;
    let method = params.method.as_deref().unwrap_or("linear");

    if times.len() != conc.len() || times.len() < 2 {
        return Ok(json_result(
            json!({"success": false, "error": "times and concentrations must have equal length >= 2"}),
        ));
    }
    let mut total = 0.0;
    for i in 1..times.len() {
        let dt = times[i] - times[i - 1];
        if dt <= 0.0 {
            return Ok(json_result(json!({
                "success": false,
                "error": format!(
                    "Time points must be strictly increasing: times[{}]={} <= times[{}]={}",
                    i, times[i], i - 1, times[i - 1]
                )
            })));
        }
        let area = match method {
            "log-linear" | "log_linear"
                if conc[i] > 0.0 && conc[i - 1] > 0.0 && (conc[i] - conc[i - 1]).abs() > 1e-15 =>
            {
                (conc[i - 1] - conc[i]) * dt / (conc[i - 1] / conc[i]).ln()
            }
            _ => (conc[i - 1] + conc[i]) * dt / 2.0,
        };
        total += area;
    }
    Ok(json_result(json!({
        "success": true,
        "auc": total,
        "method": method,
        "time_points": times.len(),
        "unit": "concentration*time"
    })))
}

/// Time to steady state: t_ss = multiplier * t_half.
pub fn compute_pk_steady_state(
    params: KellnrPkSteadyStateParams,
) -> Result<CallToolResult, McpError> {
    let half_life = params.half_life;
    let multiplier = params.multiplier.unwrap_or(5.0);
    let t_ss = multiplier * half_life;
    let ke = 0.693 / half_life;
    Ok(json_result(json!({
        "success": true,
        "time_to_steady_state": t_ss,
        "half_life": half_life,
        "multiplier": multiplier,
        "elimination_rate_constant": ke,
        "fraction_at_steady_state": 1.0 - (-ke * t_ss).exp()
    })))
}

/// Henderson-Hasselbalch ionization.
pub fn compute_pk_ionization(params: KellnrPkIonizationParams) -> Result<CallToolResult, McpError> {
    let is_acid = params.is_acid.unwrap_or(true);
    let ratio = if is_acid {
        10.0_f64.powf(params.ph - params.pka)
    } else {
        10.0_f64.powf(params.pka - params.ph)
    };
    let ionized_fraction = ratio / (1.0 + ratio);
    let unionized_fraction = 1.0 - ionized_fraction;
    Ok(json_result(json!({
        "success": true,
        "pka": params.pka,
        "ph": params.ph,
        "is_acid": is_acid,
        "ionized_fraction": ionized_fraction,
        "unionized_fraction": unionized_fraction,
        "log_ratio": ratio.log10()
    })))
}

/// Clearance: CL = (F * Dose) / AUC.
pub fn compute_pk_clearance(params: KellnrPkClearanceParams) -> Result<CallToolResult, McpError> {
    let bioavailability = params.bioavailability.unwrap_or(1.0);
    if params.auc <= 0.0 {
        return Ok(json_result(
            json!({"success": false, "error": "AUC must be > 0"}),
        ));
    }
    let cl = (bioavailability * params.dose) / params.auc;
    Ok(json_result(json!({
        "success": true,
        "clearance": cl,
        "dose": params.dose,
        "auc": params.auc,
        "bioavailability": bioavailability,
        "unit": "volume/time"
    })))
}

/// Volume of distribution: Vd = (F * Dose) / C0.
pub fn compute_pk_volume_distribution(
    params: KellnrPkVolumeDistributionParams,
) -> Result<CallToolResult, McpError> {
    let bioavailability = params.bioavailability.unwrap_or(1.0);
    if params.initial_concentration <= 0.0 {
        return Ok(json_result(
            json!({"success": false, "error": "initial_concentration must be > 0"}),
        ));
    }
    let vd = (bioavailability * params.dose) / params.initial_concentration;
    Ok(json_result(json!({
        "success": true,
        "volume_distribution": vd,
        "dose": params.dose,
        "initial_concentration": params.initial_concentration,
        "bioavailability": bioavailability,
        "unit": "volume"
    })))
}

/// Michaelis-Menten: V = Vmax * [S] / (Km + [S]).
pub fn compute_pk_michaelis_menten(
    params: KellnrPkMichaelisMentenParams,
) -> Result<CallToolResult, McpError> {
    if params.km + params.substrate_concentration <= 0.0 {
        return Ok(json_result(
            json!({"success": false, "error": "Km + [S] must be > 0"}),
        ));
    }
    let v =
        params.vmax * params.substrate_concentration / (params.km + params.substrate_concentration);
    let fraction_vmax = v / params.vmax;
    Ok(json_result(json!({
        "success": true,
        "velocity": v,
        "vmax": params.vmax,
        "km": params.km,
        "substrate_concentration": params.substrate_concentration,
        "fraction_of_vmax": fraction_vmax,
        "saturation": if fraction_vmax > 0.9 { "near saturation" } else if fraction_vmax > 0.5 { "moderate" } else { "linear range" }
    })))
}
