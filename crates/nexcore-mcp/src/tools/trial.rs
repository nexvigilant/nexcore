//! TRIAL Framework MCP Tools
//!
//! 10 tools implementing the universal experimentation framework.
//! Derived from FDA clinical trial methodology (ICH E9 R1, E20, E3).

use crate::params::trial::{
    TrialAdaptDecideParams, TrialBlindVerifyParams, TrialEndpointEvaluateParams,
    TrialInterimAnalyzeParams, TrialMultiplicityAdjustParams, TrialPowerAnalysisParams,
    TrialProtocolRegisterParams, TrialRandomizeParams, TrialReportGenerateParams,
    TrialSafetyCheckParams,
};
use nexcore_trial::{
    adaptation::evaluate_adaptation,
    block_randomize, bonferroni_adjust, check_safety_boundary, evaluate_interim,
    evaluate_two_means, evaluate_two_proportions, generate_report, hochberg_adjust, holm_adjust,
    benjamini_hochberg_adjust, lan_demets_alpha_spent, obrien_fleming_boundary,
    posterior_probability_superiority, power::{sample_size_survival, sample_size_two_mean, sample_size_two_proportion},
    randomize::stratified_randomize, register_protocol, safety_event_rate,
    simple_randomize, verify_blinding,
    types::{
        Adaptation, Arm, BlindingLevel, Endpoint, EndpointDirection, EndpointResult, InterimData,
        Protocol, ProtocolRequest, SafetyRule, SpendingFunction, Stratum,
    },
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

// ── 1. trial_protocol_register ────────────────────────────────────────────────

/// Register a new trial protocol. Validates all fields and generates a UUID trial ID.
pub fn protocol_register(params: TrialProtocolRegisterParams) -> Result<CallToolResult, McpError> {
    let primary_endpoint: Endpoint =
        serde_json::from_str(&params.primary_endpoint_json)
            .map_err(|e| McpError::invalid_params(format!("primary_endpoint_json: {e}"), None))?;

    let arms: Vec<Arm> = serde_json::from_str(&params.arms_json)
        .map_err(|e| McpError::invalid_params(format!("arms_json: {e}"), None))?;

    let safety_boundary: SafetyRule = serde_json::from_str(&params.safety_boundary_json)
        .map_err(|e| McpError::invalid_params(format!("safety_boundary_json: {e}"), None))?;

    let adaptation_rules: Vec<Adaptation> = params
        .adaptation_rules_json
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(|e| McpError::invalid_params(format!("adaptation_rules_json: {e}"), None))?
        .unwrap_or_default();

    let blinding = parse_blinding(&params.blinding)?;

    let req = ProtocolRequest {
        hypothesis: params.hypothesis,
        population: params.population,
        primary_endpoint,
        secondary_endpoints: vec![],
        arms,
        sample_size: params.sample_size,
        power: params.power,
        alpha: params.alpha,
        duration_days: params.duration_days,
        safety_boundary,
        adaptation_rules,
        blinding,
    };

    match register_protocol(req) {
        Ok(protocol) => {
            let json = serde_json::to_string_pretty(&protocol)
                .unwrap_or_else(|_| format!("{{\"id\":\"{}\"}}", protocol.id));
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Protocol registered successfully.\n\nProtocol ID: {}\nCreated: {}\n\n```json\n{json}\n```",
                protocol.id, protocol.created_at
            ))]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Protocol registration failed: {e}"
        ))])),
    }
}

// ── 2. trial_power_analysis ───────────────────────────────────────────────────

/// Compute per-arm sample size for the specified test type.
pub fn power_analysis(params: TrialPowerAnalysisParams) -> Result<CallToolResult, McpError> {
    let result = match params.test_type.as_str() {
        "two_proportion" => {
            let p1 = params.p1.ok_or_else(|| McpError::invalid_params("p1 required for two_proportion", None))?;
            let p2 = params.p2.ok_or_else(|| McpError::invalid_params("p2 required for two_proportion", None))?;
            sample_size_two_proportion(p1, p2, params.alpha, params.power)
        }
        "two_mean" => {
            let d = params.effect_size.ok_or_else(|| McpError::invalid_params("effect_size required for two_mean", None))?;
            sample_size_two_mean(d, params.alpha, params.power)
        }
        "survival" => {
            let hr = params.hazard_ratio.ok_or_else(|| McpError::invalid_params("hazard_ratio required for survival", None))?;
            let ep = params.event_prob.ok_or_else(|| McpError::invalid_params("event_prob required for survival", None))?;
            sample_size_survival(hr, params.alpha, params.power, ep)
        }
        other => return Ok(CallToolResult::error(vec![Content::text(format!(
            "Unknown test_type '{other}'. Use: two_proportion | two_mean | survival"
        ))])),
    };

    match result {
        Ok(n) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Power Analysis ({}):\n  Per-arm sample size: **{n}**\n  α={:.3}, power={:.0}%\n  Total (2 arms): {}",
            params.test_type, params.alpha, params.power * 100.0, n * 2
        ))])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!("Power analysis failed: {e}"))])),
    }
}

// ── 3. trial_randomize ────────────────────────────────────────────────────────

/// Randomize subjects to arms using simple, block, or stratified schemes.
pub fn randomize(params: TrialRandomizeParams) -> Result<CallToolResult, McpError> {
    let block_size = params.block_size.unwrap_or(params.arms * 2);

    let result = match params.method.as_str() {
        "simple" => simple_randomize(params.n, params.arms, params.seed),
        "block" => block_randomize(params.n, params.arms, block_size, params.seed),
        "stratified" => {
            let strata: Vec<Stratum> = params
                .strata_json
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(|e| McpError::invalid_params(format!("strata_json: {e}"), None))?
                .unwrap_or_default();
            if strata.is_empty() {
                return Ok(CallToolResult::error(vec![Content::text(
                    "strata_json required for stratified randomization".into(),
                )]));
            }
            stratified_randomize(&strata, params.arms, block_size, params.seed)
        }
        other => return Ok(CallToolResult::error(vec![Content::text(format!(
            "Unknown method '{other}'. Use: simple | block | stratified"
        ))])),
    };

    match result {
        Ok(assignments) => {
            let arm_counts: Vec<usize> = (0..params.arms)
                .map(|i| assignments.iter().filter(|a| a.arm_index == i).count())
                .collect();
            let counts_str = arm_counts
                .iter()
                .enumerate()
                .map(|(i, c)| format!("arm_{i}: {c}"))
                .collect::<Vec<_>>()
                .join(", ");
            let json = serde_json::to_string(&assignments)
                .unwrap_or_else(|_| "[]".into());
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Randomization complete ({}, n={}):\n  Distribution: {counts_str}\n\nAssignments JSON:\n```json\n{json}\n```",
                params.method, assignments.len()
            ))]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!("Randomization failed: {e}"))])),
    }
}

// ── 4. trial_blind_verify ─────────────────────────────────────────────────────

/// Verify blinding integrity of a randomization assignment list.
pub fn blind_verify(params: TrialBlindVerifyParams) -> Result<CallToolResult, McpError> {
    let assignments: Vec<nexcore_trial::types::ArmAssignment> =
        serde_json::from_str(&params.assignments_json)
            .map_err(|e| McpError::invalid_params(format!("assignments_json: {e}"), None))?;

    let protocol: Protocol = serde_json::from_str(&params.protocol_json)
        .map_err(|e| McpError::invalid_params(format!("protocol_json: {e}"), None))?;

    match verify_blinding(&assignments, &protocol) {
        Ok(report) => {
            let status = if report.violations.is_empty() {
                "PASS"
            } else {
                "VIOLATIONS DETECTED"
            };
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Blinding Verification: **{status}**\n  Level: {:?}\n  Integrity score: {:.2}\n  Violations: {}",
                report.level,
                report.integrity_score,
                if report.violations.is_empty() {
                    "None".into()
                } else {
                    report.violations.join("; ")
                }
            ))]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!("Blinding verification failed: {e}"))])),
    }
}

// ── 5. trial_interim_analyze ──────────────────────────────────────────────────

/// Run interim analysis using OBF or Pocock boundaries.
pub fn interim_analyze(params: TrialInterimAnalyzeParams) -> Result<CallToolResult, McpError> {
    let protocol: Protocol = serde_json::from_str(&params.protocol_json)
        .map_err(|e| McpError::invalid_params(format!("protocol_json: {e}"), None))?;

    let data = InterimData {
        information_fraction: params.information_fraction,
        treatment_successes: params.treatment_successes,
        treatment_n: params.treatment_n,
        control_successes: params.control_successes,
        control_n: params.control_n,
        safety_events: params.safety_events,
    };

    // Also compute spending
    let spending_fn = if params.method == "pocock" {
        SpendingFunction::Pocock
    } else {
        SpendingFunction::OBrienFleming
    };
    let spent = lan_demets_alpha_spent(params.information_fraction, protocol.alpha, spending_fn);
    let boundary = obrien_fleming_boundary(params.information_fraction, protocol.alpha);
    let posterior = posterior_probability_superiority(
        params.treatment_successes,
        params.treatment_n,
        params.control_successes,
        params.control_n,
    );

    match evaluate_interim(&data, &protocol) {
        Ok(result) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Interim Analysis (t={:.2}, method={}):\n\
             Decision: **{:?}**\n\
             OBF boundary: {:.4}\n\
             Test statistic: {:.4}\n\
             Alpha spent: {:.4}\n\
             Posterior P(superiority): {:.3}\n\
             Rationale: {}",
            params.information_fraction,
            params.method,
            result.decision,
            boundary,
            result.test_statistic,
            spent,
            posterior,
            result.rationale
        ))])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!("Interim analysis failed: {e}"))])),
    }
}

// ── 6. trial_safety_check ─────────────────────────────────────────────────────

/// Check an observed safety metric against its pre-specified threshold.
pub fn safety_check(params: TrialSafetyCheckParams) -> Result<CallToolResult, McpError> {
    let rule = SafetyRule {
        metric: params.metric,
        threshold: params.threshold,
        description: params.description.unwrap_or_else(|| format!("Stop if > {}", params.threshold)),
    };

    let result = check_safety_boundary(&rule, params.observed_value);
    let status = if result.is_safe { "SAFE — continue" } else { "STOP — boundary crossed" };

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Safety Check: **{status}**\n\
         Metric: {}\n\
         Observed: {:.4}\n\
         Threshold: {:.4}\n\
         Margin: {:.4} ({})",
        result.metric,
        result.observed,
        result.threshold,
        result.margin,
        if result.margin >= 0.0 { "below threshold" } else { "EXCEEDED" }
    ))]))
}

// ── 7. trial_endpoint_evaluate ────────────────────────────────────────────────

/// Evaluate a primary or secondary endpoint with statistical testing.
pub fn endpoint_evaluate(params: TrialEndpointEvaluateParams) -> Result<CallToolResult, McpError> {
    let mut result = match params.test_type.as_str() {
        "two_proportion" => {
            let s1 = params.s1.ok_or_else(|| McpError::invalid_params("s1 required", None))?;
            let n1 = params.n1.ok_or_else(|| McpError::invalid_params("n1 required", None))?;
            let s2 = params.s2.ok_or_else(|| McpError::invalid_params("s2 required", None))?;
            let n2 = params.n2.ok_or_else(|| McpError::invalid_params("n2 required", None))?;
            evaluate_two_proportions(s1, n1, s2, n2, params.alpha)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
        }
        "two_mean" => {
            let mean1 = params.mean1.ok_or_else(|| McpError::invalid_params("mean1 required", None))?;
            let sd1 = params.sd1.ok_or_else(|| McpError::invalid_params("sd1 required", None))?;
            let n1 = params.n1.ok_or_else(|| McpError::invalid_params("n1 required", None))?;
            let mean2 = params.mean2.ok_or_else(|| McpError::invalid_params("mean2 required", None))?;
            let sd2 = params.sd2.ok_or_else(|| McpError::invalid_params("sd2 required", None))?;
            let n2 = params.n2.ok_or_else(|| McpError::invalid_params("n2 required", None))?;
            evaluate_two_means(mean1, sd1, n1, mean2, sd2, n2, params.alpha)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
        }
        other => return Ok(CallToolResult::error(vec![Content::text(format!(
            "Unknown test_type '{other}'. Use: two_proportion | two_mean"
        ))])),
    };

    if let Some(name) = params.endpoint_name {
        result.name = name;
    }

    let sig_label = if result.significant { "SIGNIFICANT" } else { "not significant" };
    let nnt_str = result.nnt.map_or("N/A".into(), |n| format!("{n:.1}"));
    let json = serde_json::to_string_pretty(&result)
        .unwrap_or_else(|_| "{}".into());

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Endpoint Evaluation: **{sig_label}**\n\
         Test statistic: {:.4}\n\
         p-value: {:.4}\n\
         Effect size: {:.4}\n\
         95% CI: [{:.4}, {:.4}]\n\
         NNT: {nnt_str}\n\n```json\n{json}\n```",
        result.test_statistic,
        result.p_value,
        result.effect_size,
        result.ci_lower,
        result.ci_upper,
    ))]))
}

// ── 8. trial_multiplicity_adjust ──────────────────────────────────────────────

/// Apply multiplicity adjustment to a set of p-values.
pub fn multiplicity_adjust(
    params: TrialMultiplicityAdjustParams,
) -> Result<CallToolResult, McpError> {
    let p_values: Vec<f64> = params
        .p_values
        .split(',')
        .map(|s| s.trim().parse::<f64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| McpError::invalid_params(format!("p_values parse error: {e}"), None))?;

    if p_values.is_empty() {
        return Ok(CallToolResult::error(vec![Content::text("p_values must not be empty".into())]));
    }

    let results = match params.method.as_str() {
        "bonferroni" => bonferroni_adjust(&p_values, params.alpha),
        "holm" => holm_adjust(&p_values, params.alpha),
        "hochberg" => hochberg_adjust(&p_values, params.alpha),
        "bh" | "benjamini_hochberg" => benjamini_hochberg_adjust(&p_values, params.alpha),
        other => return Ok(CallToolResult::error(vec![Content::text(format!(
            "Unknown method '{other}'. Use: bonferroni | holm | hochberg | bh"
        ))])),
    };

    let sig_count = results.iter().filter(|r| r.significant).count();
    let rows = results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            format!(
                "  H{}: p={:.4} (threshold={:.4}) → {}",
                i + 1,
                r.original_p,
                r.adjusted_threshold,
                if r.significant { "SIGNIFICANT" } else { "not significant" }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Multiplicity Adjustment ({}, α={:.3}):\n{rows}\n\n{sig_count}/{} hypotheses significant",
        params.method, params.alpha, results.len()
    ))]))
}

// ── 9. trial_adapt_decide ─────────────────────────────────────────────────────

/// Evaluate an adaptive modification against pre-specified protocol rules.
pub fn adapt_decide(params: TrialAdaptDecideParams) -> Result<CallToolResult, McpError> {
    let protocol: Protocol = serde_json::from_str(&params.protocol_json)
        .map_err(|e| McpError::invalid_params(format!("protocol_json: {e}"), None))?;

    let interim_data: InterimData = serde_json::from_str(&params.interim_data_json)
        .map_err(|e| McpError::invalid_params(format!("interim_data_json: {e}"), None))?;

    match evaluate_adaptation(&protocol, &params.adaptation_type, &interim_data) {
        Ok(decision) => {
            let verdict = if decision.approved { "APPROVED" } else { "REJECTED" };
            let params_str = decision
                .new_parameters
                .as_ref()
                .map(|p| format!("\nNew parameters: {p}"))
                .unwrap_or_default();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Adaptation Decision: **{verdict}**\n\
                 Type: {}\n\
                 Rationale: {}{}",
                params.adaptation_type, decision.rationale, params_str
            ))]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Adaptation rejected: {e}"
        ))])),
    }
}

// ── 10. trial_report_generate ─────────────────────────────────────────────────

/// Generate a CONSORT-style Markdown report for a completed trial.
pub fn report_generate(params: TrialReportGenerateParams) -> Result<CallToolResult, McpError> {
    let protocol: Protocol = serde_json::from_str(&params.protocol_json)
        .map_err(|e| McpError::invalid_params(format!("protocol_json: {e}"), None))?;

    let results: Vec<EndpointResult> = serde_json::from_str(&params.results_json)
        .map_err(|e| McpError::invalid_params(format!("results_json: {e}"), None))?;

    let report = generate_report(&protocol, &results);
    Ok(CallToolResult::success(vec![Content::text(report)]))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_blinding(s: &str) -> Result<BlindingLevel, McpError> {
    match s {
        "Open" => Ok(BlindingLevel::Open),
        "Single" => Ok(BlindingLevel::Single),
        "Double" => Ok(BlindingLevel::Double),
        "Triple" => Ok(BlindingLevel::Triple),
        other => Err(McpError::invalid_params(
            format!("Unknown blinding level '{other}'. Use: Open | Single | Double | Triple"),
            None,
        )),
    }
}
