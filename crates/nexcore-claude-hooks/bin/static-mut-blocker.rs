//! Static Mut Blocker - Security Hook
//!
//! PreToolUse hook that blocks `static mut` declarations.
//! Static mut is inherently unsafe and causes data races.
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

const HOOK_NAME: &str = "static-mut-blocker";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };
    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);
    require_rust_file(file_path);

    let content = content_or_pass_limited(&input);
    let re = regex_or_pass(r"static\s+mut\s+\w+");
    let result = scan_lines(&content, &re, "static mut", None);

    if result.is_empty() {
        pass();
    }

    let mut msg = format_violations("static mut declaration detected", &result.violations);
    append_scan_notice(
        &mut msg,
        &content,
        result.truncated_lines,
        result.truncated_hits,
    );
    msg.push_str("\n`static mut` is inherently unsafe and causes data races.\n");
    msg.push_str("Use thread-safe alternatives instead:\n\n");
    msg.push_str("  - std::sync::OnceLock<T> for lazy initialization\n");
    msg.push_str("  - std::sync::atomic::Atomic* for primitive values\n");
    msg.push_str("  - std::sync::Mutex<T> for complex data\n");
    msg.push_str("  - thread_local! {} for thread-local state");

    // Emit cytokine signal before blocking (IL-1 = alarm, security threat)
    emit_error(HOOK_NAME, "static mut detected", HookSeverity::Critical);

    block(&msg);
}
