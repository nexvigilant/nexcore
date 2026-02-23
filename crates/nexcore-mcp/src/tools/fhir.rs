//! FHIR R4 MCP tools.
//!
//! Parse FHIR resources, convert AdverseEvents to signal-detection-ready
//! SignalInputs, validate resource structure, and extract Bundle contents.

use nexcore_fhir::resources::{AdverseEvent, Bundle};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::fhir::{
    FhirAdverseEventToSignalParams, FhirBatchToSignalsParams, FhirParseBundleParams,
    FhirValidateResourceParams,
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

fn signal_to_json(s: &nexcore_fhir::adapter::SignalInput) -> serde_json::Value {
    json!({
        "fhir_id": s.fhir_id,
        "actuality": s.actuality,
        "meddra_term": {
            "preferred_term": s.meddra_term.preferred_term,
            "code": s.meddra_term.code,
            "is_coded": s.meddra_term.is_coded,
        },
        "drug": {
            "name": s.drug.name,
            "causality": s.drug.causality,
        },
        "severity": {
            "tier": format!("{:?}", s.severity.tier),
            "is_serious": s.severity.is_serious,
        },
        "outcome": {
            "code": s.outcome.code,
            "is_fatal": s.outcome.is_fatal,
            "is_resolved": s.outcome.is_resolved,
        },
        "event_date": s.event_date,
        "recorded_date": s.recorded_date,
    })
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Parse a FHIR AdverseEvent JSON and convert to a SignalInput for PV signal detection.
pub fn fhir_adverse_event_to_signal(
    p: FhirAdverseEventToSignalParams,
) -> Result<CallToolResult, McpError> {
    let ae: AdverseEvent = match serde_json::from_str(&p.adverse_event_json) {
        Ok(ae) => ae,
        Err(e) => {
            return err_result(&["Failed to parse AdverseEvent JSON: ", &e.to_string()].concat());
        }
    };
    let signal = nexcore_fhir::adverse_event_to_signal(&ae);
    ok_json(json!({
        "signal": signal_to_json(&signal),
    }))
}

/// Batch convert multiple FHIR AdverseEvent JSONs to SignalInputs.
pub fn fhir_batch_to_signals(p: FhirBatchToSignalsParams) -> Result<CallToolResult, McpError> {
    let mut events = Vec::with_capacity(p.adverse_events_json.len());
    for (i, json_str) in p.adverse_events_json.iter().enumerate() {
        match serde_json::from_str::<AdverseEvent>(json_str) {
            Ok(ae) => events.push(ae),
            Err(e) => {
                return err_result(
                    &[
                        "Parse error at index ",
                        &i.to_string(),
                        ": ",
                        &e.to_string(),
                    ]
                    .concat(),
                );
            }
        }
    }
    let signals = nexcore_fhir::adverse_events_to_signals(&events);
    ok_json(json!({
        "total": signals.len(),
        "signals": signals.iter().map(signal_to_json).collect::<Vec<_>>(),
    }))
}

/// Parse a FHIR Bundle JSON and extract resource summaries.
pub fn fhir_parse_bundle(p: FhirParseBundleParams) -> Result<CallToolResult, McpError> {
    let bundle: Bundle = match serde_json::from_str(&p.bundle_json) {
        Ok(b) => b,
        Err(e) => return err_result(&["Failed to parse Bundle JSON: ", &e.to_string()].concat()),
    };

    let entries: Vec<serde_json::Value> = bundle
        .entry
        .iter()
        .map(|entry| {
            let resource_type = entry
                .resource
                .as_ref()
                .and_then(|r| r.get("resourceType"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let id = entry
                .resource
                .as_ref()
                .and_then(|r| r.get("id"))
                .and_then(|v| v.as_str());
            json!({
                "full_url": entry.full_url,
                "resource_type": resource_type,
                "id": id,
            })
        })
        .collect();

    ok_json(json!({
        "type": bundle.r#type,
        "total": bundle.total,
        "entry_count": entries.len(),
        "entries": entries,
    }))
}

/// Validate a FHIR resource JSON structure (checks required fields, resourceType).
pub fn fhir_validate_resource(p: FhirValidateResourceParams) -> Result<CallToolResult, McpError> {
    let value: serde_json::Value = match serde_json::from_str(&p.resource_json) {
        Ok(v) => v,
        Err(e) => return err_result(&["Invalid JSON: ", &e.to_string()].concat()),
    };

    let mut issues: Vec<serde_json::Value> = Vec::new();
    let mut valid = true;

    // Check resourceType field exists
    let resource_type = value.get("resourceType").and_then(|v| v.as_str());
    if resource_type.is_none() {
        issues.push(json!({"severity": "error", "field": "resourceType", "message": "Missing required field 'resourceType'"}));
        valid = false;
    }

    // Check expected type matches
    if let (Some(expected), Some(actual)) = (&p.expected_type, resource_type) {
        if expected != actual {
            let msg = ["Expected '", expected, "' but found '", actual, "'"].concat();
            issues.push(json!({
                "severity": "error",
                "field": "resourceType",
                "message": msg,
            }));
            valid = false;
        }
    }

    // Try parsing as known resource types
    if let Some(rt) = resource_type {
        let parse_result = match rt {
            "AdverseEvent" => serde_json::from_value::<AdverseEvent>(value.clone()).err(),
            "Bundle" => serde_json::from_value::<Bundle>(value.clone()).err(),
            "Patient" => {
                serde_json::from_value::<nexcore_fhir::resources::Patient>(value.clone()).err()
            }
            "Medication" => {
                serde_json::from_value::<nexcore_fhir::resources::Medication>(value.clone()).err()
            }
            "Observation" => {
                serde_json::from_value::<nexcore_fhir::resources::Observation>(value.clone()).err()
            }
            "Condition" => {
                serde_json::from_value::<nexcore_fhir::resources::Condition>(value.clone()).err()
            }
            _ => None, // Unknown type — structural validation only
        };
        if let Some(e) = parse_result {
            let msg = ["Failed to parse as ", rt, ": ", &e.to_string()].concat();
            issues.push(json!({
                "severity": "error",
                "field": "structure",
                "message": msg,
            }));
            valid = false;
        }
    }

    ok_json(json!({
        "valid": valid,
        "resource_type": resource_type,
        "issues": issues,
    }))
}
