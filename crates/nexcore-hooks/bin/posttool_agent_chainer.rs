//! # Agent Chainer Hook
//!
//! Chains agent completions to trigger follow-up agents automatically.
//!
//! ## Event
//! - **Primary**: `PostToolUse` (matcher: `Task`) - Runs in main agent context
//! - **Fallback**: `SubagentStop` (matcher: `rust-.*`) - For subagent completions
//!
//! ## Chain Rules
//! - `rust-toolchain` (with errors) → `rust-compiler-doctor`
//! - `rust-compiler-doctor` → `rust-test-architect`
//! - `rust-migrator` → `rust-reviewer`
//! - `rust-reviewer` → `rust-test-architect`
//!
//! ## Exit Codes
//! - `0` (allow): No chain triggered or chain message injected
//! - `1` (warn): SubagentStop chain triggered (stderr message)
//!
//! ## Integration
//! Uses `nexcore_hooks::agent_triggers::get_next_agent()` for chain lookup.

use nexcore_hooks::agent_triggers::get_next_agent;
use nexcore_hooks::protocol::HookOutput;
use nexcore_hooks::{exit_success_auto, read_input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const FORGE_MAX_ITERATIONS: u32 = 5;

/// Minimal ForgeState for cycle tracking (mirrors prompt_forge_activator)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ForgeState {
    active: bool,
    paused: bool,
    goal: Option<String>,
    cycle_count: u32,
    primitives_mined: Vec<String>,
    artifacts_generated: Vec<String>,
    last_verification: String,
    last_activity: Option<String>,
}

impl ForgeState {
    fn state_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".claude")
            .join("brain")
            .join("forge")
            .join("state.json")
    }

    fn load() -> Self {
        let path = Self::state_path();
        if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::state_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Determine the agent type from either:
    // 1. PostToolUse:Task → tool_input.subagent_type
    // 2. SubagentStop → agent_type
    let agent_type = if input.tool_name.as_deref() == Some("Task") {
        // PostToolUse:Task - get subagent_type from tool_input
        input
            .tool_input
            .as_ref()
            .and_then(|ti| ti.get("subagent_type"))
            .and_then(|v| v.as_str())
            .map(String::from)
    } else {
        // SubagentStop - use agent_type directly
        input.agent_type.clone()
    };

    let agent_type = match agent_type {
        Some(name) => name,
        None => exit_success_auto(), // Not a relevant event
    };

    // Check if it's a chained agent (rust-* or forge-*)
    if !agent_type.starts_with("rust-") && !agent_type.starts_with("forge-") {
        exit_success_auto();
    }

    // Check tool response for errors (for conditional chaining)
    let has_errors = input
        .tool_response
        .as_ref()
        .map(|r| {
            // Check various places where error info might be
            let output = r.get("output").and_then(|v| v.as_str()).unwrap_or("");
            let result = r.get("result").and_then(|v| v.as_str()).unwrap_or("");
            let combined = format!("{} {}", output, result);
            combined.contains("error") || combined.contains("Error") || combined.contains("ERROR")
        })
        .unwrap_or(false);

    // For forge agents: track cycles and enforce max iterations
    if agent_type.starts_with("forge-") {
        let mut state = ForgeState::load();

        // Increment cycle on validator completion (one full loop = one cycle)
        if agent_type == "forge-validator" {
            state.cycle_count += 1;
            state.last_verification = if has_errors {
                "needs_refinement".to_string()
            } else {
                "pass".to_string()
            };
            state.last_activity = Some(chrono::Utc::now().to_rfc3339());
            if let Err(e) = state.save() {
                eprintln!("Warning: Could not save FORGE state: {e}");
            }
        }

        // Max iteration guard: stop looping after FORGE_MAX_ITERATIONS
        if state.cycle_count >= FORGE_MAX_ITERATIONS && has_errors {
            state.active = false;
            state.last_verification = format!("stopped_at_max_{FORGE_MAX_ITERATIONS}");
            if let Err(e) = state.save() {
                eprintln!("Warning: Could not save FORGE state: {e}");
            }

            let context = format!(
                r#"
⛔ FORGE MAX ITERATIONS REACHED ({FORGE_MAX_ITERATIONS})
═══════════════════════════════════════════════════════════
Goal: {}
Cycles completed: {}
Last verdict: needs_refinement

The refinement loop has been stopped. Review the best
iteration in ~/.claude/forge-sessions/ and refine manually.
═══════════════════════════════════════════════════════════"#,
                state.goal.as_deref().unwrap_or("(none)"),
                state.cycle_count,
            );

            let output = HookOutput {
                decision: None,
                reason: Some(context),
                ..Default::default()
            };
            output.emit();
            std::process::exit(1);
        }
    }

    // Get the next agent in the chain
    if let Some(chain) = get_next_agent(&agent_type, has_errors) {
        let context = format!(
            r#"
🔗 AGENT CHAIN TRIGGERED ──────────────────────────────────
   Completed: {}
   Next: {}
   Condition: {}

   ⚡ You MUST invoke the next agent now.

   Use Task tool with:
     subagent_type: "{}"
     prompt: "Continue from {} - apply findings and verify"
───────────────────────────────────────────────────────────"#,
            agent_type, chain.next, chain.condition, chain.next, agent_type
        );

        // For PostToolUse, use JSON output with additionalContext
        // This injects the message into the main agent's context
        if input.tool_name.as_deref() == Some("Task") {
            let output = HookOutput {
                decision: Some(nexcore_hooks::protocol::HookDecision::Block),
                reason: Some(context),
                ..Default::default()
            };
            output.emit();
            std::process::exit(0);
        } else {
            // For SubagentStop, use stderr (less reliable)
            eprintln!("{}", context);
            std::process::exit(1);
        }
    }

    exit_success_auto();
}
