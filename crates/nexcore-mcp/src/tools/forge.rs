//! Forge MCP tools: Primitive-first technology construction
//!
//! The Forge is an autonomous technology constructor that mines primitives,
//! generates grounded Rust code, and validates through compiler gates.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Forge Loop | Recursion | ρ |
//! | Primitive Extraction | Mapping | μ |
//! | Code Generation | Causality | → |
//! | Validation Gate | Boundary | ∂ |
//! | Technology Output | Sum | Σ |

use crate::params::{
    ForgeInitParams, ForgeMineParams, ForgePromptParams, ForgeReferenceParams, ForgeSuggestParams,
    ForgeSummaryParams,
};
use nexcore_forge_strategy::ForgeDecision;
use nexcore_vigil::llm::forge_harness::{ForgeHarness, ForgeTask, LEX_PRIMITIVA, Tier};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::sync::Mutex;

// Session state (thread-safe singleton for MCP)
static FORGE_SESSION: Mutex<Option<ForgeHarness>> = Mutex::new(None);

// ============================================================================
// Tool Implementations
// ============================================================================

/// Initialize a new Forge session
pub fn forge_init(params: ForgeInitParams) -> Result<CallToolResult, McpError> {
    let session_id = params
        .session_id
        .unwrap_or_else(|| format!("forge-{}", chrono::Utc::now().timestamp()));

    let harness = ForgeHarness::new(&session_id);

    let mut guard = FORGE_SESSION.lock().map_err(|e| {
        McpError::internal_error(format!("Failed to acquire forge lock: {}", e), None)
    })?;
    *guard = Some(harness);

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "status": "initialized",
            "session_id": session_id,
            "primitives_available": 16,
            "tier_system": {
                "T1": { "primitives": 1, "transfer": 1.0 },
                "T2-P": { "primitives": "2-3", "transfer": 0.9 },
                "T2-C": { "primitives": "4-5", "transfer": 0.7 },
                "T3": { "primitives": "6+", "transfer": 0.4 }
            }
        })
        .to_string(),
    )]))
}

/// Get the primitive reference card
pub fn forge_reference(_params: ForgeReferenceParams) -> Result<CallToolResult, McpError> {
    let reference = ForgeHarness::primitive_reference();
    let primitives: Vec<_> = LEX_PRIMITIVA
        .iter()
        .map(|(sym, name, desc)| {
            json!({
                "symbol": sym,
                "name": name,
                "description": desc
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "count": 16,
            "primitives": primitives,
            "formatted": reference
        })
        .to_string(),
    )]))
}

/// Mine primitives from a concept
pub fn forge_mine(params: ForgeMineParams) -> Result<CallToolResult, McpError> {
    let mut guard = FORGE_SESSION.lock().map_err(|e| {
        McpError::internal_error(format!("Failed to acquire forge lock: {}", e), None)
    })?;

    let harness = guard.as_mut().ok_or_else(|| {
        McpError::invalid_request(
            "No forge session active. Call forge_init first.".to_string(),
            None,
        )
    })?;

    // Convert primitives to &str slice
    let primitives: Vec<&str> = params.primitives.iter().map(String::as_str).collect();
    let extraction = harness.mine(&params.concept, primitives, &params.decomposition);

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "concept": extraction.concept,
            "primitives": extraction.primitives,
            "tier": extraction.tier.code(),
            "transfer_confidence": extraction.transfer_confidence,
            "decomposition": extraction.decomposition
        })
        .to_string(),
    )]))
}

/// Generate forge prompt for a task (now strategy-aware via session harness)
pub fn forge_prompt(params: ForgePromptParams) -> Result<CallToolResult, McpError> {
    let task = ForgeTask {
        name: params.name.clone(),
        description: params.description.clone(),
        domain: params.domain.unwrap_or_else(|| "general".to_string()),
        target_tier: params.target_tier.map(|t| match t.as_str() {
            "T1" => Tier::T1,
            "T2-P" => Tier::T2P,
            "T2-C" => Tier::T2C,
            "T3" => Tier::T3,
            _ => Tier::T2P, // Default to T2-P for best transfer
        }),
    };

    // Use the session harness if available (includes evolved strategy params);
    // otherwise create a temporary one with default evolved strategy.
    let guard = FORGE_SESSION.lock().map_err(|e| {
        McpError::internal_error(format!("Failed to acquire forge lock: {}", e), None)
    })?;
    let temp_harness;
    let harness = match guard.as_ref() {
        Some(h) => h,
        None => {
            temp_harness = ForgeHarness::new("forge-temp");
            &temp_harness
        }
    };

    let prompt = harness.forge_prompt(&task);

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "task": {
                "name": task.name,
                "description": task.description,
                "domain": task.domain,
                "target_tier": task.target_tier.map(|t| t.code())
            },
            "prompt": prompt,
            "strategy": format!("{}", harness.strategy)
        })
        .to_string(),
    )]))
}

/// Get session summary
pub fn forge_summary(_params: ForgeSummaryParams) -> Result<CallToolResult, McpError> {
    let guard = FORGE_SESSION.lock().map_err(|e| {
        McpError::internal_error(format!("Failed to acquire forge lock: {}", e), None)
    })?;

    let harness = guard.as_ref().ok_or_else(|| {
        McpError::invalid_request(
            "No forge session active. Call forge_init first.".to_string(),
            None,
        )
    })?;

    let summary = harness.summary();
    let mined: Vec<_> = harness
        .mined_primitives
        .iter()
        .map(|ext| {
            json!({
                "concept": ext.concept,
                "primitives": ext.primitives,
                "tier": ext.tier.code(),
                "transfer_confidence": ext.transfer_confidence
            })
        })
        .collect();

    let code_generated: Vec<_> = harness.generated_code.keys().cloned().collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "session_id": harness.session_id,
            "primitives_mined": mined,
            "code_generated": code_generated,
            "formatted": summary
        })
        .to_string(),
    )]))
}

/// Get the Gemini system prompt for Forge mode
pub fn forge_system_prompt() -> Result<CallToolResult, McpError> {
    let prompt = nexcore_vigil::llm::forge_harness::gemini_forge_system_prompt();
    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "system_prompt": prompt,
            "protocol": ["MINE", "DECOMPOSE", "GENERATE", "VALIDATE", "REFINE"],
            "action_format": {
                "mine": "[ACTION: forge_mine]{...}[/ACTION]",
                "generate": "[ACTION: forge_generate]{...}[/ACTION]",
                "validate": "[ACTION: forge_validate]{...}[/ACTION]",
                "shell": "[ACTION: shell]cargo check[/ACTION]"
            }
        })
        .to_string(),
    )]))
}

/// Suggest the next forge action based on evolved strategy parameters.
///
/// Maps current forge state to a decision using parameters evolved through
/// 12,000 simulated games in the Primitive Depths genetic algorithm.
pub fn forge_suggest(params: ForgeSuggestParams) -> Result<CallToolResult, McpError> {
    let guard = FORGE_SESSION.lock().map_err(|e| {
        McpError::internal_error(format!("Failed to acquire forge lock: {}", e), None)
    })?;

    let harness = guard.as_ref().ok_or_else(|| {
        McpError::invalid_request(
            "No forge session active. Call forge_init first.".to_string(),
            None,
        )
    })?;

    let decision = harness.suggest_action(
        params.blocker_count.unwrap_or(0),
        params.warning_count.unwrap_or(0),
        params.primitives_available.unwrap_or(0),
        params.confidence.unwrap_or(0.9),
    );

    let guidance = match decision {
        ForgeDecision::Abandon => "Confidence too low — abandon this generation and restart",
        ForgeDecision::FixBlocker => "Fix blocking compiler errors immediately (easiest first)",
        ForgeDecision::Refactor => "Quality below floor — refactor to 84% before resuming",
        ForgeDecision::LintFix => "Fix clippy warnings in adjacent code",
        ForgeDecision::Decompose => "Mine and decompose available primitives",
        ForgeDecision::Promote => "Current tier complete — promote to next tier",
        ForgeDecision::Explore => "Try alternative decompositions or approaches",
        ForgeDecision::Stuck => "No progress possible — needs external intervention",
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "decision": format!("{decision}"),
            "guidance": guidance,
            "strategy": format!("{}", harness.strategy),
            "state": {
                "blockers": params.blocker_count.unwrap_or(0),
                "warnings": params.warning_count.unwrap_or(0),
                "primitives_available": params.primitives_available.unwrap_or(0),
                "confidence": params.confidence.unwrap_or(0.9)
            }
        })
        .to_string(),
    )]))
}

/// Classify a tier from primitive count
pub fn forge_tier_classify(count: usize) -> Result<CallToolResult, McpError> {
    let tier = Tier::from_count(count);
    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "count": count,
            "tier": tier.code(),
            "transfer_confidence": tier.transfer_confidence(),
            "classification_rules": {
                "T1": "1 primitive → 100% transfer",
                "T2-P": "2-3 primitives → 90% transfer",
                "T2-C": "4-5 primitives → 70% transfer",
                "T3": "6+ primitives → 40% transfer"
            }
        })
        .to_string(),
    )]))
}
