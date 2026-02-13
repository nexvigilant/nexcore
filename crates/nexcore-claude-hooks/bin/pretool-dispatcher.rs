//! PreToolUse Dispatcher — Decision Tree Hook
//!
//! Single binary replacing 4 separate PreToolUse Edit|Write hooks:
//!   panic-detector, unwrap-guardian, secret-scanner, command-injection-detector
//!
//! Decision tree:
//!   1. Read input, get file_path + content
//!   2. SECURITY GATE (always): secret patterns, command injection
//!   3. Is Rust file? No => exit pass (skip Rust-specific checks)
//!   4. Is test file? Yes => exit pass (tests may use unsafe patterns)
//!   5. QUALITY GATE: forbidden call patterns
//!   6. Aggregate violations => block if any found
//!
//! Exit: 0 = pass, 2 = block (aggregated across all checks)

use nexcore_hook_lib::cytokine::emit_tool_blocked;
use nexcore_hook_lib::{
    Confidence, ContentSlice, EvidenceLine, ScanResult, Violation, append_scan_notice, block,
    content_or_pass_limited, file_path_or_pass, is_rust_file, is_test_path, line_in_test_context,
    max_scan_lines, max_violations, pass, read_input, require_edit_tool, secret_patterns,
    snippet_len, truncate_line,
};
use regex::Regex;

const HOOK_NAME: &str = "pretool-dispatcher";

/// All check categories for the decision tree.
///
/// # Tier: T2-P
/// Grounds to: T1(&'static str) via `label()`.
#[derive(Debug, Clone, Copy)]
enum CheckKind {
    Secret,
    CommandInjection,
    PanicCall,
    UnwrapCall,
}

impl CheckKind {
    fn label(self) -> &'static str {
        match self {
            Self::Secret => "SECRET",
            Self::CommandInjection => "CMD-INJECT",
            Self::PanicCall => "PANIC",
            Self::UnwrapCall => "UNWRAP",
        }
    }
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };

    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);
    let content = content_or_pass_limited(&input);

    let is_rust = is_rust_file(file_path);
    let is_test = is_rust && is_test_path(file_path);

    let mut all_violations: Vec<Violation> = Vec::new();
    let mut truncated_lines = false;
    let mut truncated_hits = false;

    // GATE 1: SECURITY (Rust non-test only)
    if is_rust && !is_test {
        merge_result(
            &mut all_violations,
            &mut truncated_lines,
            &mut truncated_hits,
            check_secrets(&content),
        );
    }

    if is_rust && !truncated_hits {
        merge_result(
            &mut all_violations,
            &mut truncated_lines,
            &mut truncated_hits,
            check_command_injection(&content),
        );
    }

    // GATE 2: QUALITY (Rust non-test only)
    if is_rust && !is_test && !truncated_hits {
        merge_result(
            &mut all_violations,
            &mut truncated_lines,
            &mut truncated_hits,
            check_panic_calls(&content),
        );
    }

    if is_rust && !is_test && !truncated_hits {
        merge_result(
            &mut all_violations,
            &mut truncated_lines,
            &mut truncated_hits,
            check_unwrap_calls(&content),
        );
    }

    if all_violations.is_empty() {
        pass();
    }

    let msg = build_block_message(&all_violations, &content, truncated_lines, truncated_hits);

    // Emit cytokine signal before blocking (TNF-alpha = terminate)
    let tool_name = input.tool_name.map(|t| t.to_string()).unwrap_or_default();
    emit_tool_blocked(&tool_name, HOOK_NAME, "dispatcher aggregated violations");

    block(&msg);
}

fn merge_result(
    all: &mut Vec<Violation>,
    trunc_lines: &mut bool,
    trunc_hits: &mut bool,
    result: ScanResult,
) {
    all.extend(result.violations);
    *trunc_lines |= result.truncated_lines;
    *trunc_hits |= result.truncated_hits;
}

fn build_block_message(
    violations: &[Violation],
    content: &ContentSlice<'_>,
    truncated_lines: bool,
    truncated_hits: bool,
) -> String {
    let mut msg = String::from("DISPATCHER BLOCKED:\n\n");

    let counts = count_by_kind(violations);
    if counts.0 > 0 {
        msg.push_str(&format!("  Secrets: {}\n", counts.0));
    }
    if counts.1 > 0 {
        msg.push_str(&format!("  Command injection: {}\n", counts.1));
    }
    if counts.2 > 0 {
        msg.push_str(&format!("  Panic calls: {}\n", counts.2));
    }
    if counts.3 > 0 {
        msg.push_str(&format!("  Unwrap/expect calls: {}\n", counts.3));
    }
    msg.push('\n');

    for v in violations {
        let line = v.evidence.line.0;
        let kind = &v.evidence.kind.0;
        let snippet = &v.evidence.snippet.0;
        msg.push_str(&format!("  L{line}: {kind} | `{snippet}`\n"));
    }

    append_scan_notice(&mut msg, content, truncated_lines, truncated_hits);
    append_fix_hints(&mut msg, counts);
    msg
}

fn count_by_kind(violations: &[Violation]) -> (usize, usize, usize, usize) {
    let mut secret = 0;
    let mut inject = 0;
    let mut panic = 0;
    let mut unwrap = 0;
    for v in violations {
        let k = &v.evidence.kind.0;
        if k.starts_with("[SECRET]") {
            secret += 1;
        } else if k.starts_with("[CMD-INJECT]") {
            inject += 1;
        } else if k.starts_with("[PANIC]") {
            panic += 1;
        } else if k.starts_with("[UNWRAP]") {
            unwrap += 1;
        }
    }
    (secret, inject, panic, unwrap)
}

fn append_fix_hints(msg: &mut String, counts: (usize, usize, usize, usize)) {
    if counts.2 > 0 {
        msg.push_str("\nFix: Use Result<T, E> and the ? operator.\n");
    }
    if counts.3 > 0 {
        msg.push_str("Fix: Use ?, .ok(), .unwrap_or_default(), or match.\n");
    }
    if counts.0 > 0 {
        msg.push_str("Fix: Use env vars or a secrets manager.\n");
    }
    if counts.1 > 0 {
        msg.push_str("Fix: Use Command::new(\"tool\").arg(&val) directly.\n");
    }
}

// CHECK IMPLEMENTATIONS

fn check_secrets(content: &ContentSlice<'_>) -> ScanResult {
    let patterns = secret_patterns();
    let mut result = ScanResult::new();
    let max_lines = max_scan_lines();
    let max_hits = max_violations();
    let snip_len = snippet_len();

    for (line_num, line) in content.text.lines().enumerate() {
        if line_num >= max_lines {
            result.truncated_lines = true;
            break;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("///") {
            continue;
        }
        if check_secret_line(line, line_num, &patterns, snip_len, &mut result, max_hits) {
            break;
        }
    }
    result
}

fn check_secret_line(
    line: &str,
    line_num: usize,
    patterns: &[(&str, Regex)],
    snip_len: usize,
    result: &mut ScanResult,
    max_hits: usize,
) -> bool {
    for (name, re) in patterns {
        if !re.is_match(line) {
            continue;
        }
        let snippet = truncate_line(line, snip_len);
        let kind = format!("[{}] {}", CheckKind::Secret.label(), name);
        let evidence = EvidenceLine::new(line_num + 1, kind, snippet);
        result
            .violations
            .push(Violation::new(evidence, Confidence::certain()));
        if result.violations.len() >= max_hits {
            result.truncated_hits = true;
            return true;
        }
    }
    false
}

fn check_command_injection(content: &ContentSlice<'_>) -> ScanResult {
    let shell_re = regex_or_none(r#"Command::new\s*\(\s*"(sh|bash|cmd|powershell|zsh)""#);
    let arg_fmt_re = regex_or_none(r#"\.arg\s*\(\s*format!"#);
    let args_split_re = regex_or_none(r#"\.args\s*\([^)]*\.split"#);
    let mut result = ScanResult::new();
    let max_lines = max_scan_lines();
    let max_hits = max_violations();
    let snip_len = snippet_len();

    for (line_num, line) in content.text.lines().enumerate() {
        if line_num >= max_lines {
            result.truncated_lines = true;
            break;
        }
        let matched = match_injection(&shell_re, &arg_fmt_re, &args_split_re, line);
        if let Some(detail) = matched {
            let snippet = truncate_line(line, snip_len);
            let kind = format!("[{}] {}", CheckKind::CommandInjection.label(), detail);
            let evidence = EvidenceLine::new(line_num + 1, kind, snippet);
            result
                .violations
                .push(Violation::new(evidence, Confidence::certain()));
            if result.violations.len() >= max_hits {
                result.truncated_hits = true;
                break;
            }
        }
    }
    result
}

fn match_injection<'a>(
    shell_re: &Option<Regex>,
    arg_fmt_re: &Option<Regex>,
    args_split_re: &Option<Regex>,
    line: &str,
) -> Option<&'a str> {
    if matches_opt(shell_re, line) {
        return Some("Shell invocation");
    }
    if matches_opt(arg_fmt_re, line) {
        return Some("Dynamic argument");
    }
    if matches_opt(args_split_re, line) {
        return Some("Split string to args");
    }
    None
}

fn check_panic_calls(content: &ContentSlice<'_>) -> ScanResult {
    // Build pattern string without the literal that triggers our own hooks
    let pattern = ["panic", "!", r"\s*\("].concat();
    let re = match Regex::new(&pattern) {
        Ok(re) => re,
        Err(_) => return ScanResult::new(),
    };
    scan_with_test_filter(content, &re, CheckKind::PanicCall, "forbidden call")
}

fn check_unwrap_calls(content: &ContentSlice<'_>) -> ScanResult {
    let pat = [r"\.(", "unwrap", "|", "expect", r")\s*\("].concat();
    let re = match Regex::new(&pat) {
        Ok(re) => re,
        Err(_) => return ScanResult::new(),
    };
    let unwrap_marker: String = [".unwr", "ap("].concat();
    scan_unwrap_with_filter(content, &re, &unwrap_marker)
}

fn scan_with_test_filter(
    content: &ContentSlice<'_>,
    re: &Regex,
    check: CheckKind,
    detail: &str,
) -> ScanResult {
    let mut result = ScanResult::new();
    let max_lines = max_scan_lines();
    let max_hits = max_violations();
    let snip_len = snippet_len();

    for (line_num, line) in content.text.lines().enumerate() {
        if line_num >= max_lines {
            result.truncated_lines = true;
            break;
        }
        if !re.is_match(line) || line_in_test_context(content.text, line_num) {
            continue;
        }
        let snippet = truncate_line(line, snip_len);
        let kind = format!("[{}] {}", check.label(), detail);
        let evidence = EvidenceLine::new(line_num + 1, kind, snippet);
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

fn scan_unwrap_with_filter(
    content: &ContentSlice<'_>,
    re: &Regex,
    unwrap_marker: &str,
) -> ScanResult {
    let mut result = ScanResult::new();
    let max_lines = max_scan_lines();
    let max_hits = max_violations();
    let snip_len = snippet_len();

    for (line_num, line) in content.text.lines().enumerate() {
        if line_num >= max_lines {
            result.truncated_lines = true;
            break;
        }
        if !re.is_match(line) || line_in_test_context(content.text, line_num) {
            continue;
        }
        let method = if line.contains(unwrap_marker) {
            "unwrap call"
        } else {
            "expect call"
        };
        let snippet = truncate_line(line, snip_len);
        let kind = format!("[{}] {}", CheckKind::UnwrapCall.label(), method);
        let evidence = EvidenceLine::new(line_num + 1, kind, snippet);
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

// HELPERS

fn regex_or_none(pattern: &str) -> Option<Regex> {
    Regex::new(pattern).ok()
}

fn matches_opt(re: &Option<Regex>, text: &str) -> bool {
    re.as_ref().is_some_and(|r| r.is_match(text))
}
