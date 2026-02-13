//! Atomic Agent Enforcer — SubagentStop Hook
//!
//! Logs agent completion for telemetry.
//! Actual enforcement is via max_turns=1 at Task call site.
//!
//! Exit: 0 = pass (always)

use nexcore_hook_lib::cytokine::emit_hook_completed;
use nexcore_hook_lib::{pass, read_input};

const HOOK_NAME: &str = "atomic-agent-enforcer";

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => pass(),
    };

    // Emit cytokine signal (TGF-beta = regulation, agent completed)
    emit_hook_completed(HOOK_NAME, 0, "agent_stop_observed");

    // SubagentStop receives different payload - just pass for now
    // Actual atomic enforcement is via max_turns parameter
    pass();
}
