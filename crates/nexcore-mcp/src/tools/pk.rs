//! PV Pharmacokinetics MCP tools (6)
//!
//! Clinical pharmacokinetic calculations for drug exposure, elimination,
//! distribution, and metabolism. Each tool returns structured JSON with
//! forensic metadata for downstream PV reasoning.

use crate::params::pk::{
    PvPkAucParams, PvPkClearanceParams, PvPkHalfLifeParams, PvPkIonizationParams,
    PvPkMichaelisMentenParams, PvPkSteadyStateParams,
};
use crate::tooling::attach_forensic_meta;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ============================================================================
// AUC — Area Under the Curve
// ============================================================================

/// Calculate AUC via linear or log-linear trapezoidal rule.
///
/// AUC is the primary measure of total systemic drug exposure.
/// Linear trapezoidal is standard; log-linear is preferred during
/// the terminal elimination phase where concentrations decay exponentially.
pub fn pk_auc(params: PvPkAucParams) -> Result<CallToolResult, McpError> {
    let times = &params.times;
    let conc = &params.concentrations;

    if times.len() != conc.len() {
        return Err(McpError::invalid_params(
            "times and concentrations must have equal length",
            None,
        ));
    }
    if times.len() < 2 {
        return Err(McpError::invalid_params(
            "at least 2 time-concentration points required",
            None,
        ));
    }

    let method = params.method.as_str();
    let mut total = 0.0_f64;

    for i in 1..times.len() {
        let dt = times[i] - times[i - 1];
        if dt <= 0.0 {
            return Err(McpError::invalid_params(
                "time points must be monotonically increasing",
                None,
            ));
        }

        let area = match method {
            "log-linear" | "log_linear"
                if conc[i] > 0.0
                    && conc[i - 1] > 0.0
                    && (conc[i] - conc[i - 1]).abs() > 1e-15 =>
            {
                // Log-linear trapezoidal: area = (C1 - C2) × dt / ln(C1/C2)
                (conc[i - 1] - conc[i]) * dt / (conc[i - 1] / conc[i]).ln()
            }
            _ => {
                // Linear trapezoidal: area = (C1 + C2) × dt / 2
                (conc[i - 1] + conc[i]) * dt / 2.0
            }
        };
        total += area;
    }

    let json_val = json!({
        "auc": total,
        "method": method,
        "time_points": times.len(),
        "time_range_h": times.last().unwrap_or(&0.0) - times.first().unwrap_or(&0.0),
        "cmax": conc.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        "unit": "concentration·time (e.g. mg·h/L)",
    });

    // Confidence scales with data density
    let confidence = (times.len() as f64 / 20.0).min(1.0).max(0.3);
    let mut res = CallToolResult::success(vec![Content::text(json_val.to_string())]);
    attach_forensic_meta(&mut res, confidence, None, "pv_pk");
    Ok(res)
}

// ============================================================================
// Clearance — CL = F × Dose / AUC
// ============================================================================

/// Calculate systemic clearance from dose, AUC, and bioavailability.
///
/// CL represents the volume of plasma completely cleared of drug per unit time.
/// For IV: F = 1.0. For oral: F < 1.0 reflects first-pass metabolism.
pub fn pk_clearance(params: PvPkClearanceParams) -> Result<CallToolResult, McpError> {
    if params.dose <= 0.0 {
        return Err(McpError::invalid_params("dose must be > 0", None));
    }
    if params.auc <= 0.0 {
        return Err(McpError::invalid_params("AUC must be > 0", None));
    }
    let f = params.bioavailability.clamp(0.0, 1.0);

    let cl = (f * params.dose) / params.auc;

    let json_val = json!({
        "clearance_L_per_h": cl,
        "dose_mg": params.dose,
        "auc_mg_h_per_L": params.auc,
        "bioavailability": f,
        "formula": "CL = (F × Dose) / AUC",
    });

    let mut res = CallToolResult::success(vec![Content::text(json_val.to_string())]);
    attach_forensic_meta(&mut res, 0.9, None, "pv_pk");
    Ok(res)
}

// ============================================================================
// Half-Life — t½ = 0.693 × Vd / CL
// ============================================================================

/// Calculate elimination half-life from volume of distribution and clearance.
///
/// t½ is the time required for plasma concentration to decrease by 50%.
/// Clinically: ~4–5 half-lives to reach steady state or eliminate drug.
pub fn pk_half_life(params: PvPkHalfLifeParams) -> Result<CallToolResult, McpError> {
    if params.volume_distribution <= 0.0 {
        return Err(McpError::invalid_params(
            "volume_distribution must be > 0",
            None,
        ));
    }
    if params.clearance <= 0.0 {
        return Err(McpError::invalid_params("clearance must be > 0", None));
    }

    let t_half = 0.693 * params.volume_distribution / params.clearance;
    let ke = 0.693 / t_half; // elimination rate constant

    let json_val = json!({
        "half_life_h": t_half,
        "elimination_rate_constant_per_h": ke,
        "volume_distribution_L": params.volume_distribution,
        "clearance_L_per_h": params.clearance,
        "time_to_steady_state_h": 4.5 * t_half,
        "time_to_elimination_h": 5.0 * t_half,
        "formula": "t½ = 0.693 × Vd / CL",
    });

    let mut res = CallToolResult::success(vec![Content::text(json_val.to_string())]);
    attach_forensic_meta(&mut res, 0.95, None, "pv_pk");
    Ok(res)
}

// ============================================================================
// Steady-State Concentration — Css = F × Dose / (CL × τ)
// ============================================================================

/// Calculate average steady-state plasma concentration.
///
/// Css_avg represents the time-averaged drug level during repeated dosing.
/// Reached after ~4–5 half-lives. Used to verify therapeutic window compliance.
pub fn pk_steady_state(params: PvPkSteadyStateParams) -> Result<CallToolResult, McpError> {
    if params.dose <= 0.0 {
        return Err(McpError::invalid_params("dose must be > 0", None));
    }
    if params.clearance <= 0.0 {
        return Err(McpError::invalid_params("clearance must be > 0", None));
    }
    if params.tau <= 0.0 {
        return Err(McpError::invalid_params(
            "dosing interval (tau) must be > 0",
            None,
        ));
    }

    let f = params.bioavailability.clamp(0.0, 1.0);
    let css = (f * params.dose) / (params.clearance * params.tau);

    // Also compute accumulation factor: R = 1 / (1 - e^(-ke×τ))
    // ke = CL / Vd, but we don't have Vd here, so provide Css only
    let json_val = json!({
        "css_avg_mg_per_L": css,
        "dose_mg": params.dose,
        "clearance_L_per_h": params.clearance,
        "tau_h": params.tau,
        "bioavailability": f,
        "daily_dose_mg": params.dose * (24.0 / params.tau),
        "formula": "Css_avg = (F × Dose) / (CL × τ)",
    });

    let mut res = CallToolResult::success(vec![Content::text(json_val.to_string())]);
    attach_forensic_meta(&mut res, 0.9, None, "pv_pk");
    Ok(res)
}

// ============================================================================
// Ionization — Henderson-Hasselbalch
// ============================================================================

/// Calculate ionization ratio via the Henderson-Hasselbalch equation.
///
/// For weak acids: ratio = 10^(pH − pKa). For weak bases: ratio = 10^(pKa − pH).
/// Unionized fraction crosses membranes; ionized fraction is trapped.
/// Critical for predicting absorption (stomach vs intestine) and renal excretion.
pub fn pk_ionization(params: PvPkIonizationParams) -> Result<CallToolResult, McpError> {
    let ratio = if params.is_acid {
        10.0_f64.powf(params.ph - params.pka)
    } else {
        10.0_f64.powf(params.pka - params.ph)
    };

    let ionized_fraction = ratio / (1.0 + ratio);
    let unionized_fraction = 1.0 - ionized_fraction;

    let drug_type = if params.is_acid {
        "weak acid"
    } else {
        "weak base"
    };

    let json_val = json!({
        "ionized_fraction": ionized_fraction,
        "unionized_fraction": unionized_fraction,
        "pka": params.pka,
        "ph": params.ph,
        "drug_type": drug_type,
        "log_ratio": ratio.log10(),
        "membrane_permeable_fraction": unionized_fraction,
        "interpretation": if unionized_fraction > 0.5 {
            "Majority unionized — favors membrane permeation (absorption/distribution)"
        } else {
            "Majority ionized — favors ion trapping (reduced permeation)"
        },
    });

    let mut res = CallToolResult::success(vec![Content::text(json_val.to_string())]);
    attach_forensic_meta(&mut res, 0.95, None, "pv_pk");
    Ok(res)
}

// ============================================================================
// Michaelis-Menten — v = Vmax × [S] / (Km + [S])
// ============================================================================

/// Calculate metabolic rate via Michaelis-Menten enzyme kinetics.
///
/// Models saturable metabolism (e.g. phenytoin, ethanol, aspirin at high doses).
/// When [S] << Km: first-order (rate ∝ [S]).
/// When [S] >> Km: zero-order (rate ≈ Vmax, capacity-limited).
pub fn pk_michaelis_menten(
    params: PvPkMichaelisMentenParams,
) -> Result<CallToolResult, McpError> {
    if params.km <= 0.0 {
        return Err(McpError::invalid_params("Km must be > 0", None));
    }
    if params.vmax <= 0.0 {
        return Err(McpError::invalid_params("Vmax must be > 0", None));
    }
    if params.substrate_concentration < 0.0 {
        return Err(McpError::invalid_params(
            "substrate_concentration must be >= 0",
            None,
        ));
    }

    let denom = params.km + params.substrate_concentration;
    let v = params.vmax * params.substrate_concentration / denom;
    let fraction_vmax = v / params.vmax;

    let kinetic_order = if fraction_vmax > 0.9 {
        "zero-order (near saturation)"
    } else if fraction_vmax > 0.5 {
        "mixed-order (transitional)"
    } else {
        "first-order (linear range)"
    };

    let json_val = json!({
        "velocity": v,
        "vmax": params.vmax,
        "km": params.km,
        "substrate_concentration": params.substrate_concentration,
        "fraction_of_vmax": fraction_vmax,
        "kinetic_order": kinetic_order,
        "intrinsic_clearance": params.vmax / params.km,
        "formula": "v = Vmax × [S] / (Km + [S])",
    });

    // Higher confidence when in linear (predictable) range
    let confidence = if fraction_vmax < 0.5 { 0.95 } else { 0.75 };
    let mut res = CallToolResult::success(vec![Content::text(json_val.to_string())]);
    attach_forensic_meta(&mut res, confidence, None, "pv_pk");
    Ok(res)
}
