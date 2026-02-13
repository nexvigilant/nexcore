//! Panic Detector - Atomic Hook
//!
//! PreToolUse hook that blocks `panic!()` in non-test Rust code.
//! Test files and #[test] contexts are allowed.
//!
//! Action: Block panic! outside tests
//! Exit: Verdict::Pass or Verdict::Block
//!
//! # Cytokine Integration
//! - **Block**: Emits TNF-alpha (terminate) via cytokine bridge

use nexcore_hook_lib::cytokine::emit_tool_blocked;
use nexcore_hook_lib::{
    append_scan_notice, block, content_or_pass_limited, file_path_or_pass, format_violations,
    is_test_path, line_in_test_context, pass, read_input, regex_or_pass, require_edit_tool,
    require_rust_file, scan_lines,
};

const HOOK_NAME: &str = "panic-detector";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };

    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);
    require_rust_file(file_path);

    if is_test_path(file_path) {
        pass();
    }

    let content = content_or_pass_limited(&input);
    let re = regex_or_pass(r"panic!\s*\(");
    let result = scan_lines(&content, &re, "panic!", Some(line_in_test_context));

    if result.is_empty() {
        pass();
    }

    let mut msg = format_violations("panic!() detected in non-test code", &result.violations);
    append_scan_notice(
        &mut msg,
        &content,
        result.truncated_lines,
        result.truncated_hits,
    );
    msg.push_str("\nUse Result<T, E> and ? operator instead of panic!().\n");
    msg.push_str("If intentional, move to a #[test] function or #[cfg(test)] module.");

    // Emit cytokine signal before blocking (TNF-alpha = terminate)
    let tool_name = input.tool_name.map(|t| t.to_string()).unwrap_or_default();
    emit_tool_blocked(&tool_name, HOOK_NAME, "panic!() in non-test code");

    block(&msg);
}
