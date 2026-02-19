//! Nervous System MCP tools — signal routing, reflex arcs, myelination.
//!
//! Maps Claude Code's hook-based event system to the biological nervous system:
//! - Reflex arcs: trigger→response patterns (hooks)
//! - Myelination: cached/optimized hot paths (compiled Rust hooks vs shell)
//! - Signal latency: processing chain timing
//!
//! ## T1 Primitive Grounding
//! - Reflex: →(Causality) + σ(Sequence)
//! - Myelination: ν(Frequency) + π(Persistence)
//! - Latency: N(Quantity) + ∂(Boundary)

use crate::params::nervous::{
    NervousHealthParams, NervousLatencyParams, NervousMyelinationParams, NervousReflexParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;

/// Analyze a reflex arc (trigger → response pattern).
pub fn reflex(params: NervousReflexParams) -> Result<CallToolResult, McpError> {
    let trigger = &params.trigger;
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    let hooks_dir = format!("{}/.claude/hooks/core-hooks/target/release", home);

    // Known reflex arcs (hook trigger → response patterns)
    let known_reflexes: Vec<(&str, &str, &str)> = vec![
        ("Bash", "reflex-mcp-prefer", "BLOCK: redirect to MCP tool"),
        (
            "Bash",
            "reflex-compute",
            "BLOCK: PV/stats computation via Bash",
        ),
        ("Bash", "reflex-guardian", "WARN: destructive commands"),
        ("Bash", "reflex-foundation", "WARN: foundation ops via Bash"),
        (
            "Write",
            "reflex-persist",
            "PASS: brain track on nexcore files",
        ),
        (
            "Edit",
            "reflex-persist",
            "PASS: brain track on nexcore files",
        ),
        (
            "Bash",
            "reflex-skill-suggest",
            "PASS: suggest relevant skill",
        ),
        (
            "*",
            "reflex-compound-tracker",
            "PASS: bigram/trigram tracking",
        ),
        ("Bash", "secret-scanner", "BLOCK: secrets in commands"),
        (
            "Write",
            "python-creation-blocker",
            "BLOCK: .py file creation",
        ),
    ];

    let matching: Vec<serde_json::Value> = known_reflexes
        .iter()
        .filter(|(t, _, _)| trigger.contains(t) || *t == "*")
        .map(|(t, hook, response)| {
            let binary_path = format!("{}/{}", hooks_dir, hook);
            let is_myelinated = Path::new(&binary_path).exists();
            json!({
                "trigger": t,
                "hook": hook,
                "response": response,
                "myelinated": is_myelinated,
                "latency_class": if is_myelinated { "fast (<5ms)" } else { "moderate (<50ms)" },
            })
        })
        .collect();

    let expected = params.expected_response.as_deref();
    let validated = expected.map(|exp| {
        matching
            .iter()
            .any(|m| m["response"].as_str().is_some_and(|r| r.contains(exp)))
    });

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "trigger": trigger,
            "matching_reflexes": matching,
            "reflex_count": matching.len(),
            "validation": validated.map(|v| if v { "PASS" } else { "FAIL" }),
        })
        .to_string(),
    )]))
}

/// Measure signal latency through a processing chain.
pub fn latency(params: NervousLatencyParams) -> Result<CallToolResult, McpError> {
    let chain = &params.chain;
    let latencies = params.latencies_ms.unwrap_or_default();

    let stages: Vec<serde_json::Value> = chain
        .iter()
        .enumerate()
        .map(|(i, stage)| {
            let measured = latencies.get(i).copied();
            let estimated = measured.unwrap_or_else(|| estimate_stage_latency(stage));
            json!({
                "stage": stage,
                "index": i,
                "latency_ms": estimated,
                "measured": measured.is_some(),
                "classification": classify_latency(estimated),
            })
        })
        .collect();

    let total: f64 = stages.iter().filter_map(|s| s["latency_ms"].as_f64()).sum();

    let bottleneck = stages.iter().max_by(|a, b| {
        a["latency_ms"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&b["latency_ms"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "chain": stages,
            "total_latency_ms": (total * 100.0).round() / 100.0,
            "stage_count": chain.len(),
            "bottleneck": bottleneck,
            "classification": classify_latency(total),
        })
        .to_string(),
    )]))
}

/// Check myelination status (compiled Rust hooks vs shell scripts).
pub fn myelination(params: NervousMyelinationParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    let release_dir = format!("{}/.claude/hooks/core-hooks/target/release", home);
    let shell_dir = format!("{}/.claude/hooks", home);

    let filter = params.path.as_deref();

    // Count compiled (myelinated) vs shell (unmyelinated) hooks
    let mut myelinated = 0u64;
    let mut unmyelinated = 0u64;
    let mut details = Vec::new();

    // Check compiled hooks
    if let Ok(entries) = std::fs::read_dir(&release_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.file_type().is_ok_and(|ft| ft.is_file())
                && !name.contains('.')
                && !name.starts_with("lib")
                && !name.starts_with("build")
            {
                if filter.is_none() || name.contains(filter.unwrap_or("")) {
                    myelinated += 1;
                    details
                        .push(json!({"hook": name, "myelinated": true, "type": "compiled_rust"}));
                }
            }
        }
    }

    // Check shell hooks
    if let Ok(entries) = std::fs::read_dir(&shell_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".sh") {
                let base = name.trim_end_matches(".sh");
                if filter.is_none() || base.contains(filter.unwrap_or("")) {
                    unmyelinated += 1;
                    details
                        .push(json!({"hook": base, "myelinated": false, "type": "shell_script"}));
                }
            }
        }
    }

    let total = myelinated + unmyelinated;
    let ratio = if total > 0 {
        myelinated as f64 / total as f64
    } else {
        0.0
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "myelination": {
                "compiled_rust": myelinated,
                "shell_scripts": unmyelinated,
                "total": total,
                "ratio": (ratio * 100.0).round() / 100.0,
                "classification": if ratio > 0.8 { "highly_myelinated" }
                    else if ratio > 0.5 { "moderately_myelinated" }
                    else { "poorly_myelinated" },
            },
            "filter": filter,
            "details": if details.len() <= 20 { details.clone() } else { details[..20].to_vec() },
        })
        .to_string(),
    )]))
}

/// Get nervous system health overview.
pub fn health(_params: NervousHealthParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());

    let guardian_state =
        std::fs::read_to_string(format!("{}/.claude/hooks/state/guardian-state.json", home))
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());

    let hook_log = format!("{}/.claude/hooks/state/hook_executions.jsonl", home);
    let hook_lines = std::fs::read_to_string(&hook_log)
        .map(|c| c.lines().count())
        .unwrap_or(0);

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "nervous_system": {
                "status": "operational",
                "hook_executions_logged": hook_lines,
                "guardian_active": guardian_state.is_some(),
                "guardian_iteration": guardian_state
                    .as_ref()
                    .and_then(|g| g.get("iteration").and_then(|i| i.as_u64())),
                "threat_level": guardian_state
                    .as_ref()
                    .and_then(|g| g.get("threat_level").and_then(|t| t.as_str().map(String::from))),
            },
            "analog": {
                "reflex_arcs": "PreToolUse hooks (trigger → response)",
                "myelination": "Compiled Rust hooks vs shell scripts",
                "neurotransmitters": "Cytokine events (IL, TNF, IFN, TGF, CSF)",
                "central_nervous": "Guardian homeostasis loop",
                "peripheral_nervous": "PostToolUse async hooks",
            },
        })
        .to_string(),
    )]))
}

fn estimate_stage_latency(stage: &str) -> f64 {
    match stage {
        s if s.contains("hook") => 5.0,
        s if s.contains("dispatch") => 1.0,
        s if s.contains("tool") => 15.0,
        s if s.contains("response") => 2.0,
        s if s.contains("network") => 50.0,
        s if s.contains("disk") || s.contains("io") => 10.0,
        s if s.contains("compile") || s.contains("build") => 5000.0,
        _ => 10.0,
    }
}

fn classify_latency(ms: f64) -> &'static str {
    if ms < 10.0 {
        "fast"
    } else if ms < 100.0 {
        "normal"
    } else if ms < 500.0 {
        "slow"
    } else {
        "critical"
    }
}
