//! FAERS Advanced Analytics MCP tools (Algorithms A77, A78, A79, A80, A81, A82).
//!
//! Novel signal detection exploiting previously unused FAERS data dimensions:
//! - A82: Outcome-conditioned signal strength (→+∝+ς+κ)
//! - A77: Signal velocity detection (σ+ν+→+N)
//! - A80: Seriousness cascade detection (∂+κ+∝+→+ς)
//! - A78: Polypharmacy interaction signal (×+∂+κ+N)
//! - A79: Reporter-weighted disproportionality (∃+κ+N+∂)
//! - A81: Geographic signal divergence (λ+κ+ν+∂)

use crate::params::{
    FaersGeographicDivergenceParams, FaersOutcomeConditionedParams, FaersPolypharmacyParams,
    FaersReporterWeightedParams, FaersSeriousnessCascadeParams, FaersSignalVelocityParams,
};
use nexcore_faers_etl::analytics::{
    CascadeConfig, CaseSeriousness, DrugCharacterization, GeographicCase, GeographicConfig,
    OutcomeCase, OutcomeConditionedConfig, PolypharmacyCase, PolypharmacyConfig, ReporterCase,
    ReporterWeightedConfig, SeriousnessCase, SeriousnessFlag, TemporalCase, VelocityConfig,
    compute_geographic_divergence, compute_outcome_conditioned, compute_polypharmacy_signals,
    compute_reporter_weighted, compute_seriousness_cascade, compute_signal_velocity,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::HashMap;

/// Round to 4 decimal places for readability.
fn round4(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}

fn text_result(value: &serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string()),
    )])
}

// =============================================================================
// A82 — Outcome-Conditioned Signal Strength
// =============================================================================

/// Compute outcome-conditioned signals (Algorithm A82).
pub fn outcome_conditioned(
    params: FaersOutcomeConditionedParams,
) -> Result<CallToolResult, McpError> {
    if params.cases.is_empty() {
        return Err(McpError::invalid_params("cases array is empty", None));
    }

    // Convert params to domain types
    let cases: Vec<OutcomeCase> = params
        .cases
        .iter()
        .map(|c| OutcomeCase {
            drug: c.drug.clone(),
            event: c.event.clone(),
            outcome_code: c.outcome_code.clone(),
        })
        .collect();

    let mut standard_prrs: HashMap<(String, String), f64> = HashMap::new();
    for p in &params.standard_prrs {
        standard_prrs.insert((p.drug.to_uppercase(), p.event.to_uppercase()), p.prr);
    }

    let config = OutcomeConditionedConfig {
        prr_threshold: params.prr_threshold.unwrap_or(2.0),
        min_cases: params.min_cases.unwrap_or(3),
    };

    let results = compute_outcome_conditioned(&cases, &standard_prrs, &config);

    let signals: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            let distribution: serde_json::Value = r
                .outcome_distribution
                .iter()
                .map(|(k, v)| (format!("{k}"), json!(v)))
                .collect::<serde_json::Map<String, serde_json::Value>>()
                .into();

            json!({
                "drug": r.drug,
                "event": r.event,
                "total_cases": r.total_cases,
                "outcome_severity_index": round4(r.outcome_severity_index),
                "standard_prr": round4(r.standard_prr),
                "adjusted_prr": round4(r.adjusted_prr),
                "adjustment_factor": round4(r.adjustment_factor),
                "fatality_rate": round4(r.fatality_rate),
                "is_signal": r.is_signal,
                "outcome_distribution": distribution,
            })
        })
        .collect();

    let signal_count = results.iter().filter(|r| r.is_signal).count();

    Ok(text_result(&json!({
        "algorithm": "A82",
        "name": "Outcome-Conditioned Signal Strength",
        "description": "Adjusts standard PRR by reaction outcome severity. Fatal outcomes amplify signals; recovered outcomes dampen them.",
        "total_pairs": results.len(),
        "signals_detected": signal_count,
        "config": {
            "prr_threshold": config.prr_threshold,
            "min_cases": config.min_cases,
        },
        "results": signals,
    })))
}

// =============================================================================
// A77 — Signal Velocity Detector
// =============================================================================

/// Compute signal velocities (Algorithm A77).
pub fn signal_velocity(params: FaersSignalVelocityParams) -> Result<CallToolResult, McpError> {
    if params.cases.is_empty() {
        return Err(McpError::invalid_params("cases array is empty", None));
    }

    let cases: Vec<TemporalCase> = params
        .cases
        .iter()
        .map(|c| TemporalCase {
            drug: c.drug.clone(),
            event: c.event.clone(),
            receipt_date: c.receipt_date.clone(),
        })
        .collect();

    let mut known_prrs: HashMap<(String, String), f64> = HashMap::new();
    for p in &params.known_prrs {
        known_prrs.insert((p.drug.to_uppercase(), p.event.to_uppercase()), p.prr);
    }

    let config = VelocityConfig {
        min_months: params.min_months.unwrap_or(3),
        min_cases: params.min_cases.unwrap_or(3),
        acceleration_threshold: params.acceleration_threshold.unwrap_or(0.5),
        known_prrs,
        prr_early_warning_threshold: 2.0,
    };

    let results = compute_signal_velocity(&cases, &config);

    let velocities: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            let monthly: Vec<serde_json::Value> = r
                .monthly_counts
                .iter()
                .map(|(m, c)| json!({"month": m.as_str(), "count": c}))
                .collect();

            json!({
                "drug": r.drug,
                "event": r.event,
                "total_cases": r.total_cases,
                "active_months": r.active_months,
                "current_velocity": round4(r.current_velocity),
                "current_acceleration": round4(r.current_acceleration),
                "mean_velocity": round4(r.mean_velocity),
                "peak_velocity": round4(r.peak_velocity),
                "peak_month": r.peak_month.as_ref().map(|m| m.as_str()),
                "is_accelerating": r.is_accelerating,
                "is_early_warning": r.is_early_warning,
                "monthly_counts": monthly,
            })
        })
        .collect();

    let accelerating = results.iter().filter(|r| r.is_accelerating).count();
    let early_warnings = results.iter().filter(|r| r.is_early_warning).count();

    Ok(text_result(&json!({
        "algorithm": "A77",
        "name": "Signal Velocity Detector",
        "description": "Detects emerging signals by measuring temporal acceleration in reporting frequency. Early warnings flag accelerating signals below PRR threshold.",
        "total_pairs": results.len(),
        "accelerating_signals": accelerating,
        "early_warnings": early_warnings,
        "config": {
            "min_months": config.min_months,
            "min_cases": config.min_cases,
            "acceleration_threshold": config.acceleration_threshold,
        },
        "results": velocities,
    })))
}

// =============================================================================
// A80 — Seriousness Cascade Detector
// =============================================================================

/// Compute seriousness cascades (Algorithm A80).
pub fn seriousness_cascade(
    params: FaersSeriousnessCascadeParams,
) -> Result<CallToolResult, McpError> {
    if params.cases.is_empty() {
        return Err(McpError::invalid_params("cases array is empty", None));
    }

    let cases: Vec<SeriousnessCase> = params
        .cases
        .iter()
        .map(|c| {
            let seriousness = CaseSeriousness::from_openfda(
                c.seriousness_death.as_deref(),
                c.seriousness_hospitalization.as_deref(),
                c.seriousness_disabling.as_deref(),
                c.seriousness_congenital.as_deref(),
                c.seriousness_life_threatening.as_deref(),
                c.seriousness_other.as_deref(),
            );
            SeriousnessCase {
                drug: c.drug.clone(),
                event: c.event.clone(),
                seriousness,
                receipt_date: c.receipt_date.clone(),
            }
        })
        .collect();

    let config = CascadeConfig {
        min_cases: params.min_cases.unwrap_or(3),
        death_rate_review_threshold: params.death_rate_threshold.unwrap_or(0.1),
        ..Default::default()
    };

    let results = compute_seriousness_cascade(&cases, &config);

    let cascades: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            let flag_rates: serde_json::Value = r
                .flag_rates
                .iter()
                .map(|(k, v)| (format!("{k}"), json!(round4(*v))))
                .collect::<serde_json::Map<String, serde_json::Value>>()
                .into();

            let flag_counts: serde_json::Value = r
                .flag_distribution
                .iter()
                .map(|(k, v)| (format!("{k}"), json!(v)))
                .collect::<serde_json::Map<String, serde_json::Value>>()
                .into();

            let monthly: Vec<serde_json::Value> = r
                .monthly_cascade_scores
                .iter()
                .map(|(m, s)| json!({"month": m.as_str(), "cascade_score": round4(*s)}))
                .collect();

            json!({
                "drug": r.drug,
                "event": r.event,
                "total_cases": r.total_cases,
                "mean_cascade_score": round4(r.mean_cascade_score),
                "death_rate": round4(r.death_rate),
                "cascade_velocity": round4(r.cascade_velocity),
                "is_escalating": r.is_escalating,
                "max_observed_severity": r.max_observed_severity.map(|f| format!("{f}")),
                "requires_immediate_review": r.requires_immediate_review,
                "flag_rates": flag_rates,
                "flag_counts": flag_counts,
                "monthly_cascade_scores": monthly,
            })
        })
        .collect();

    let escalating = results.iter().filter(|r| r.is_escalating).count();
    let requiring_review = results
        .iter()
        .filter(|r| r.requires_immediate_review)
        .count();

    // P0 flag severity levels for reference
    let severity_reference: Vec<serde_json::Value> = SeriousnessFlag::all()
        .iter()
        .map(|f| json!({"flag": format!("{f}"), "weight": f.weight()}))
        .collect();

    Ok(text_result(&json!({
        "algorithm": "A80",
        "name": "Seriousness Cascade Detector",
        "description": "Detects signals escalating in severity using all 6 FAERS seriousness flags. P0 patient safety: flags requiring immediate human review when death rate exceeds threshold.",
        "total_pairs": results.len(),
        "escalating_signals": escalating,
        "requiring_immediate_review": requiring_review,
        "config": {
            "min_cases": config.min_cases,
            "death_rate_threshold": config.death_rate_review_threshold,
        },
        "severity_weights": severity_reference,
        "results": cascades,
    })))
}

// =============================================================================
// A78 — Polypharmacy Interaction Signal
// =============================================================================

/// Compute polypharmacy interaction signals (Algorithm A78).
pub fn polypharmacy(params: FaersPolypharmacyParams) -> Result<CallToolResult, McpError> {
    if params.cases.is_empty() {
        return Err(McpError::invalid_params("cases array is empty", None));
    }

    let cases: Vec<PolypharmacyCase> = params
        .cases
        .iter()
        .map(|c| PolypharmacyCase {
            case_id: c.case_id.clone(),
            drugs: c
                .drugs
                .iter()
                .map(|d| {
                    let char = DrugCharacterization::from_code(&d.characterization)
                        .unwrap_or(DrugCharacterization::Suspect);
                    (d.name.clone(), char)
                })
                .collect(),
            event: c.event.clone(),
        })
        .collect();

    let config = PolypharmacyConfig {
        min_pair_count: params.min_pair_count.unwrap_or(3),
        interaction_threshold: params.interaction_threshold.unwrap_or(1.0),
    };

    let results = compute_polypharmacy_signals(&cases, &config);

    let signals: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            json!({
                "drug_a": r.drug_a,
                "drug_b": r.drug_b,
                "event": r.event,
                "pair_count": r.pair_count,
                "drug_a_only_count": r.drug_a_only_count,
                "drug_b_only_count": r.drug_b_only_count,
                "total_event_cases": r.total_event_cases,
                "pair_prr": round4(r.pair_prr),
                "individual_prr_a": round4(r.individual_prr_a),
                "individual_prr_b": round4(r.individual_prr_b),
                "interaction_signal": round4(r.interaction_signal),
                "is_synergistic": r.is_synergistic,
            })
        })
        .collect();

    let synergistic_count = results.iter().filter(|r| r.is_synergistic).count();

    Ok(text_result(&json!({
        "algorithm": "A78",
        "name": "Polypharmacy Interaction Signal",
        "description": "Detects drug pairs with disproportionate co-occurrence signals. Positive interaction_signal indicates synergistic toxicity invisible to single-drug analysis.",
        "total_pairs": results.len(),
        "synergistic_signals": synergistic_count,
        "config": {
            "min_pair_count": config.min_pair_count,
            "interaction_threshold": config.interaction_threshold,
        },
        "results": signals,
    })))
}

// =============================================================================
// A79 — Reporter-Weighted Disproportionality
// =============================================================================

/// Compute reporter-weighted signals (Algorithm A79).
pub fn reporter_weighted(params: FaersReporterWeightedParams) -> Result<CallToolResult, McpError> {
    if params.cases.is_empty() {
        return Err(McpError::invalid_params("cases array is empty", None));
    }

    let cases: Vec<ReporterCase> = params
        .cases
        .iter()
        .map(|c| ReporterCase {
            drug: c.drug.clone(),
            event: c.event.clone(),
            qualification_code: c.qualification_code.clone(),
        })
        .collect();

    let config = ReporterWeightedConfig {
        min_cases: params.min_cases.unwrap_or(3),
        diversity_threshold: params.diversity_threshold.unwrap_or(0.5),
    };

    let results = compute_reporter_weighted(&cases, &config);

    let signals: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            let distribution: serde_json::Value = r
                .reporter_distribution
                .iter()
                .map(|(k, v)| (format!("{k}"), json!(v)))
                .collect::<serde_json::Map<String, serde_json::Value>>()
                .into();

            json!({
                "drug": r.drug,
                "event": r.event,
                "raw_count": r.raw_count,
                "weighted_count": round4(r.weighted_count),
                "reporter_distribution": distribution,
                "reporter_diversity_index": round4(r.reporter_diversity_index),
                "normalized_diversity": round4(r.normalized_diversity),
                "mean_reporter_weight": round4(r.mean_reporter_weight),
                "is_multi_source_confirmed": r.is_multi_source_confirmed,
                "confidence_bonus": round4(r.confidence_bonus),
            })
        })
        .collect();

    let multi_source = results
        .iter()
        .filter(|r| r.is_multi_source_confirmed)
        .count();

    Ok(text_result(&json!({
        "algorithm": "A79",
        "name": "Reporter-Weighted Disproportionality",
        "description": "Weights cases by reporter qualification (Physician=1.0, Pharmacist=0.9, OtherHP=0.8, Consumer=0.6, Lawyer=0.5). Shannon entropy measures reporter diversity; multi-source confirmation increases signal confidence.",
        "total_pairs": results.len(),
        "multi_source_confirmed": multi_source,
        "config": {
            "min_cases": config.min_cases,
            "diversity_threshold": config.diversity_threshold,
        },
        "results": signals,
    })))
}

// =============================================================================
// A81 — Geographic Signal Divergence
// =============================================================================

/// Compute geographic signal divergences (Algorithm A81).
pub fn geographic_divergence(
    params: FaersGeographicDivergenceParams,
) -> Result<CallToolResult, McpError> {
    if params.cases.is_empty() {
        return Err(McpError::invalid_params("cases array is empty", None));
    }

    let cases: Vec<GeographicCase> = params
        .cases
        .iter()
        .map(|c| GeographicCase {
            drug: c.drug.clone(),
            event: c.event.clone(),
            country: c.country.clone(),
        })
        .collect();

    let config = GeographicConfig {
        min_cases: params.min_cases.unwrap_or(5),
        min_countries: params.min_countries.unwrap_or(2),
        divergence_threshold: params.divergence_threshold.unwrap_or(3.0),
        p_value_threshold: params.p_value_threshold.unwrap_or(0.05),
        min_country_cases: params.min_country_cases.unwrap_or(2),
    };

    let results = compute_geographic_divergence(&cases, &config);

    let divergences: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            let countries: Vec<serde_json::Value> = r
                .country_signals
                .iter()
                .map(|cs| {
                    json!({
                        "country": cs.country,
                        "count": cs.count,
                        "reporting_rate": round4(cs.reporting_rate),
                    })
                })
                .collect();

            json!({
                "drug": r.drug,
                "event": r.event,
                "total_cases": r.total_cases,
                "reporting_countries": r.reporting_countries,
                "divergence_ratio": round4(r.divergence_ratio),
                "highest_country": r.highest_country,
                "lowest_country": r.lowest_country,
                "chi_squared": round4(r.chi_squared),
                "heterogeneity_p": round4(r.heterogeneity_p),
                "is_heterogeneous": r.is_heterogeneous,
                "is_divergent": r.is_divergent,
                "country_signals": countries,
            })
        })
        .collect();

    let divergent_count = results.iter().filter(|r| r.is_divergent).count();
    let heterogeneous_count = results.iter().filter(|r| r.is_heterogeneous).count();

    Ok(text_result(&json!({
        "algorithm": "A81",
        "name": "Geographic Signal Divergence",
        "description": "Detects drug-event pairs with significantly different reporting rates across countries. Suggests pharmacogenomic effects, regulatory gaps, or reporting biases. Uses chi-squared heterogeneity test with Wilson-Hilferty p-value approximation.",
        "total_pairs": results.len(),
        "divergent_signals": divergent_count,
        "heterogeneous_signals": heterogeneous_count,
        "config": {
            "min_cases": config.min_cases,
            "min_countries": config.min_countries,
            "divergence_threshold": config.divergence_threshold,
            "p_value_threshold": config.p_value_threshold,
            "min_country_cases": config.min_country_cases,
        },
        "results": divergences,
    })))
}
