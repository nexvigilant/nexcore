//! Ghost privacy MCP tools.
//!
//! Privacy-by-design pseudonymization, PII detection, anonymization
//! boundary checking, and data scrubbing for GDPR/HIPAA compliance.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::ghost::{
    GhostBoundaryCheckParams, GhostCategoryPolicyParams, GhostModeInfoParams, GhostScanPiiParams,
    GhostScrubFieldsParams,
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

fn parse_mode(s: &str) -> Option<nexcore_ghost::GhostMode> {
    match s.to_lowercase().as_str() {
        "off" => Some(nexcore_ghost::GhostMode::Off),
        "standard" => Some(nexcore_ghost::GhostMode::Standard),
        "strict" => Some(nexcore_ghost::GhostMode::Strict),
        "maximum" => Some(nexcore_ghost::GhostMode::Maximum),
        _ => None,
    }
}

fn parse_category(s: &str) -> Option<nexcore_ghost::DataCategory> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Check anonymization boundary violations for given metrics.
pub fn ghost_boundary_check(p: GhostBoundaryCheckParams) -> Result<CallToolResult, McpError> {
    let mode = match parse_mode(&p.mode) {
        Some(m) => m,
        None => {
            return err_result(
                &[
                    "Unknown mode '",
                    &p.mode,
                    "'. Use: Off, Standard, Strict, Maximum",
                ]
                .concat(),
            );
        }
    };
    let boundary = nexcore_ghost::AnonymizationBoundary::from_mode(mode);
    let result = boundary.check_all(p.risk, p.k, p.l);
    ok_json(json!({
        "mode": p.mode,
        "boundary": {
            "max_risk": boundary.max_risk,
            "k_anonymity": boundary.k_anonymity,
            "l_diversity": boundary.l_diversity,
        },
        "observed": {
            "risk": p.risk,
            "k": p.k,
            "l": p.l,
        },
        "risk_ok": result.risk_ok,
        "k_ok": result.k_ok,
        "l_ok": result.l_ok,
        "all_ok": result.all_ok(),
        "violation_count": result.violation_count(),
        "risk_margin": boundary.risk_margin(p.risk),
    }))
}

/// Get properties for a ghost mode.
pub fn ghost_mode_info(p: GhostModeInfoParams) -> Result<CallToolResult, McpError> {
    let mode = match parse_mode(&p.mode) {
        Some(m) => m,
        None => {
            return err_result(
                &[
                    "Unknown mode '",
                    &p.mode,
                    "'. Use: Off, Standard, Strict, Maximum",
                ]
                .concat(),
            );
        }
    };
    ok_json(json!({
        "mode": p.mode,
        "label": mode.label(),
        "level": mode.level(),
        "is_active": mode.is_active(),
        "allows_reversal": mode.allows_reversal(),
        "requires_dual_auth": mode.requires_dual_auth(),
        "min_k_anonymity": mode.min_k_anonymity(),
    }))
}

/// Get the effective policy for a data category under a given mode.
pub fn ghost_category_policy(p: GhostCategoryPolicyParams) -> Result<CallToolResult, McpError> {
    let mode = match parse_mode(&p.mode) {
        Some(m) => m,
        None => {
            return err_result(
                &[
                    "Unknown mode '",
                    &p.mode,
                    "'. Use: Off, Standard, Strict, Maximum",
                ]
                .concat(),
            );
        }
    };
    let category = match parse_category(&p.category) {
        Some(c) => c,
        None => return err_result(&["Unknown category: '", &p.category, "'"].concat()),
    };
    let config = nexcore_ghost::GhostConfig {
        mode,
        ..nexcore_ghost::GhostConfig::default()
    };
    let policy = config.policy_for(category);
    ok_json(json!({
        "mode": p.mode,
        "category": p.category,
        "is_sensitive": category.is_sensitive(),
        "policy": {
            "pseudonymize": policy.pseudonymize,
            "redact": policy.redact,
            "retention_days": policy.retention_days,
            "reversal_permitted": policy.reversal_permitted,
        },
    }))
}

/// Scan fields for PII leak patterns.
pub fn ghost_scan_pii(p: GhostScanPiiParams) -> Result<CallToolResult, McpError> {
    let mode = match parse_mode(&p.mode) {
        Some(m) => m,
        None => {
            return err_result(
                &[
                    "Unknown mode '",
                    &p.mode,
                    "'. Use: Off, Standard, Strict, Maximum",
                ]
                .concat(),
            );
        }
    };
    let mut sensor = nexcore_ghost::GhostSensor::new(mode);
    let count = sensor.scan(&p.fields);
    let signals = sensor.drain();
    let items: Vec<serde_json::Value> = signals
        .iter()
        .map(|s| {
            json!({
                "pattern": format!("{:?}", s.pattern),
                "severity_weight": s.pattern.severity_weight(),
                "detected_at": s.detected_at,
                "context": s.context,
            })
        })
        .collect();
    ok_json(json!({
        "mode": p.mode,
        "fields_scanned": p.fields.len(),
        "leaks_detected": count,
        "signals": items,
    }))
}

/// Scrub PII from a set of fields according to ghost mode policy.
pub fn ghost_scrub_fields(p: GhostScrubFieldsParams) -> Result<CallToolResult, McpError> {
    let mode = match parse_mode(&p.mode) {
        Some(m) => m,
        None => {
            return err_result(
                &[
                    "Unknown mode '",
                    &p.mode,
                    "'. Use: Off, Standard, Strict, Maximum",
                ]
                .concat(),
            );
        }
    };
    let config = nexcore_ghost::GhostConfig {
        mode,
        ..nexcore_ghost::GhostConfig::default()
    };
    let scrubber = nexcore_ghost::PiiScrubber::new(config, None);
    let result = scrubber.scrub(&p.fields);

    let audit_entries: Vec<serde_json::Value> = result
        .audit
        .entries()
        .iter()
        .map(|e| {
            json!({
                "field": e.field,
                "action": e.action,
                "reason": e.reason,
                "category": e.category,
            })
        })
        .collect();

    ok_json(json!({
        "mode": p.mode,
        "input_fields": p.fields.len(),
        "modified_count": result.modified_count,
        "suppressed_count": result.suppressed_count,
        "scrubbed_fields": result.fields,
        "audit": audit_entries,
    }))
}
