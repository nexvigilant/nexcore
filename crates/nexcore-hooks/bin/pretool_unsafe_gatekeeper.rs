//! Unsafe Block Gatekeeper
//!
//! Requires documented SAFETY comments for all unsafe blocks.
//!
//! Safety Axiom: Unsafe Code Conservation
//! - All unsafe blocks must document their safety invariants
//! - Prevents undocumented memory safety risks

use nexcore_hooks::{exit_block, exit_success_auto, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip enforcement in plan mode
    if input.is_plan_mode() {
        exit_success_auto();
    }

    // Only check Rust files
    if !input.is_rust_file() {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let violations = check_unsafe_blocks(content);
    if violations.is_empty() {
        exit_success_auto();
    }

    let mut msg = String::from("UNSAFE BLOCK REQUIRES DOCUMENTATION\n\n");
    for line_num in &violations {
        msg.push_str(&format!(
            "Line {line_num}: unsafe block without SAFETY comment\n"
        ));
    }
    msg.push_str("\nREQUIRED structure:\n");
    msg.push_str("  // SAFETY: This unsafe block is sound because:\n");
    msg.push_str("  // 1. [Invariant]: explanation\n");
    msg.push_str("  // 2. [Invariant]: explanation\n");
    // Build the unsafe keyword dynamically to avoid self-detection
    let unsafe_kw = format!("un{}e", "saf");
    msg.push_str(&format!("  {} {{ ... }}\n", unsafe_kw));
    exit_block(&msg);
}

fn check_unsafe_blocks(content: &str) -> Vec<usize> {
    let lines: Vec<&str> = content.lines().collect();
    let mut violations = Vec::new();
    // Build marker at runtime to avoid self-detection
    let marker = format!("un{}e {{", "saf");

    for (i, line) in lines.iter().enumerate() {
        if line.contains(&marker) {
            // Check previous 1-2 lines for SAFETY comment
            let has_safety = (i > 0
                && lines
                    .get(i.saturating_sub(1))
                    .is_some_and(|l| l.contains("// SAFETY:")))
                || (i > 1
                    && lines
                        .get(i.saturating_sub(2))
                        .is_some_and(|l| l.contains("// SAFETY:")));

            if !has_safety {
                violations.push(i + 1);
            }
        }
    }

    violations
}
