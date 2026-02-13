//! String Error Blocker - Atomic Hook
//!
//! PreToolUse hook that blocks `Result<T, String>` return types.
//! Encourages proper error types with thiserror or anyhow.
//!
//! # Codex Compliance
//! - **Tier**: T3 (Domain-Specific Hook)
//! - **Commandments**: VI (Match), VII (Type)
//!
//! # Cytokine Integration
//! - **Block**: Emits TNF-alpha (terminate) via cytokine bridge — quality violation
//! - **Pass**: No emission (homeostasis maintained)

use nexcore_hook_lib::cytokine::emit_tool_blocked;
use nexcore_hook_lib::{
    append_scan_notice, block, content_or_pass_limited, file_path_or_pass, format_violations, pass,
    read_input, regex_or_pass, require_edit_tool, require_rust_file, scan_lines,
};

const HOOK_NAME: &str = "string-error-blocker";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };
    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);
    require_rust_file(file_path);

    let content = content_or_pass_limited(&input);
    let re = regex_or_pass(r"Result\s*<[^,>]+,\s*String\s*>");
    let result = scan_lines(&content, &re, "Result<T, String>", None);

    if result.is_empty() {
        pass();
    }

    let mut msg = format_violations("Result<T, String> detected", &result.violations);
    append_scan_notice(
        &mut msg,
        &content,
        result.truncated_lines,
        result.truncated_hits,
    );
    msg.push_str("\nString errors lose type information and can't be pattern-matched.\n");
    msg.push_str("Use instead:\n");
    msg.push_str("  - thiserror::Error enum for library code\n");
    msg.push_str("  - anyhow::Result<T> for application code\n");
    msg.push_str("  - Custom error type implementing std::error::Error");

    // Emit cytokine signal before blocking (TNF-alpha = terminate)
    let tool_name = input.tool_name.map(|t| t.to_string()).unwrap_or_default();
    emit_tool_blocked(&tool_name, HOOK_NAME, "Result<T, String> detected");

    block(&msg);
}
