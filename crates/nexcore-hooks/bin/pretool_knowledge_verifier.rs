//! # Knowledge Verification Gate
//!
//! **Hook Event**: `PreToolUse` (Edit, Write)
//! **Exit Behavior**: Warn (non-blocking) when unverified APIs detected
//!
//! ## Purpose
//!
//! Detects potential API hallucinations in Rust code before writes occur.
//! LLMs can confidently generate non-existent method calls, trait implementations,
//! or module paths. This hook:
//!
//! 1. Extracts all `use` statements to identify referenced crates
//! 2. Cross-references against a trusted crate allowlist
//! 3. Scans for risky patterns that indicate fabricated APIs
//! 4. Tracks unverified assumptions in session state for later verification
//!
//! ## Trusted Crate Allowlist
//!
//! Crates in `TRUSTED_CRATES` are assumed to have stable, known APIs:
//! - Standard library: `std`, `core`, `alloc`
//! - Serialization: `serde`, `serde_json`, `serde_yaml`, `toml`
//! - Async runtime: `tokio`, `async_std`, `futures`
//! - Error handling: `anyhow`, `thiserror`, `eyre`
//! - nexcore workspace: `nexcore_*` crates
//!
//! ## Risky Patterns
//!
//! Patterns that suggest hallucinated APIs:
//! - `._async()` - Made-up async method suffix
//! - `::from_*_async(` - Made-up async conversion
//! - `.into_*_result()` - Made-up result conversion
//! - `::very_long_module::` - Suspiciously deep paths (15+ chars)
//!
//! ## Session State Integration
//!
//! Unverified crates and risky patterns are added to `SessionState.assumptions`
//! with confidence levels. The `posttool_assumption_verifier` hook can later
//! verify these assumptions via `cargo check`.
//!
//! ## Example Output
//!
//! ```text
//! ⚠️ Knowledge verification: 2 assumption(s) added
//! Unverified crate: some_obscure_crate
//! Made-up async method: .process_async()
//! ```
//!
//! ## Configuration
//!
//! In `~/.claude/settings.json`:
//! ```json
//! {
//!   "hooks": {
//!     "PreToolUse": [{
//!       "matcher": "Edit|Write",
//!       "command": "pretool_knowledge_verifier"
//!     }]
//!   }
//! }
//! ```

use nexcore_hooks::state::SessionState;
use nexcore_hooks::{exit_success_auto, exit_warn, read_input};
use regex::Regex;
use std::collections::HashSet;

const TRUSTED_CRATES: &[&str] = &[
    "std",
    "core",
    "alloc",
    "serde",
    "serde_json",
    "serde_yaml",
    "toml",
    "tokio",
    "async_std",
    "futures",
    "anyhow",
    "thiserror",
    "eyre",
    "clap",
    "structopt",
    "log",
    "tracing",
    "env_logger",
    "reqwest",
    "hyper",
    "axum",
    "regex",
    "chrono",
    "time",
    "rand",
    "uuid",
    "itertools",
    "once_cell",
    "lazy_static",
    "parking_lot",
    "sha2",
    "walkdir",
    "tempfile",
    "rmcp",
    "nexcore_foundation",
    "nexcore_pv",
    "nexcore_skills",
    "nexcore_vigilance",
    "nexcore_hooks",
    "syn",
    "quote",
    "proc_macro2",
];

const RISKY_PATTERNS: &[(&str, &str)] = &[
    (r"\._async\(\)", "Made-up async method"),
    (r"::from_\w+_async\(", "Made-up async conversion"),
    (r"\.into_\w+_result\(\)", "Made-up result conversion"),
    (r"::\w{15,}::", "Suspiciously deep module path"),
];

fn extract_crate_uses(content: &str) -> HashSet<String> {
    let mut crates = HashSet::new();
    if let Ok(re) = Regex::new(r"use\s+(\w+)(?:::|;)") {
        for cap in re.captures_iter(content) {
            let name = &cap[1];
            if !["crate", "self", "super"].contains(&name) {
                crates.insert(name.to_string());
            }
        }
    }
    crates
}

fn detect_risky_patterns(content: &str) -> Vec<(String, String)> {
    let mut risks = Vec::new();
    for (pattern, desc) in RISKY_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            for mat in re.find_iter(content) {
                risks.push((mat.as_str().to_string(), desc.to_string()));
            }
        }
    }
    risks
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !file_path.ends_with(".rs") {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let trusted: HashSet<&str> = TRUSTED_CRATES.iter().copied().collect();
    let crates = extract_crate_uses(content);
    let mut state = SessionState::load();

    let unverified: Vec<_> = crates
        .iter()
        .filter(|c| !trusted.contains(c.as_str()) && !state.is_crate_verified(c))
        .cloned()
        .collect();

    let risks = detect_risky_patterns(content);

    // Add assumptions for unverified crates
    for crate_name in &unverified {
        state.add_assumption(
            &format!("Crate '{}' API is correct", crate_name),
            "medium",
            "Verify with: cargo check",
        );
    }

    // Add assumptions for risky patterns
    for (pattern, desc) in &risks {
        state.add_assumption(
            &format!("{}: '{}'", desc, pattern),
            "low",
            "Verify with: cargo check or API docs",
        );
    }

    let _ = state.save();

    let mut warnings = Vec::new();
    for crate_name in &unverified {
        warnings.push(format!("Unverified crate: {}", crate_name));
    }
    for (pattern, desc) in &risks {
        warnings.push(format!("{}: {}", desc, pattern));
    }

    if warnings.is_empty() {
        exit_success_auto();
    }

    // Non-blocking - these will be caught by cargo check
    exit_warn(&format!(
        "Knowledge verification: {} assumption(s) added\n{}",
        warnings.len(),
        warnings.join("\n")
    ));
}
