//! Integumentary System MCP tools — boundary protection, permissions, scarring.
//!
//! Maps Claude Code's security boundary to the skin:
//! - Epidermis: permission system (hook allow/deny/warn)
//! - Dermis: settings precedence (project > user > global)
//! - Sweat glands: sandbox cooling (resource limits)
//! - Scarring: learned restrictions from past incidents (CAPA)
//!
//! ## T1 Primitive Grounding
//! - Permissions: ∂(Boundary) + κ(Comparison)
//! - Settings: σ(Sequence) + π(Persistence)
//! - Scarring: ∝(Irreversibility) + ∃(Existence)

use crate::params::integumentary::{
    IntegumentaryHealthParams, IntegumentaryPermissionParams, IntegumentarySandboxParams,
    IntegumentaryScarringParams, IntegumentarySettingsParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Evaluate permission cascade for an action.
pub fn permission(params: IntegumentaryPermissionParams) -> Result<CallToolResult, McpError> {
    let action = &params.action;
    let target = params.target.as_deref().unwrap_or("*");

    // Known permission rules (from hook infrastructure)
    let rules = match action.to_lowercase().as_str() {
        "bash" => vec![
            json!({"rule": "reflex-mcp-prefer", "decision": "BLOCK_by_default", "exception": "allowlist: cargo,git,gh,npm,systemctl,mkdir,cp,mv,ssh,docker"}),
            json!({"rule": "reflex-compute", "decision": "BLOCK", "trigger": "PV/stats computation patterns"}),
            json!({"rule": "reflex-guardian", "decision": "WARN", "trigger": "destructive commands"}),
            json!({"rule": "secret-scanner", "decision": "BLOCK", "trigger": "secrets in commands"}),
        ],
        "write" => vec![
            json!({"rule": "python-creation-blocker", "decision": "BLOCK", "trigger": ".py file creation"}),
            json!({"rule": "reflex-persist", "decision": "PASS_async", "trigger": "brain track on nexcore files"}),
        ],
        "edit" => vec![
            json!({"rule": "reflex-persist", "decision": "PASS_async", "trigger": "brain track on nexcore files"}),
        ],
        _ => vec![
            json!({"rule": "guardian-gate", "decision": "BLOCK_on_Critical", "trigger": "threat level check"}),
        ],
    };

    let effective_decision = if rules.iter().any(|r| {
        r["decision"]
            .as_str()
            .is_some_and(|d| d.starts_with("BLOCK"))
    }) {
        "conditional_block"
    } else {
        "allow"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "permission_cascade": {
                "action": action,
                "target": target,
                "rules": rules,
                "effective_decision": effective_decision,
                "rule_count": rules.len(),
            },
            "analog": {
                "epidermis": "First line — hook matchers check action type",
                "dermis": "Second line — content/path-specific rules",
                "hypodermis": "Deep — guardian threat level gate",
            },
        })
        .to_string(),
    )]))
}

/// Analyze settings precedence stack.
pub fn settings(params: IntegumentarySettingsParams) -> Result<CallToolResult, McpError> {
    let key = params.setting_key.as_deref().unwrap_or("*");

    let precedence = vec![
        json!({
            "layer": 1,
            "scope": "project",
            "file": ".claude/settings.json (in project root)",
            "description": "Project-specific overrides — highest precedence",
        }),
        json!({
            "layer": 2,
            "scope": "user_project",
            "file": "~/.claude/projects/<hash>/settings.json",
            "description": "User's per-project settings",
        }),
        json!({
            "layer": 3,
            "scope": "user",
            "file": "~/.claude/settings.json",
            "description": "User's global settings",
        }),
        json!({
            "layer": 4,
            "scope": "system",
            "file": "Built-in defaults",
            "description": "Claude Code hardcoded defaults — lowest precedence",
        }),
    ];

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "settings_precedence": {
                "query_key": key,
                "layers": precedence,
                "merge_strategy": "deep_merge_with_override",
                "note": "Lower layer numbers override higher ones",
            },
            "mcp_config": {
                "note": "MCP servers go in ~/.claude.json NOT settings.json",
                "file": "~/.claude.json",
                "key": "mcpServers",
            },
            "analog": {
                "layers": "Skin layers — outer overrides inner",
                "epidermis": "Project settings (most exposed, most specific)",
                "dermis": "User settings (structural, personal)",
                "hypodermis": "System defaults (deep, stable)",
            },
        })
        .to_string(),
    )]))
}

/// Check sandbox isolation layers.
pub fn sandbox(_params: IntegumentarySandboxParams) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "sandbox_layers": [
                {
                    "layer": "process_isolation",
                    "description": "Claude Code runs in sandboxed subprocess",
                    "enforced_by": "Claude Code runtime",
                },
                {
                    "layer": "filesystem_restrictions",
                    "description": "Limited to working directory and allowed paths",
                    "enforced_by": "Permission system + hooks",
                },
                {
                    "layer": "network_restrictions",
                    "description": "internet-access-blocker hook prevents unauthorized network calls",
                    "enforced_by": "PreToolUse:Bash hook",
                },
                {
                    "layer": "command_allowlist",
                    "description": "reflex-mcp-prefer: default-deny Bash with explicit allowlist",
                    "enforced_by": "PreToolUse:Bash hook (BLOCK)",
                },
                {
                    "layer": "secret_protection",
                    "description": "secret-scanner blocks commands containing API keys/tokens",
                    "enforced_by": "PreToolUse:Bash hook (BLOCK)",
                },
            ],
            "analog": {
                "skin_barrier": "Multi-layer protection preventing unauthorized access",
                "sweat_cooling": "Resource limits preventing thermal runaway",
                "melanin": "Secret scanning — absorbs harmful radiation (exposed secrets)",
            },
        })
        .to_string(),
    )]))
}

/// Check scarring mechanisms (learned restrictions from past incidents).
pub fn scarring(params: IntegumentaryScarringParams) -> Result<CallToolResult, McpError> {
    let incident_type = params.incident_type.as_deref();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());

    // Known scars (CAPAs and learned restrictions)
    let mut scars = vec![
        json!({
            "scar_id": "CAPA-2026-0214",
            "incident": "Consolidation incident — mv destroyed data",
            "restriction": "NEVER use mv for consolidation, always cp -a + verify + rm",
            "severity": "critical",
            "date": "2026-02-14",
        }),
        json!({
            "scar_id": "SCAR-001",
            "incident": "INSERT OR REPLACE erased accumulated counters",
            "restriction": "brain-sync uses INSERT-only for patterns/beliefs/trust",
            "severity": "high",
            "date": "2026-02-16",
        }),
        json!({
            "scar_id": "SCAR-002",
            "incident": "Python file created by accident",
            "restriction": "python-creation-blocker hook (PreToolUse:Write BLOCK)",
            "severity": "medium",
            "date": "2026-02-14",
        }),
    ];

    // Load corrections as additional scars
    let corrections_file = format!("{}/.claude/implicit/corrections.json", home);
    if let Ok(data) = std::fs::read_to_string(&corrections_file) {
        if let Ok(corrections) = serde_json::from_str::<Vec<serde_json::Value>>(&data) {
            for (i, c) in corrections.iter().enumerate() {
                scars.push(json!({
                    "scar_id": format!("CORRECTION-{:03}", i + 1),
                    "incident": c.get("mistake").and_then(|m| m.as_str()).unwrap_or("unknown"),
                    "restriction": c.get("correction").and_then(|m| m.as_str()).unwrap_or("unknown"),
                    "severity": "info",
                    "applications": c.get("application_count").and_then(|a| a.as_u64()).unwrap_or(0),
                }));
            }
        }
    }

    // Filter by incident type if specified
    let filtered: Vec<&serde_json::Value> = if let Some(itype) = incident_type {
        scars
            .iter()
            .filter(|s| {
                s["incident"]
                    .as_str()
                    .is_some_and(|i| i.to_lowercase().contains(&itype.to_lowercase()))
                    || s["restriction"]
                        .as_str()
                        .is_some_and(|r| r.to_lowercase().contains(&itype.to_lowercase()))
            })
            .collect()
    } else {
        scars.iter().collect()
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "scarring": {
                "total_scars": scars.len(),
                "filtered": filtered.len(),
                "filter": incident_type,
                "scars": filtered,
            },
            "analog": {
                "scar_tissue": "Permanent restriction from past injury",
                "keloid": "Over-restrictive response (too many false positives)",
                "wound_healing": "CAPA process — investigate, restrict, prevent recurrence",
            },
        })
        .to_string(),
    )]))
}

/// Get integumentary system health overview.
pub fn health(_params: IntegumentaryHealthParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());

    let hooks_exist =
        std::path::Path::new(&format!("{}/.claude/hooks/core-hooks/target/release", home)).exists();

    let settings_exist = std::path::Path::new(&format!("{}/.claude/settings.json", home)).exists();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "integumentary_health": {
                "status": if hooks_exist && settings_exist { "operational" } else { "degraded" },
                "hooks_compiled": hooks_exist,
                "settings_present": settings_exist,
            },
            "components": {
                "epidermis": "Permission hooks (PreToolUse matchers)",
                "dermis": "Settings precedence stack",
                "hypodermis": "Guardian threat level gate",
                "sweat_glands": "Sandbox resource limits",
                "hair_follicles": "File watchers and change detection",
                "melanocytes": "Secret scanning (UV protection)",
                "scar_tissue": "CAPAs and learned restrictions",
            },
        })
        .to_string(),
    )]))
}
