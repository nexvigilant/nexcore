//! Skill Injector - UserPromptSubmit Hook
//!
//! Detects hook-related questions and injects the hook-lifecycle skill.
//! Uses UserPromptSubmit event to inject context before Claude responds.
//!
//! Action: Context injection (no block)
//! Exit: 0 = pass (with optional context on stderr)

use nexcore_hook_lib::cytokine::emit_hook_completed;
use nexcore_hook_lib::pass;
use regex::Regex;

const HOOK_NAME: &str = "skill-injector";
use std::env;
use std::fs;
use std::io::{self, Read};

/// UserPromptSubmit input structure
/// Tier: T2-C (cross-domain composite)
/// Grounds to: T1(String) via Option.
#[derive(Debug, serde::Deserialize)]
struct PromptInput {
    prompt: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    session_id: Option<String>,
}

fn main() {
    // Read stdin
    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() {
        pass();
    }

    if buffer.trim().is_empty() {
        pass();
    }

    // Parse input
    let input: PromptInput = match serde_json::from_str(&buffer) {
        Ok(i) => i,
        Err(_err) => pass(),
    };

    let prompt = match input.prompt {
        Some(p) => p.to_lowercase(),
        None => pass(),
    };

    // Check for hook-related keywords
    let hook_pattern = match Regex::new(
        r"(?i)(create.*hook|hook.*(lifecycle|event|block|matcher)|PreToolUse|PostToolUse|SessionStart|UserPromptSubmit|which.*hook|hook.*config)",
    ) {
        Ok(re) => re,
        Err(_err) => pass(),
    };

    if !hook_pattern.is_match(&prompt) {
        pass();
    }

    // Load and inject the hook-lifecycle skill
    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => pass(),
    };
    let skill_path = format!("{}/.claude/skills/hook-lifecycle/SKILL.md", home);

    if let Ok(skill_content) = fs::read_to_string(&skill_path) {
        // Output to stderr for context injection
        eprintln!("\n--- Hook Development Reference (auto-injected) ---\n");
        eprintln!("{}", skill_content);
        eprintln!("\n--- End Reference ---\n");
        // Emit cytokine signal (TGF-beta = regulation, successful injection)
        emit_hook_completed(HOOK_NAME, 0, "skill_injected");
    }

    pass();
}
