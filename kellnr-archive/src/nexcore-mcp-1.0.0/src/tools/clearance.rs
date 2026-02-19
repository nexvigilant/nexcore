//! Security Classification MCP tools — 5-level clearance system for NexVigilant.
//!
//! # T1 Grounding
//! - ∂ (boundary): Classification levels define security boundaries
//! - ς (state): Each level is a security state, each mode is a behavioral state
//! - κ (comparison): Priority ordering, level comparison, gate decisions
//! - π (persistence): Append-only audit trail
//! - μ (mapping): Tag targets map names to classification levels

use nexcore_clearance::{
    AccessMode, ChangeDirection, ClassificationLevel, ClearanceConfig, ClearanceGate,
    ClearancePolicy, ClearancePriority, CrossBoundaryValidator, TagTarget,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{
    ClearanceEvaluateParams, ClearanceLevelInfoParams, ClearancePolicyForParams,
    ClearanceValidateChangeParams,
};

/// Parse a classification level from a string, or return an MCP error.
fn parse_level(s: &str) -> Result<ClassificationLevel, McpError> {
    ClassificationLevel::from_str_loose(s).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Unknown level: '{s}'. Valid: Public, Internal, Confidential, Secret, TopSecret"
            ),
            None,
        )
    })
}

/// Parse an access mode from a string.
fn parse_mode(s: &str) -> Result<AccessMode, McpError> {
    match s.to_lowercase().as_str() {
        "unrestricted" => Ok(AccessMode::Unrestricted),
        "aware" => Ok(AccessMode::Aware),
        "guarded" => Ok(AccessMode::Guarded),
        "enforced" => Ok(AccessMode::Enforced),
        "lockdown" => Ok(AccessMode::Lockdown),
        _ => Err(McpError::invalid_params(
            format!("Unknown mode: '{s}'. Valid: Unrestricted, Aware, Guarded, Enforced, Lockdown"),
            None,
        )),
    }
}

/// Parse a tag target from kind + name.
fn parse_target(kind: &str, name: String) -> Result<TagTarget, McpError> {
    match kind.to_lowercase().as_str() {
        "project" => Ok(TagTarget::Project(name)),
        "crate" => Ok(TagTarget::Crate(name)),
        "file" => Ok(TagTarget::File(name)),
        "skill" => Ok(TagTarget::Skill(name)),
        "mcp_tool" => Ok(TagTarget::McpTool(name)),
        "region" => Ok(TagTarget::Region(name)),
        "data_category" => Ok(TagTarget::DataCategory(name)),
        _ => Err(McpError::invalid_params(
            format!(
                "Unknown target kind: '{kind}'. Valid: project, crate, file, skill, mcp_tool, region, data_category"
            ),
            None,
        )),
    }
}

/// Evaluate a gate operation (access, write, or external call) against classification policy.
pub fn clearance_evaluate(params: ClearanceEvaluateParams) -> Result<CallToolResult, McpError> {
    let level = parse_level(&params.level)?;
    let target = parse_target(&params.target_kind, params.target_name)?;

    let config = ClearanceConfig::with_defaults();
    let mut gate = ClearanceGate::with_config(config);

    let result = match params.operation.to_lowercase().as_str() {
        "access" => gate.evaluate_access(&target, level, &params.actor),
        "write" => gate.evaluate_write(&target, level, &params.actor),
        "external_call" => {
            let tool_name = params.tool_name.as_deref().unwrap_or("unknown");
            gate.evaluate_external_call(&target, level, tool_name, &params.actor)
        }
        other => {
            return Err(McpError::invalid_params(
                format!("Unknown operation: '{other}'. Valid: access, write, external_call"),
                None,
            ));
        }
    };

    let response = serde_json::json!({
        "result": result.to_string(),
        "is_pass": result.is_pass(),
        "is_block": result.is_block(),
        "exit_code": result.exit_code(),
        "target": target.to_string(),
        "level": level.to_string(),
        "operation": params.operation,
        "audit_entries": gate.audit().len(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Get the enforcement policy for a specific classification level.
pub fn clearance_policy_for(params: ClearancePolicyForParams) -> Result<CallToolResult, McpError> {
    let level = parse_level(&params.level)?;
    let policy = ClearancePolicy::default_for(level);
    let config = ClearanceConfig::with_defaults();
    let effective_mode = config.effective_mode(level);

    let response = serde_json::json!({
        "level": level.to_string(),
        "access_mode": policy.access_mode.to_string(),
        "effective_mode": effective_mode.to_string(),
        "audit": policy.audit,
        "warn_on_write": policy.warn_on_write,
        "block_external": policy.block_external,
        "require_dual_auth": policy.require_dual_auth,
        "block_external_tools": policy.block_external_tools,
        "level_ordinal": level.ordinal(),
        "is_restricted": level.is_restricted(),
        "requires_audit": level.requires_audit(),
        "allows_external": level.allows_external(),
        "priority": {
            "classification": ClearancePriority::Classification.to_string(),
            "outranked_by": ["P0: Patient Safety", "P1: Signal Integrity", "P2: Regulatory", "P2b: Privacy"],
            "outranks": ["P3: Data Quality", "P4: Operational", "P5: Cost"],
        },
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Validate a classification change (upgrade or downgrade).
pub fn clearance_validate_change(
    params: ClearanceValidateChangeParams,
) -> Result<CallToolResult, McpError> {
    let from = parse_level(&params.from_level)?;
    let to = parse_level(&params.to_level)?;
    let mode = parse_mode(&params.mode)?;

    let direction = CrossBoundaryValidator::direction(from, to);
    let result =
        CrossBoundaryValidator::validate_change(from, to, mode, params.downgrade_permitted);

    let response = serde_json::json!({
        "from": from.to_string(),
        "to": to.to_string(),
        "direction": match direction {
            ChangeDirection::Upgrade => "upgrade",
            ChangeDirection::Downgrade => "downgrade",
            ChangeDirection::Neutral => "neutral",
        },
        "result": result.to_string(),
        "is_permitted": result.is_permitted(),
        "is_blocked": result.is_blocked(),
        "mode": mode.to_string(),
        "downgrade_permitted": params.downgrade_permitted,
        "crosses_boundary": CrossBoundaryValidator::crosses_boundary(from, to),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Get metadata and predicates for a specific classification level.
pub fn clearance_level_info(params: ClearanceLevelInfoParams) -> Result<CallToolResult, McpError> {
    let level = parse_level(&params.level)?;
    let policy = ClearancePolicy::default_for(level);

    let response = serde_json::json!({
        "level": level.to_string(),
        "ordinal": level.ordinal(),
        "is_restricted": level.is_restricted(),
        "requires_audit": level.requires_audit(),
        "requires_dual_auth": level.requires_dual_auth(),
        "allows_external": level.allows_external(),
        "default_access_mode": policy.access_mode.to_string(),
        "enforcement_active": policy.access_mode.is_enforcement_active(),
        "allows_cross_boundary": policy.access_mode.allows_cross_boundary(),
        "allows_external_calls": policy.access_mode.allows_external_calls(),
        "requires_full_audit": policy.access_mode.requires_full_audit(),
        "all_levels": [
            {"name": "Public", "ordinal": 0, "restricted": false},
            {"name": "Internal", "ordinal": 1, "restricted": false},
            {"name": "Confidential", "ordinal": 2, "restricted": true},
            {"name": "Secret", "ordinal": 3, "restricted": true},
            {"name": "Top Secret", "ordinal": 4, "restricted": true},
        ],
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Return the full clearance system configuration with all policies.
pub fn clearance_config() -> Result<CallToolResult, McpError> {
    let config = ClearanceConfig::with_defaults();

    let levels = [
        ClassificationLevel::Public,
        ClassificationLevel::Internal,
        ClassificationLevel::Confidential,
        ClassificationLevel::Secret,
        ClassificationLevel::TopSecret,
    ];

    let mut policies = serde_json::Map::new();
    for level in &levels {
        let policy = config.policy_for(*level);
        policies.insert(
            level.to_string(),
            serde_json::json!({
                "access_mode": policy.access_mode.to_string(),
                "audit": policy.audit,
                "warn_on_write": policy.warn_on_write,
                "block_external": policy.block_external,
                "require_dual_auth": policy.require_dual_auth,
                "block_external_tools": policy.block_external_tools,
            }),
        );
    }

    let response = serde_json::json!({
        "version": config.version,
        "default_level": config.default_level.to_string(),
        "default_mode": config.default_mode.to_string(),
        "mode_override": config.mode_override.map(|m| m.to_string()),
        "policies": policies,
        "priority_hierarchy": [
            "P0: Patient Safety (supreme)",
            "P1: Signal Integrity",
            "P2: Regulatory Compliance",
            "P2b: Data Privacy (Ghost)",
            "P2c: Security Classification (Clearance)",
            "P3: Data Quality",
            "P4: Operational Efficiency",
            "P5: Cost Optimization",
        ],
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
