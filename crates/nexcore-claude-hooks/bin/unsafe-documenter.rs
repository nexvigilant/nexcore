//! Unsafe Documenter - Security Hook
//!
//! PreToolUse hook that requires SAFETY: comments for unsafe blocks.
//! Every unsafe block must document why it's safe.
//!
//! # Codex Compliance
//! - **Tier**: T3 (Security Hook)
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

const HOOK_NAME: &str = "unsafe-documenter";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };
    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);
    require_rust_file(file_path);

    let content = content_or_pass_limited(&input);
    let result = scan_undocumented_unsafe(&content);

    if result.is_empty() {
        pass();
    }

    let mut msg = format_violations("unsafe block without SAFETY: comment", &result.violations);
    append_scan_notice(
        &mut msg,
        &content,
        result.truncated_lines,
        result.truncated_hits,
    );
    msg.push_str("\nEvery unsafe block must document why it's safe.\n");
    msg.push_str("Add a comment in the 3 lines before the unsafe block:\n\n");
    msg.push_str("  // SAFETY: [Explain why this is safe]\n");
    msg.push_str("  // - Invariant 1: [why it holds]\n");
    msg.push_str("  // - Invariant 2: [why it holds]\n");
    msg.push_str("  unsafe { ... }");

    // Emit cytokine signal before blocking (TNF-alpha = terminate)
    let tool_name = input.tool_name.map(|t| t.to_string()).unwrap_or_default();
    emit_tool_blocked(
        &tool_name,
        HOOK_NAME,
        "unsafe block without SAFETY: comment",
    );

    block(&msg);
}

fn scan_undocumented_unsafe(content: &ContentSlice<'_>) -> ScanResult {
    let unsafe_re = regex_or_pass(r"unsafe\s*\{");
    let safety_re = regex_or_pass(r"//\s*SAFETY:");
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
        if !unsafe_re.is_match(line) {
            continue;
        }
        if has_safety_comment(&lines, line_num, &safety_re) {
            continue;
        }
        let snippet = truncate_line(line, snip_len);
        let evidence = EvidenceLine::new(line_num + 1, "unsafe block", snippet);
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

fn has_safety_comment(lines: &[&str], line_num: usize, safety_re: &Regex) -> bool {
    (line_num.saturating_sub(3)..line_num).any(|i| safety_re.is_match(lines[i]))
}
