// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Immunity MCP tools for antipattern detection and prevention.
//!
//! ## Primitive Grounding
//!
//! | Tool | T1 Primitives |
//! |------|---------------|
//! | immunity_scan | κ (comparison) + μ (mapping) + σ (sequence) |
//! | immunity_scan_errors | κ (comparison) + μ (mapping) |
//! | immunity_list | σ (sequence) + κ (comparison) |
//! | immunity_get | μ (mapping) |
//! | immunity_propose | π (persistence) + μ (mapping) |
//! | immunity_status | N (quantity) + ς (state) |
//!
//! ## Homeostasis Loop
//!
//! ```text
//! SENSE ──► DECIDE ──► RESPOND ──► LEARN ──► SENSE...
//!   │         │          │          │
//! [PAMP]   [Match]    [Block/     [Store
//! [DAMP]   [Pattern]   Fix]       Antibody]
//! ```

use crate::params::{
    ImmunityGetParams, ImmunityListParams, ImmunityProposeParams, ImmunityScanErrorsParams,
    ImmunityScanParams,
};
use crate::tooling::attach_forensic_meta;
use nexcore_immunity::{
    AntibodyRegistry, ImmunityScanner, ThreatLevel, ThreatType, load_default_registry,
};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;
use std::sync::OnceLock;

/// Cached scanner instance (singleton pattern for performance).
/// Uses Option to handle initialization errors gracefully.
static SCANNER: OnceLock<Result<(AntibodyRegistry, ImmunityScanner), String>> = OnceLock::new();

/// Get or initialize the scanner.
fn get_scanner() -> Result<&'static (AntibodyRegistry, ImmunityScanner), McpError> {
    let result = SCANNER.get_or_init(|| {
        let registry = match load_default_registry() {
            Ok(r) => r,
            Err(e) => return Err(format!("Failed to load antibody registry: {e}")),
        };
        let scanner = match ImmunityScanner::new(&registry) {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to create scanner: {e}")),
        };
        Ok((registry, scanner))
    });

    match result {
        Ok(pair) => Ok(pair),
        Err(e) => Err(McpError::internal_error(e.clone(), None)),
    }
}

/// Scan code content for antipatterns.
///
/// Returns threats found, their severity, and suggested responses.
pub fn immunity_scan(params: ImmunityScanParams) -> Result<CallToolResult, McpError> {
    let (_, scanner) = get_scanner()?;

    let result = scanner.scan(&params.content, params.file_path.as_deref());

    let threats_json: Vec<serde_json::Value> = result
        .threats
        .iter()
        .map(|t| {
            json!({
                "antibody_id": t.antibody_id,
                "name": t.antibody_name,
                "type": t.threat_type.to_string(),
                "severity": t.severity.to_string(),
                "line": t.location,
                "matched": t.matched_content,
                "confidence": t.confidence,
                "response": t.response.to_string()
            })
        })
        .collect();

    let response = json!({
        "clean": result.clean,
        "threats": threats_json,
        "antibodies_triggered": result.antibodies_applied,
        "metrics": {
            "scanned": result.metrics.total_scanned,
            "detected": result.metrics.threats_detected
        }
    });

    let max_confidence = result
        .threats
        .iter()
        .map(|t| t.confidence)
        .fold(0.0_f64, f64::max);
    let mut res = CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
    )]);
    attach_forensic_meta(&mut res, max_confidence, Some(!result.clean), "immunity");
    Ok(res)
}

/// Scan error output for known patterns.
///
/// Useful for learning from build failures.
pub fn immunity_scan_errors(params: ImmunityScanErrorsParams) -> Result<CallToolResult, McpError> {
    let (_, scanner) = get_scanner()?;

    let result = scanner.scan_errors(&params.stderr);

    let threats_json: Vec<serde_json::Value> = result
        .threats
        .iter()
        .map(|t| {
            json!({
                "antibody_id": t.antibody_id,
                "name": t.antibody_name,
                "severity": t.severity.to_string(),
                "matched": t.matched_content
            })
        })
        .collect();

    let response = json!({
        "known_errors": !result.clean,
        "matches": threats_json
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
    )]))
}

/// List all antibodies with optional filtering.
pub fn immunity_list(params: ImmunityListParams) -> Result<CallToolResult, McpError> {
    let (registry, _) = get_scanner()?;

    let antibodies: Vec<serde_json::Value> = registry
        .antibodies
        .iter()
        .filter(|ab| {
            // Filter by threat type if specified
            if let Some(ref tt) = params.threat_type {
                let expected = match tt.to_uppercase().as_str() {
                    "PAMP" => ThreatType::Pamp,
                    "DAMP" => ThreatType::Damp,
                    _ => return true, // Invalid filter, include all
                };
                if ab.threat_type != expected {
                    return false;
                }
            }

            // Filter by minimum severity if specified
            if let Some(ref sev) = params.min_severity {
                let min_sev = match sev.to_lowercase().as_str() {
                    "low" => ThreatLevel::Low,
                    "medium" => ThreatLevel::Medium,
                    "high" => ThreatLevel::High,
                    "critical" => ThreatLevel::Critical,
                    _ => ThreatLevel::Low, // Invalid filter, include all
                };
                if ab.severity < min_sev {
                    return false;
                }
            }

            true
        })
        .map(|ab| {
            json!({
                "id": ab.id,
                "name": ab.name,
                "type": ab.threat_type.to_string(),
                "severity": ab.severity.to_string(),
                "confidence": ab.confidence,
                "applications": ab.applications
            })
        })
        .collect();

    let response = json!({
        "count": antibodies.len(),
        "antibodies": antibodies
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
    )]))
}

/// Get a specific antibody by ID.
pub fn immunity_get(params: ImmunityGetParams) -> Result<CallToolResult, McpError> {
    let (registry, _) = get_scanner()?;

    let antibody = registry.get(&params.id).ok_or_else(|| {
        McpError::invalid_params(format!("Antibody not found: {}", params.id), None)
    })?;

    let response = json!({
        "id": antibody.id,
        "name": antibody.name,
        "type": antibody.threat_type.to_string(),
        "severity": antibody.severity.to_string(),
        "description": antibody.description,
        "detection": {
            "code_patterns": antibody.detection.code_patterns.iter().map(|p| &p.pattern).collect::<Vec<_>>(),
            "error_patterns": &antibody.detection.error_patterns,
            "file_contexts": &antibody.detection.file_contexts,
            "exceptions": &antibody.detection.exceptions
        },
        "response": {
            "strategy": antibody.response.strategy.to_string(),
            "description": antibody.response.description,
            "rust_template": antibody.response.rust_template
        },
        "confidence": antibody.confidence,
        "applications": antibody.applications,
        "false_positives": antibody.false_positives,
        "learned_from": antibody.learned_from,
        "reference": antibody.reference
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
    )]))
}

/// Propose a new antibody from an observed error/fix pair.
pub fn immunity_propose(params: ImmunityProposeParams) -> Result<CallToolResult, McpError> {
    let proposals_path = std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
        .join(".claude/immunity/proposals.yaml");

    // Generate a new proposal ID
    let id = format!(
        "AUTO-{}",
        nexcore_chrono::DateTime::now()
            .format("%Y%m%d%H%M%S")
            .unwrap_or_default()
    );

    let severity = params
        .severity
        .as_deref()
        .unwrap_or("medium")
        .to_lowercase();

    let proposal = format!(
        r#"
# Proposed: {timestamp}
- id: {id}
  name: proposed-from-error
  trigger:
    error_pattern: "{error_pattern}"
    context: "{context}"
  proposed_fix: |
    {fix}
  severity: {severity}
  status: pending
"#,
        timestamp = nexcore_chrono::DateTime::now()
            .format("%Y-%m-%dT%H:%M:%SZ")
            .unwrap_or_default(),
        id = id,
        error_pattern = params.error_pattern.replace('"', r#"\""#),
        context = params.context.as_deref().unwrap_or("unknown"),
        fix = params.fix_applied.replace('\n', "\n    "),
        severity = severity,
    );

    // Append to proposals file
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&proposals_path)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(proposal.as_bytes())
        })
        .map_err(|e| McpError::internal_error(format!("Failed to write proposal: {e}"), None))?;

    let response = json!({
        "status": "proposed",
        "id": id,
        "path": proposals_path.display().to_string(),
        "message": "Antibody proposal saved. Review with `/antipattern-immunity review`."
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
    )]))
}

/// Get immunity system status and statistics.
pub fn immunity_status() -> Result<CallToolResult, McpError> {
    let (registry, scanner) = get_scanner()?;

    let stats = scanner.stats();

    let response = json!({
        "registry_version": registry.version,
        "antibodies": {
            "total": stats.get("total").copied().unwrap_or(0),
            "pamp": stats.get("pamp").copied().unwrap_or(0),
            "damp": stats.get("damp").copied().unwrap_or(0),
            "critical": stats.get("critical").copied().unwrap_or(0),
            "high": stats.get("high").copied().unwrap_or(0)
        },
        "registry_path": nexcore_immunity::DEFAULT_REGISTRY_PATH,
        "homeostasis_loop": "SENSE → DECIDE → RESPOND → LEARN"
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string()),
    )]))
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immunity_scan_clean() {
        let params = ImmunityScanParams {
            content: "fn foo() -> Result<(), Error> { Ok(()) }".to_string(),
            file_path: Some("test.rs".to_string()),
        };
        let result = immunity_scan(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_immunity_list() {
        let params = ImmunityListParams::default();
        let result = immunity_list(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_immunity_status() {
        let result = immunity_status();
        assert!(result.is_ok());
    }
}
