//! Hormone Stimulus Emitter Hook
//!
//! Event: PostToolUse (matcher: all tools)
//!
//! Automatically emits stimuli to the endocrine system based on tool outcomes:
//! - Tool errors → cortisol (stress response)
//! - Successful completions → dopamine (reward)
//! - Write/Edit completions → dopamine + pattern success

use nexcore_hooks::{exit_success_auto, read_input};
use nexcore_hormones::{EndocrineState, Stimulus};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Get tool name
    let tool_name = input.tool_name.as_deref().unwrap_or("");

    // Get response
    let response = match &input.tool_response {
        Some(v) => v,
        None => exit_success_auto(),
    };

    // Determine if there was an error
    let is_error = response
        .get("is_error")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let exit_code = response
        .get("exit_code")
        .or_else(|| response.get("exitCode"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    // Load endocrine state
    let mut state = EndocrineState::load();

    // Apply stimuli based on outcome
    if is_error || exit_code != 0 {
        // Error occurred → stress response
        let severity = if exit_code > 1 { 0.7 } else { 0.4 };
        let stimulus = Stimulus::ErrorEncountered { severity };
        stimulus.apply(&mut state);
    } else {
        // Success → reward response
        let complexity = match tool_name {
            "Write" | "Edit" => 0.6,         // Code changes are rewarding
            "Bash" => 0.4,                   // Commands moderately rewarding
            "Task" => 0.8,                   // Completed subagent very rewarding
            "Read" | "Glob" | "Grep" => 0.2, // Info gathering mildly rewarding
            _ => 0.3,
        };
        let stimulus = Stimulus::TaskCompleted { complexity };
        stimulus.apply(&mut state);

        // Write/Edit also counts as pattern success
        if tool_name == "Write" || tool_name == "Edit" {
            let pattern_stimulus = Stimulus::PatternSuccess { reuse_count: 1 };
            pattern_stimulus.apply(&mut state);
        }
    }

    // Save updated state (best-effort, non-blocking)
    if let Err(e) = state.save() {
        eprintln!("hormone_stimulus: failed to save state: {e}");
    }

    exit_success_auto();
}
