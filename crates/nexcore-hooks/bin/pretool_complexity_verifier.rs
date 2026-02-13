//! # Complexity Bound Verifier (Hook 42)
//!
//! PreToolUse hook that enforces algorithmic complexity documentation and verification.
//! Prevents O(n²) algorithms from sneaking into O(n) code paths.
//!
//! ## Event
//! - **Hook**: PreToolUse
//! - **Matcher**: Write, Edit (Rust files only)
//! - **Skip**: Test files, plan mode
//!
//! ## Behavior
//!
//! 1. **Complexity Analysis**: Parses Rust functions for loop nesting, recursion, and iterator chains
//! 2. **Documentation Check**: Looks for `/// O(...)` complexity annotations
//! 3. **Mismatch Detection**: Compares documented vs inferred complexity
//!
//! ## Exit Codes
//!
//! | Code | Condition | Action |
//! |------|-----------|--------|
//! | 0 | No issues or non-Rust file | Allow |
//! | 1 | Undocumented O(n²) or worse | Warn (show in output) |
//! | 2 | Documented complexity doesn't match implementation | Block |
//!
//! ## Examples
//!
//! ### Passing (documented complexity)
//! ```rust
//! /// O(n) - single pass
//! fn sum_list(items: &[i32]) -> i32 {
//!     items.iter().sum()
//! }
//! ```
//!
//! ### Warning (undocumented high complexity)
//! ```rust
//! fn find_pairs(items: &[i32]) -> Vec<(i32, i32)> {
//!     // Nested loop inferred as O(n²) - needs documentation
//!     items.iter().flat_map(|a| items.iter().map(|b| (*a, *b))).collect()
//! }
//! ```
//!
//! ### Blocking (mismatch)
//! ```rust
//! /// O(n) - WRONG: actually O(n²)
//! fn nested_search(matrix: &[Vec<i32>], target: i32) -> bool {
//!     for row in matrix {
//!         for val in row {
//!             if *val == target { return true; }
//!         }
//!     }
//!     false
//! }
//! ```
//!
//! ## Complexity Inference
//!
//! | Pattern | Inferred Complexity |
//! |---------|---------------------|
//! | Single loop | O(n) |
//! | Nested loops (2 levels) | O(n²) |
//! | Nested loops (3 levels) | O(n³) |
//! | Recursive without memo | O(2^n) potential |
//! | `.iter().flat_map(...)` chains | Depends on nesting |
//!
//! ## Integration
//!
//! Works with `posttool_complexity_regression_detector` to prevent complexity regressions
//! across commits.

use nexcore_hooks::parser::performance::analyze_complexity;
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

    let annotations = analyze_complexity(content);
    if annotations.is_empty() {
        exit_success_auto();
    }

    // Separate mismatches from undocumented high complexity
    let mismatches: Vec<_> = annotations.iter().filter(|a| !a.matches).collect();
    let undocumented: Vec<_> = annotations
        .iter()
        .filter(|a| a.documented.is_none() && a.inferred.contains("n²"))
        .collect();

    if !mismatches.is_empty() {
        let mut msg = format!("COMPLEXITY MISMATCH - {} function(s)\n\n", mismatches.len());
        for a in &mismatches {
            msg.push_str(&format!(
                "Function: {} (line {})\n  Documented: {:?}\n  Inferred: {}\n\n",
                a.function, a.line, a.documented, a.inferred
            ));
        }
        msg.push_str("Documented complexity doesn't match implementation.\n");
        msg.push_str("Fix implementation or update documentation.");
        exit_block(&msg);
    }

    if !undocumented.is_empty() {
        let mut msg = format!(
            "HIGH COMPLEXITY - {} function(s) without docs\n\n",
            undocumented.len()
        );
        for a in &undocumented {
            msg.push_str(&format!(
                "Function: {} (line {})\n  Inferred: {} (confidence: {})\n\n",
                a.function, a.line, a.inferred, a.confidence
            ));
        }
        msg.push_str("Document complexity with /// O(n²) or optimize.");
        exit_warn(&msg);
    }

    exit_success_auto();
}
