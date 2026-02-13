//! Requirements Crystallization Gate (HOOK-M1-01)
//!
//! Intercepts implementation-oriented prompts and injects a requirements verification
//! checkpoint before Claude begins coding. Prevents "build first, ask questions later"
//! anti-pattern by ensuring requirements are crystallized upfront.
//!
//! # Hook Configuration
//!
//! | Property | Value |
//! |----------|-------|
//! | **Event** | `UserPromptSubmit` |
//! | **Tier** | `dev` (development workflow) |
//! | **Timeout** | 5000ms |
//! | **Matchers** | All user prompts (content-based filtering) |
//!
//! # Exit Codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | 0 | Skip - not an implementation request, or requirements already verified |
//! | 0 + context | Inject requirements checkpoint into conversation |
//!
//! # Detection Patterns
//!
//! The hook detects implementation requests using these regex patterns:
//!
//! - `create|build|implement|write|develop|code|make` + `function|struct|module|api|service|feature|system`
//! - `add|implement` + `to`
//! - `write code|rust|python`
//! - `let's build|create|implement`
//! - `can you create|build|write|implement`
//!
//! # Session State
//!
//! Uses `SessionState.requirements_verified` to track if requirements have been
//! confirmed for the current task. Once verified, the hook stops injecting checkpoints.
//!
//! # Injected Checkpoint
//!
//! When triggered, injects a structured requirements verification prompt asking:
//! - What exactly should this do?
//! - What are the inputs/outputs?
//! - What constraints exist?
//! - What's explicitly out of scope?
//!
//! # Rationale
//!
//! The "Preflight Clarification Protocol" ensures:
//! - Requirements are explicit before implementation
//! - Scope creep is prevented by defining boundaries
//! - Rework is minimized by catching misunderstandings early
//!
//! This implements the CLAUDE.md principle: "NEVER propose changes to code you haven't read."

use nexcore_hooks::output::format_requirements_checkpoint;
use nexcore_hooks::state::SessionState;
use nexcore_hooks::{exit_skip_prompt, exit_with_context, read_input};
use regex::Regex;

/// Implementation-oriented language patterns
const IMPL_PATTERNS: &[&str] = &[
    r"\b(create|build|implement|write|develop|code|make)\b.*\b(function|struct|module|api|service|feature|system)\b",
    r"\b(add|implement)\b.*\bto\b",
    r"\bwrite\s+(code|rust|python)\b",
    r"\blet'?s\s+(build|create|implement)\b",
    r"\bcan\s+you\s+(create|build|write|implement)\b",
];

fn detect_implementation_request(prompt: &str) -> bool {
    let prompt_lower = prompt.to_lowercase();
    for pattern in IMPL_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&prompt_lower) {
                return true;
            }
        }
    }
    false
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_skip_prompt(),
    };

    let prompt = match input.get_prompt() {
        Some(p) => p,
        None => exit_skip_prompt(),
    };

    let state = SessionState::load();

    // If requirements already verified, skip
    if state.requirements_verified {
        exit_skip_prompt();
    }

    // Check for implementation request
    if detect_implementation_request(prompt) {
        let checkpoint = format_requirements_checkpoint();
        exit_with_context(&checkpoint);
    }

    exit_skip_prompt();
}
