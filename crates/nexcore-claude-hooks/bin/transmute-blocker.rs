//! Transmute Blocker - Security Hook
//!
//! PreToolUse hook that blocks all `mem::transmute` usage.
//! Transmute is fundamentally unsafe and bypasses type safety.
//!
//! # Codex Compliance
//! - **Tier**: T3 (Security Hook)
//! - **Commandments**: VI (Match), VII (Type)
//!
//! # Cytokine Integration
//! - **Block**: Emits IL-1 (alarm) via cytokine bridge — security threat
//! - **Pass**: No emission (homeostasis maintained)

use nexcore_hook_lib::cytokine::{HookSeverity, emit_error};
use nexcore_hook_lib::{
    append_scan_notice, block, content_or_pass_limited, file_path_or_pass, format_violations, pass,
    read_input, regex_or_pass, require_edit_tool, require_rust_file, scan_lines,
};

const HOOK_NAME: &str = "transmute-blocker";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };
    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);
    require_rust_file(file_path);

    let content = content_or_pass_limited(&input);
    let re = regex_or_pass(r"(mem::)?transmute\s*[:<(]");
    let result = scan_lines(&content, &re, "mem::transmute", None);

    if result.is_empty() {
        pass();
    }

    let mut msg = format_violations("mem::transmute detected", &result.violations);
    append_scan_notice(
        &mut msg,
        &content,
        result.truncated_lines,
        result.truncated_hits,
    );
    msg.push_str("\nmem::transmute bypasses ALL type safety guarantees.\n");
    msg.push_str("It can cause:\n");
    msg.push_str("  - Type confusion vulnerabilities\n");
    msg.push_str("  - Memory corruption\n");
    msg.push_str("  - Undefined behavior\n\n");
    msg.push_str("Alternatives:\n");
    msg.push_str("  - as casts for numeric conversions\n");
    msg.push_str("  - From/Into traits for safe conversions\n");
    msg.push_str("  - bytemuck crate for zero-copy casts");

    // Emit cytokine signal before blocking (IL-1 = alarm, security threat)
    emit_error(HOOK_NAME, "mem::transmute detected", HookSeverity::Critical);

    block(&msg);
}
