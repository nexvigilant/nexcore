//! SessionStart hook: Auto Bond Executor
//!
//! Automatically executes low-energy pending bonds on session start.
//! Creates zero-friction continuous improvement - system improves every session.
//!
//! Process:
//! 1. Read ~/.claude/bonds/pending_actions.json
//! 2. Filter bonds with activation_energy ≤ 20 and status == "Pending"
//! 3. Output instructions for Claude to execute
//!
//! ToV Alignment:
//! - Feedback Loop (ℱ): Closes improvement cycle automatically
//! - Self-Improvement: System evolves without manual intervention
//!
//! Exit codes:
//! - 0: Success (context injected or skipped if no bonds)

use nexcore_hooks::{exit_skip_session, exit_with_session_context, read_input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Maximum activation energy for auto-execution
const AUTO_EXECUTE_THRESHOLD: u32 = 20;

/// Maximum bonds to auto-execute per session (prevent overload)
const MAX_BONDS_PER_SESSION: usize = 3;

#[derive(Debug, Deserialize, Serialize)]
struct PendingActions {
    version: String,
    last_updated: u64,
    actions: Vec<Bond>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Bond {
    bond_id: String,
    created_at: u64,
    bond_type: String,
    catalyst: String,
    activation_energy: u32,
    cause: BondCause,
    effect: BondEffect,
    context_transfer: Vec<String>,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    completed_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct BondCause {
    capability_type: String,
    target: String,
    description: String,
    path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct BondEffect {
    improvement: String,
    description: String,
    quick_action: String,
    mcp_tool: Option<String>,
    mcp_args: Option<serde_json::Value>,
}

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_skip_session(),
    };

    let pending = match load_pending_actions() {
        Some(p) => p,
        None => exit_skip_session(),
    };

    // Filter: low energy + pending status
    let auto_bonds: Vec<&Bond> = pending
        .actions
        .iter()
        .filter(|b| b.activation_energy <= AUTO_EXECUTE_THRESHOLD && b.status == "Pending")
        .take(MAX_BONDS_PER_SESSION)
        .collect();

    if auto_bonds.is_empty() {
        exit_skip_session();
    }

    // Build execution context
    let mut context = String::from("🔄 **AUTO-BOND EXECUTOR** ─────────────────────────────────\n");
    context.push_str(&format!(
        "   {} low-energy bond(s) ready for auto-execution:\n\n",
        auto_bonds.len()
    ));

    for (i, bond) in auto_bonds.iter().enumerate() {
        context.push_str(&format!(
            "   {}. **{}** (energy: {})\n",
            i + 1,
            bond.bond_id,
            bond.activation_energy
        ));
        context.push_str(&format!(
            "      Target: {} ({})\n",
            bond.cause.target, bond.cause.capability_type
        ));
        context.push_str(&format!("      Action: {}\n", bond.effect.quick_action));

        if let Some(ref tool) = bond.effect.mcp_tool {
            context.push_str(&format!("      MCP Tool: {}\n", tool));
        }
        context.push('\n');
    }

    context.push_str("   ⚡ **EXECUTE NOW**: Complete these bonds before user prompt.\n");
    context.push_str("   After each: Update pending_actions.json status → \"Completed\"\n");
    context.push_str("───────────────────────────────────────────────────────────\n");

    exit_with_session_context(&context);
}

fn bonds_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".claude")
        .join("bonds")
        .join("pending_actions.json")
}

fn load_pending_actions() -> Option<PendingActions> {
    let path = bonds_path();
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}
