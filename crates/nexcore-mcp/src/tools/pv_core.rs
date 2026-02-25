//! PV Core tools — IVF axiom assessment, severity classification
//!
//! Unique modules from nexcore-pv-core not already exposed through pv.rs/vigilance.rs.

use crate::params::{IvfAssessParams, IvfAxiomsParams, PvFdrAdjustParams, SeverityAssessParams};
use nexcore_pv_core::classification::{SeverityCriteria, full_assessment};
use nexcore_pv_core::ivf::{InterventionCharacteristics, IvfAxiom, assess_ivf_axioms};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Assess all 5 IVF axioms for an intervention (ToV §35)
pub fn ivf_assess(params: IvfAssessParams) -> Result<CallToolResult, McpError> {
    let characteristics = InterventionCharacteristics::new()
        .with_potency(params.potency)
        .with_emergence_uncertainty(params.emergence_uncertainty)
        .with_vulnerability_exposure(params.vulnerability_exposure)
        .with_deployment_scale(params.deployment_scale)
        .with_testing_completeness(params.testing_completeness);

    let assessment = assess_ivf_axioms(&characteristics);

    let axiom_results: Vec<_> = assessment
        .axiom_results
        .iter()
        .map(|r| {
            json!({
                "axiom": format!("{}", r.axiom),
                "level": format!("{}", r.level),
                "risk_score": (r.risk_score * 100.0).round() / 100.0,
                "rationale": r.rationale,
                "requires_vigilance": r.level.requires_vigilance(),
            })
        })
        .collect();

    let json = json!({
        "overall_risk": (assessment.overall_risk * 100.0).round() / 100.0,
        "vigilance_required": assessment.vigilance_required,
        "monitoring_intensity": format!("{}", assessment.monitoring_intensity),
        "axiom_results": axiom_results,
        "inputs": {
            "potency": params.potency,
            "emergence_uncertainty": params.emergence_uncertainty,
            "vulnerability_exposure": params.vulnerability_exposure,
            "deployment_scale": params.deployment_scale,
            "testing_completeness": params.testing_completeness,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// List the 5 IVF axioms with ToV mappings and formal statements
pub fn ivf_axioms(_params: IvfAxiomsParams) -> Result<CallToolResult, McpError> {
    let axioms: Vec<_> = IvfAxiom::all()
        .iter()
        .map(|a| {
            json!({
                "number": a.number(),
                "name": format!("{a}"),
                "statement": a.statement(),
                "tov_mapping": a.tov_mapping(),
            })
        })
        .collect();

    let json = json!({
        "axioms": axioms,
        "count": 5,
        "framework": "Intervention Vigilance Framework (ToV §35)",
        "note": "Generalizes pharmacovigilance methodology to all intervention domains",
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Assess adverse event severity using Hartwig-Siegel scale (levels 1-7)
pub fn severity_assess(params: SeverityAssessParams) -> Result<CallToolResult, McpError> {
    let mut criteria = SeverityCriteria::new();

    if params.treatment_changed {
        criteria = criteria.with_treatment_change();
    }
    if params.antidote_required {
        criteria = criteria.with_antidote();
    }
    if params.hospitalization_required {
        criteria = criteria.with_hospitalization();
    }
    if params.icu_required {
        criteria = criteria.with_icu();
    }
    if params.permanent_harm {
        criteria = criteria.with_permanent_harm();
    }
    if params.death {
        criteria = criteria.with_death();
    }

    let result = full_assessment(&criteria);

    let json = json!({
        "level": result.level.level(),
        "level_name": format!("{:?}", result.level),
        "category": format!("{:?}", result.category),
        "is_serious": result.is_serious,
        "priority_weight": result.priority_weight,
        "description": result.level.description(),
        "clinical_action": result.level.clinical_action(),
        "criteria_met": {
            "treatment_changed": params.treatment_changed,
            "antidote_required": params.antidote_required,
            "hospitalization_required": params.hospitalization_required,
            "icu_required": params.icu_required,
            "permanent_harm": params.permanent_harm,
            "death": params.death,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

// ═══════════════════════════════════════════════════════════════════════════════
// SURVIVAL ANALYSIS TOOLS (B.7/B.8 — Directive 002)
// ═══════════════════════════════════════════════════════════════════════════════

use crate::params::{
    CoxObservationParam, PvCoreCoxParams, PvCoreCumulativeIncidenceParams, PvCoreHazardRatioParams,
    PvCoreKaplanMeierParams, PvCoreLogRankParams, SurvivalObservationParam,
};
use nexcore_pv_core::signals::survival::measured;
use nexcore_pv_core::signals::survival::{self, CoxConfig, CoxObservation, SurvivalObservation};

/// Convert param observations to domain type.
fn to_obs(params: &[SurvivalObservationParam]) -> Vec<SurvivalObservation> {
    params
        .iter()
        .map(|p| SurvivalObservation::new(p.time, p.event))
        .collect()
}

/// Kaplan-Meier survival estimation with Measured<T> confidence.
///
/// Observation-level input (time, event/censored). Returns survival curve,
/// Greenwood SE, log-log CI, median survival, and per-point confidence scores.
pub fn survival_kaplan_meier(params: PvCoreKaplanMeierParams) -> Result<CallToolResult, McpError> {
    if params.observations.is_empty() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Error: no observations provided".to_string(),
        )]));
    }

    let obs = to_obs(&params.observations);
    let result = measured::kaplan_meier_measured(&obs);

    let curve: Vec<serde_json::Value> = result
        .raw
        .curve
        .iter()
        .zip(result.measured_curve.iter())
        .map(|(pt, m)| {
            json!({
                "time": pt.time,
                "survival": (pt.survival * 1e6).round() / 1e6,
                "se": (pt.se * 1e6).round() / 1e6,
                "ci_lower": (pt.ci_lower * 1e6).round() / 1e6,
                "ci_upper": (pt.ci_upper * 1e6).round() / 1e6,
                "n_risk": pt.n_risk,
                "n_events": pt.n_events,
                "n_censored": pt.n_censored,
                "confidence": (m.confidence.value() * 1e4).round() / 1e4,
            })
        })
        .collect();

    let output = json!({
        "curve": curve,
        "n_total": result.raw.n_total,
        "n_events": result.raw.n_events,
        "n_censored": result.raw.n_censored,
        "median_survival": result.raw.median_survival,
        "overall_confidence": (result.overall_confidence.value() * 1e4).round() / 1e4,
        "method": "Kaplan-Meier with Greenwood SE and Measured<T> confidence",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_default(),
    )]))
}

/// Log-rank test comparing two survival groups with Measured<T> confidence.
///
/// Returns chi-squared, p-value, Mantel-Haenszel hazard ratio,
/// significance flag, and overall confidence score.
pub fn survival_log_rank(params: PvCoreLogRankParams) -> Result<CallToolResult, McpError> {
    if params.group0.is_empty() || params.group1.is_empty() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Error: both groups must have at least one observation".to_string(),
        )]));
    }

    let g0 = to_obs(&params.group0);
    let g1 = to_obs(&params.group1);
    let result = measured::log_rank_measured(&g0, &g1);

    let output = json!({
        "chi_squared": (result.chi_squared * 1e4).round() / 1e4,
        "p_value": result.p_value,
        "hazard_ratio": (result.hazard_ratio.value * 1e4).round() / 1e4,
        "significant": result.significant,
        "confidence": (result.confidence.value() * 1e4).round() / 1e4,
        "group0_n": params.group0.len(),
        "group1_n": params.group1.len(),
        "method": "Log-rank (Mantel-Cox) with Measured<T> confidence",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_default(),
    )]))
}

/// Cumulative incidence estimation: CI(t) = 1 - S(t), with Measured<T> confidence.
///
/// Complementary view to Kaplan-Meier. Safety scientists often think in
/// cumulative incidence (probability of event by time t).
pub fn survival_cumulative_incidence(
    params: PvCoreCumulativeIncidenceParams,
) -> Result<CallToolResult, McpError> {
    if params.observations.is_empty() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Error: no observations provided".to_string(),
        )]));
    }

    let obs = to_obs(&params.observations);
    let result = survival::cumulative_incidence_measured(&obs);

    let points: Vec<serde_json::Value> = result
        .raw
        .points
        .iter()
        .zip(result.measured_points.iter())
        .map(|(pt, m)| {
            json!({
                "time": pt.time,
                "incidence": (pt.incidence * 1e6).round() / 1e6,
                "se": (pt.se * 1e6).round() / 1e6,
                "ci_lower": (pt.ci_lower * 1e6).round() / 1e6,
                "ci_upper": (pt.ci_upper * 1e6).round() / 1e6,
                "n_risk": pt.n_risk,
                "n_events": pt.n_events,
                "n_censored": pt.n_censored,
                "confidence": (m.confidence.value() * 1e4).round() / 1e4,
            })
        })
        .collect();

    let output = json!({
        "points": points,
        "total_incidence": (result.raw.total_incidence * 1e6).round() / 1e6,
        "event_count": result.raw.event_count,
        "censored_count": result.raw.censored_count,
        "n_total": result.raw.n_total,
        "overall_confidence": (result.overall_confidence.value() * 1e4).round() / 1e4,
        "method": "Cumulative Incidence (1 - KM) with Measured<T> confidence",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_default(),
    )]))
}

/// Cox proportional hazards regression with Measured<T> confidence.
///
/// Fits a Cox PH model via partial likelihood (Newton-Raphson).
/// Returns hazard ratios, confidence intervals, and Measured confidence per coefficient.
pub fn survival_cox(params: PvCoreCoxParams) -> Result<CallToolResult, McpError> {
    if params.observations.is_empty() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Error: no observations provided".to_string(),
        )]));
    }

    // Convert to CoxObservation
    let obs: Vec<CoxObservation> = params
        .observations
        .iter()
        .map(|o| CoxObservation::new(o.time, o.event, o.covariates.clone()))
        .collect();

    let config = CoxConfig {
        max_iterations: params.max_iterations,
        tolerance: params.tolerance,
        ..CoxConfig::default()
    };

    let result = match measured::cox_measured(&obs, &config) {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Cox regression failed: {e}"
            ))]));
        }
    };

    let coefficients: Vec<serde_json::Value> = result
        .raw
        .coefficients
        .iter()
        .zip(result.measured_hazard_ratios.iter())
        .enumerate()
        .map(|(i, (c, m))| {
            json!({
                "index": i,
                "coefficient": (c.coefficient * 1e6).round() / 1e6,
                "se": (c.se * 1e6).round() / 1e6,
                "hazard_ratio": (c.hazard_ratio * 1e4).round() / 1e4,
                "hr_ci_lower": (c.hr_ci_lower * 1e4).round() / 1e4,
                "hr_ci_upper": (c.hr_ci_upper * 1e4).round() / 1e4,
                "z_score": (c.z_statistic * 1e4).round() / 1e4,
                "p_value": c.p_value,
                "confidence": (m.confidence.value() * 1e4).round() / 1e4,
            })
        })
        .collect();

    let output = json!({
        "coefficients": coefficients,
        "n_observations": result.raw.n_observations,
        "n_events": result.raw.n_events,
        "converged": result.raw.converged,
        "iterations": result.raw.iterations,
        "log_likelihood": result.raw.log_likelihood,
        "concordance": (result.raw.concordance * 1e4).round() / 1e4,
        "overall_confidence": (result.overall_confidence.value() * 1e4).round() / 1e4,
        "method": "Cox PH (Newton-Raphson) with Measured<T> confidence",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_default(),
    )]))
}

/// Quick two-group hazard ratio with Measured<T> confidence.
///
/// Simplified interface: provide treatment/control times and event flags.
/// Wraps a single-covariate Cox model internally.
pub fn survival_hazard_ratio(params: PvCoreHazardRatioParams) -> Result<CallToolResult, McpError> {
    if params.treatment_times.len() != params.treatment_events.len() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Error: treatment_times and treatment_events must have equal length".to_string(),
        )]));
    }
    if params.control_times.len() != params.control_events.len() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Error: control_times and control_events must have equal length".to_string(),
        )]));
    }

    let result = match measured::hazard_ratio_measured(
        &params.treatment_times,
        &params.treatment_events,
        &params.control_times,
        &params.control_events,
    ) {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Hazard ratio estimation failed: {e}"
            ))]));
        }
    };

    let output = json!({
        "hazard_ratio": (result.hazard_ratio.value * 1e4).round() / 1e4,
        "confidence": (result.hazard_ratio.confidence.value() * 1e4).round() / 1e4,
        "coefficient": (result.raw.coefficient * 1e6).round() / 1e6,
        "se": (result.raw.se * 1e6).round() / 1e6,
        "hr_ci_lower": (result.raw.hr_ci_lower * 1e4).round() / 1e4,
        "hr_ci_upper": (result.raw.hr_ci_upper * 1e4).round() / 1e4,
        "z_score": (result.raw.z_statistic * 1e4).round() / 1e4,
        "p_value": result.raw.p_value,
        "treatment_n": params.treatment_times.len(),
        "control_n": params.control_times.len(),
        "method": "Quick HR (single-covariate Cox) with Measured<T> confidence",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_default(),
    )]))
}

// =============================================================================
// FDR / MULTIPLE TESTING CORRECTION
// =============================================================================

/// Adjust p-values for multiple comparisons.
///
/// Supports BH (FDR control), Bonferroni, Holm, and Šidák (FWER control).
/// Standalone tool — works with p-values from any statistical analysis.
pub fn fdr_adjust(params: PvFdrAdjustParams) -> Result<CallToolResult, McpError> {
    use nexcore_pv_core::signals::adjustment::{
        bh_adjust, bonferroni_adjust, holm_adjust, sidak_adjust,
    };

    if params.p_values.is_empty() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Error: p_values must be non-empty".to_string(),
        )]));
    }

    // Validate p-values are in [0, 1]
    for (i, &p) in params.p_values.iter().enumerate() {
        if !(0.0..=1.0).contains(&p) {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: p_values[{i}] = {p} is outside [0, 1]"
            ))]));
        }
    }

    let method = params.method.to_lowercase();
    let fdr_level = params.fdr_level;

    let (adjusted, rejected, n_rejected, method_name) = match method.as_str() {
        "bh" | "benjamini-hochberg" | "benjamini_hochberg" | "fdr" => {
            let r = bh_adjust(&params.p_values, fdr_level);
            (
                r.q_values,
                r.rejected,
                r.n_rejected,
                "Benjamini-Hochberg (FDR)",
            )
        }
        "bonferroni" => {
            let r = bonferroni_adjust(&params.p_values, fdr_level);
            (
                r.adjusted_p_values,
                r.rejected,
                r.n_rejected,
                "Bonferroni (FWER)",
            )
        }
        "holm" | "holm-bonferroni" => {
            let r = holm_adjust(&params.p_values, fdr_level);
            (
                r.adjusted_p_values,
                r.rejected,
                r.n_rejected,
                "Holm-Bonferroni (FWER)",
            )
        }
        "sidak" | "šidák" => {
            let r = sidak_adjust(&params.p_values, fdr_level);
            (
                r.adjusted_p_values,
                r.rejected,
                r.n_rejected,
                "Šidák (FWER)",
            )
        }
        _ => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: unknown method '{}'. Use: bh, bonferroni, holm, sidak",
                params.method
            ))]));
        }
    };

    let output = json!({
        "adjusted_p_values": adjusted.iter().map(|v| (v * 1e6).round() / 1e6).collect::<Vec<_>>(),
        "rejected": rejected,
        "num_rejected": n_rejected,
        "num_tested": params.p_values.len(),
        "method": method_name,
        "fdr_level": fdr_level,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_default(),
    )]))
}

// =============================================================================
// BAYESIAN UPDATE (Conjugate Models)
// =============================================================================

use crate::params::{
    PvCoreBetaBinomialParams, PvCoreGammaPoissonParams, PvCoreSequentialBetaBinomialParams,
};
use nexcore_pv_core::signals::bayesian::update::{
    BayesianUpdate, BetaParams, BinomialEvidence, ConjugateBetaBinomial, GammaParams,
    GammaPoissonMixture, PoissonEvidence,
};

/// Beta-Binomial conjugate Bayesian update with Measured<f64> confidence.
///
/// Prior: Beta(alpha, beta). Evidence: binomial successes/failures.
/// Posterior: Beta(alpha + successes, beta + failures).
/// Returns posterior mean, variance, 95% credible interval, and confidence score.
pub fn bayesian_beta_binomial(
    params: PvCoreBetaBinomialParams,
) -> Result<CallToolResult, McpError> {
    if params.prior_alpha <= 0.0 || params.prior_beta <= 0.0 {
        return Ok(CallToolResult::error(vec![Content::text(
            "Error: prior_alpha and prior_beta must be positive".to_string(),
        )]));
    }

    let prior = BetaParams::new(params.prior_alpha, params.prior_beta);
    let evidence = BinomialEvidence {
        successes: params.successes,
        failures: params.failures,
    };
    let posterior = ConjugateBetaBinomial::update(&prior, &evidence);
    let summary = ConjugateBetaBinomial::summarize(&posterior);

    let n = params.successes + params.failures;
    let output = json!({
        "prior": {
            "alpha": prior.alpha,
            "beta": prior.beta,
            "mean": prior.mean(),
        },
        "evidence": {
            "successes": params.successes,
            "failures": params.failures,
            "n": n,
        },
        "posterior": {
            "alpha": posterior.alpha,
            "beta": posterior.beta,
            "mean": (posterior.mean() * 1e6).round() / 1e6,
            "variance": (posterior.variance() * 1e8).round() / 1e8,
        },
        "measured": {
            "value": (summary.value * 1e6).round() / 1e6,
            "confidence": (summary.confidence.value() * 1e4).round() / 1e4,
        },
        "model": "Beta-Binomial conjugate",
        "grounding": "→(Causality) + ρ(Recursion) + ∂(Boundary)",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_default(),
    )]))
}

/// Gamma-Poisson conjugate Bayesian update with Measured<f64> confidence.
///
/// Prior: Gamma(shape, rate). Evidence: Poisson count over exposure.
/// Posterior: Gamma(shape + count, rate + exposure).
/// Returns posterior mean rate, variance, and confidence score.
pub fn bayesian_gamma_poisson(
    params: PvCoreGammaPoissonParams,
) -> Result<CallToolResult, McpError> {
    if params.prior_shape <= 0.0 || params.prior_rate <= 0.0 {
        return Ok(CallToolResult::error(vec![Content::text(
            "Error: prior_shape and prior_rate must be positive".to_string(),
        )]));
    }
    if params.exposure <= 0.0 {
        return Ok(CallToolResult::error(vec![Content::text(
            "Error: exposure must be positive".to_string(),
        )]));
    }

    let prior = GammaParams::new(params.prior_shape, params.prior_rate);
    let evidence = PoissonEvidence {
        count: params.count,
        exposure: params.exposure,
    };
    let posterior = GammaPoissonMixture::update(&prior, &evidence);
    let summary = GammaPoissonMixture::summarize(&posterior);

    let output = json!({
        "prior": {
            "shape": prior.shape,
            "rate": prior.rate,
            "mean": prior.mean(),
        },
        "evidence": {
            "count": params.count,
            "exposure": params.exposure,
            "observed_rate": params.count as f64 / params.exposure,
        },
        "posterior": {
            "shape": posterior.shape,
            "rate": posterior.rate,
            "mean": (posterior.mean() * 1e6).round() / 1e6,
            "variance": (posterior.variance() * 1e8).round() / 1e8,
        },
        "measured": {
            "value": (summary.value * 1e6).round() / 1e6,
            "confidence": (summary.confidence.value() * 1e4).round() / 1e4,
        },
        "model": "Gamma-Poisson conjugate",
        "grounding": "→(Causality) + ρ(Recursion) + ∂(Boundary)",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_default(),
    )]))
}

/// Sequential Beta-Binomial update: process multiple evidence batches.
///
/// Demonstrates that sequential Bayesian updates are equivalent to a single
/// update with pooled evidence (order-invariance for conjugate models).
/// Returns intermediate posteriors at each step plus the final result.
pub fn bayesian_sequential_beta_binomial(
    params: PvCoreSequentialBetaBinomialParams,
) -> Result<CallToolResult, McpError> {
    if params.prior_alpha <= 0.0 || params.prior_beta <= 0.0 {
        return Ok(CallToolResult::error(vec![Content::text(
            "Error: prior_alpha and prior_beta must be positive".to_string(),
        )]));
    }
    if params.evidence_sequence.is_empty() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Error: evidence_sequence must be non-empty".to_string(),
        )]));
    }

    let prior = BetaParams::new(params.prior_alpha, params.prior_beta);
    let evidence: Vec<BinomialEvidence> = params
        .evidence_sequence
        .iter()
        .map(|e| BinomialEvidence {
            successes: e[0],
            failures: e[1],
        })
        .collect();

    // Sequential update — track intermediate posteriors
    let mut current: BetaParams = prior.clone();
    let mut steps: Vec<serde_json::Value> = Vec::new();
    for (i, ev) in evidence.iter().enumerate() {
        current = ConjugateBetaBinomial::update(&current, ev);
        let summary = ConjugateBetaBinomial::summarize(&current);
        steps.push(json!({
            "step": i + 1,
            "evidence": { "successes": ev.successes, "failures": ev.failures },
            "posterior_alpha": current.alpha,
            "posterior_beta": current.beta,
            "posterior_mean": (current.mean() * 1e6).round() / 1e6,
            "confidence": (summary.confidence.value() * 1e4).round() / 1e4,
        }));
    }

    let final_summary = ConjugateBetaBinomial::summarize(&current);

    let output = json!({
        "prior": {
            "alpha": params.prior_alpha,
            "beta": params.prior_beta,
        },
        "steps": steps,
        "final_posterior": {
            "alpha": current.alpha,
            "beta": current.beta,
            "mean": (current.mean() * 1e6).round() / 1e6,
            "variance": (current.variance() * 1e8).round() / 1e8,
        },
        "final_measured": {
            "value": (final_summary.value * 1e6).round() / 1e6,
            "confidence": (final_summary.confidence.value() * 1e4).round() / 1e4,
        },
        "n_steps": params.evidence_sequence.len(),
        "model": "Sequential Beta-Binomial conjugate",
        "property": "Order-invariant: final posterior depends only on total evidence, not sequence order",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_default(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{
        CoxObservationParam, IvfAssessParams, IvfAxiomsParams, PvCoreBetaBinomialParams,
        PvCoreCoxParams, PvCoreCumulativeIncidenceParams, PvCoreGammaPoissonParams,
        PvCoreHazardRatioParams, PvCoreKaplanMeierParams, PvCoreLogRankParams,
        PvCoreSequentialBetaBinomialParams, PvFdrAdjustParams, SeverityAssessParams,
        SurvivalObservationParam,
    };

    fn extract_json(result: &CallToolResult) -> serde_json::Value {
        let content = &result.content[0];
        let text = content.as_text().expect("expected text content");
        serde_json::from_str(&text.text).expect("valid JSON")
    }

    #[test]
    fn ivf_assess_high_risk() {
        let r = ivf_assess(IvfAssessParams {
            potency: 0.95,
            emergence_uncertainty: 0.9,
            vulnerability_exposure: 0.9,
            deployment_scale: 0.9,
            testing_completeness: 0.1,
        })
        .expect("ok");
        let j = extract_json(&r);
        let risk = j["overall_risk"].as_f64().expect("f64");
        assert!(risk > 0.5, "high-risk → high overall_risk: {risk}");
        assert!(j["vigilance_required"].as_bool().expect("bool"));
    }

    #[test]
    fn ivf_assess_low_risk() {
        let r = ivf_assess(IvfAssessParams {
            potency: 0.1,
            emergence_uncertainty: 0.1,
            vulnerability_exposure: 0.1,
            deployment_scale: 0.1,
            testing_completeness: 0.9,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["overall_risk"].as_f64().expect("f64") < 0.5);
    }

    #[test]
    fn ivf_assess_all_zero() {
        let r = ivf_assess(IvfAssessParams {
            potency: 0.0,
            emergence_uncertainty: 0.0,
            vulnerability_exposure: 0.0,
            deployment_scale: 0.0,
            testing_completeness: 0.0,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["overall_risk"].as_f64().is_some());
        assert!(j["axiom_results"].as_array().is_some());
    }

    #[test]
    fn ivf_assess_all_one() {
        let r = ivf_assess(IvfAssessParams {
            potency: 1.0,
            emergence_uncertainty: 1.0,
            vulnerability_exposure: 1.0,
            deployment_scale: 1.0,
            testing_completeness: 1.0,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["axiom_results"].as_array().expect("arr").len(), 5);
    }

    #[test]
    fn ivf_assess_returns_five_axioms() {
        let r = ivf_assess(IvfAssessParams {
            potency: 0.5,
            emergence_uncertainty: 0.5,
            vulnerability_exposure: 0.5,
            deployment_scale: 0.5,
            testing_completeness: 0.5,
        })
        .expect("ok");
        let j = extract_json(&r);
        let axioms = j["axiom_results"].as_array().expect("arr");
        assert_eq!(axioms.len(), 5);
        for a in axioms {
            assert!(a["axiom"].as_str().is_some());
            assert!(a["level"].as_str().is_some());
            assert!(a["risk_score"].as_f64().is_some());
        }
    }

    #[test]
    fn ivf_axioms_lists_five() {
        let r = ivf_axioms(IvfAxiomsParams {}).expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["count"], 5);
        let axioms = j["axioms"].as_array().expect("arr");
        assert_eq!(axioms.len(), 5);
        for a in axioms {
            assert!(a["number"].as_u64().is_some());
            assert!(a["name"].as_str().is_some());
            assert!(a["statement"].as_str().is_some());
            assert!(a["tov_mapping"].as_str().is_some());
        }
    }

    #[test]
    fn ivf_axioms_framework() {
        let r = ivf_axioms(IvfAxiomsParams {}).expect("ok");
        let j = extract_json(&r);
        assert!(
            j["framework"]
                .as_str()
                .expect("str")
                .contains("Intervention Vigilance")
        );
    }

    #[test]
    fn severity_all_false_is_mild() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: false,
            antidote_required: false,
            hospitalization_required: false,
            icu_required: false,
            permanent_harm: false,
            death: false,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["level"].as_u64().expect("u64"), 1);
        assert!(!j["is_serious"].as_bool().expect("bool"));
    }

    #[test]
    fn severity_death_is_lethal() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: false,
            antidote_required: false,
            hospitalization_required: false,
            icu_required: false,
            permanent_harm: false,
            death: true,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["level"].as_u64().expect("u64"), 7);
        assert!(j["is_serious"].as_bool().expect("bool"));
    }

    #[test]
    fn severity_hospitalization() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: false,
            antidote_required: false,
            hospitalization_required: true,
            icu_required: false,
            permanent_harm: false,
            death: false,
        })
        .expect("ok");
        let j = extract_json(&r);
        let lvl = j["level"].as_u64().expect("u64");
        assert!(lvl >= 3 && lvl <= 4, "hospitalization → 3-4: {lvl}");
        assert!(j["is_serious"].as_bool().expect("bool"));
    }

    #[test]
    fn severity_icu() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: false,
            antidote_required: false,
            hospitalization_required: false,
            icu_required: true,
            permanent_harm: false,
            death: false,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["level"].as_u64().expect("u64") >= 5);
    }

    #[test]
    fn severity_permanent_harm() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: false,
            antidote_required: false,
            hospitalization_required: false,
            icu_required: false,
            permanent_harm: true,
            death: false,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["level"].as_u64().expect("u64") >= 6);
    }

    #[test]
    fn severity_all_true_death_wins() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: true,
            antidote_required: true,
            hospitalization_required: true,
            icu_required: true,
            permanent_harm: true,
            death: true,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["level"].as_u64().expect("u64"), 7);
    }

    #[test]
    fn severity_treatment_change_only() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: true,
            antidote_required: false,
            hospitalization_required: false,
            icu_required: false,
            permanent_harm: false,
            death: false,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["level"].as_u64().expect("u64"), 2);
    }

    // ── Survival Analysis Tests ──────────────────────────────────────

    #[test]
    fn survival_km_basic() {
        let r = survival_kaplan_meier(PvCoreKaplanMeierParams {
            observations: vec![
                SurvivalObservationParam {
                    time: 1.0,
                    event: true,
                },
                SurvivalObservationParam {
                    time: 2.0,
                    event: true,
                },
                SurvivalObservationParam {
                    time: 3.0,
                    event: false,
                },
                SurvivalObservationParam {
                    time: 4.0,
                    event: true,
                },
                SurvivalObservationParam {
                    time: 5.0,
                    event: false,
                },
            ],
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["curve"].as_array().expect("arr").len() > 0);
        assert_eq!(j["n_total"].as_u64().expect("u64"), 5);
        assert!(j["overall_confidence"].as_f64().expect("f64") >= 0.05);
    }

    #[test]
    fn survival_km_empty_errors() {
        let r = survival_kaplan_meier(PvCoreKaplanMeierParams {
            observations: vec![],
        })
        .expect("ok");
        let text = r.content[0].as_text().expect("text");
        assert!(text.text.contains("Error"));
    }

    #[test]
    fn survival_log_rank_different_groups() {
        let g0 = vec![
            SurvivalObservationParam {
                time: 5.0,
                event: true,
            },
            SurvivalObservationParam {
                time: 6.0,
                event: true,
            },
            SurvivalObservationParam {
                time: 8.0,
                event: true,
            },
            SurvivalObservationParam {
                time: 10.0,
                event: true,
            },
        ];
        let g1 = vec![
            SurvivalObservationParam {
                time: 1.0,
                event: true,
            },
            SurvivalObservationParam {
                time: 2.0,
                event: true,
            },
            SurvivalObservationParam {
                time: 3.0,
                event: true,
            },
            SurvivalObservationParam {
                time: 4.0,
                event: true,
            },
        ];
        let r = survival_log_rank(PvCoreLogRankParams {
            group0: g0,
            group1: g1,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["chi_squared"].as_f64().expect("f64") > 0.0);
        assert!(j["p_value"].as_f64().is_some());
        assert!(j["hazard_ratio"].as_f64().expect("f64") > 0.0);
    }

    #[test]
    fn survival_cumulative_incidence_basic() {
        let r = survival_cumulative_incidence(PvCoreCumulativeIncidenceParams {
            observations: vec![
                SurvivalObservationParam {
                    time: 1.0,
                    event: true,
                },
                SurvivalObservationParam {
                    time: 2.0,
                    event: true,
                },
                SurvivalObservationParam {
                    time: 3.0,
                    event: false,
                },
                SurvivalObservationParam {
                    time: 4.0,
                    event: true,
                },
            ],
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["points"].as_array().expect("arr").len() > 0);
        let incidence = j["total_incidence"].as_f64().expect("f64");
        assert!(incidence > 0.0 && incidence <= 1.0);
    }

    #[test]
    fn survival_cox_basic() {
        let r = survival_cox(PvCoreCoxParams {
            observations: vec![
                CoxObservationParam {
                    time: 1.0,
                    event: true,
                    covariates: vec![1.0],
                },
                CoxObservationParam {
                    time: 2.0,
                    event: true,
                    covariates: vec![0.0],
                },
                CoxObservationParam {
                    time: 3.0,
                    event: false,
                    covariates: vec![1.0],
                },
                CoxObservationParam {
                    time: 4.0,
                    event: true,
                    covariates: vec![0.0],
                },
                CoxObservationParam {
                    time: 5.0,
                    event: true,
                    covariates: vec![1.0],
                },
                CoxObservationParam {
                    time: 6.0,
                    event: false,
                    covariates: vec![0.0],
                },
                CoxObservationParam {
                    time: 7.0,
                    event: true,
                    covariates: vec![1.0],
                },
                CoxObservationParam {
                    time: 8.0,
                    event: true,
                    covariates: vec![0.0],
                },
            ],
            max_iterations: 25,
            tolerance: 1e-6,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["n_observations"].as_u64().expect("u64"), 8);
        assert!(j["coefficients"].as_array().expect("arr").len() > 0);
        assert!(j["overall_confidence"].as_f64().expect("f64") >= 0.05);
    }

    #[test]
    fn survival_hazard_ratio_basic() {
        let r = survival_hazard_ratio(PvCoreHazardRatioParams {
            treatment_times: vec![1.0, 3.0, 5.0, 7.0],
            treatment_events: vec![true, false, true, true],
            control_times: vec![2.0, 4.0, 6.0, 8.0],
            control_events: vec![true, true, false, true],
        })
        .expect("ok");
        let j = extract_json(&r);
        // HR can be very small if Cox diverges on small datasets — validate it's finite
        assert!(j["hazard_ratio"].as_f64().expect("f64").is_finite());
        assert!(j["confidence"].as_f64().expect("f64") >= 0.05);
    }

    #[test]
    fn survival_hazard_ratio_length_mismatch() {
        let r = survival_hazard_ratio(PvCoreHazardRatioParams {
            treatment_times: vec![1.0, 3.0],
            treatment_events: vec![true], // mismatch
            control_times: vec![2.0],
            control_events: vec![true],
        })
        .expect("ok");
        let text = r.content[0].as_text().expect("text");
        assert!(text.text.contains("Error"));
    }

    #[test]
    fn severity_output_fields_complete() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: true,
            antidote_required: false,
            hospitalization_required: true,
            icu_required: false,
            permanent_harm: false,
            death: false,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["level"].as_u64().is_some());
        assert!(j["level_name"].as_str().is_some());
        assert!(j["category"].as_str().is_some());
        assert!(j["is_serious"].is_boolean());
        assert!(j["priority_weight"].as_f64().is_some());
        assert!(j["description"].as_str().is_some());
        assert!(j["clinical_action"].as_str().is_some());
        assert!(j["criteria_met"].is_object());
    }

    // =========================================================================
    // FDR TOOL TESTS
    // =========================================================================

    #[test]
    fn fdr_adjust_bh_basic() {
        let r = fdr_adjust(PvFdrAdjustParams {
            p_values: vec![0.001, 0.008, 0.039, 0.041, 0.05],
            method: "bh".to_string(),
            fdr_level: 0.05,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["num_tested"], 5);
        assert!(j["num_rejected"].as_u64().is_some());
        let adjusted = j["adjusted_p_values"].as_array().expect("arr");
        assert_eq!(adjusted.len(), 5);
        // Adjusted p-values >= raw p-values
        let raw = [0.001, 0.008, 0.039, 0.041, 0.05];
        for (i, adj) in adjusted.iter().enumerate() {
            assert!(adj.as_f64().expect("f64") >= raw[i] - 1e-10);
        }
    }

    #[test]
    fn fdr_adjust_bonferroni() {
        let r = fdr_adjust(PvFdrAdjustParams {
            p_values: vec![0.01, 0.02, 0.03],
            method: "bonferroni".to_string(),
            fdr_level: 0.05,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["method"], "Bonferroni (FWER)");
        let adjusted = j["adjusted_p_values"].as_array().expect("arr");
        // Bonferroni: p_adj = p * m (capped at 1.0)
        // 0.01 * 3 = 0.03, 0.02 * 3 = 0.06, 0.03 * 3 = 0.09
        assert!((adjusted[0].as_f64().expect("f64") - 0.03).abs() < 1e-4);
    }

    #[test]
    fn fdr_adjust_empty_errors() {
        let r = fdr_adjust(PvFdrAdjustParams {
            p_values: vec![],
            method: "bh".to_string(),
            fdr_level: 0.05,
        })
        .expect("ok");
        let text = r.content[0].as_text().expect("text");
        assert!(text.text.contains("Error"));
    }

    #[test]
    fn fdr_adjust_invalid_method_errors() {
        let r = fdr_adjust(PvFdrAdjustParams {
            p_values: vec![0.01],
            method: "invalid".to_string(),
            fdr_level: 0.05,
        })
        .expect("ok");
        let text = r.content[0].as_text().expect("text");
        assert!(text.text.contains("Error"));
    }

    // =================================================================
    // BAYESIAN UPDATE TOOLS
    // =================================================================

    #[test]
    fn beta_binomial_jeffreys_prior() {
        let r = bayesian_beta_binomial(PvCoreBetaBinomialParams {
            prior_alpha: 0.5,
            prior_beta: 0.5,
            successes: 10,
            failures: 90,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["posterior"]["alpha"], 10.5);
        assert_eq!(j["posterior"]["beta"], 90.5);
        assert!(j["measured"]["confidence"].as_f64().expect("f64") > 0.0);
    }

    #[test]
    fn beta_binomial_invalid_prior_errors() {
        let r = bayesian_beta_binomial(PvCoreBetaBinomialParams {
            prior_alpha: 0.0,
            prior_beta: 0.5,
            successes: 5,
            failures: 5,
        })
        .expect("ok");
        let text = r.content[0].as_text().expect("text");
        assert!(text.text.contains("Error"));
    }

    #[test]
    fn gamma_poisson_basic() {
        let r = bayesian_gamma_poisson(PvCoreGammaPoissonParams {
            prior_shape: 0.5,
            prior_rate: 0.5,
            count: 20,
            exposure: 100.0,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["posterior"]["shape"], 20.5);
        assert_eq!(j["posterior"]["rate"], 100.5);
        assert!(j["measured"]["confidence"].as_f64().expect("f64") > 0.0);
    }

    #[test]
    fn gamma_poisson_invalid_exposure_errors() {
        let r = bayesian_gamma_poisson(PvCoreGammaPoissonParams {
            prior_shape: 0.5,
            prior_rate: 0.5,
            count: 5,
            exposure: 0.0,
        })
        .expect("ok");
        let text = r.content[0].as_text().expect("text");
        assert!(text.text.contains("Error"));
    }

    #[test]
    fn sequential_beta_binomial_multi_step() {
        let r = bayesian_sequential_beta_binomial(PvCoreSequentialBetaBinomialParams {
            prior_alpha: 0.5,
            prior_beta: 0.5,
            evidence_sequence: vec![[5, 15], [3, 7], [2, 8]],
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["n_steps"], 3);
        let steps = j["steps"].as_array().expect("arr");
        assert_eq!(steps.len(), 3);
        // Final posterior = Jeffreys + (5+3+2, 15+7+8) = Beta(10.5, 30.5)
        assert_eq!(j["final_posterior"]["alpha"], 10.5);
        assert_eq!(j["final_posterior"]["beta"], 30.5);
    }

    #[test]
    fn sequential_beta_binomial_empty_errors() {
        let r = bayesian_sequential_beta_binomial(PvCoreSequentialBetaBinomialParams {
            prior_alpha: 0.5,
            prior_beta: 0.5,
            evidence_sequence: vec![],
        })
        .expect("ok");
        let text = r.content[0].as_text().expect("text");
        assert!(text.text.contains("Error"));
    }
}
