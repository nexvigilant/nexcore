//! Escape Word Detector Hook
//!
//! UserPromptSubmit hook that detects escape words for autonomous loop termination.
//!
//! # Event
//! UserPromptSubmit
//!
//! # Escape Words
//! - "forge complete" - Normal completion, accept results
//! - "refinery halt" - Emergency stop, discard current iteration
//! - "primitive stable" - Accept primitives as stable
//!
//! # Behavior
//! When detected, injects context to signal loop termination and clears state.
//!
//! # Exit Codes
//! - 0: No escape word or escape processed successfully

use nexcore_hooks::protocol::HookOutput;
use nexcore_hooks::{exit_success_auto, read_input};
use std::fs;
use std::path::PathBuf;

/// Escape words and their termination modes
const ESCAPE_WORDS: &[(&str, TerminationMode)] = &[
    // FORGE loop control
    ("forge complete", TerminationMode::Accept),
    ("stop forge", TerminationMode::Halt),
    ("halt", TerminationMode::Halt),
    ("enough", TerminationMode::Halt),
    // Refinery/primitive loop control
    ("refinery halt", TerminationMode::Halt),
    ("primitive stable", TerminationMode::Stable),
    ("stop mining", TerminationMode::Halt),
    ("end refinement", TerminationMode::Accept),
    // Generic escape
    ("exit loop", TerminationMode::Halt),
    ("break loop", TerminationMode::Halt),
];

#[derive(Debug, Clone, Copy)]
enum TerminationMode {
    /// Accept current results and exit loop
    Accept,
    /// Emergency halt, discard current iteration
    Halt,
    /// Mark primitives as stable, exit loop
    Stable,
}

impl TerminationMode {
    fn description(&self) -> &'static str {
        match self {
            Self::Accept => "Accepting current refinement results",
            Self::Halt => "Emergency halt - discarding current iteration",
            Self::Stable => "Primitives marked stable - exiting refinement loop",
        }
    }

    fn action(&self) -> &'static str {
        match self {
            Self::Accept => "Save extracted primitives and generate summary",
            Self::Halt => "Discard current work and restore previous state",
            Self::Stable => "Commit primitives to skill registry and close loop",
        }
    }
}

fn get_refinement_state_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".claude")
        .join("brain")
        .join("refinement_state.json")
}

fn clear_refinement_state() {
    let path = get_refinement_state_path();
    if path.exists() {
        // Best-effort cleanup - log but don't fail if removal fails
        if let Err(e) = fs::remove_file(&path) {
            eprintln!("Warning: could not clear refinement state: {}", e);
        }
    }
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Get the user's prompt text
    let prompt = input.prompt.as_deref().unwrap_or("");
    let prompt_lower = prompt.to_lowercase();

    // Check for escape words
    for (word, mode) in ESCAPE_WORDS {
        if prompt_lower.contains(word) {
            // Clear refinement state
            clear_refinement_state();

            let context = format!(
                r#"
===============================================================
REFINEMENT LOOP TERMINATION DETECTED
===============================================================

Escape word detected: "{}"
Mode: {:?}
Description: {}

ACTION REQUIRED: {}

The autonomous refinement loop has been terminated by user command.
Proceeding with normal conversation flow.

===============================================================
"#,
                word,
                mode,
                mode.description(),
                mode.action()
            );

            // Emit with context injection via reason field
            // Exit 1 = warn (shows message but allows action)
            let output = HookOutput {
                decision: None, // No decision needed for UserPromptSubmit
                reason: Some(context),
                ..Default::default()
            };
            output.emit();
            std::process::exit(1); // Exit 1 = warn with message
        }
    }

    // No escape word found - allow prompt to continue
    exit_success_auto();
}
