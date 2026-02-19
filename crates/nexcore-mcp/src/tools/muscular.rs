//! Muscular System MCP tools — tool execution patterns.
//!
//! Maps Claude Code's tool usage to the muscular system:
//! - Skeletal muscle: voluntary tools (Read, Write, Edit, Bash)
//! - Smooth muscle: autonomic tools (hooks, pipelines)
//! - Cardiac muscle: event loop (guardian tick, heartbeat)
//!
//! ## T1 Primitive Grounding
//! - Execution: →(Causality) + σ(Sequence)
//! - Size Principle: N(Quantity) + κ(Comparison)
//! - Fatigue: ν(Frequency) + ∂(Boundary)

use crate::params::muscular::{
    MuscularClassifyParams, MuscularFatigueParams, MuscularHealthParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Classify a tool by muscle type following the Size Principle.
pub fn classify(params: MuscularClassifyParams) -> Result<CallToolResult, McpError> {
    let tool = params.tool.to_lowercase();

    let (muscle_type, motor_unit_size, recruitment_order) = match tool.as_str() {
        // Smallest motor units first (Size Principle)
        "read" | "glob" | "grep" => ("skeletal", "small", 1),
        "edit" => ("skeletal", "small", 2),
        "write" => ("skeletal", "medium", 3),
        "bash" => ("skeletal", "large", 4),
        "task" => ("skeletal", "large", 5),

        // Smooth muscle (autonomic)
        t if t.contains("hook") => ("smooth", "autonomic", 0),
        t if t.contains("guardian") => ("cardiac", "involuntary", 0),
        t if t.contains("reflex") => ("smooth", "autonomic", 0),

        // MCP tools by domain
        t if t.contains("foundation") || t.contains("levenshtein") => ("skeletal", "small", 1),
        t if t.contains("pv_") || t.contains("faers") => ("skeletal", "medium", 3),
        t if t.contains("wolfram") || t.contains("perplexity") => ("skeletal", "large", 5),

        _ => ("skeletal", "medium", 3),
    };

    let antagonist = match tool.as_str() {
        "read" => Some("write"),
        "write" => Some("read"),
        "edit" => Some("read"),
        "bash" => Some("mcp_tool"),
        _ => None,
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "classification": {
                "tool": params.tool,
                "muscle_type": muscle_type,
                "motor_unit_size": motor_unit_size,
                "recruitment_order": recruitment_order,
                "antagonist": antagonist,
                "size_principle_compliant": recruitment_order <= 3,
            },
            "analog": {
                "skeletal": "Voluntary tools — user-directed actions",
                "smooth": "Autonomic tools — hooks, reflexes, pipelines",
                "cardiac": "Event loop tools — guardian tick, heartbeat monitoring",
                "size_principle": "Recruit smallest motor unit sufficient for the task",
            },
        })
        .to_string(),
    )]))
}

/// Check fatigue level for the current session.
pub fn fatigue(params: MuscularFatigueParams) -> Result<CallToolResult, McpError> {
    let calls = params.total_calls;
    let consumed = params.tokens_consumed;
    let window = params.context_window;

    let utilization = if window > 0 {
        consumed as f64 / window as f64
    } else {
        0.0
    };

    let fatigue_level = if utilization > 0.9 {
        "exhausted"
    } else if utilization > 0.75 {
        "fatigued"
    } else if utilization > 0.5 {
        "moderate"
    } else {
        "fresh"
    };

    // Refractory period check (too many calls too fast)
    let needs_refractory = calls > 100 && utilization > 0.7;

    // ATP analog: remaining capacity
    let atp_remaining = window.saturating_sub(consumed);
    let atp_ratio = if window > 0 {
        atp_remaining as f64 / window as f64
    } else {
        0.0
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "fatigue": {
                "level": fatigue_level,
                "total_calls": calls,
                "tokens_consumed": consumed,
                "context_window": window,
                "utilization": (utilization * 100.0).round() / 100.0,
                "atp_remaining": atp_remaining,
                "atp_ratio": (atp_ratio * 100.0).round() / 100.0,
                "needs_refractory": needs_refractory,
            },
            "recommendations": match fatigue_level {
                "exhausted" => vec!["Context window nearly full — /compact or start new session"],
                "fatigued" => vec!["Consider compacting context", "Prioritize remaining actions"],
                "moderate" => vec!["Good endurance — continue at current pace"],
                _ => vec!["Full strength — operate freely"],
            },
        })
        .to_string(),
    )]))
}

/// Get muscular system health overview.
pub fn health(_params: MuscularHealthParams) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "muscular_health": {
                "status": "operational",
                "muscle_groups": {
                    "skeletal_voluntary": {
                        "tools": ["Read", "Write", "Edit", "Glob", "Grep", "Bash", "Task"],
                        "description": "User-directed tool invocations",
                    },
                    "smooth_autonomic": {
                        "tools": ["PreToolUse hooks", "PostToolUse hooks", "Pipelines"],
                        "description": "Automatic background processing (peristalsis)",
                    },
                    "cardiac_involuntary": {
                        "tools": ["guardian_homeostasis_tick", "session lifecycle"],
                        "description": "Continuous monitoring loop (heartbeat)",
                    },
                },
                "size_principle": {
                    "description": "Recruit smallest motor unit first: Read → Edit → Write → Bash → Task",
                    "benefit": "Minimizes resource usage per action",
                },
                "antagonistic_pairs": [
                    {"agonist": "Read", "antagonist": "Write", "joint": "File I/O"},
                    {"agonist": "Bash", "antagonist": "MCP tools", "joint": "Computation"},
                    {"agonist": "Glob", "antagonist": "Grep", "joint": "Search"},
                ],
            },
        })
        .to_string(),
    )]))
}
