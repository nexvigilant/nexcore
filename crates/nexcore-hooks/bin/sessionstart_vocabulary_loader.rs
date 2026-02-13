//! SessionStart hook: Vocabulary Loader
//!
//! Injects active vocabulary shorthands into session context.
//! Loads vocabulary.json and formats as context for Claude.
//!
//! ToV Alignment:
//! - Monitoring Apparatus (ℳ): Provides observation context
//! - Recognition Presence (R): Preloads pattern recognition vocabulary
//!
//! Exit codes:
//! - 0: Success (context injected or skipped)

use nexcore_hooks::{exit_skip_session, exit_with_session_context, read_input};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Vocabulary {
    shorthands: HashMap<String, Shorthand>,
}

#[derive(Deserialize)]
struct Shorthand {
    expansion: Vec<String>,
    compression_ratio: usize,
    domain: String,
}

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_skip_session(),
    };

    let vocabulary = match load_vocabulary() {
        Some(v) => v,
        None => exit_skip_session(),
    };

    // Check for pending proposals
    let proposals = load_proposals();

    // Build context injection
    let mut context = String::from("Vocabulary System Active:\n");

    // List available shorthands
    context.push_str("\nShorthands loaded (use these - I'll expand automatically):\n");
    for (name, shorthand) in &vocabulary.shorthands {
        context.push_str(&format!(
            "  • {} → {} constraints ({})\n",
            name, shorthand.compression_ratio, shorthand.domain
        ));
    }

    // Note pending proposals if any
    if !proposals.is_empty() {
        context.push_str(&format!(
            "\n⚠ {} vocabulary proposal(s) pending review. Use /vocabulary-review to curate.\n",
            proposals.len()
        ));
    }

    exit_with_session_context(&context);
}

fn implicit_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude").join("implicit")
}

fn load_vocabulary() -> Option<Vocabulary> {
    let path = implicit_path().join("vocabulary.json");
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

fn load_proposals() -> Vec<String> {
    let path = implicit_path().join("vocabulary_proposals.jsonl");
    fs::read_to_string(&path)
        .map(|content| content.lines().map(String::from).collect())
        .unwrap_or_default()
}
