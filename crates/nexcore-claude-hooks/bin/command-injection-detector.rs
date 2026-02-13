//! Command Injection Detector - Security Hook
//!
//! PreToolUse hook that detects command injection patterns.
//! Catches shell invocations with dynamic arguments.
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
    read_input, regex_or_pass, require_edit_tool, require_rust_file, scan_lines_multi,
};

const HOOK_NAME: &str = "command-injection-detector";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };
    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);
    require_rust_file(file_path);

    let content = content_or_pass_limited(&input);

    let shell_cmd_re = regex_or_pass(r#"Command::new\s*\(\s*"(sh|bash|cmd|powershell|zsh)""#);
    let arg_format_re = regex_or_pass(r#"\.arg\s*\(\s*format!"#);
    let args_split_re = regex_or_pass(r#"\.args\s*\([^)]*\.split"#);

    let patterns: Vec<(&str, &regex::Regex)> = vec![
        ("Shell invocation", &shell_cmd_re),
        ("Dynamic argument", &arg_format_re),
        ("Split string to args", &args_split_re),
    ];
    let result = scan_lines_multi(&content, &patterns, false);

    if result.is_empty() {
        pass();
    }

    let mut msg = format_violations("Potential command injection detected", &result.violations);
    append_scan_notice(
        &mut msg,
        &content,
        result.truncated_lines,
        result.truncated_hits,
    );
    msg.push_str("\nShell commands with dynamic input are injection vectors.\n");
    msg.push_str("Safe patterns:\n\n");
    msg.push_str("  // BAD: Command::new(\"sh\").arg(\"-c\").arg(format!(\"ls {}\", dir))\n");
    msg.push_str("  // GOOD: Command::new(\"ls\").arg(&dir)  // Direct invocation\n");
    msg.push_str("  // GOOD: Command::new(\"git\").args([\"clone\", \"--depth\", \"1\", url])");

    // Emit cytokine signal before blocking (IL-1 = alarm, security threat)
    emit_error(
        HOOK_NAME,
        "command injection pattern detected",
        HookSeverity::Critical,
    );

    block(&msg);
}
