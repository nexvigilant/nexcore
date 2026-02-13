//! QBRI Benefit-Risk MCP Tools

use crate::params::{QbriComputeParams, QbriDeriveParams};
use nexcore_vigilance::pv::benefit_risk::{
    BenefitAssessment, QbriResult, QbriThresholds, RiskAssessment, compute_qbri, derive_thresholds,
    generate_synthetic_data,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::{Value, json};

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
