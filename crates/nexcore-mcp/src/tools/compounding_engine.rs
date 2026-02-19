//! Compounding Engine MCP tools — learning pipeline velocity metrics.
//!
//! Measures compound growth across the learning loop:
//! CE→RO→AC→AE (Kolb) × SessionStart→Work→Stop (lifecycle).
//!
//! ## T1 Primitive Grounding
//! - Velocity: ν(Frequency) + N(Quantity)
//! - Loop health: σ(Sequence) + κ(Comparison)
//! - Metrics: π(Persistence) + μ(Mapping)

use crate::params::compounding_engine::{
    CompoundingLoopHealthParams, CompoundingMetricsParams, CompoundingVelocityParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;

/// Measure compounding velocity across learning dimensions.
pub fn velocity(params: CompoundingVelocityParams) -> Result<CallToolResult, McpError> {
    let window = params.window_hours.unwrap_or(24);
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());

    // Read brain.db stats
    let brain_path = format!("{}/.claude/brain/brain.db", home);
    let brain_exists = Path::new(&brain_path).exists();

    // Read hook execution stats
    let hook_log = format!("{}/.claude/hooks/state/hook_executions.jsonl", home);
    let hook_lines = count_recent_lines(&hook_log, window);

    // Read pattern stats
    let patterns_file = format!("{}/.claude/implicit/patterns.json", home);
    let pattern_count = count_json_array_items(&patterns_file);

    // Read beliefs
    let beliefs_file = format!("{}/.claude/implicit/beliefs.json", home);
    let belief_count = count_json_array_items(&beliefs_file);

    // Read corrections
    let corrections_file = format!("{}/.claude/implicit/corrections.json", home);
    let correction_count = count_json_array_items(&corrections_file);

    // Compound velocity = weighted sum of growth rates
    let velocity_score = (hook_lines as f64 * 0.1
        + pattern_count as f64 * 0.3
        + belief_count as f64 * 0.4
        + correction_count as f64 * 0.2)
        / window.max(1) as f64;

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "window_hours": window,
            "velocity_score": (velocity_score * 100.0).round() / 100.0,
            "dimensions": {
                "hook_executions": hook_lines,
                "patterns_detected": pattern_count,
                "beliefs_formed": belief_count,
                "corrections_learned": correction_count,
                "brain_db_active": brain_exists,
            },
            "growth_rate": {
                "per_hour": (velocity_score * 100.0).round() / 100.0,
                "classification": if velocity_score > 5.0 { "accelerating" }
                    else if velocity_score > 1.0 { "steady" }
                    else if velocity_score > 0.1 { "slow" }
                    else { "stagnant" },
            },
        })
        .to_string(),
    )]))
}

/// Check Kolb learning loop health (CE→RO→AC→AE phases).
pub fn loop_health(_params: CompoundingLoopHealthParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());

    // CE (Concrete Experience) — tool usage count
    let hook_log = format!("{}/.claude/hooks/state/hook_executions.jsonl", home);
    let ce_signal = count_recent_lines(&hook_log, 24);

    // RO (Reflective Observation) — patterns detected
    let patterns_file = format!("{}/.claude/implicit/patterns.json", home);
    let ro_signal = count_json_array_items(&patterns_file);

    // AC (Abstract Conceptualization) — beliefs formed
    let beliefs_file = format!("{}/.claude/implicit/beliefs.json", home);
    let ac_signal = count_json_array_items(&beliefs_file);

    // AE (Active Experimentation) — corrections applied
    let corrections_file = format!("{}/.claude/implicit/corrections.json", home);
    let ae_signal = count_json_array_items(&corrections_file);

    let total = (ce_signal + ro_signal + ac_signal + ae_signal).max(1) as f64;
    let balance = [
        ("CE", ce_signal as f64 / total),
        ("RO", ro_signal as f64 / total),
        ("AC", ac_signal as f64 / total),
        ("AE", ae_signal as f64 / total),
    ];

    // Ideal balance is roughly equal across phases
    let min_phase = balance.iter().map(|(_, v)| *v).fold(f64::MAX, f64::min);
    let max_phase = balance.iter().map(|(_, v)| *v).fold(f64::MIN, f64::max);
    let imbalance = max_phase - min_phase;

    let weakest = balance
        .iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(name, _)| *name)
        .unwrap_or("unknown");

    let diagnosis = match weakest {
        "CE" => "Insufficient experience — need more tool usage and experimentation",
        "RO" => "Insufficient reflection — patterns not being extracted from experience",
        "AC" => "Insufficient theorizing — observations not forming into beliefs",
        "AE" => "Insufficient action — beliefs not being tested through corrections",
        _ => "Unknown imbalance",
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "kolb_cycle": {
                "CE_concrete_experience": { "count": ce_signal, "ratio": (balance[0].1 * 100.0).round() / 100.0 },
                "RO_reflective_observation": { "count": ro_signal, "ratio": (balance[1].1 * 100.0).round() / 100.0 },
                "AC_abstract_conceptualization": { "count": ac_signal, "ratio": (balance[2].1 * 100.0).round() / 100.0 },
                "AE_active_experimentation": { "count": ae_signal, "ratio": (balance[3].1 * 100.0).round() / 100.0 },
            },
            "balance": {
                "imbalance_score": (imbalance * 100.0).round() / 100.0,
                "weakest_phase": weakest,
                "diagnosis": diagnosis,
                "health": if imbalance < 0.15 { "balanced" }
                    else if imbalance < 0.3 { "moderate_imbalance" }
                    else { "severe_imbalance" },
            },
        })
        .to_string(),
    )]))
}

/// Get compounding metrics summary.
pub fn metrics(params: CompoundingMetricsParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    let include_trend = params.include_trend.unwrap_or(false);

    let daemon_state_path = format!("{}/.claude/hooks/state/learning-daemon-state.json", home);
    let daemon_state = std::fs::read_to_string(&daemon_state_path).ok();

    let trend_path = format!("{}/.claude/hooks/state/trend-analysis.json", home);
    let trend_data = if include_trend {
        std::fs::read_to_string(&trend_path).ok()
    } else {
        None
    };

    let patterns_file = format!("{}/.claude/implicit/patterns.json", home);
    let beliefs_file = format!("{}/.claude/implicit/beliefs.json", home);

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "compounding_infrastructure": {
                "learning_daemon": daemon_state.is_some(),
                "patterns_file": Path::new(&patterns_file).exists(),
                "beliefs_file": Path::new(&beliefs_file).exists(),
                "trend_analysis": Path::new(&trend_path).exists(),
            },
            "counts": {
                "patterns": count_json_array_items(&patterns_file),
                "beliefs": count_json_array_items(&beliefs_file),
            },
            "daemon_state": daemon_state.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
            "trend": trend_data.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
        })
        .to_string(),
    )]))
}

fn count_recent_lines(path: &str, _hours: u64) -> u64 {
    // Count total lines as a proxy (proper time filtering would need parsing)
    std::fs::read_to_string(path)
        .map(|c| c.lines().count() as u64)
        .unwrap_or(0)
}

fn count_json_array_items(path: &str) -> u64 {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
        .and_then(|v| v.as_array().map(|a| a.len() as u64))
        .unwrap_or(0)
}
