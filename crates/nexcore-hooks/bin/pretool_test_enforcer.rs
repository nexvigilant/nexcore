//! Test-First Enforcer (HOOK-M2-17)
//!
//! Implements Guardian homeostasis pattern for test enforcement:
//!
//! ```text
//! ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//! │   SENSING   │───▶│  DECISION   │───▶│  RESPONSE   │
//! │ (Detect src │    │ (Check test │    │ (Warn/Block)│
//! │  changes)   │    │  coverage)  │    │             │
//! └─────────────┘    └─────────────┘    └─────────────┘
//! ```
//!
//! # Behavior
//! - **PAMP Detection**: Source file modification without corresponding test
//! - **Decision Engine**: Check if test file exists for the source
//! - **Response**: Warn (exit 1) if no test, pass (exit 0) if covered
//!
//! # Test File Patterns
//! For `src/foo.rs`, checks:
//! - `src/foo_test.rs`
//! - `tests/foo.rs`
//! - `tests/foo_test.rs`
//! - `src/foo.rs` contains `#[cfg(test)]` module
//!
//! # Event
//! `PreToolUse:Edit|Write`
//!
//! # Exit Codes
//! - 0: Test coverage exists OR non-source file
//! - 1: Warning - no test coverage detected (allows proceed)
//!
//! # Guardian Pattern Applied
//! - Sensing: Detect modifications to `.rs` files in `src/`
//! - Decision: Check test coverage patterns
//! - Response: Emit warning with guidance
//! - Feedback: Track warned files to avoid repetition

use nexcore_hooks::{exit_success_auto, exit_warn, read_input};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// State file for tracking already-warned files (feedback loop)
fn state_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude/state/test_enforcer_warned.json")
}

/// Persistent state for the enforcer (Guardian feedback loop)
#[derive(Debug, Default, Serialize, Deserialize)]
struct EnforcerState {
    /// Files we've already warned about this session
    warned_files: HashSet<String>,
    /// Session ID to reset warnings per session
    session_id: Option<String>,
}

impl EnforcerState {
    fn load() -> Self {
        let path = state_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    fn save(&self) {
        let path = state_path();
        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                return;
            }
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if fs::write(&path, json).is_err() {
                eprintln!("⚠️  Failed to save test enforcer state");
            }
        }
    }

    fn has_warned(&self, file: &str) -> bool {
        self.warned_files.contains(file)
    }

    fn mark_warned(&mut self, file: &str) {
        self.warned_files.insert(file.to_string());
        self.save();
    }
}

/// SENSING: Detect if this is a source file that needs test coverage
fn is_testable_source(path: &str) -> bool {
    // Must be a Rust file
    if !path.ends_with(".rs") {
        return false;
    }

    // Must be in src/ directory
    if !path.contains("/src/") && !path.starts_with("src/") {
        return false;
    }

    // Exclude test files themselves
    if path.contains("_test.rs") || path.contains("/tests/") {
        return false;
    }

    // Exclude mod.rs (usually just re-exports)
    if path.ends_with("mod.rs") {
        return false;
    }

    // Exclude lib.rs and main.rs (entry points, tested via integration)
    let filename = Path::new(path).file_name().and_then(|f| f.to_str());
    if matches!(filename, Some("lib.rs") | Some("main.rs")) {
        return false;
    }

    true
}

/// DECISION: Check if test coverage exists for a source file
fn has_test_coverage(source_path: &str) -> bool {
    let path = Path::new(source_path);

    // Pattern 1: Inline #[cfg(test)] module in the source file
    if let Ok(content) = fs::read_to_string(path) {
        if content.contains("#[cfg(test)]") {
            return true;
        }
    }

    // Pattern 2: Adjacent _test.rs file (src/foo.rs → src/foo_test.rs)
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if let Some(parent) = path.parent() {
        let adjacent_test = parent.join(format!("{}_test.rs", stem));
        if adjacent_test.exists() {
            return true;
        }
    }

    // Pattern 3: tests/ directory at crate root
    // Find crate root by looking for Cargo.toml
    let mut search_dir = path.parent();
    while let Some(dir) = search_dir {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            // Found crate root
            let tests_dir = dir.join("tests");

            // Check tests/foo.rs
            let test_file = tests_dir.join(format!("{}.rs", stem));
            if test_file.exists() {
                return true;
            }

            // Check tests/foo_test.rs
            let test_file = tests_dir.join(format!("{}_test.rs", stem));
            if test_file.exists() {
                return true;
            }

            break;
        }
        search_dir = dir.parent();
    }

    false
}

/// RESPONSE: Generate appropriate response based on sensing and decision
fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only process Edit/Write tools
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Edit" && tool_name != "Write" {
        exit_success_auto();
    }

    // Get the file being modified
    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    // SENSING: Check if this is a testable source file
    if !is_testable_source(&file_path) {
        exit_success_auto();
    }

    // FEEDBACK: Check if we've already warned about this file
    let mut state = EnforcerState::load();
    if state.has_warned(&file_path) {
        exit_success_auto();
    }

    // DECISION: Check test coverage
    if has_test_coverage(&file_path) {
        exit_success_auto();
    }

    // RESPONSE: No test coverage detected - warn user
    state.mark_warned(&file_path);

    let filename = Path::new(&file_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("file");

    let stem = Path::new(&file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");

    exit_warn(&format!(
        "⚠️  No test coverage for `{}`

Consider adding tests:
  1. Inline: Add `#[cfg(test)] mod tests {{ ... }}` to {}
  2. Adjacent: Create {}_test.rs
  3. Integration: Add tests/{}.rs

Proceed with caution - untested code is technical debt.",
        filename, filename, stem, stem
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_testable_source() {
        // Should be testable
        assert!(is_testable_source("src/foo.rs"));
        assert!(is_testable_source("/path/to/src/bar.rs"));
        assert!(is_testable_source("crate/src/module/thing.rs"));

        // Should NOT be testable
        assert!(!is_testable_source("src/foo_test.rs"));
        assert!(!is_testable_source("tests/foo.rs"));
        assert!(!is_testable_source("src/mod.rs"));
        assert!(!is_testable_source("src/lib.rs"));
        assert!(!is_testable_source("src/main.rs"));
        assert!(!is_testable_source("config.json"));
        assert!(!is_testable_source("README.md"));
    }

    #[test]
    fn test_state_serialization() {
        let mut state = EnforcerState::default();
        state.mark_warned("src/foo.rs");
        assert!(state.has_warned("src/foo.rs"));
        assert!(!state.has_warned("src/bar.rs"));
    }
}
