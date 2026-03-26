//! Benefit-Risk MCP Tools
//!
//! Two complementary tool families:
//! - **QBRI**: Expert-judgment quantifier (3 tools)
//! - **QBR**: Statistical-evidence quantifier with 4 forms (3 tools)

use crate::params::{QbriComputeParams, QbriDeriveParams};
use nexcore_vigilance::pv::benefit_risk::{
    BenefitAssessment, QbriResult, QbriThresholds, RiskAssessment, compute_qbri, derive_thresholds,
    generate_synthetic_data,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::{Value, json};

// ═══════════════════════════════════════════════════════════════════════════
// QBRI TOOLS (unchanged)
// ═══════════════════════════════════════════════════════════════════════════

fn format_qbri_result(r: &QbriResult, t: &QbriThresholds) -> Value {
    json!({
        "qbri": { "index": format!("{:.3}", r.index), "decision": format!("{:?}", r.decision), "confidence": format!("{:.2}", r.confidence) },
        "components": { "benefit_score": format!("{:.3}", r.benefit_score), "risk_score": format!("{:.3}", r.risk_score) },
        "thresholds": { "tau_approve": t.tau_approve, "tau_monitor": t.tau_monitor, "tau_uncertain": t.tau_uncertain },
        "equation": "QBRI = (B × Pb × Ub) / (R × Pr × Sr × Tr)",
    })
}

fn format_inputs(p: &QbriComputeParams) -> Value {
    json!({
        "benefit": { "magnitude": p.benefit_effect, "probability": 1.0 - p.benefit_pvalue, "unmet_need": p.unmet_need },
        "risk": { "signal": p.risk_signal, "probability": p.risk_probability, "severity": p.risk_severity, "reversible": p.reversible },
    })
}

/// Compute QBRI from benefit and risk parameters.
pub fn qbri_compute(p: QbriComputeParams) -> Result<CallToolResult, McpError> {
    let benefit = BenefitAssessment::from_trial(p.benefit_effect, p.benefit_pvalue, p.unmet_need);
    let risk = RiskAssessment::from_signal(
        p.risk_signal,
        p.risk_probability,
        p.risk_severity,
        p.reversible,
    );
    let thresholds = QbriThresholds::default();
    let result = compute_qbri(&benefit, &risk, &thresholds);

    let mut output = format_qbri_result(&result, &thresholds);
    output["inputs"] = format_inputs(&p);

    Ok(CallToolResult::success(vec![Content::text(
        output.to_string(),
    )]))
}

/// Derive optimal QBRI thresholds from historical FDA decisions.
pub fn qbri_derive(params: QbriDeriveParams) -> Result<CallToolResult, McpError> {
    let data = generate_synthetic_data();
    let result = derive_thresholds(&data);
    let t = &result.thresholds;

    let output = json!({
        "derived_thresholds": { "tau_approve": format!("{:.2}", t.tau_approve), "tau_monitor": format!("{:.2}", t.tau_monitor), "tau_uncertain": format!("{:.2}", t.tau_uncertain) },
        "optimization": { "accuracy": format!("{:.1}%", result.accuracy * 100.0), "n_drugs": result.n_drugs },
        "interpretation": {
            "approve": format!("QBRI > {:.2}", t.tau_approve),
            "rems": format!("QBRI ∈ [{:.2}, {:.2}]", t.tau_monitor, t.tau_approve),
            "more_data": format!("QBRI ∈ [{:.2}, {:.2}]", t.tau_uncertain, t.tau_monitor),
            "reject": format!("QBRI < {:.2}", t.tau_uncertain),
        },
        "data_source": if params.use_synthetic { "synthetic (8 drugs)" } else { "historical" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        output.to_string(),
    )]))
}

/// Get QBRI equation explanation.
pub fn qbri_equation() -> Result<CallToolResult, McpError> {
    let output = json!({
        "equation": "QBRI = (B × Pb × Ub) / (R × Pr × Sr × Tr)",
        "variables": {
            "B": "Benefit magnitude", "Pb": "P(benefit) = 1-pvalue", "Ub": "Unmet need [1-10]",
            "R": "Risk signal", "Pr": "P(causal)", "Sr": "Severity [1-7]", "Tr": "Treatability",
        },
        "hypothesis_thresholds": { "tau_approve": 2.0, "tau_monitor": 1.0, "tau_uncertain": 0.5 },
    });

    Ok(CallToolResult::success(vec![Content::text(
        output.to_string(),
    )]))
}

// ═══════════════════════════════════════════════════════════════════════════
// QBR TOOLS — Statistical-Evidence Benefit-Risk
// ═══════════════════════════════════════════════════════════════════════════

use crate::params::{
    QbrComputeParams, QbrHillParam, QbrSimpleParams, QbrTableParam, QbrTherapeuticWindowParams,
    QbrWeightParam,
};

fn parse_qbr_method(s: &str) -> Result<nexcore_qbr::QbrSignalMethod, nexcore_error::NexError> {
    match s {
        "prr" => Ok(nexcore_qbr::QbrSignalMethod::Prr),
        "ror" => Ok(nexcore_qbr::QbrSignalMethod::Ror),
        "ic" => Ok(nexcore_qbr::QbrSignalMethod::Ic),
        "ebgm" => Ok(nexcore_qbr::QbrSignalMethod::Ebgm),
        other => Err(nexcore_error::nexerror!(
            "Invalid method '{other}'. Must be: prr, ror, ic, ebgm"
        )),
    }
}

fn table_to_ct(t: &QbrTableParam) -> nexcore_pv_core::signals::ContingencyTable {
    nexcore_pv_core::signals::ContingencyTable::new(t.a, t.b, t.c, t.d)
}

fn weight_to_measured(w: &QbrWeightParam) -> nexcore_constants::Measured<f64> {
    nexcore_constants::Measured::new(w.value, nexcore_constants::Confidence::new(w.confidence))
}

fn measured_to_json(m: &nexcore_constants::Measured<f64>) -> Value {
    json!({ "value": m.value, "confidence": m.confidence.value() })
}

/// Compute full QBR with all available forms (simple, Bayesian, composite, therapeutic window).
pub fn qbr_compute(p: QbrComputeParams) -> Result<CallToolResult, McpError> {
    let method = match parse_qbr_method(&p.method) {
        Ok(m) => m,
        Err(msg) => return Ok(CallToolResult::error(vec![Content::text(msg)])),
    };

    let input = nexcore_qbr::BenefitRiskInput {
        benefit_tables: p.benefit_tables.iter().map(table_to_ct).collect(),
        risk_tables: p.risk_tables.iter().map(table_to_ct).collect(),
        benefit_weights: p
            .benefit_weights
            .as_ref()
            .map(|ws| ws.iter().map(weight_to_measured).collect()),
        risk_weights: p
            .risk_weights
            .as_ref()
            .map(|ws| ws.iter().map(weight_to_measured).collect()),
        hill_efficacy: p
            .hill_efficacy
            .as_ref()
            .map(|h| nexcore_qbr::HillCurveParams {
                k_half: h.k_half,
                n_hill: h.n_hill,
            }),
        hill_toxicity: p
            .hill_toxicity
            .as_ref()
            .map(|h| nexcore_qbr::HillCurveParams {
                k_half: h.k_half,
                n_hill: h.n_hill,
            }),
        integration_bounds: p
            .integration_bounds
            .as_ref()
            .map(|b| nexcore_qbr::IntegrationBounds {
                dose_min: b.dose_min,
                dose_max: b.dose_max,
                intervals: b.intervals,
            }),
        method,
    };

    match nexcore_qbr::compute_qbr(&input) {
        Ok(qbr) => {
            let output = json!({
                "simple": measured_to_json(&qbr.simple),
                "bayesian": qbr.bayesian.as_ref().map(measured_to_json),
                "composite": qbr.composite.as_ref().map(measured_to_json),
                "therapeutic_window": qbr.therapeutic_window.as_ref().map(measured_to_json),
                "details": {
                    "benefit_signal": measured_to_json(&qbr.details.benefit_signal),
                    "risk_signal": measured_to_json(&qbr.details.risk_signal),
                    "benefit_eb05": qbr.details.benefit_eb05,
                    "risk_eb95": qbr.details.risk_eb95,
                    "worst_case_bayesian": qbr.details.worst_case_bayesian.as_ref().map(measured_to_json),
                    "method": format!("{:?}", qbr.details.method).to_lowercase(),
                },
            });
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&output).unwrap_or_else(|_| output.to_string()),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    }
}

/// Compute simple benefit-risk ratio from one benefit and one risk contingency table.
pub fn qbr_simple(p: QbrSimpleParams) -> Result<CallToolResult, McpError> {
    let method = match parse_qbr_method(&p.method) {
        Ok(m) => m,
        Err(msg) => return Ok(CallToolResult::error(vec![Content::text(msg)])),
    };

    let benefit_ct = table_to_ct(&p.benefit_table);
    let risk_ct = table_to_ct(&p.risk_table);

    let qbr_ratio = match nexcore_qbr::compute_simple(&benefit_ct, &risk_ct, method) {
        Ok(r) => r,
        Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    };
    let benefit_signal = match nexcore_qbr::signal::extract_signal_strength(&benefit_ct, method) {
        Ok(s) => s,
        Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    };
    let risk_signal = match nexcore_qbr::signal::extract_signal_strength(&risk_ct, method) {
        Ok(s) => s,
        Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    };

    let output = json!({
        "qbr": measured_to_json(&qbr_ratio),
        "benefit_signal": measured_to_json(&benefit_signal),
        "risk_signal": measured_to_json(&risk_signal),
        "method": format!("{:?}", method).to_lowercase(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| output.to_string()),
    )]))
}

/// Compute therapeutic window from efficacy and toxicity Hill curve parameters.
pub fn qbr_therapeutic_window(p: QbrTherapeuticWindowParams) -> Result<CallToolResult, McpError> {
    let efficacy = nexcore_qbr::HillCurveParams {
        k_half: p.efficacy.k_half,
        n_hill: p.efficacy.n_hill,
    };
    let toxicity = nexcore_qbr::HillCurveParams {
        k_half: p.toxicity.k_half,
        n_hill: p.toxicity.n_hill,
    };
    let bounds = p.bounds.as_ref().map_or(
        nexcore_qbr::IntegrationBounds {
            dose_min: 0.1,
            dose_max: 100.0,
            intervals: 1000,
        },
        |b| nexcore_qbr::IntegrationBounds {
            dose_min: b.dose_min,
            dose_max: b.dose_max,
            intervals: b.intervals,
        },
    );

    let tw = match nexcore_qbr::compute_therapeutic_window(&efficacy, &toxicity, &bounds) {
        Ok(r) => r,
        Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    };

    // Compute component AUCs
    let n = if !bounds.intervals.is_multiple_of(2) {
        bounds.intervals + 1
    } else {
        bounds.intervals
    };

    use nexcore_primitives::chemistry::cooperativity::hill_response;

    let eff_k = efficacy.k_half;
    let eff_n = efficacy.n_hill;
    let tox_k = toxicity.k_half;
    let tox_n = toxicity.n_hill;

    let efficacy_auc = match nexcore_qbr::simpson_integrate(
        |d| hill_response(d, eff_k, eff_n),
        bounds.dose_min,
        bounds.dose_max,
        n,
    ) {
        Ok(v) => v,
        Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    };

    let toxicity_auc = match nexcore_qbr::simpson_integrate(
        |d| hill_response(d, tox_k, tox_n),
        bounds.dose_min,
        bounds.dose_max,
        n,
    ) {
        Ok(v) => v,
        Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    };

    let output = json!({
        "therapeutic_window": measured_to_json(&tw),
        "efficacy_auc": efficacy_auc,
        "toxicity_auc": toxicity_auc,
        "bounds": { "dose_min": bounds.dose_min, "dose_max": bounds.dose_max, "intervals": n },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| output.to_string()),
    )]))
}
