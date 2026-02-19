//! Model Delegation MCP tools — task→model routing with scoring.
//!
//! Routes tasks to optimal AI models based on complexity, error tolerance,
//! latency requirements, and token budget.
//!
//! ## T1 Primitive Grounding
//! - Routing: μ(Mapping) + κ(Comparison)
//! - Scoring: N(Quantity) + σ(Sequence)
//! - Selection: Σ(Sum) + ∂(Boundary)

use crate::params::model_delegation::{ModelCompareParams, ModelListParams, ModelRouteParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

struct ModelProfile {
    id: &'static str,
    name: &'static str,
    strengths: &'static [&'static str],
    max_complexity: u8,  // 1-5
    error_tolerance: u8, // 1=none, 5=high
    latency_class: u8,   // 1=real-time, 3=batch
    cost_per_mtok: f64,  // relative cost
    context_window: u64, // tokens
}

const MODELS: &[ModelProfile] = &[
    ModelProfile {
        id: "claude-opus-4-6",
        name: "Opus 4.6",
        strengths: &[
            "complex reasoning",
            "architecture",
            "multi-step",
            "code generation",
            "analysis",
        ],
        max_complexity: 5,
        error_tolerance: 1,
        latency_class: 3,
        cost_per_mtok: 75.0,
        context_window: 200000,
    },
    ModelProfile {
        id: "claude-sonnet-4-5-20250929",
        name: "Sonnet 4.5",
        strengths: &["balanced", "code generation", "reasoning", "fast iteration"],
        max_complexity: 4,
        error_tolerance: 2,
        latency_class: 2,
        cost_per_mtok: 15.0,
        context_window: 200000,
    },
    ModelProfile {
        id: "claude-haiku-4-5-20251001",
        name: "Haiku 4.5",
        strengths: &[
            "quick tasks",
            "classification",
            "extraction",
            "simple transforms",
        ],
        max_complexity: 2,
        error_tolerance: 3,
        latency_class: 1,
        cost_per_mtok: 1.25,
        context_window: 200000,
    },
    ModelProfile {
        id: "gemini-2.5-pro",
        name: "Gemini 2.5 Pro",
        strengths: &[
            "long context",
            "multimodal",
            "search integration",
            "reasoning",
        ],
        max_complexity: 4,
        error_tolerance: 2,
        latency_class: 2,
        cost_per_mtok: 10.0,
        context_window: 1000000,
    },
    ModelProfile {
        id: "gemini-2.5-flash",
        name: "Gemini 2.5 Flash",
        strengths: &["fast", "cost-effective", "simple tasks", "bulk processing"],
        max_complexity: 2,
        error_tolerance: 4,
        latency_class: 1,
        cost_per_mtok: 0.50,
        context_window: 1000000,
    },
];

/// Route a task to the optimal model.
pub fn route(params: ModelRouteParams) -> Result<CallToolResult, McpError> {
    let complexity = parse_complexity(params.complexity.as_deref().unwrap_or("moderate"));
    let error_tol = parse_error_tolerance(params.error_tolerance.as_deref().unwrap_or("low"));
    let latency = parse_latency(params.latency.as_deref().unwrap_or("interactive"));
    let budget = params.token_budget.unwrap_or(50000);

    let mut scored: Vec<(f64, &ModelProfile)> = MODELS
        .iter()
        .filter(|m| m.context_window >= budget)
        .map(|m| {
            let mut score = 0.0f64;

            // Complexity fit (40% weight)
            if m.max_complexity >= complexity {
                score += 0.4 * (1.0 - (m.max_complexity - complexity) as f64 * 0.1);
            }

            // Error tolerance fit (25% weight) — lower tolerance model preferred for low-tolerance tasks
            let tol_diff = (m.error_tolerance as i8 - error_tol as i8).unsigned_abs() as f64;
            score += 0.25 * (1.0 - tol_diff * 0.2).max(0.0);

            // Latency fit (20% weight)
            if m.latency_class <= latency {
                score += 0.2;
            } else {
                score += 0.2 * (1.0 - (m.latency_class - latency) as f64 * 0.3).max(0.0);
            }

            // Cost efficiency (15% weight) — prefer cheaper for simple tasks
            let cost_score = if complexity <= 2 {
                1.0 / (1.0 + m.cost_per_mtok / 10.0)
            } else {
                0.5 // Cost matters less for complex tasks
            };
            score += 0.15 * cost_score;

            (score, m)
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let recommendations: Vec<serde_json::Value> = scored
        .iter()
        .map(|(score, m)| {
            json!({
                "model_id": m.id,
                "name": m.name,
                "fit_score": (*score * 100.0).round() / 100.0,
                "strengths": m.strengths,
                "cost_per_mtok": m.cost_per_mtok,
                "context_window": m.context_window,
            })
        })
        .collect();

    let best = scored
        .first()
        .map(|(_, m)| m.id)
        .unwrap_or("claude-sonnet-4-5-20250929");

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "task": params.task,
            "recommended_model": best,
            "task_profile": {
                "complexity": complexity,
                "error_tolerance": error_tol,
                "latency_class": latency,
                "token_budget": budget,
            },
            "rankings": recommendations,
        })
        .to_string(),
    )]))
}

/// Compare models for a specific task.
pub fn compare(params: ModelCompareParams) -> Result<CallToolResult, McpError> {
    let filter: Option<Vec<&str>> = params
        .models
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect());

    let models: Vec<serde_json::Value> = MODELS
        .iter()
        .filter(|m| {
            filter.as_ref().map_or(true, |f| {
                f.iter().any(|&id| m.id.contains(id) || m.name.contains(id))
            })
        })
        .map(|m| {
            json!({
                "model_id": m.id,
                "name": m.name,
                "strengths": m.strengths,
                "max_complexity": m.max_complexity,
                "error_tolerance": m.error_tolerance,
                "latency_class": m.latency_class,
                "cost_per_mtok": m.cost_per_mtok,
                "context_window": m.context_window,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "task": params.task,
            "models": models,
            "comparison_note": "Lower latency_class = faster. Higher max_complexity = handles harder tasks.",
        })
        .to_string(),
    )]))
}

/// List all available models.
pub fn list(_params: ModelListParams) -> Result<CallToolResult, McpError> {
    let models: Vec<serde_json::Value> = MODELS
        .iter()
        .map(|m| {
            json!({
                "model_id": m.id,
                "name": m.name,
                "strengths": m.strengths,
                "max_complexity": m.max_complexity,
                "cost_per_mtok": m.cost_per_mtok,
                "context_window": m.context_window,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({ "models": models, "count": models.len() }).to_string(),
    )]))
}

fn parse_complexity(s: &str) -> u8 {
    match s {
        "trivial" => 1,
        "simple" => 2,
        "moderate" => 3,
        "complex" => 4,
        "expert" => 5,
        _ => 3,
    }
}

fn parse_error_tolerance(s: &str) -> u8 {
    match s {
        "none" => 1,
        "low" => 2,
        "medium" => 3,
        "high" => 4,
        _ => 2,
    }
}

fn parse_latency(s: &str) -> u8 {
    match s {
        "real-time" => 1,
        "interactive" => 2,
        "batch" => 3,
        _ => 2,
    }
}
