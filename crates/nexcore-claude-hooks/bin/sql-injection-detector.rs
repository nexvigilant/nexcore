//! SQL Injection Detector - Security Hook
//!
//! PreToolUse hook that detects SQL injection patterns.
//! Catches format! macros with SQL keywords containing placeholders.
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

const HOOK_NAME: &str = "sql-injection-detector";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };
    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);
    require_rust_file(file_path);

    let content = content_or_pass_limited(&input);

    let sql_format_re = regex_or_pass(
        r#"format!\s*\(\s*"[^"]*(?i)(SELECT|INSERT|UPDATE|DELETE|DROP|CREATE|ALTER)[^"]*\{[^}]*\}[^"]*""#,
    );
    let sql_concat_re = regex_or_pass(r#"(?i)"(SELECT|INSERT|UPDATE|DELETE)[^"]*"\s*\+"#);

    let patterns: Vec<(&str, &regex::Regex)> = vec![
        ("format! with SQL", &sql_format_re),
        ("SQL string concatenation", &sql_concat_re),
    ];
    let result = scan_lines_multi(&content, &patterns, false);

    if result.is_empty() {
        pass();
    }

    let mut msg = format_violations("Potential SQL injection detected", &result.violations);
    append_scan_notice(
        &mut msg,
        &content,
        result.truncated_lines,
        result.truncated_hits,
    );
    msg.push_str("\nDynamic SQL construction is vulnerable to injection.\n");
    msg.push_str("Use parameterized queries instead:\n\n");
    msg.push_str("  // BAD: format!(\"SELECT * FROM users WHERE id = {}\", id)\n");
    msg.push_str("  // GOOD: sqlx::query!(\"SELECT * FROM users WHERE id = ?\", id)\n");
    msg.push_str("  // GOOD: sqlx::query_as!(User, \"SELECT * FROM users WHERE id = $1\", id)");

    // Emit cytokine signal before blocking (IL-1 = alarm, security threat)
    emit_error(
        HOOK_NAME,
        "SQL injection pattern detected",
        HookSeverity::Critical,
    );

    block(&msg);
}
