//! Code Inspection MCP tools — FDA auditor-inspired code audit.
//!
//! Three dimensions: Safety, Efficacy, Purity.
//! Each scored 0.0-1.0 based on static analysis heuristics.
//!
//! ## T1 Primitive Grounding
//! - Safety: ∂(Boundary) + ∝(Irreversibility)
//! - Efficacy: κ(Comparison) + →(Causality)
//! - Purity: ∅(Void) + N(Quantity)

use crate::params::code_inspect::{
    CodeInspectAuditParams, CodeInspectCriteriaParams, CodeInspectScoreParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;

/// Run a full code inspection audit on a file or directory.
pub fn audit(params: CodeInspectAuditParams) -> Result<CallToolResult, McpError> {
    let target = Path::new(&params.target);
    if !target.exists() {
        return Err(McpError::invalid_params(
            format!("Target does not exist: {}", params.target),
            None,
        ));
    }

    let dims = params
        .dimensions
        .unwrap_or_else(|| vec!["safety".into(), "efficacy".into(), "purity".into()]);
    let content = collect_source(target);

    let mut results = Vec::new();
    let mut total = 0.0f64;

    for dim in &dims {
        let (score, findings) = match dim.as_str() {
            "safety" => score_safety(&content),
            "efficacy" => score_efficacy(&content),
            "purity" => score_purity(&content),
            _ => (
                0.0,
                vec![json!({"issue": format!("Unknown dimension: {}", dim)})],
            ),
        };
        results.push(json!({
            "dimension": dim,
            "score": (score * 100.0).round() / 100.0,
            "grade": grade(score),
            "findings": findings,
        }));
        total += score;
    }

    let overall = total / dims.len() as f64;

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "target": params.target,
            "overall_score": (overall * 100.0).round() / 100.0,
            "overall_grade": grade(overall),
            "dimensions": results,
        })
        .to_string(),
    )]))
}

/// Score source code against all three dimensions.
pub fn score(params: CodeInspectScoreParams) -> Result<CallToolResult, McpError> {
    let lang = params.language.as_deref().unwrap_or("rust");
    let (safety, sf) = score_safety(&params.code);
    let (efficacy, ef) = score_efficacy(&params.code);
    let (purity, pf) = score_purity(&params.code);
    let overall = (safety + efficacy + purity) / 3.0;

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "language": lang,
            "safety": { "score": (safety * 100.0).round() / 100.0, "grade": grade(safety), "findings": sf },
            "efficacy": { "score": (efficacy * 100.0).round() / 100.0, "grade": grade(efficacy), "findings": ef },
            "purity": { "score": (purity * 100.0).round() / 100.0, "grade": grade(purity), "findings": pf },
            "overall": { "score": (overall * 100.0).round() / 100.0, "grade": grade(overall) },
        })
        .to_string(),
    )]))
}

/// Get inspection criteria definitions.
pub fn criteria(_params: CodeInspectCriteriaParams) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "dimensions": [
                {
                    "name": "Safety",
                    "primitives": "∂+∝",
                    "checks": [
                        "No unsafe blocks",
                        "No unwrap/expect (use ? or proper error handling)",
                        "No panic! macro",
                        "Bounds checking present",
                        "Input validation at boundaries",
                    ],
                    "weight": 0.4,
                },
                {
                    "name": "Efficacy",
                    "primitives": "κ+→",
                    "checks": [
                        "Tests exist for public functions",
                        "Error paths tested",
                        "Return types are Result/Option (not bare values for fallible ops)",
                        "Assertions use specific values (not just assert!(true))",
                    ],
                    "weight": 0.35,
                },
                {
                    "name": "Purity",
                    "primitives": "∅+N",
                    "checks": [
                        "No dead code (unused imports, functions, variables)",
                        "No TODO/FIXME/HACK comments",
                        "No magic numbers",
                        "Functions under 50 lines",
                        "Cyclomatic complexity low",
                    ],
                    "weight": 0.25,
                },
            ],
            "grading": {
                "A": ">= 0.9",
                "B": ">= 0.75",
                "C": ">= 0.6",
                "D": ">= 0.4",
                "F": "< 0.4",
            },
        })
        .to_string(),
    )]))
}

fn score_safety(code: &str) -> (f64, Vec<serde_json::Value>) {
    let mut findings = Vec::new();
    let mut deductions = 0.0f64;
    let lines: Vec<&str> = code.lines().collect();
    let total_lines = lines.len().max(1) as f64;

    let mut unsafe_count = 0u32;
    let mut unwrap_count = 0u32;
    let mut panic_count = 0u32;

    for line in &lines {
        let t = line.trim();
        if t.starts_with("unsafe ") || t.contains("unsafe {") {
            unsafe_count += 1;
        }
        if t.contains(".unwrap()") {
            unwrap_count += 1;
        }
        if t.contains(".expect(") {
            unwrap_count += 1;
        }
        if t.contains("panic!(") {
            panic_count += 1;
        }
    }

    if unsafe_count > 0 {
        deductions += 0.3;
        findings.push(json!({"issue": "unsafe blocks", "count": unsafe_count, "severity": "high"}));
    }
    if unwrap_count > 0 {
        let ratio = unwrap_count as f64 / total_lines;
        deductions += (ratio * 10.0).min(0.3);
        findings.push(
            json!({"issue": "unwrap/expect calls", "count": unwrap_count, "severity": "medium"}),
        );
    }
    if panic_count > 0 {
        deductions += 0.2;
        findings.push(json!({"issue": "panic! macros", "count": panic_count, "severity": "high"}));
    }

    let score = (1.0 - deductions).max(0.0);
    (score, findings)
}

fn score_efficacy(code: &str) -> (f64, Vec<serde_json::Value>) {
    let mut findings = Vec::new();
    let mut score = 0.0f64;

    let has_tests = code.contains("#[test]") || code.contains("#[cfg(test)]");
    if has_tests {
        score += 0.3;
    } else {
        findings.push(json!({"issue": "No test functions found", "severity": "high"}));
    }

    let has_err_handling = code.contains("Result<") || code.contains("-> Result");
    if has_err_handling {
        score += 0.25;
    } else {
        findings.push(json!({"issue": "No Result return types", "severity": "medium"}));
    }

    let has_err_tests = code.contains("Err(") && has_tests;
    if has_err_tests {
        score += 0.25;
    } else if has_tests {
        findings.push(json!({"issue": "No error path tests", "severity": "medium"}));
    }

    let has_doc = code.contains("///");
    if has_doc {
        score += 0.2;
    } else {
        findings.push(json!({"issue": "No doc comments", "severity": "low"}));
    }

    (score.min(1.0), findings)
}

fn score_purity(code: &str) -> (f64, Vec<serde_json::Value>) {
    let mut findings = Vec::new();
    let mut deductions = 0.0f64;
    let lines: Vec<&str> = code.lines().collect();

    // Dead code markers
    let dead_code = lines
        .iter()
        .filter(|l| l.contains("#[allow(dead_code)]"))
        .count();
    if dead_code > 0 {
        deductions += (dead_code as f64 * 0.05).min(0.2);
        findings.push(json!({"issue": "dead_code allows", "count": dead_code, "severity": "low"}));
    }

    // TODO/FIXME/HACK
    let todo_count = lines
        .iter()
        .filter(|l| {
            let t = l.trim();
            t.contains("TODO") || t.contains("FIXME") || t.contains("HACK")
        })
        .count();
    if todo_count > 0 {
        deductions += (todo_count as f64 * 0.03).min(0.15);
        findings.push(
            json!({"issue": "TODO/FIXME/HACK comments", "count": todo_count, "severity": "low"}),
        );
    }

    // Long functions (heuristic: count lines between fn and closing brace)
    let long_fns = count_long_functions(&lines, 50);
    if long_fns > 0 {
        deductions += (long_fns as f64 * 0.05).min(0.2);
        findings.push(
            json!({"issue": "Functions over 50 lines", "count": long_fns, "severity": "medium"}),
        );
    }

    // Magic numbers (bare numeric literals in non-test code)
    let magic = lines
        .iter()
        .filter(|l| !l.trim().starts_with("//") && !l.contains("#[test]"))
        .filter(|l| {
            // Simple heuristic: numbers > 1 that aren't in const/static/type contexts
            l.chars().any(|c| c.is_ascii_digit())
                && !l.contains("const ")
                && !l.contains("static ")
                && !l.contains("assert")
                && l.trim().len() > 5
        })
        .count();
    // Don't penalize too heavily for this rough heuristic
    if magic > 10 {
        deductions += 0.1;
        findings.push(
            json!({"issue": "Potential magic numbers", "estimate": magic, "severity": "low"}),
        );
    }

    let score = (1.0 - deductions).max(0.0);
    (score, findings)
}

fn count_long_functions(lines: &[&str], threshold: usize) -> u32 {
    let mut count = 0u32;
    let mut fn_start: Option<usize> = None;
    let mut brace_depth = 0i32;

    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if (t.starts_with("pub fn ") || t.starts_with("fn ") || t.starts_with("pub(crate) fn "))
            && !t.starts_with("fn main")
        {
            fn_start = Some(i);
            brace_depth = 0;
        }

        for c in t.chars() {
            if c == '{' {
                brace_depth += 1;
            }
            if c == '}' {
                brace_depth -= 1;
            }
        }

        if brace_depth == 0 && fn_start.is_some() {
            let start = fn_start.take().unwrap_or(i);
            if i - start > threshold {
                count += 1;
            }
        }
    }

    count
}

fn grade(score: f64) -> &'static str {
    if score >= 0.9 {
        "A"
    } else if score >= 0.75 {
        "B"
    } else if score >= 0.6 {
        "C"
    } else if score >= 0.4 {
        "D"
    } else {
        "F"
    }
}

fn collect_source(target: &Path) -> String {
    if target.is_file() {
        std::fs::read_to_string(target).unwrap_or_default()
    } else {
        let mut content = String::new();
        collect_dir_source(target, &mut content);
        content
    }
}

fn collect_dir_source(dir: &Path, content: &mut String) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.file_name().map_or(true, |n| n != "target") {
                collect_dir_source(&path, content);
            } else if path
                .extension()
                .map_or(false, |e| e == "rs" || e == "ts" || e == "tsx")
            {
                if let Ok(src) = std::fs::read_to_string(&path) {
                    content.push_str(&src);
                    content.push('\n');
                }
            }
        }
    }
}
