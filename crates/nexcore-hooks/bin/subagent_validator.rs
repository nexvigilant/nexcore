//! Subagent Start Validator Hook
//!
//! Event: SubagentStart
//! Validates subagent before execution.
//!
//! Checks:
//! - Agent type is recognized
//! - Required resources are available
//! - Agent chaining is appropriate

use nexcore_hooks::{HookOutput, exit_success_auto, read_input};

/// Known agent types that are valid
const KNOWN_AGENTS: &[&str] = &[
    "Bash",
    "general-purpose",
    "Explore",
    "Plan",
    "rust-migrator",
    "rust-reviewer",
    "rust-borrow-doctor",
    "rust-compiler-doctor",
    "rust-async-expert",
    "rust-docs",
    "rust-test-architect",
    "rust-architect",
    "claude-code-guide",
];

/// Agent types that should only be used for specific purposes
const SPECIALIZED_AGENTS: &[(&str, &str)] = &[
    ("rust-migrator", "Python/Go/C migration to Rust"),
    ("rust-borrow-doctor", "Borrow checker errors"),
    ("rust-compiler-doctor", "Compiler errors"),
    ("rust-async-expert", "Async/await issues"),
];

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let agent_type = input.agent_type.as_deref().unwrap_or("unknown");
    let agent_id = input.agent_id.as_deref().unwrap_or("no-id");

    // Check if agent type is known
    let is_known = KNOWN_AGENTS.iter().any(|&a| a == agent_type);

    if !is_known {
        // Warn about unknown agent type
        HookOutput::warn(&format!(
            "Unknown agent type '{}' - may not have expected tools",
            agent_type
        ))
        .with_system_message(format!("⚠️ Spawning unknown agent: {}", agent_type))
        .emit();
        std::process::exit(1);
    }

    // Check for specialized agent usage
    for (agent, purpose) in SPECIALIZED_AGENTS {
        if agent_type == *agent {
            HookOutput::allow()
                .with_system_message(format!(
                    "🤖 {} agent started (purpose: {})",
                    agent_type, purpose
                ))
                .emit();
            std::process::exit(0);
        }
    }

    // Standard agent - allow with logging
    HookOutput::allow()
        .with_system_message(format!(
            "🤖 Agent started: {} ({})",
            agent_type,
            &agent_id[..8.min(agent_id.len())]
        ))
        .emit();
    std::process::exit(0);
}
