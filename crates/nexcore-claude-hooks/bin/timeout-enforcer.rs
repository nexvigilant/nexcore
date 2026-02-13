//! Timeout Enforcer — PreToolUse Hook
//!
//! Enforces time limits on Bash and Task tool calls:
//!   - Bash: timeout ≤ BASH_MAX_TIMEOUT_MS (default 120000ms = 2min)
//!   - Task: max_turns ≤ MAX_TASK_TURNS (default 3)
//!
//! Environment variables:
//!   BASH_MAX_TIMEOUT_MS  — hard ceiling for Bash timeout (default: 120000)
//!   BASH_WARN_TIMEOUT_MS — soft warning threshold (default: 60000)
//!   MAX_TASK_TURNS       — hard ceiling for Task max_turns (default: 3)
//!
//! Exit: 0 = pass, 1 = warn, 2 = block

use nexcore_hook_lib::cytokine::emit_threshold_exceeded;
use nexcore_hook_lib::{block, pass, read_input, warn};

#[allow(dead_code)]
const HOOK_NAME: &str = "timeout-enforcer";
const DEFAULT_BASH_MAX_MS: u64 = 120_000;
const DEFAULT_BASH_WARN_MS: u64 = 60_000;
const DEFAULT_MAX_TURNS: u64 = 30;
const ATOMIC_MAX_TURNS: u64 = 1;
const ATOMIC_REQUIRED_MODEL: &str = "opus";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };

    let tool_name = match &input.tool_name {
        Some(t) => format!("{t}"),
        None => pass(),
    };

    let tool_input = match &input.tool_input {
        Some(ti) => ti,
        None => pass(),
    };

    match tool_name.as_str() {
        "Bash" | "run_shell_command" => check_bash_timeout(tool_input),
        "Task" | "task" => check_task_limits(tool_input),
        _ => pass(),
    }
}

fn check_bash_timeout(ti: &nexcore_hook_lib::ToolInput) -> ! {
    let max_ms = env_u64("BASH_MAX_TIMEOUT_MS", DEFAULT_BASH_MAX_MS);
    let warn_ms = env_u64("BASH_WARN_TIMEOUT_MS", DEFAULT_BASH_WARN_MS);

    match ti.timeout {
        Some(t) if t > max_ms => {
            // Emit cytokine signal (IL-10 = threshold exceeded)
            emit_threshold_exceeded("bash_timeout_ms", t as f64, max_ms as f64);
            block(&format!(
                "🚫 TIMEOUT LIMIT: Bash timeout={t}ms exceeds max={max_ms}ms.\n\
                 Set timeout ≤ {max_ms} or omit for default."
            ));
        }
        Some(t) if t > warn_ms => {
            let cmd_preview = ti.command.as_deref().unwrap_or("(unknown)");
            let preview = if cmd_preview.len() > 40 {
                &cmd_preview[..40]
            } else {
                cmd_preview
            };
            warn(&format!(
                "⚠️ Long Bash timeout: {t}ms (warn threshold: {warn_ms}ms)\n\
                 Command: {preview}..."
            ));
        }
        _ => pass(),
    }
}

fn check_turn_ceiling(turns: u64, max: u64) {
    if turns > max {
        // Emit cytokine signal (IL-10 = threshold exceeded)
        emit_threshold_exceeded("task_max_turns", turns as f64, max as f64);
        block(&format!("🚫 max_turns={turns} exceeds ceiling={max}"));
    }
}

fn check_missing_turns(strict: bool, max: u64) {
    if strict {
        block(&format!("🚫 Missing max_turns. Required ≤ {max}"));
    }
    warn(&format!("⚠️ Missing max_turns. Policy: ≤ {max}"));
}

fn check_atomic_model(turns: u64, model: &str, enforce: bool) {
    if turns > ATOMIC_MAX_TURNS || model == ATOMIC_REQUIRED_MODEL {
        return;
    }
    let msg = format!("Atomic (max_turns={turns}) requires model='opus', got '{model}'");
    if enforce {
        block(&format!("🚫 {msg}"));
    }
    warn(&format!("⚠️ {msg}"));
}

fn check_task_limits(ti: &nexcore_hook_lib::ToolInput) -> ! {
    let max_turns = env_u64("MAX_TASK_TURNS", DEFAULT_MAX_TURNS);
    let strict = std::env::var("STRICT_TASK_LIMITS").is_ok();
    let enforce_model = true; // Maximum enforcement: atomic tasks require opus

    match ti.max_turns {
        Some(n) => {
            check_turn_ceiling(n, max_turns);
            check_atomic_model(n, ti.model.as_deref().unwrap_or("sonnet"), enforce_model);
        }
        None => check_missing_turns(strict, max_turns),
    }
    pass();
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
