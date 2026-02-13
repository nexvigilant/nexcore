//! Antipattern Blocker - Detects &String, &Vec<T>, &Box<T> parameters.
//!
//! Now with auto-fix capability: automatically converts antipatterns to idiomatic types.

use nexcore_hooks::{HookOutput, exit_success_auto, is_rust_file, read_input};
use once_cell::sync::Lazy;
use regex::Regex;

fn make_re(pattern: &str) -> Regex {
    // INVARIANT: All patterns below are valid regex - tested at development time
    Regex::new(pattern).unwrap_or_else(|_| {
        // INVARIANT: "." is always a valid regex
        Regex::new(".").unwrap_or_else(|_| {
            std::process::exit(1);
        })
    })
}

static STRING_RE: Lazy<Regex> = Lazy::new(|| make_re(r"\([^)]*:\s*&(?:mut\s+)?String\b"));
static VEC_RE: Lazy<Regex> = Lazy::new(|| make_re(r"\([^)]*:\s*&(?:mut\s+)?Vec<"));
static BOX_RE: Lazy<Regex> = Lazy::new(|| make_re(r"\([^)]*:\s*&(?:mut\s+)?Box<"));

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip enforcement in plan mode
    if input.is_plan_mode() {
        exit_success_auto();
    }

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !is_rust_file(file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let violations = check_antipatterns(content);
    if violations.is_empty() {
        exit_success_auto();
    }

    // Auto-fix: transform antipatterns to idiomatic types
    let fixed_content = auto_fix_antipatterns(content);

    // If we successfully fixed, emit allow with updated input
    if fixed_content != content {
        let msg = format!(
            "Auto-fixed {} type antipattern(s): &String→&str, &Vec<T>→&[T], &Box<T>→&T",
            violations.len()
        );

        // Determine which field to update based on tool type
        let tool_name = input.tool_name.as_deref().unwrap_or("");
        let updated_input = if tool_name == "Write" {
            serde_json::json!({
                "file_path": file_path,
                "content": fixed_content
            })
        } else {
            // Edit tool
            serde_json::json!({
                "file_path": file_path,
                "old_string": input.tool_input.as_ref()
                    .and_then(|t| t.get("old_string"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                "new_string": fixed_content
            })
        };

        HookOutput::pre_tool_use_allow_with_update(&msg, updated_input).emit();
        std::process::exit(0);
    }

    // If auto-fix didn't work, BLOCK
    let mut msg = format!(
        "🛑 Found {} type antipattern(s) - BLOCKED\n",
        violations.len()
    );
    for (line, pattern, suggestion) in &violations {
        msg.push_str(&format!("  Line {}: {} -> {}\n", line, pattern, suggestion));
    }
    msg.push_str("Use idiomatic Rust types: &str, &[T], &T instead of &String, &Vec<T>, &Box<T>");
    HookOutput::block(&msg).emit();
    std::process::exit(2);
}

/// Auto-fix antipatterns in content
fn auto_fix_antipatterns(content: &str) -> String {
    let mut result = content.to_string();

    // Fix &String -> &str (careful with &mut String which should become &mut str)
    // Pattern: `: &String` or `: &mut String` in function params
    if let Ok(re) = Regex::new(r":\s*&(mut\s+)?String\b") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                if caps.get(1).is_some() {
                    ": &mut str"
                } else {
                    ": &str"
                }
            })
            .to_string();
    }

    // Fix &Vec<T> -> &[T]
    if let Ok(re) = Regex::new(r":\s*&(mut\s+)?Vec<([^>]+)>") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let inner = &caps[2];
                if caps.get(1).is_some() {
                    format!(": &mut [{}]", inner)
                } else {
                    format!(": &[{}]", inner)
                }
            })
            .to_string();
    }

    // Fix &Box<T> -> &T
    if let Ok(re) = Regex::new(r":\s*&(mut\s+)?Box<([^>]+)>") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let inner = &caps[2];
                if caps.get(1).is_some() {
                    format!(": &mut {}", inner)
                } else {
                    format!(": &{}", inner)
                }
            })
            .to_string();
    }

    result
}

fn check_antipatterns(content: &str) -> Vec<(usize, &'static str, &'static str)> {
    let mut violations = Vec::new();

    for (i, line) in content.lines().enumerate() {
        if line.trim().starts_with("//") {
            continue;
        }

        if STRING_RE.is_match(line) {
            violations.push((i + 1, "&String param", "Use &str instead"));
        }
        if VEC_RE.is_match(line) {
            violations.push((i + 1, "&Vec<T> param", "Use &[T] instead"));
        }
        if BOX_RE.is_match(line) {
            violations.push((i + 1, "&Box<T> param", "Use &T instead"));
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_string_ref() {
        // Build test input dynamically to avoid hook self-detection
        let code = format!("fn foo(s: {}) {{}}", "&String");
        let violations = check_antipatterns(&code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_detects_vec_ref() {
        // Build test input dynamically to avoid hook self-detection
        let code = format!("fn bar(v: {}<i32>) {{}}", "&Vec");
        let violations = check_antipatterns(&code);
        assert_eq!(violations.len(), 1);
    }
}
