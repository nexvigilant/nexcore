//! # Wheel Reinvention Detector
//!
//! A PreToolUse hook that prevents implementing functionality that already exists
//! in well-maintained crates. Enforces the "don't reinvent the wheel" principle.
//!
//! ## Purpose
//!
//! Detects patterns in Rust code that suggest reimplementation of common functionality:
//! - Retry logic → use `backoff` crate
//! - Rate limiting → use `governor` crate
//! - Thread pools → use `rayon` crate
//! - CLI parsing → use `clap` crate
//! - UUID generation → use `uuid` crate
//! - Base64 encoding → use `base64` crate
//! - Custom hashing → use `std::hash` (HARD BLOCK)
//! - Custom crypto → use `ring` crate (HARD BLOCK)
//!
//! ## Hook Event
//!
//! - **Event**: `PreToolUse` (Write, Edit)
//! - **Matcher**: `*.rs` files only
//! - **Mode**: Skipped in plan mode
//!
//! ## Exit Codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | 0    | No wheel reinvention detected (allow) |
//! | 2    | Pattern detected (block with justification prompt) |
//!
//! ## Justification
//!
//! When blocked, provide one of:
//! - **dependency tree**: Adding the crate would cause conflicts
//! - **unmet need**: The crate doesn't support your specific use case
//! - **benchmark**: You've proven your impl is faster for your use case
//! - **license**: The crate's license is incompatible
//!
//! ## Pattern Detection
//!
//! Patterns are constructed at runtime to avoid self-detection (the hook checking
//! itself would trigger false positives). Keywords are split across format strings.
//!
//! ## Example Trigger
//!
//! ```rust
//! // This would be blocked:
//! fn retry<T>(f: impl Fn() -> Result<T>) -> Result<T> { ... }
//!
//! // Suggestion: use `backoff` crate instead
//! ```

use nexcore_hooks::{exit_block, exit_success_auto, is_rust_file, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip enforcement in plan mode
    if input.is_plan_mode() {
        exit_success_auto();
    }

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !is_rust_file(file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    if content.is_empty() {
        exit_success_auto();
    }

    let violations = check_patterns(content);
    if violations.is_empty() {
        exit_success_auto();
    }

    let mut msg = String::from("WHEEL REINVENTION DETECTED\n\n");
    for (name, crate_name, hard) in &violations {
        msg.push_str(&format!("Pattern: {name}\nUse: `{crate_name}` crate\n"));
        msg.push_str(if *hard { "HARD BLOCK\n\n" } else { "BLOCK\n\n" });
    }
    msg.push_str("Justify: dependency tree | unmet need | benchmark | license");
    exit_block(&msg);
}

/// Build patterns at runtime to avoid self-detection
fn get_patterns() -> Vec<(Vec<String>, &'static str, &'static str, bool)> {
    vec![
        (
            vec![
                format!("fn re{}", "try"),
                format!("with_re{}", "try"),
                format!("Re{}Policy", "try"),
            ],
            "retry logic",
            "backoff",
            false,
        ),
        (
            vec![format!("Rate{}er", "Limit"), format!("Token{}", "Bucket")],
            "rate limiter",
            "governor",
            false,
        ),
        (
            vec![
                format!("struct Thread{}", "Pool"),
                format!("Worker{}", "Pool"),
            ],
            "thread pool",
            "rayon",
            false,
        ),
        (
            vec![format!("Arg{}", "Parser"), format!("fn parse_{}", "args")],
            "CLI parsing",
            "clap",
            false,
        ),
        (
            vec![format!("fn generate_{}", "uuid"), format!("{}_v4", "uuid")],
            "UUID",
            "uuid",
            false,
        ),
        (
            vec![
                format!("fn {}_encode", "base64"),
                format!("{}_CHARS", "BASE64"),
            ],
            "base64",
            "base64",
            false,
        ),
        (
            vec![
                format!("fn custom_{}", "hash"),
                format!("impl {}", "Hasher"),
            ],
            "hash",
            "std::hash",
            true,
        ),
        (
            vec![format!("fn en{}", "crypt"), format!("fn de{}", "crypt")],
            "crypto",
            "ring",
            true,
        ),
    ]
}

fn check_patterns(content: &str) -> Vec<(&'static str, &'static str, bool)> {
    let patterns = get_patterns();

    patterns
        .iter()
        .filter(|(keywords, _, _, _)| keywords.iter().any(|k| content.contains(k)))
        .map(|(_, name, crate_name, hard)| (*name, *crate_name, *hard))
        .collect()
}
