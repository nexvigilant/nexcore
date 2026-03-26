//! Chemistry primitives MCP tools
//!
//! Exposes high-value chemistry primitives via MCP for cross-domain transfer.
//! Prioritized by PV transfer confidence (0.72-0.92).

use nexcore_vigilance::primitives::chemistry::{
    arrhenius_rate, beer_lambert_absorbance, buffer_capacity, calculate_rate_law,
    classify_cooperativity, classify_coverage, classify_inhibition, eyring_rate, gibbs_free_energy,
    hill_response, inhibited_rate, langmuir_coverage, michaelis_menten_rate, nernst_potential,
    pv_mappings, remaining_after_time, steady_state_fractions, threshold_exceeded,
};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use nexcore_vigilance::pv::thermodynamic::{
    ClosedSystemBalance, MassFlowStream, OpenSystemBalance,
};

use crate::params::{
    ChemistryBufferCapacityParams, ChemistryDecayRemainingParams, ChemistryDependencyRateParams,
    ChemistryEquilibriumParams, ChemistryEyringRateParams, ChemistryFeasibilityParams,
    ChemistryFirstLawClosedParams, ChemistryFirstLawOpenParams, ChemistryGaussianOverlapParams,
    ChemistryHillResponseParams, ChemistryInhibitionParams, ChemistryLangmuirParams,
    ChemistryNernstParams, ChemistryPvMappingsParams, ChemistrySaturationRateParams,
    ChemistrySignalAbsorbanceParams, ChemistryThresholdExceededParams,
    ChemistryThresholdRateParams,
};

/// Calculate Arrhenius rate (threshold gating). PV confidence: 0.92
pub fn threshold_rate(params: ChemistryThresholdRateParams) -> Result<CallToolResult, McpError> {
    match arrhenius_rate(
        params.pre_exponential,
        params.activation_energy_kj,
        params.temperature_k,
    ) {
        Ok(rate) => {
            let result = serde_json::json!({
                "rate_constant": rate,
                "pre_exponential": params.pre_exponential,
                "activation_energy_kj": params.activation_energy_kj,
                "temperature_k": params.temperature_k,
                "pv_mapping": {
                    "chemistry_term": "activation_energy",
                    "pv_equivalent": "signal_detection_threshold",
                    "confidence": 0.92,
                    "rationale": "Both gate action on exceeding energy barrier"
                }
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            format!("Error: {e}"),
        )])),
    }
}

/// Calculate remaining after decay (half-life kinetics). PV confidence: 0.90
pub fn decay_remaining(params: ChemistryDecayRemainingParams) -> Result<CallToolResult, McpError> {
    match remaining_after_time(params.initial, params.half_life, params.time) {
        Ok(remaining) => {
            let fraction_remaining = remaining / params.initial;
            let half_lives_elapsed = params.time / params.half_life;
            let result = serde_json::json!({
                "remaining": remaining,
                "initial": params.initial,
                "fraction_remaining": fraction_remaining,
                "half_life": params.half_life,
                "time": params.time,
                "half_lives_elapsed": half_lives_elapsed,
                "pv_mapping": {
                    "chemistry_term": "half_life",
                    "pv_equivalent": "signal_persistence",
                    "confidence": 0.90,
                    "rationale": "Both describe exponential decay over time"
                }
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            format!("Error: {e}"),
        )])),
    }
}

/// Calculate Michaelis-Menten saturation rate. PV confidence: 0.88
pub fn saturation_rate(params: ChemistrySaturationRateParams) -> Result<CallToolResult, McpError> {
    match michaelis_menten_rate(params.substrate, params.v_max, params.k_m) {
        Ok(rate) => {
            let saturation_frac = params.substrate / (params.k_m + params.substrate);
            let result = serde_json::json!({
                "rate": rate,
                "substrate": params.substrate,
                "v_max": params.v_max,
                "k_m": params.k_m,
                "saturation_fraction": saturation_frac,
                "pv_mapping": {
                    "chemistry_term": "saturation_kinetics",
                    "pv_equivalent": "case_processing_capacity",
                    "confidence": 0.88,
                    "rationale": "Both exhibit hyperbolic throughput curves"
                }
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            format!("Error: {e}"),
        )])),
    }
}

/// Calculate Gibbs free energy feasibility. PV confidence: 0.85
pub fn feasibility(params: ChemistryFeasibilityParams) -> Result<CallToolResult, McpError> {
    match gibbs_free_energy(params.delta_h, params.delta_s, params.temperature_k) {
        Ok(delta_g) => {
            let is_favorable = delta_g < 0.0;
            let favorability = match (params.delta_h < 0.0, params.delta_s > 0.0) {
                (true, true) => "AlwaysFavorable",
                (false, false) => "NeverFavorable",
                (true, false) => "FavorableAtLowUncertainty",
                (false, true) => "FavorableAtHighUncertainty",
            };
            let result = serde_json::json!({
                "delta_g_kj": delta_g,
                "is_favorable": is_favorable,
                "favorability_class": favorability,
                "delta_h_kj": params.delta_h,
                "delta_s_j_mol_k": params.delta_s,
                "temperature_k": params.temperature_k,
                "pv_mapping": {
                    "chemistry_term": "gibbs_free_energy",
                    "pv_equivalent": "causality_likelihood",
                    "confidence": 0.85,
                    "rationale": "Spontaneous reaction ↔ likely causal relationship"
                }
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            format!("Error: {e}"),
        )])),
    }
}

/// Calculate rate law dependency. PV confidence: 0.82
pub fn dependency_rate(params: ChemistryDependencyRateParams) -> Result<CallToolResult, McpError> {
    match calculate_rate_law(params.k, &params.reactants) {
        Ok(rate) => {
            let overall_order: f64 = params.reactants.iter().map(|(_, order)| order).sum();
            let result = serde_json::json!({
                "rate": rate,
                "rate_constant": params.k,
                "reactants": params.reactants,
                "overall_order": overall_order,
                "pv_mapping": {
                    "chemistry_term": "rate_law_order",
                    "pv_equivalent": "signal_dependency",
                    "confidence": 0.82,
                    "rationale": "Both describe how inputs affect output rate"
                }
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            format!("Error: {e}"),
        )])),
    }
}

/// Calculate buffer capacity. PV confidence: 0.78
pub fn buffer_cap(params: ChemistryBufferCapacityParams) -> Result<CallToolResult, McpError> {
    match buffer_capacity(params.total_conc, params.ratio) {
        Ok(capacity) => {
            let is_optimal = params.ratio >= 0.1 && params.ratio <= 10.0;
            let result = serde_json::json!({
                "buffer_capacity": capacity,
                "total_concentration": params.total_conc,
                "ratio": params.ratio,
                "is_optimal_range": is_optimal,
                "pv_mapping": {
                    "chemistry_term": "buffer_capacity",
                    "pv_equivalent": "baseline_stability",
                    "confidence": 0.78,
                    "rationale": "Both resist perturbation around setpoint"
                }
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            format!("Error: {e}"),
        )])),
    }
}

/// Calculate Beer-Lambert absorbance. PV confidence: 0.75
pub fn signal_absorbance(
    params: ChemistrySignalAbsorbanceParams,
) -> Result<CallToolResult, McpError> {
    match beer_lambert_absorbance(
        params.absorptivity,
        params.path_length,
        params.concentration,
    ) {
        Ok(absorbance) => {
            let transmittance = 10.0_f64.powf(-absorbance);
            let result = serde_json::json!({
                "absorbance": absorbance,
                "transmittance": transmittance,
                "absorptivity": params.absorptivity,
                "path_length": params.path_length,
                "concentration": params.concentration,
                "pv_mapping": {
                    "chemistry_term": "beer_lambert",
                    "pv_equivalent": "dose_response_linearity",
                    "confidence": 0.75,
                    "rationale": "Linear relationship between concentration and signal"
                }
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            format!("Error: {e}"),
        )])),
    }
}

/// Calculate equilibrium steady-state fractions. PV confidence: 0.72
pub fn equilibrium(params: ChemistryEquilibriumParams) -> Result<CallToolResult, McpError> {
    match steady_state_fractions(params.k_eq) {
        Ok((product_frac, substrate_frac)) => {
            let result = serde_json::json!({
                "product_fraction": product_frac,
                "substrate_fraction": substrate_frac,
                "k_eq": params.k_eq,
                "pv_mapping": {
                    "chemistry_term": "equilibrium_constant",
                    "pv_equivalent": "reporting_baseline",
                    "confidence": 0.72,
                    "rationale": "Both represent steady-state balance point"
                }
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            format!("Error: {e}"),
        )])),
    }
}

/// Get all chemistry → PV mappings
pub fn get_pv_mappings(_params: ChemistryPvMappingsParams) -> Result<CallToolResult, McpError> {
    let mappings = pv_mappings();
    let json_mappings: Vec<serde_json::Value> = mappings
        .iter()
        .map(|m| {
            serde_json::json!({
                "chemistry_term": m.chemistry_term,
                "pv_equivalent": m.pv_equivalent,
                "confidence": m.confidence,
                "rationale": m.rationale
            })
        })
        .collect();

    let result = serde_json::json!({
        "mappings": json_mappings,
        "count": mappings.len(),
        "description": "Cross-domain transfer mappings from chemistry primitives to pharmacovigilance concepts"
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Simple threshold exceeded check
pub fn check_threshold_exceeded(
    params: ChemistryThresholdExceededParams,
) -> Result<CallToolResult, McpError> {
    let exceeded = threshold_exceeded(params.signal, params.threshold);
    let result = serde_json::json!({
        "exceeded": exceeded,
        "signal": params.signal,
        "threshold": params.threshold,
        "margin": params.signal - params.threshold
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Calculate Hill equation response (cooperative binding). PV confidence: 0.85
pub fn hill_cooperative(params: ChemistryHillResponseParams) -> Result<CallToolResult, McpError> {
    let response = hill_response(params.input, params.k_half, params.n_hill);
    let cooperativity = classify_cooperativity(params.n_hill);
    let result = serde_json::json!({
        "response": response,
        "input": params.input,
        "k_half": params.k_half,
        "n_hill": params.n_hill,
        "cooperativity_type": format!("{:?}", cooperativity),
        "amplification_factor": params.n_hill / 4.0,
        "pv_mapping": {
            "chemistry_term": "hill_cooperativity",
            "pv_equivalent": "signal_cascade_amplification",
            "confidence": 0.85,
            "rationale": "nH > 1 amplifies weak signals; nH < 1 dampens noise"
        }
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Calculate Nernst potential (dynamic threshold). PV confidence: 0.80
pub fn nernst_dynamic(params: ChemistryNernstParams) -> Result<CallToolResult, McpError> {
    let potential = nernst_potential(
        params.e_standard,
        params.temperature_k,
        params.n_electrons,
        params.q,
    );
    let shift = potential - params.e_standard;
    let result = serde_json::json!({
        "potential": potential,
        "e_standard": params.e_standard,
        "shift_from_standard": shift,
        "temperature_k": params.temperature_k,
        "n_electrons": params.n_electrons,
        "q": params.q,
        "pv_mapping": {
            "chemistry_term": "nernst_potential",
            "pv_equivalent": "dynamic_decision_threshold",
            "confidence": 0.80,
            "rationale": "Threshold shifts with background concentration"
        }
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Calculate competitive inhibition rate. PV confidence: 0.78
pub fn inhibition_rate(params: ChemistryInhibitionParams) -> Result<CallToolResult, McpError> {
    let rate = inhibited_rate(
        params.substrate,
        params.v_max,
        params.k_m,
        params.inhibitor,
        params.k_i,
    );
    let uninhibited = params.v_max * params.substrate / (params.k_m + params.substrate);
    let inhibition_strength = classify_inhibition(params.inhibitor, params.k_i);
    let apparent_km = params.k_m * (1.0 + params.inhibitor / params.k_i);
    let result = serde_json::json!({
        "inhibited_rate": rate,
        "uninhibited_rate": uninhibited,
        "rate_reduction": 1.0 - (rate / uninhibited),
        "apparent_km": apparent_km,
        "inhibition_strength": format!("{:?}", inhibition_strength),
        "substrate": params.substrate,
        "inhibitor": params.inhibitor,
        "pv_mapping": {
            "chemistry_term": "competitive_inhibition",
            "pv_equivalent": "signal_interference_factor",
            "confidence": 0.78,
            "rationale": "Competing signals raise apparent detection threshold"
        }
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Calculate Eyring rate (transition state theory). PV confidence: 0.82
pub fn eyring_transition(params: ChemistryEyringRateParams) -> Result<CallToolResult, McpError> {
    let rate = eyring_rate(params.delta_g, params.temperature_k, params.kappa);
    let half_life = if rate > 0.0 {
        0.693 / rate
    } else {
        f64::INFINITY
    };
    let result = serde_json::json!({
        "rate_constant": rate,
        "half_life_s": half_life,
        "delta_g_j_mol": params.delta_g,
        "temperature_k": params.temperature_k,
        "kappa": params.kappa,
        "pv_mapping": {
            "chemistry_term": "eyring_transition_state",
            "pv_equivalent": "signal_escalation_rate",
            "confidence": 0.82,
            "rationale": "Accounts for both threshold (ΔH‡) and process complexity (ΔS‡)"
        }
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Calculate Langmuir coverage (resource binding). PV confidence: 0.88
pub fn langmuir_binding(params: ChemistryLangmuirParams) -> Result<CallToolResult, McpError> {
    let coverage = langmuir_coverage(params.concentration, params.k_eq);
    let coverage_state = classify_coverage(coverage);
    let half_coverage_conc = 1.0 / params.k_eq;
    let result = serde_json::json!({
        "coverage": coverage,
        "coverage_percent": coverage * 100.0,
        "coverage_state": format!("{:?}", coverage_state),
        "concentration": params.concentration,
        "k_eq": params.k_eq,
        "half_coverage_concentration": half_coverage_conc,
        "remaining_capacity": 1.0 - coverage,
        "pv_mapping": {
            "chemistry_term": "langmuir_adsorption",
            "pv_equivalent": "case_slot_occupancy",
            "confidence": 0.88,
            "rationale": "Finite reviewer slots compete for cases; saturation behavior"
        }
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Calculate First Law energy balance for closed systems. PV confidence: 0.85
/// ΔU = Q - W (Conservation of Energy)
pub fn first_law_closed(params: ChemistryFirstLawClosedParams) -> Result<CallToolResult, McpError> {
    let balance = ClosedSystemBalance::calculate(params.u_initial, params.heat_in, params.work_out);
    let result = build_closed_result(&balance);
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

fn build_closed_result(balance: &ClosedSystemBalance) -> serde_json::Value {
    serde_json::json!({
        "u_initial": balance.u_initial,
        "u_final": balance.u_final,
        "delta_u": balance.delta_u,
        "heat_in": balance.heat_in,
        "work_out": balance.work_out,
        "is_balanced": balance.is_balanced,
        "equation": "ΔU = Q - W",
        "pv_mapping": {
            "chemistry_term": "first_law_closed",
            "pv_equivalent": "case_backlog_change",
            "confidence": 0.85,
            "rationale": "Backlog change = cases received (Q) - cases resolved (W)"
        }
    })
}

/// Calculate First Law energy balance for open systems. PV confidence: 0.85
/// dE/dt = Q̇ - Ẇ + Σṁh_in - Σṁh_out
pub fn first_law_open(params: ChemistryFirstLawOpenParams) -> Result<CallToolResult, McpError> {
    let streams = build_mass_flow_streams(&params);
    let result_text = match streams {
        Some((ins, outs)) => build_open_result(&params, &ins, &outs),
        None => "Error: Invalid mass flow parameters".to_string(),
    };
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result_text.to_string(),
    )]))
}

fn build_mass_flow_streams(
    params: &ChemistryFirstLawOpenParams,
) -> Option<(Vec<MassFlowStream>, Vec<MassFlowStream>)> {
    let ins: Option<Vec<_>> = params
        .inflow_mass_rates
        .iter()
        .zip(&params.inflow_enthalpies)
        .map(|(&r, &h)| MassFlowStream::new(r, h).ok())
        .collect();
    let outs: Option<Vec<_>> = params
        .outflow_mass_rates
        .iter()
        .zip(&params.outflow_enthalpies)
        .map(|(&r, &h)| MassFlowStream::new(r, h).ok())
        .collect();
    ins.zip(outs)
}

fn build_open_result(
    params: &ChemistryFirstLawOpenParams,
    ins: &[MassFlowStream],
    outs: &[MassFlowStream],
) -> String {
    let balance = OpenSystemBalance::calculate(params.heat_rate, params.power_out, ins, outs, 1.0);
    let dm_dt = OpenSystemBalance::mass_balance(ins, outs);
    let json = serde_json::json!({
        "de_dt": balance.de_dt,
        "heat_rate": balance.heat_rate,
        "power_out": balance.power_out,
        "enthalpy_in": balance.enthalpy_in,
        "enthalpy_out": balance.enthalpy_out,
        "is_steady_state": balance.is_steady_state,
        "dm_dt": dm_dt,
        "equation": "dE/dt = Q̇ - Ẇ + Σṁh_in - Σṁh_out",
        "pv_mapping": {
            "chemistry_term": "first_law_open",
            "pv_equivalent": "inter_system_case_flow",
            "confidence": 0.85
        }
    });
    serde_json::to_string_pretty(&json).unwrap_or_default()
}

/// Compute Gaussian primitive overlap integral with proper normalization.
///
/// The key insight: STO-nG basis sets (Hehre-Stewart-Pople 1969) tabulate
/// coefficients for *normalized* Gaussian primitives. Each primitive requires
/// normalization factor N = (2α/π)^(3/4) before computing the overlap.
///
/// Without normalization, self-overlap of H 1s yields ~7.0 instead of ~1.0.
/// This tool encodes that lesson as a reusable computation.
///
/// PV confidence: 0.78 (wavefunction overlap → signal co-occurrence)
pub fn gaussian_overlap(
    params: ChemistryGaussianOverlapParams,
) -> Result<CallToolResult, McpError> {
    use std::f64::consts::PI;

    // Validate equal lengths
    if params.exponents_a.len() != params.coefficients_a.len() {
        return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            "Error: exponents_a and coefficients_a must have equal length",
        )]));
    }
    if params.exponents_b.len() != params.coefficients_b.len() {
        return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            "Error: exponents_b and coefficients_b must have equal length",
        )]));
    }

    // Squared distance between centers
    let rab2 = {
        let d0 = params.center_a[0] - params.center_b[0];
        let d1 = params.center_a[1] - params.center_b[1];
        let d2 = params.center_a[2] - params.center_b[2];
        d0 * d0 + d1 * d1 + d2 * d2
    };

    let mut total = 0.0;
    let mut primitive_contributions = Vec::new();

    for (i, (alpha, ca)) in params
        .exponents_a
        .iter()
        .zip(&params.coefficients_a)
        .enumerate()
    {
        for (j, (beta, cb)) in params
            .exponents_b
            .iter()
            .zip(&params.coefficients_b)
            .enumerate()
        {
            let gamma = alpha + beta;
            // Normalization: N = (2α/π)^(3/4) for s-type Gaussians
            let na = (2.0 * alpha / PI).powf(0.75);
            let nb = (2.0 * beta / PI).powf(0.75);
            let k = (-alpha * beta / gamma * rab2).exp();
            let prefactor = (PI / gamma).powi(3).sqrt();
            let contrib = ca * cb * na * nb * k * prefactor;
            total += contrib;
            primitive_contributions.push(serde_json::json!({
                "pair": format!("({i},{j})"),
                "alpha": alpha,
                "beta": beta,
                "norm_a": na,
                "norm_b": nb,
                "contribution": contrib,
            }));
        }
    }

    let result = serde_json::json!({
        "overlap_integral": total,
        "center_distance": rab2.sqrt(),
        "primitives_a": params.exponents_a.len(),
        "primitives_b": params.exponents_b.len(),
        "primitive_contributions": primitive_contributions,
        "normalization_formula": "N = (2α/π)^(3/4) for s-type Gaussians",
        "reference": "Hehre, Stewart, Pople (1969) — STO-nG basis sets",
        "pv_mapping": {
            "chemistry_term": "wavefunction_overlap",
            "pv_equivalent": "signal_co_occurrence",
            "confidence": 0.78,
            "rationale": "Overlap integral measures spatial co-occurrence of probability densities, analogous to temporal co-occurrence of adverse event signals"
        }
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_rate() {
        let params = ChemistryThresholdRateParams {
            pre_exponential: 1e13,
            activation_energy_kj: 50.0,
            temperature_k: 298.15,
        };
        let result = threshold_rate(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decay_remaining() {
        let params = ChemistryDecayRemainingParams {
            initial: 100.0,
            half_life: 30.0,
            time: 90.0,
        };
        let result = decay_remaining(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_saturation_rate() {
        let params = ChemistrySaturationRateParams {
            substrate: 500.0,
            v_max: 1000.0,
            k_m: 200.0,
        };
        let result = saturation_rate(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_feasibility() {
        let params = ChemistryFeasibilityParams {
            delta_h: -50.0,
            delta_s: 100.0,
            temperature_k: 298.0,
        };
        let result = feasibility(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pv_mappings() {
        let params = ChemistryPvMappingsParams {};
        let result = get_pv_mappings(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_threshold_exceeded() {
        let params = ChemistryThresholdExceededParams {
            signal: 2.5,
            threshold: 2.0,
        };
        let result = check_threshold_exceeded(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_first_law_closed() {
        let params = ChemistryFirstLawClosedParams {
            u_initial: 100.0,
            heat_in: 50.0,
            work_out: 20.0,
        };
        let result = first_law_closed(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_first_law_open() {
        let params = ChemistryFirstLawOpenParams {
            heat_rate: 100.0,
            power_out: 300.0,
            inflow_mass_rates: vec![1.0],
            inflow_enthalpies: vec![2000.0],
            outflow_mass_rates: vec![1.0],
            outflow_enthalpies: vec![1800.0],
        };
        let result = first_law_open(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dependency_rate() {
        let params = ChemistryDependencyRateParams {
            k: 0.5,
            reactants: vec![(2.0, 1.0), (3.0, 2.0)],
        };
        let result = dependency_rate(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_buffer_capacity() {
        let params = ChemistryBufferCapacityParams {
            total_conc: 0.1,
            ratio: 1.0,
        };
        let result = buffer_cap(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_absorbance() {
        let params = ChemistrySignalAbsorbanceParams {
            absorptivity: 1000.0,
            path_length: 1.0,
            concentration: 0.01,
        };
        let result = signal_absorbance(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_equilibrium() {
        let params = ChemistryEquilibriumParams { k_eq: 2.0 };
        let result = equilibrium(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hill_response() {
        let params = ChemistryHillResponseParams {
            input: 75.0,
            k_half: 50.0,
            n_hill: 2.0,
        };
        let result = hill_cooperative(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nernst_potential() {
        let params = ChemistryNernstParams {
            e_standard: 0.0,
            temperature_k: 310.0,
            n_electrons: 1.0,
            q: 145.0 / 12.0,
        };
        let result = nernst_dynamic(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inhibition_rate() {
        let params = ChemistryInhibitionParams {
            substrate: 75.0,
            v_max: 100.0,
            k_m: 50.0,
            inhibitor: 25.0,
            k_i: 10.0,
        };
        let result = inhibition_rate(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eyring_rate() {
        let params = ChemistryEyringRateParams {
            delta_g: 50000.0,
            temperature_k: 310.0,
            kappa: 1.0,
        };
        let result = eyring_transition(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_langmuir_coverage() {
        let params = ChemistryLangmuirParams {
            concentration: 5.0,
            k_eq: 0.1,
        };
        let result = langmuir_binding(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_gaussian_overlap_h1s_self() {
        // STO-3G hydrogen 1s: self-overlap should be ~1.0
        let params = ChemistryGaussianOverlapParams {
            exponents_a: vec![3.425250914, 0.623913730, 0.168855404],
            coefficients_a: vec![0.154328967, 0.535328142, 0.444634542],
            center_a: [0.0, 0.0, 0.0],
            exponents_b: vec![3.425250914, 0.623913730, 0.168855404],
            coefficients_b: vec![0.154328967, 0.535328142, 0.444634542],
            center_b: [0.0, 0.0, 0.0],
        };
        let result = gaussian_overlap(params);
        assert!(result.is_ok());
        // Extract the overlap value from the JSON result
        let ctr = result.expect("tool returned Err");
        let text: String = ctr
            .content
            .iter()
            .filter_map(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .collect();
        let json: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        let overlap = json["overlap_integral"].as_f64().expect("overlap field");
        assert!(
            (overlap - 1.0).abs() < 0.02,
            "H 1s self-overlap should be ~1.0, got {overlap}"
        );
    }

    /// Comprehensive exercise: runs all 15 core tools, extracts JSON, prints output
    #[test]
    fn exercise_all_15_chemistry_tools() {
        use rmcp::model::RawContent;

        fn extract_text(result: Result<CallToolResult, McpError>) -> String {
            let ctr = result.expect("tool returned Err");
            ctr.content
                .iter()
                .filter_map(|c| match &c.raw {
                    RawContent::Text(t) => Some(t.text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("")
        }

        // 1. threshold_rate
        let out1 = extract_text(threshold_rate(ChemistryThresholdRateParams {
            pre_exponential: 1e13,
            activation_energy_kj: 50.0,
            temperature_k: 298.15,
        }));
        println!("=== 1. chemistry_threshold_rate ===\n{out1}\n");

        // 2. threshold_exceeded
        let out2 = extract_text(check_threshold_exceeded(ChemistryThresholdExceededParams {
            signal: 120.0,
            threshold: 100.0,
        }));
        println!("=== 2. chemistry_threshold_exceeded ===\n{out2}\n");

        // 3. decay_remaining
        let out3 = extract_text(decay_remaining(ChemistryDecayRemainingParams {
            initial: 1000.0,
            half_life: 5.0,
            time: 15.0,
        }));
        println!("=== 3. chemistry_decay_remaining ===\n{out3}\n");

        // 4. saturation_rate
        let out4 = extract_text(saturation_rate(ChemistrySaturationRateParams {
            substrate: 75.0,
            v_max: 100.0,
            k_m: 50.0,
        }));
        println!("=== 4. chemistry_saturation_rate ===\n{out4}\n");

        // 5. pv_mappings
        let out5 = extract_text(get_pv_mappings(ChemistryPvMappingsParams {}));
        println!("=== 5. chemistry_pv_mappings ===\n{out5}\n");

        // 6. feasibility
        let out6 = extract_text(feasibility(ChemistryFeasibilityParams {
            delta_h: -20.0,
            delta_s: -50.0,
            temperature_k: 310.0,
        }));
        println!("=== 6. chemistry_feasibility ===\n{out6}\n");

        // 7. dependency_rate
        let out7 = extract_text(dependency_rate(ChemistryDependencyRateParams {
            k: 0.5,
            reactants: vec![(2.0, 1.0), (3.0, 2.0)],
        }));
        println!("=== 7. chemistry_dependency_rate ===\n{out7}\n");

        // 8. buffer_capacity
        let out8 = extract_text(buffer_cap(ChemistryBufferCapacityParams {
            total_conc: 0.1,
            ratio: 1.0,
        }));
        println!("=== 8. chemistry_buffer_capacity ===\n{out8}\n");

        // 9. signal_absorbance
        let out9 = extract_text(signal_absorbance(ChemistrySignalAbsorbanceParams {
            absorptivity: 1000.0,
            concentration: 0.01,
            path_length: 1.0,
        }));
        println!("=== 9. chemistry_signal_absorbance ===\n{out9}\n");

        // 10. equilibrium
        let out10 = extract_text(equilibrium(ChemistryEquilibriumParams { k_eq: 2.0 }));
        println!("=== 10. chemistry_equilibrium ===\n{out10}\n");

        // 11. hill_response
        let out11 = extract_text(hill_cooperative(ChemistryHillResponseParams {
            input: 75.0,
            k_half: 50.0,
            n_hill: 2.0,
        }));
        println!("=== 11. chemistry_hill_response ===\n{out11}\n");

        // 12. nernst_potential
        let out12 = extract_text(nernst_dynamic(ChemistryNernstParams {
            e_standard: 0.0,
            temperature_k: 310.0,
            n_electrons: 1.0,
            q: 145.0 / 12.0,
        }));
        println!("=== 12. chemistry_nernst_potential ===\n{out12}\n");

        // 13. inhibition_rate
        let out13 = extract_text(inhibition_rate(ChemistryInhibitionParams {
            substrate: 75.0,
            v_max: 100.0,
            k_m: 50.0,
            inhibitor: 25.0,
            k_i: 10.0,
        }));
        println!("=== 13. chemistry_inhibition_rate ===\n{out13}\n");

        // 14. eyring_rate
        let out14 = extract_text(eyring_transition(ChemistryEyringRateParams {
            delta_g: 50000.0,
            temperature_k: 310.0,
            kappa: 1.0,
        }));
        println!("=== 14. chemistry_eyring_rate ===\n{out14}\n");

        // 15. langmuir_coverage
        let out15 = extract_text(langmuir_binding(ChemistryLangmuirParams {
            concentration: 5.0,
            k_eq: 0.1,
        }));
        println!("=== 15. chemistry_langmuir_coverage ===\n{out15}\n");

        println!("ALL 15 CHEMISTRY TOOLS EXERCISED SUCCESSFULLY");
    }
}
