//! Clone Detector Hook
//!
//! Detects unnecessary `.clone()` calls that waste CPU and memory. Enforces
//! Rust's ownership model by identifying clones that can be eliminated through
//! better borrowing, references, or ownership transfer.
//!
//! # Hook Configuration
//!
//! | Property | Value |
//! |----------|-------|
//! | **Event** | `PreToolUse` |
//! | **Tier** | `dev` (performance optimization) |
//! | **Timeout** | 5000ms |
//! | **Matchers** | `tool_name: Edit, Write` for `*.rs` files (excludes tests) |
//!
//! # Exit Codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | 0 | Success - no problematic clones found |
//! | 1 | Warn - clones detected but not in hot paths |
//! | 2 | Block - clones detected inside loops (high impact) |
//!
//! # Clone Classifications
//!
//! | Classification | Impact | Action |
//! |----------------|--------|--------|
//! | `InLoop` | **Critical** | Blocks - clone cost multiplied per iteration |
//! | `Unnecessary` | Medium | Warns - can use reference or move |
//! | `ChainedClone` | Medium | Warns - `.clone().clone()` pattern |
//! | `Necessary` | None | Allowed - genuinely needed (e.g., shared ownership) |
//!
//! # Justification Comment
//!
//! To suppress a warning, add a justification comment:
//!
//! ```rust
//! // CLONE: Required for shared ownership across threads
//! let shared = data.clone();
//! ```
//!
//! # Performance Impact
//!
//! Clones in loops are particularly expensive:
//!
//! ```text
//! for item in items {           // 1000 iterations
//!     let copy = item.clone();  // 1000 allocations!
//! }
//!
//! // Better: clone once outside loop, or use references
//! let items_ref = &items;
//! for item in items_ref { ... }
//! ```
//!
//! # Rationale
//!
//! This hook enforces the Rust principle: "Don't pay for what you don't use."
//! Unnecessary clones indicate ownership confusion and should be refactored.

use nexcore_hooks::parser::performance::{CloneClassification, detect_clones};
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

    let clones = detect_clones(content);
    if clones.is_empty() {
        exit_success_auto();
    }

    let loop_clones: Vec<_> = clones
        .iter()
        .filter(|c| c.classification == CloneClassification::InLoop)
        .collect();

    let other_clones: Vec<_> = clones
        .iter()
        .filter(|c| {
            c.classification != CloneClassification::InLoop
                && c.classification != CloneClassification::Necessary
        })
        .collect();

    if !loop_clones.is_empty() {
        let mut msg = format!(
            "CLONE DETECTOR - {} clone(s) in loops\n\n",
            loop_clones.len()
        );
        for c in &loop_clones {
            msg.push_str(&format!(
                "Line {}: {}\n  Suggestion: {}\n\n",
                c.line, c.code, c.suggestion
            ));
        }
        msg.push_str("Clones in loops multiply cost per iteration.\n");
        msg.push_str("Justify with // CLONE: or hoist outside loop.");
        exit_block(&msg);
    }

    if !other_clones.is_empty() {
        let mut msg = format!(
            "CLONE DETECTOR - {} clone(s) to review\n\n",
            other_clones.len()
        );
        for c in &other_clones {
            msg.push_str(&format!(
                "Line {}: {}\n  Classification: {:?}\n  Suggestion: {}\n\n",
                c.line, c.code, c.classification, c.suggestion
            ));
        }
        exit_warn(&msg);
    }

    exit_success_auto();
}
