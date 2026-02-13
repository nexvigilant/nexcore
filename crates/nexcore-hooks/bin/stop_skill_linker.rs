//! Stop Skill Linker
//!
//! **Event:** `Stop`
//! **Action:** Ensures a skill is referenced in the output before session ends
//!
//! # Purpose
//!
//! Enforces skill-aware responses by blocking session end until a relevant
//! skill is linked. This promotes skill discoverability and ensures users
//! know about available capabilities.
//!
//! # Behavior
//!
//! 1. Reads the conversation transcript
//! 2. Searches for skill references (slash commands, Skill tool calls, trigger patterns)
//! 3. Blocks if no skill was referenced, suggesting relevant skills
//! 4. Allows if a skill was invoked or mentioned
//!
//! # Patterns Detected
//!
//! - Slash commands: `/forge`, `/ctvp`, `/rust-dev`, etc.
//! - Skill tool calls: `Skill tool`, `invoke skill`
//! - Skill mentions: `skill-name skill`, `use the X skill`
//!
//! # Exit Codes
//!
//! - `0`: Allow stop (skill was referenced)
//! - `2`: Block stop (no skill referenced - suggest one)

use nexcore_hooks::{exit_block, exit_success, read_input};
use std::fs;

/// Skill slash command patterns (most common skills)
const SKILL_SLASH_PATTERNS: &[&str] = &[
    "/forge",
    "/ctvp",
    "/rust-dev",
    "/rust-anatomy",
    "/strat-dev",
    "/strategy",
    "/primitive",
    "/hook",
    "/skill",
    "/brain",
    "/mcp",
    "/config",
    "/vigilance",
    "/guardian",
    "/cep",
    "/vdag",
    "/systems",
    "/compendious",
    "/persona",
    "/advisor",
    "/vocabulary",
    "/translate",
    "/socratic",
];

/// Patterns indicating skill tool usage
const SKILL_TOOL_PATTERNS: &[&str] = &[
    "Skill tool",
    "invoke skill",
    "skill:",
    "skill-dev",
    "skill-audit",
    "skill-advisor",
    "skill_list",
    "skill_get",
    "skill_validate",
];

/// Context patterns that exempt from skill requirement
const EXEMPT_PATTERNS: &[&str] = &[
    "compilation error",
    "fix before",
    "cannot find",
    "error[E",
    "FAILED",
    "Critical issue",
    // Simple acknowledgments don't need skill links
    "Got it",
    "Understood",
    "OK",
];

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success(),
    };

    // Prevent infinite loop
    if input.is_stop_hook_active() {
        exit_success();
    }

    // Get transcript
    let transcript_path = match &input.transcript_path {
        Some(p) => p,
        None => exit_success(),
    };

    let transcript = match fs::read_to_string(transcript_path) {
        Ok(content) => content,
        Err(_) => exit_success(),
    };

    // Get last ~100 lines for context
    let lines: Vec<&str> = transcript.lines().collect();
    let recent_lines = if lines.len() > 100 {
        &lines[lines.len() - 100..]
    } else {
        &lines[..]
    };
    let recent_content = recent_lines.join("\n");
    let lower_content = recent_content.to_lowercase();

    // Check for exempt patterns (error fixing, simple acks)
    for pattern in EXEMPT_PATTERNS {
        if recent_content.contains(pattern) || lower_content.contains(&pattern.to_lowercase()) {
            exit_success();
        }
    }

    // Check for skill slash commands
    for pattern in SKILL_SLASH_PATTERNS {
        if recent_content.contains(pattern) {
            exit_success();
        }
    }

    // Check for skill tool usage patterns
    for pattern in SKILL_TOOL_PATTERNS {
        if recent_content.contains(pattern) || lower_content.contains(&pattern.to_lowercase()) {
            exit_success();
        }
    }

    // No skill found - block with suggestion
    exit_block(
        "No skill referenced in output. Consider linking a relevant skill:\n\
         • /skill-advisor - Get skill recommendations for your context\n\
         • /forge - Autonomous Rust development\n\
         • /ctvp - Validate code with CTVP framework\n\
         • /rust-dev - Rust patterns and guidance\n\
         • /hook-lifecycle - Hook development\n\
         • Type '/help skills' to see all available skills",
    );
}
