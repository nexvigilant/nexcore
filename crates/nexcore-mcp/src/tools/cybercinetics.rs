//! Cybercinetics feedback controller tools
//!
//! Exposes the ∂(→(ν, ς, ρ)) typed feedback controller to MCP.
//!
//! ## Tools
//! - feedback_controller_tick: Run one tick of the feedback controller, returns verdict
//! - feedback_registry_status: Report hook-binary binding registry health

use crate::params::cybercinetics::{FeedbackControllerTickParams, FeedbackRegistryStatusParams};
use nexcore_cybercinetics::{Arrow, BindingRegistry, Controller, HookBinding, Nu, Rho, Verdict};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Run one tick of the feedback controller with provided parameters.
pub fn controller_tick(params: FeedbackControllerTickParams) -> Result<CallToolResult, McpError> {
    let mut ctrl: Controller<String> = Controller::new(
        "tick".to_string(),
        params.nu_rate,
        params.nu_floor,
        params.rho_ceiling,
        params.f_min,
    );

    // Set recursion depth
    for _ in 0..params.rho_depth {
        ctrl.observe();
    }

    // Build causal chain
    for link in &params.causal_links {
        ctrl.arrow.push(&link.cause, &link.effect, link.fidelity);
    }

    let verdict = ctrl.tick();

    let result = json!({
        "verdict": verdict.to_string(),
        "stable": verdict == Verdict::Stable,
        "nu": {
            "rate": ctrl.nu.rate,
            "floor": ctrl.nu.floor,
            "health_ratio": ctrl.nu.health_ratio(),
            "decayed": ctrl.nu.is_decayed(),
        },
        "rho": {
            "depth": ctrl.rho.depth,
            "ceiling": ctrl.rho.ceiling,
            "saturated": ctrl.rho.is_saturated(),
        },
        "arrow": {
            "hops": ctrl.arrow.len(),
            "f_total": ctrl.arrow.f_total(),
            "f_min": params.f_min,
            "below_threshold": !ctrl.arrow.is_empty() && ctrl.arrow.f_total() < params.f_min,
            "weakest": ctrl.arrow.weakest().map(|w| json!({
                "cause": w.cause,
                "effect": w.effect,
                "fidelity": w.fidelity,
            })),
        },
        "primitive_grounding": "∂(→(ν, ς, ρ))",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Report the health of the hook-binary binding registry.
///
/// Reads settings.json to discover registered hooks, builds bindings,
/// and reports aggregate fidelity and degraded bindings.
pub fn registry_status(params: FeedbackRegistryStatusParams) -> Result<CallToolResult, McpError> {
    // Read settings.json for hook registrations
    let settings_path = std::env::var("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".claude/settings.json"))
        .unwrap_or_default();

    let hooks = match std::fs::read_to_string(&settings_path) {
        Ok(content) => {
            let v: serde_json::Value = serde_json::from_str(&content).unwrap_or(json!({}));
            extract_hooks(&v)
        }
        Err(_) => Vec::new(),
    };

    let ctrl: Controller<String> = Controller::new("registry".to_string(), 1.0, 0.1, 3, 0.80);
    let mut reg = BindingRegistry::new(ctrl);

    for (hook, event) in &hooks {
        let binding = HookBinding::new(hook.as_str(), "claude-code", event.as_str());
        reg.register(binding);
    }

    let degraded: Vec<_> = reg
        .degraded_bindings(params.degraded_threshold)
        .iter()
        .map(|b| {
            json!({
                "hook": b.hook,
                "event": b.event,
                "fidelity": b.fidelity,
            })
        })
        .collect();

    let result = json!({
        "total_bindings": reg.bindings.len(),
        "aggregate_fidelity": reg.aggregate_fidelity(),
        "degraded_count": degraded.len(),
        "degraded_threshold": params.degraded_threshold,
        "degraded_bindings": degraded,
        "bindings": reg.bindings.iter().map(|b| json!({
            "hook": b.hook,
            "event": b.event,
            "fidelity": b.fidelity,
        })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Extract hook script paths and events from settings.json.
fn extract_hooks(settings: &serde_json::Value) -> Vec<(String, String)> {
    let mut hooks = Vec::new();
    if let Some(hook_map) = settings.get("hooks").and_then(|h| h.as_object()) {
        for (event, entries) in hook_map {
            if let Some(arr) = entries.as_array() {
                for entry in arr {
                    if let Some(cmd) = entry.get("command").and_then(|c| c.as_str()) {
                        hooks.push((cmd.to_string(), event.clone()));
                    }
                }
            }
        }
    }
    hooks
}
