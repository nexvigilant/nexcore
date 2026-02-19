//! SOP-Anatomy-Code MCP tools
//!
//! 4 tools exposing the triple mapping, capability transfer, codebase audit,
//! and coverage report.
//!
//! ## Primitive Foundation
//! - σ (Sequence): Ordered 18-section governance pipeline
//! - μ (Mapping): Cross-domain translation (SOP <-> Anatomy <-> Code)
//! - κ (Comparison): Chirality checks during transfer

use crate::params::sop_anatomy::{
    SopAnatomyAuditParams, SopAnatomyBridgeParams, SopAnatomyCoverageParams, SopAnatomyMapParams,
};
use nexcore_sop_anatomy::audit;
use nexcore_sop_anatomy::mapping::{CoverageReport, Domain, SopSection};
use nexcore_sop_anatomy::transfer;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use std::path::Path;

// ─── sop_anatomy_map ───────────────────────────────────────────────────────

/// Look up the SOP-Anatomy-Code triple mapping for one or all 18 sections.
pub fn sop_anatomy_map(params: SopAnatomyMapParams) -> Result<CallToolResult, McpError> {
    let json = match params.section {
        Some(n) => match SopSection::from_number(n) {
            Some(s) => serde_json::to_string_pretty(&s.mapping())
                .map_err(|e| McpError::internal_error(e.to_string(), None))?,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Invalid section number: {n}. Valid range: 1-18."
                ))]));
            }
        },
        None => {
            let all: Vec<_> = SopSection::ALL.iter().map(|s| s.mapping()).collect();
            serde_json::to_string_pretty(&all).map_err(|e| McpError::internal_error(e.to_string(), None))?
        }
    };

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

// ─── sop_anatomy_bridge ────────────────────────────────────────────────────

/// Cross-domain transfer using the Capability Transfer Protocol
/// (FISSION -> CHIRALITY -> FUSION -> TITRATION).
pub fn sop_anatomy_bridge(params: SopAnatomyBridgeParams) -> Result<CallToolResult, McpError> {
    let source = match Domain::from_str_loose(&params.source_domain) {
        Some(d) => d,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Unknown source domain: '{}'. Use: sop, anatomy, or code.",
                params.source_domain
            ))]));
        }
    };

    let target = match Domain::from_str_loose(&params.target_domain) {
        Some(d) => d,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Unknown target domain: '{}'. Use: sop, anatomy, or code.",
                params.target_domain
            ))]));
        }
    };

    let result = transfer::transfer(source, &params.concept, target);
    let json =
        serde_json::to_string_pretty(&result).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

// ─── sop_anatomy_audit ─────────────────────────────────────────────────────

/// Audit a crate/project directory against all 18 SOP governance sections.
pub fn sop_anatomy_audit(params: SopAnatomyAuditParams) -> Result<CallToolResult, McpError> {
    let path = Path::new(&params.path);
    if !path.exists() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Path does not exist: {}",
            params.path
        ))]));
    }

    let report = audit::audit_path(path);
    let json =
        serde_json::to_string_pretty(&report).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

// ─── sop_anatomy_coverage ──────────────────────────────────────────────────

/// Full 18-section coverage report with bio-crate wiring status.
pub fn sop_anatomy_coverage(
    _params: SopAnatomyCoverageParams,
) -> Result<CallToolResult, McpError> {
    let report = CoverageReport::generate();
    let json =
        serde_json::to_string_pretty(&report).map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}
