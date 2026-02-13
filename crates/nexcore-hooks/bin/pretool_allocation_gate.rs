//! Allocation Audit Gate (Hook 39)
//!
//! Detects hidden allocations in performance-critical code paths.
//! Blocks allocations in hot paths, warns on loop allocations.
//! Runs as PreToolUse hook on Write/Edit of Rust files.
//!
//! ## Patterns Detected
//!
//! - `Vec::new()` / `vec![]` inside `#[hot]` or `#[inline(always)]` functions
//! - `String::from()` / `.to_string()` / `format!()` in tight loops
//! - `Box::new()` / `Arc::new()` in hot paths without justification
//! - `.collect::<Vec<_>>()` inside loops (prefer extending pre-allocated)
//!
//! ## Exit Codes
//!
//! - 0: No allocations found, non-Rust file, test file, or plan mode (skip)
//! - 1: Warning — allocation in loop (not hot path)
//! - 2: Block — allocation in hot path without `// ALLOC:` justification
//!
//! ## Integration
//!
//! Event: `PreToolUse` (Write, Edit)
//! Tier: `dev` (default), `review` (stricter)
//! Parser: `nexcore_hooks::parser::performance::detect_allocations`

use nexcore_hooks::parser::performance::detect_allocations;
use nexcore_hooks::{
    exit_block, exit_success_auto, exit_warn, is_rust_file, is_test_file, read_input,
};

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

    if !is_rust_file(file_path) || is_test_file(file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let allocations = detect_allocations(content);
    if allocations.is_empty() {
        exit_success_auto();
    }

    // Separate by severity
    let hot_path: Vec<_> = allocations.iter().filter(|a| a.in_hot_path).collect();
    let in_loop: Vec<_> = allocations
        .iter()
        .filter(|a| a.in_loop && !a.in_hot_path)
        .collect();

    // Hot path allocations are blocking
    if !hot_path.is_empty() {
        let mut msg = format!(
            "ALLOCATION AUDIT - {} hot path allocation(s)\n\n",
            hot_path.len()
        );
        for a in &hot_path {
            msg.push_str(&format!(
                "Line {}: {}\n  Pattern: {}\n  Note: {}\n\n",
                a.line, a.code, a.pattern, a.note
            ));
        }
        msg.push_str("Hot path allocations must be justified with // ALLOC: comment\n");
        msg.push_str("Or hoist allocation outside hot path.");
        exit_block(&msg);
    }

    // Loop allocations are warnings
    if !in_loop.is_empty() {
        let mut msg = format!(
            "ALLOCATION AUDIT - {} loop allocation(s)\n\n",
            in_loop.len()
        );
        for a in &in_loop {
            msg.push_str(&format!(
                "Line {}: {}\n  Pattern: {}\n  Note: {}\n\n",
                a.line, a.code, a.pattern, a.note
            ));
        }
        msg.push_str("Consider pre-allocating or hoisting outside loop.");
        exit_warn(&msg);
    }

    exit_success_auto();
}
