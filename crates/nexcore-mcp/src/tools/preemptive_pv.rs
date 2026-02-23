//! Preemptive Pharmacovigilance MCP tools.
//!
//! Three-tier signal detection (Reactive → Predictive → Preemptive) with Gibbs
//! thermodynamic modeling, Hill trajectory amplification, irreversibility weighting,
//! noise floor correction, and competitive inhibition intervention modeling.

use nexcore_preemptive_pv::{
    self, GibbsParams, NoiseParams, ReportingCounts, ReportingDataPoint, Seriousness,
};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::preemptive_pv::{
    PreemptiveEvaluateParams, PreemptiveGibbsParams, PreemptiveInterventionParams,
    PreemptiveNoiseParams, PreemptiveOmegaTableParams, PreemptivePredictiveParams,
    PreemptiveReactiveParams, PreemptiveRequiredStrengthParams, PreemptiveSeverityParams,
    PreemptiveTrajectoryParams,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

fn parse_seriousness(s: &str) -> Option<Seriousness> {
    match s.to_lowercase().as_str() {
        "non_serious" | "nonserious" | "none" => Some(Seriousness::NonSerious),
        "hospitalization" | "hospital" => Some(Seriousness::Hospitalization),
        "disability" => Some(Seriousness::Disability),
        "life_threatening" | "lifethreatening" => Some(Seriousness::LifeThreatening),
        "fatal" | "death" => Some(Seriousness::Fatal),
        _ => None,
    }
}

fn data_points(
    data: &[crate::params::preemptive_pv::TrajectoryDataPoint],
) -> Vec<ReportingDataPoint> {
    data.iter()
        .map(|d| ReportingDataPoint::new(d.time, d.rate))
        .collect()
}

// ── Tools ───────────────────────────────────────────────────────────────────

/// Tier 1: Reactive signal strength (S = observed/expected) with chi-square test.
pub fn preemptive_reactive(p: PreemptiveReactiveParams) -> Result<CallToolResult, McpError> {
    let counts = ReportingCounts::new(p.a, p.b, p.c, p.d);
    let strength = nexcore_preemptive_pv::reactive::signal_strength(&counts);
    let chi2 = nexcore_preemptive_pv::reactive::chi_squared(&counts);
    let threshold = p.threshold.unwrap_or(2.0);
    let is_signal = nexcore_preemptive_pv::reactive::is_signal(&counts, threshold);
    let chi2_sig = nexcore_preemptive_pv::reactive::chi_squared_significant(&counts);

    ok_json(serde_json::json!({
        "tier": 1,
        "tier_name": "Reactive",
        "signal_strength": strength,
        "is_signal": is_signal,
        "threshold": threshold,
        "chi_squared": chi2,
        "chi_squared_significant": chi2_sig,
        "chi_squared_critical": nexcore_preemptive_pv::reactive::CHI2_CRITICAL_005,
        "counts": { "a": p.a, "b": p.b, "c": p.c, "d": p.d, "total": counts.total(), "expected": counts.expected() },
    }))
}

/// Gibbs free energy: signal emergence feasibility. Negative delta_g = favorable.
pub fn preemptive_gibbs(p: PreemptiveGibbsParams) -> Result<CallToolResult, McpError> {
    let params = GibbsParams::new(p.delta_h_mechanism, p.t_exposure, p.delta_s_information);
    let dg = nexcore_preemptive_pv::gibbs::delta_g(&params);
    let favorable = nexcore_preemptive_pv::gibbs::is_favorable(&params);
    let score = nexcore_preemptive_pv::gibbs::feasibility_score(&params);

    ok_json(serde_json::json!({
        "delta_g": dg,
        "is_favorable": favorable,
        "feasibility_score": score,
        "interpretation": if favorable { "Signal emergence thermodynamically favorable" } else { "Signal emergence thermodynamically unfavorable" },
        "params": { "delta_h": p.delta_h_mechanism, "t_exposure": p.t_exposure, "delta_s": p.delta_s_information },
    }))
}

/// Trajectory with Hill amplification from time-series reporting data.
pub fn preemptive_trajectory(p: PreemptiveTrajectoryParams) -> Result<CallToolResult, McpError> {
    if p.data.len() < 2 {
        return err_result("Need at least 2 data points for trajectory computation");
    }

    let points = data_points(&p.data);
    let alpha = p
        .alpha
        .unwrap_or(nexcore_preemptive_pv::trajectory::DEFAULT_ALPHA);
    let gamma_raw = nexcore_preemptive_pv::trajectory::gamma(&points, alpha);
    let gamma_amp = nexcore_preemptive_pv::trajectory::gamma_amplified(&points);

    let n_h = p
        .hill_n
        .unwrap_or(nexcore_preemptive_pv::trajectory::DEFAULT_HILL_COEFFICIENT);
    let k_half = p
        .hill_k_half
        .unwrap_or(nexcore_preemptive_pv::trajectory::DEFAULT_K_HALF);
    let hill_applied = nexcore_preemptive_pv::trajectory::hill_amplify(gamma_raw, n_h, k_half);

    ok_json(serde_json::json!({
        "gamma_raw": gamma_raw,
        "gamma_amplified": gamma_amp,
        "hill_applied": hill_applied,
        "data_points": p.data.len(),
        "params": { "alpha": alpha, "hill_n": n_h, "hill_k_half": k_half },
        "interpretation": if gamma_amp > 0.0 { "Signal trajectory is accelerating" } else { "Signal trajectory is decelerating or flat" },
    }))
}

/// Severity/irreversibility weighting (Omega). Higher = more severe + irreversible.
pub fn preemptive_severity(p: PreemptiveSeverityParams) -> Result<CallToolResult, McpError> {
    match parse_seriousness(&p.seriousness) {
        Some(s) => {
            let omega = nexcore_preemptive_pv::severity::omega(s);
            let omega_norm = nexcore_preemptive_pv::severity::omega_normalized(s);
            ok_json(serde_json::json!({
                "seriousness": p.seriousness,
                "omega": omega,
                "omega_normalized": omega_norm,
                "severity_score": s.severity_score(),
                "irreversibility_factor": s.irreversibility_factor(),
                "formula": "Omega = S * (1 + irreversibility_factor)",
            }))
        }
        None => err_result(&format!(
            "Unknown seriousness: {}. Use: non_serious, hospitalization, disability, life_threatening, fatal",
            p.seriousness
        )),
    }
}

/// Noise floor correction (eta). High eta = stimulated reporting dominates.
pub fn preemptive_noise(p: PreemptiveNoiseParams) -> Result<CallToolResult, McpError> {
    let params = NoiseParams::new(p.r_stimulated, p.r_baseline);
    let params = if let Some(k) = p.k {
        NoiseParams::with_k(p.r_stimulated, p.r_baseline, k)
    } else {
        params
    };

    let eta = nexcore_preemptive_pv::noise::eta(&params);
    let retention = nexcore_preemptive_pv::noise::signal_retention(&params);
    let organic = nexcore_preemptive_pv::noise::is_organic(&params);

    ok_json(serde_json::json!({
        "eta": eta,
        "signal_retention": retention,
        "is_organic": organic,
        "interpretation": if organic {
            "Reporting appears organic (eta < 0.5)"
        } else {
            "Reporting appears stimulated (eta >= 0.5) — signal may be noise-inflated"
        },
        "params": { "r_stimulated": p.r_stimulated, "r_baseline": p.r_baseline, "k": p.k.unwrap_or(5.0) },
    }))
}

/// Tier 2: Predictive signal potential (Psi = feasibility * trajectory * signal_retention).
pub fn preemptive_predictive(p: PreemptivePredictiveParams) -> Result<CallToolResult, McpError> {
    if p.data.len() < 2 {
        return err_result("Need at least 2 data points for predictive computation");
    }

    let gibbs = GibbsParams::new(p.delta_h_mechanism, p.t_exposure, p.delta_s_information);
    let points = data_points(&p.data);
    let noise = if let Some(k) = p.k {
        NoiseParams::with_k(p.r_stimulated, p.r_baseline, k)
    } else {
        NoiseParams::new(p.r_stimulated, p.r_baseline)
    };

    let config = nexcore_preemptive_pv::predictive::PredictiveConfig {
        alpha: p
            .alpha
            .unwrap_or(nexcore_preemptive_pv::trajectory::DEFAULT_ALPHA),
        hill_n: p
            .hill_n
            .unwrap_or(nexcore_preemptive_pv::trajectory::DEFAULT_HILL_COEFFICIENT),
        hill_k_half: p
            .hill_k_half
            .unwrap_or(nexcore_preemptive_pv::trajectory::DEFAULT_K_HALF),
        use_hill_amplification: true,
    };

    let result = nexcore_preemptive_pv::predictive::psi(&gibbs, &points, &noise, &config);

    ok_json(serde_json::json!({
        "tier": 2,
        "tier_name": "Predictive",
        "psi": result.psi,
        "components": {
            "delta_g": result.delta_g,
            "feasibility": result.feasibility,
            "gamma_raw": result.gamma_raw,
            "gamma_amplified": result.gamma_amplified,
            "eta": result.eta,
            "signal_retention": result.signal_retention,
        },
        "formula": "Psi = feasibility * gamma_amplified * signal_retention",
        "interpretation": if result.psi > 0.0 {
            "Signal has predictive potential — emergence feasible and trajectory positive"
        } else {
            "Signal lacks predictive potential — emergence unfavorable or trajectory declining"
        },
    }))
}

/// Tier 3: Full three-tier preemptive evaluation. The crown jewel.
pub fn preemptive_evaluate(p: PreemptiveEvaluateParams) -> Result<CallToolResult, McpError> {
    if p.data.len() < 2 {
        return err_result("Need at least 2 data points for preemptive evaluation");
    }

    let seriousness = match parse_seriousness(&p.seriousness) {
        Some(s) => s,
        None => {
            return err_result(&format!(
                "Unknown seriousness: {}. Use: non_serious, hospitalization, disability, life_threatening, fatal",
                p.seriousness
            ));
        }
    };

    let gibbs = GibbsParams::new(p.delta_h_mechanism, p.t_exposure, p.delta_s_information);
    let points = data_points(&p.data);
    let noise = NoiseParams::new(p.r_stimulated, p.r_baseline);

    let mut config = nexcore_preemptive_pv::preemptive::PreemptiveConfig::default();
    if let Some(cost) = p.intervention_cost {
        config.intervention_cost = cost;
    }
    if let Some(threshold) = p.detection_threshold {
        config.detection_threshold = threshold;
    }

    let result =
        nexcore_preemptive_pv::preemptive::evaluate(&gibbs, &points, &noise, seriousness, &config);

    let decision_str = format!("{:?}", result.decision);
    let tier = result.decision.tier();

    ok_json(serde_json::json!({
        "decision": decision_str,
        "decision_tier": tier,
        "predictive": {
            "psi": result.predictive.psi,
            "delta_g": result.predictive.delta_g,
            "feasibility": result.predictive.feasibility,
            "gamma_raw": result.predictive.gamma_raw,
            "gamma_amplified": result.predictive.gamma_amplified,
            "eta": result.predictive.eta,
            "signal_retention": result.predictive.signal_retention,
        },
        "severity": {
            "omega": result.omega,
            "seriousness": p.seriousness,
        },
        "threshold": {
            "safety_lambda": result.safety_lambda,
            "preemptive_threshold": result.preemptive_threshold,
        },
        "intervention": {
            "pi": result.pi,
            "benefit": result.intervention.as_ref().map(|i| i.reduction_percentage),
        },
        "requires_intervention": result.decision.requires_intervention(),
    }))
}

/// Competitive inhibition: model intervention effect on harm rate.
pub fn preemptive_intervention(
    p: PreemptiveInterventionParams,
) -> Result<CallToolResult, McpError> {
    let k_m = p
        .k_m
        .unwrap_or(nexcore_preemptive_pv::intervention::DEFAULT_K_M);
    let k_i = p
        .k_i
        .unwrap_or(nexcore_preemptive_pv::intervention::DEFAULT_K_I);

    let result = nexcore_preemptive_pv::intervention::intervention_effect(
        p.v_max,
        p.substrate,
        p.inhibitor,
        k_m,
        k_i,
    );

    let uninhibited =
        nexcore_preemptive_pv::intervention::uninhibited_rate(p.v_max, p.substrate, k_m);

    ok_json(serde_json::json!({
        "uninhibited_rate": uninhibited,
        "inhibited_rate": result.inhibited_rate,
        "original_rate": result.original_rate,
        "reduction_fraction": result.reduction_fraction,
        "reduction_percentage": result.reduction_percentage,
        "params": { "v_max": p.v_max, "substrate": p.substrate, "inhibitor": p.inhibitor, "k_m": k_m, "k_i": k_i },
        "reference_interventions": {
            "none": nexcore_preemptive_pv::intervention::INTERVENTION_NONE,
            "dhpc": nexcore_preemptive_pv::intervention::INTERVENTION_DHPC,
            "rems": nexcore_preemptive_pv::intervention::INTERVENTION_REMS,
            "withdrawal": nexcore_preemptive_pv::intervention::INTERVENTION_WITHDRAWAL,
        },
    }))
}

/// Solve for required intervention strength to achieve target harm reduction.
pub fn preemptive_required_strength(
    p: PreemptiveRequiredStrengthParams,
) -> Result<CallToolResult, McpError> {
    let k_m = p
        .k_m
        .unwrap_or(nexcore_preemptive_pv::intervention::DEFAULT_K_M);
    let k_i = p
        .k_i
        .unwrap_or(nexcore_preemptive_pv::intervention::DEFAULT_K_I);

    match nexcore_preemptive_pv::intervention::required_intervention_strength(
        p.v_max,
        p.substrate,
        p.target_reduction,
        k_m,
        k_i,
    ) {
        Some(strength) => ok_json(serde_json::json!({
            "required_strength": strength,
            "target_reduction_fraction": p.target_reduction,
            "target_reduction_percentage": p.target_reduction * 100.0,
            "equivalent_intervention": if strength < 5.0 { "Below DHPC" }
                else if strength < 15.0 { "DHPC-level" }
                else if strength < 50.0 { "REMS-level" }
                else { "Withdrawal-level" },
            "params": { "v_max": p.v_max, "substrate": p.substrate, "k_m": k_m, "k_i": k_i },
        })),
        None => err_result("Cannot achieve target reduction with competitive inhibition model"),
    }
}

/// Get the omega table for all seriousness levels.
pub fn preemptive_omega_table(_p: PreemptiveOmegaTableParams) -> Result<CallToolResult, McpError> {
    let table = nexcore_preemptive_pv::severity::omega_table();
    ok_json(serde_json::json!({
        "levels": table.iter().map(|(s, omega)| serde_json::json!({
            "seriousness": format!("{s:?}"),
            "omega": omega,
            "omega_normalized": nexcore_preemptive_pv::severity::omega_normalized(*s),
            "severity_score": s.severity_score(),
            "irreversibility_factor": s.irreversibility_factor(),
        })).collect::<Vec<_>>(),
        "formula": "Omega = S * (1 + irreversibility_factor)",
    }))
}
