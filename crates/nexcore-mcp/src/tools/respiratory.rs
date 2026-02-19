//! Respiratory System MCP tools — context window management, gas exchange.
//!
//! Maps Claude Code's context window to the respiratory system:
//! - Inhalation: loading context (files, tool results, user prompts)
//! - Gas exchange: extracting useful information from context
//! - Dead space: wasted tokens (irrelevant context, stale data)
//! - Exhalation: output generation (responses, tool calls)
//!
//! ## T1 Primitive Grounding
//! - Exchange: μ(Mapping) + κ(Comparison)
//! - Dead space: ∅(Void) + N(Quantity)
//! - Tidal volume: ν(Frequency) + ∂(Boundary)

use crate::params::respiratory::{
    RespiratoryDeadSpaceParams, RespiratoryExchangeParams, RespiratoryHealthParams,
    RespiratoryTidalParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Analyze gas exchange (useful info extracted from context).
pub fn exchange(params: RespiratoryExchangeParams) -> Result<CallToolResult, McpError> {
    let inhaled = params.inhaled_tokens;
    let extracted = params.extracted_tokens.unwrap_or(inhaled / 4); // Default: 25% extraction

    let exchange_ratio = if inhaled > 0 {
        extracted as f64 / inhaled as f64
    } else {
        0.0
    };

    let wasted = inhaled.saturating_sub(extracted);

    let efficiency = if exchange_ratio > 0.5 {
        "excellent"
    } else if exchange_ratio > 0.25 {
        "normal"
    } else if exchange_ratio > 0.1 {
        "impaired"
    } else {
        "critical"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "gas_exchange": {
                "inhaled_tokens": inhaled,
                "extracted_useful": extracted,
                "wasted_tokens": wasted,
                "exchange_ratio": (exchange_ratio * 100.0).round() / 100.0,
                "efficiency": efficiency,
            },
            "analog": {
                "O2_in": "Useful information absorbed from context",
                "CO2_out": "Irrelevant/stale tokens discarded",
                "exchange_ratio": "Information extraction efficiency",
            },
            "recommendations": match efficiency {
                "critical" => vec!["Context is mostly waste — reduce irrelevant file reads", "Focus context on task-relevant files"],
                "impaired" => vec!["Improve context targeting — read specific sections, not whole files", "Use Grep before Read"],
                _ => vec!["Context exchange operating normally"],
            },
        })
        .to_string(),
    )]))
}

/// Detect dead space in context (wasted tokens).
pub fn dead_space(params: RespiratoryDeadSpaceParams) -> Result<CallToolResult, McpError> {
    let context_size = params.context_size;
    let active = params.active_tokens.unwrap_or(context_size * 3 / 4);

    let dead_space = context_size.saturating_sub(active);
    let dead_ratio = if context_size > 0 {
        dead_space as f64 / context_size as f64
    } else {
        0.0
    };

    // Biological reference: ~30% dead space is normal (anatomical dead space)
    let classification = if dead_ratio < 0.15 {
        "minimal"
    } else if dead_ratio < 0.30 {
        "normal"
    } else if dead_ratio < 0.50 {
        "elevated"
    } else {
        "pathological"
    };

    // Vital capacity: max usable context
    let vital_capacity = context_size.saturating_sub(dead_space);

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "dead_space": {
                "total_context": context_size,
                "active_tokens": active,
                "dead_space_tokens": dead_space,
                "dead_ratio": (dead_ratio * 100.0).round() / 100.0,
                "classification": classification,
            },
            "vital_capacity": vital_capacity,
            "analog": {
                "anatomical_dead_space": "System prompts, CLAUDE.md, fixed overhead (~15-20%)",
                "alveolar_dead_space": "Stale context from compaction, irrelevant prior turns",
                "vital_capacity": "Maximum usable context for productive work",
            },
            "recommendations": if dead_ratio > 0.4 {
                vec!["High dead space — consider /compact to reclaim context", "Reduce system prompt size if possible"]
            } else {
                vec!["Dead space within normal limits"]
            },
        })
        .to_string(),
    )]))
}

/// Measure tidal volume (per-turn context usage).
pub fn tidal_volume(params: RespiratoryTidalParams) -> Result<CallToolResult, McpError> {
    let turns = &params.tokens_per_turn;

    if turns.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "No turn data provided"}).to_string(),
        )]));
    }

    let total: u64 = turns.iter().sum();
    let count = turns.len() as f64;
    let avg = total as f64 / count;
    let max = turns.iter().copied().max().unwrap_or(0);
    let min = turns.iter().copied().min().unwrap_or(0);

    // Variance
    let variance = turns.iter().map(|&t| (t as f64 - avg).powi(2)).sum::<f64>() / count;
    let std_dev = variance.sqrt();

    // Breathing rate analog: consistency of turn sizes
    let regularity = if std_dev / avg.max(1.0) < 0.3 {
        "regular"
    } else if std_dev / avg.max(1.0) < 0.7 {
        "variable"
    } else {
        "irregular"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "tidal_volume": {
                "mean_tokens_per_turn": (avg * 10.0).round() / 10.0,
                "max_tokens": max,
                "min_tokens": min,
                "std_dev": (std_dev * 10.0).round() / 10.0,
                "total_tokens": total,
                "turn_count": turns.len(),
            },
            "breathing_pattern": {
                "regularity": regularity,
                "coefficient_of_variation": ((std_dev / avg.max(1.0)) * 100.0).round() / 100.0,
            },
            "analog": {
                "tidal_volume": "Tokens processed per breathing cycle (turn)",
                "minute_ventilation": format!("{} tokens across {} turns", total, turns.len()),
                "regularity": "Consistent turn sizes = efficient processing",
            },
        })
        .to_string(),
    )]))
}

/// Get respiratory system health overview.
pub fn health(_params: RespiratoryHealthParams) -> Result<CallToolResult, McpError> {
    let context_window = 200_000u64; // Claude default
    let system_overhead_est = 30_000u64; // CLAUDE.md + system prompts + skills

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "respiratory_health": {
                "status": "operational",
                "context_window": context_window,
                "estimated_system_overhead": system_overhead_est,
                "estimated_vital_capacity": context_window - system_overhead_est,
            },
            "components": {
                "trachea": "Message transport (user ↔ model)",
                "alveoli": "Context slots where information exchange occurs",
                "diaphragm": "Compaction mechanism (/compact)",
                "dead_space": "Fixed system prompts, stale context",
                "surfactant": "Context summarization (reduces surface tension)",
            },
            "rates": {
                "breathing_rate": "Turns per session",
                "tidal_volume": "Tokens per turn",
                "vital_capacity": "Max usable context per session",
            },
        })
        .to_string(),
    )]))
}
