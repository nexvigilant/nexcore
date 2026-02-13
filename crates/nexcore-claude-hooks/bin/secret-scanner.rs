//! Secret Scanner - Security Hook
//!
//! PreToolUse hook that blocks credentials and secrets in source code.
//! Detects API keys, tokens, private keys, and database credentials.
//!
//! Action: Block on secret detection
//! Exit: 0 = pass, 2 = block
//!
//! # Cytokine Integration
//! - **Block**: Emits IL-1 (alarm) - critical security threat detected

use nexcore_hook_lib::cytokine::{HookSeverity, emit_error};
use nexcore_hook_lib::{
    append_scan_notice, block, content_or_pass_limited, file_path_or_pass, format_violations,
    is_test_path, pass, read_input, require_edit_tool, require_rust_file, scan_lines_multi,
    secret_patterns,
};

const HOOK_NAME: &str = "secret-scanner";

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
    let owned_patterns = secret_patterns();
    let patterns: Vec<(&str, &regex::Regex)> = owned_patterns
        .iter()
        .map(|(name, re)| (*name, re))
        .collect();
    let result = scan_lines_multi(&content, &patterns, true);

    if result.is_empty() {
        pass();
    }

    let mut msg = format_violations("Potential secret/credential detected", &result.violations);
    append_scan_notice(
        &mut msg,
        &content,
        result.truncated_lines,
        result.truncated_hits,
    );
    msg.push_str("\nSecrets should never be in source code.\n");
    msg.push_str("Use environment variables or a secrets manager instead.");

    // Emit IL-1 (alarm) cytokine - critical security threat
    emit_error(
        HOOK_NAME,
        "secret/credential detected in source",
        HookSeverity::Critical,
    );

    block(&msg);
}
