//! Agent Intent Detector Hook
//!
//! Event: UserPromptSubmit
//!
//! Analyzes user prompts for keywords that indicate they need
//! specialized Rust agents:
//! - "migrate from python" → rust-migrator
//! - "async", "tokio" → rust-async-expert
//! - "optimize", "performance" → rust-optimize
//! - "unsafe", "ffi" → rust-unsafe-specialist
//! - etc.

use nexcore_hooks::agent_triggers::intents::detect_intent;
use nexcore_hooks::{exit_skip_prompt, exit_with_context, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_skip_prompt(),
    };

    // Get the user's prompt
    let prompt = match input.get_prompt() {
        Some(p) => p,
        None => exit_skip_prompt(),
    };

    // Detect intent from prompt
    if let Some(detection) = detect_intent(prompt) {
        exit_with_context(&detection.to_context());
    }

    exit_skip_prompt();
}
