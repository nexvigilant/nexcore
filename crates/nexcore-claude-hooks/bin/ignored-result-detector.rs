//! Ignored Result Detector - Atomic Hook
//!
//! PreToolUse hook that blocks `let _ = expr` without explicit allow.
//! Prevents silent swallowing of Results/Options.
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
    Confidence, ContentSlice, EvidenceLine, ScanResult, Violation, append_scan_notice, block,
    content_or_pass_limited, file_path_or_pass, format_violations, max_scan_lines, max_violations,
    pass, read_input, regex_or_pass, require_edit_tool, require_rust_file, snippet_len,
    truncate_line,
};
use regex::Regex;

const HOOK_NAME: &str = "ignored-result-detector";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };
    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);
    require_rust_file(file_path);

    let content = content_or_pass_limited(&input);
    let result = scan_ignored_results(&content);

    if result.is_empty() {
        pass();
    }

    let mut msg = format_violations("Ignored result without #[allow]", &result.violations);
    append_scan_notice(
        &mut msg,
        &content,
        result.truncated_lines,
        result.truncated_hits,
    );
    msg.push_str("\n`let _ = expr` silently discards Results/Options.\n");
    msg.push_str("Either:\n");
    msg.push_str("  1. Handle the result: `result?` or `if let Err(e) = result { ... }`\n");
    msg.push_str("  2. Document intentional ignore:\n");
    msg.push_str("     #[allow(unused_results)] // Reason: best-effort save\n");
    msg.push_str("     let _ = state.save();");

    // Emit cytokine signal before blocking (TNF-alpha = terminate)
    let tool_name = input.tool_name.map(|t| t.to_string()).unwrap_or_default();
    emit_tool_blocked(&tool_name, HOOK_NAME, "ignored result without #[allow]");

    block(&msg);
}

fn scan_ignored_results(content: &ContentSlice<'_>) -> ScanResult {
    let ignored_re = regex_or_pass(r"let\s+_\s*=");
    let allow_re = regex_or_pass(r"#\[allow\(");
    let lines: Vec<&str> = content.text.lines().collect();
    let mut result = ScanResult::new();
    let max_lines = max_scan_lines();
    let max_hits = max_violations();
    let snip_len = snippet_len();

    for (line_num, line) in lines.iter().enumerate() {
        if line_num >= max_lines {
            result.truncated_lines = true;
            break;
        }
        if !ignored_re.is_match(line) {
            continue;
        }
        if has_allow_annotation(&lines, line_num, line, &allow_re) {
            continue;
        }
        let snippet = truncate_line(line, snip_len);
        let evidence = EvidenceLine::new(line_num + 1, "ignored result", snippet);
        result
            .violations
            .push(Violation::new(evidence, Confidence::certain()));
        if result.violations.len() >= max_hits {
            result.truncated_hits = true;
            break;
        }
    }
    result
}

fn has_allow_annotation(lines: &[&str], line_num: usize, line: &str, allow_re: &Regex) -> bool {
    let prev_has_allow = line_num > 0 && allow_re.is_match(lines[line_num - 1]);
    let inline_allow = line.contains("#[allow(");
    prev_has_allow || inline_allow
}
