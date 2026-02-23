//! Antibodies MCP tools — adaptive immune recognition.
//!
//! Pure-function wrappers: affinity computation, Ig class classification,
//! and class info lookup. Stateful repertoire management is not exposed.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::antibodies::{
    AntibodyAffinityParams, AntibodyClassifyResponseParams, AntibodyIgCatalogParams,
    AntibodyIgInfoParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

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

fn parse_severity(s: &str) -> Option<nexcore_antibodies::ThreatSeverity> {
    use nexcore_antibodies::ThreatSeverity;
    match s.to_lowercase().trim() {
        "low" => Some(ThreatSeverity::Low),
        "medium" => Some(ThreatSeverity::Medium),
        "high" => Some(ThreatSeverity::High),
        "critical" => Some(ThreatSeverity::Critical),
        _ => None,
    }
}

fn parse_ig_class(s: &str) -> Option<nexcore_antibodies::ImmunoglobulinClass> {
    use nexcore_antibodies::ImmunoglobulinClass;
    match s.trim() {
        "IgG" | "igg" => Some(ImmunoglobulinClass::IgG),
        "IgM" | "igm" => Some(ImmunoglobulinClass::IgM),
        "IgA" | "iga" => Some(ImmunoglobulinClass::IgA),
        "IgD" | "igd" => Some(ImmunoglobulinClass::IgD),
        "IgE" | "ige" => Some(ImmunoglobulinClass::IgE),
        _ => None,
    }
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Compute binding affinity between a paratope matcher and epitope signature.
pub fn antibody_compute_affinity(p: AntibodyAffinityParams) -> Result<CallToolResult, McpError> {
    let paratope = nexcore_antibodies::Paratope {
        id: "mcp-paratope".to_string(),
        matcher: p.paratope_matcher.clone(),
        specificity: 1,
    };
    let epitope = nexcore_antibodies::Epitope {
        id: "mcp-epitope".to_string(),
        signature: p.epitope_signature.clone(),
        domain: "mcp".to_string(),
    };

    let affinity = nexcore_antibodies::compute_affinity(&paratope, &epitope);

    ok_json(json!({
        "paratope_matcher": p.paratope_matcher,
        "epitope_signature": p.epitope_signature,
        "affinity": affinity.value(),
        "exceeds_igg_threshold": affinity.exceeds_threshold(0.6),
        "exceeds_igm_threshold": affinity.exceeds_threshold(0.4),
        "exceeds_ige_threshold": affinity.exceeds_threshold(0.3),
    }))
}

/// Classify which immunoglobulin response class to use for a threat.
pub fn antibody_classify_response(
    p: AntibodyClassifyResponseParams,
) -> Result<CallToolResult, McpError> {
    let severity = match parse_severity(&p.severity) {
        Some(s) => s,
        None => return err_result("severity must be 'low', 'medium', 'high', or 'critical'"),
    };

    let class = nexcore_antibodies::classify_response(severity, p.is_novel);

    ok_json(json!({
        "severity": p.severity,
        "is_novel": p.is_novel,
        "class": format!("{class}"),
        "default_threshold": class.default_threshold(),
        "requires_escalation": class.requires_escalation(),
    }))
}

/// Get information about an immunoglobulin class.
pub fn antibody_ig_info(p: AntibodyIgInfoParams) -> Result<CallToolResult, McpError> {
    let class = match parse_ig_class(&p.class) {
        Some(c) => c,
        None => return err_result("class must be IgG, IgM, IgA, IgD, or IgE"),
    };

    ok_json(json!({
        "class": format!("{class}"),
        "default_threshold": class.default_threshold(),
        "requires_escalation": class.requires_escalation(),
    }))
}

/// List all immunoglobulin classes with their properties.
pub fn antibody_ig_catalog(_p: AntibodyIgCatalogParams) -> Result<CallToolResult, McpError> {
    use nexcore_antibodies::ImmunoglobulinClass;

    let classes = [
        ImmunoglobulinClass::IgG,
        ImmunoglobulinClass::IgM,
        ImmunoglobulinClass::IgA,
        ImmunoglobulinClass::IgD,
        ImmunoglobulinClass::IgE,
    ];

    let catalog: Vec<serde_json::Value> = classes
        .iter()
        .map(|c| {
            json!({
                "class": format!("{c}"),
                "default_threshold": c.default_threshold(),
                "requires_escalation": c.requires_escalation(),
            })
        })
        .collect();

    ok_json(json!({ "immunoglobulin_classes": catalog }))
}
