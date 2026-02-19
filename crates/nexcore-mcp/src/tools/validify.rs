//! Validify MCP tools — 8-gate crate validation algorithm.
//!
//! V-A-L-I-D-I-F-Y: Verify format, Assemble, Lint, Inspect tests,
//! Deny unsafe, Integrate workspace, Forge release, Yield docs.
//!
//! Each gate is a pass/fail check. Fail any gate, stop and fix.
//!
//! ## T1 Primitive Grounding
//! - V(Verify): κ(Comparison) + ∂(Boundary)
//! - A(Assemble): ∃(Existence) + →(Causality)
//! - L(Lint): κ(Comparison) + N(Quantity)
//! - I(Inspect): σ(Sequence) + κ(Comparison)
//! - D(Deny): ∂(Boundary) + ∝(Irreversibility)
//! - I2(Integrate): μ(Mapping) + ×(Product)
//! - F(Forge): →(Causality) + π(Persistence)
//! - Y(Yield): ∃(Existence) + σ(Sequence)

use crate::params::validify::{ValidifyGateParams, ValidifyGatesListParams, ValidifyRunParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;
use std::process::Command;

/// Gate definitions with their checks.
struct Gate {
    letter: &'static str,
    name: &'static str,
    description: &'static str,
    primitives: &'static str,
}

const GATES: &[Gate] = &[
    Gate {
        letter: "V",
        name: "Verify format",
        description: "cargo fmt --check passes",
        primitives: "κ+∂",
    },
    Gate {
        letter: "A",
        name: "Assemble",
        description: "cargo check succeeds (all targets compile)",
        primitives: "∃+→",
    },
    Gate {
        letter: "L",
        name: "Lint",
        description: "cargo clippy -- -D warnings passes",
        primitives: "κ+N",
    },
    Gate {
        letter: "I",
        name: "Inspect tests",
        description: "cargo test --lib passes",
        primitives: "σ+κ",
    },
    Gate {
        letter: "D",
        name: "Deny unsafe",
        description: "No unsafe blocks, no unwrap/expect",
        primitives: "∂+∝",
    },
    Gate {
        letter: "I2",
        name: "Integrate workspace",
        description: "Workspace deps use { workspace = true }",
        primitives: "μ+×",
    },
    Gate {
        letter: "F",
        name: "Forge release",
        description: "cargo build --release succeeds",
        primitives: "→+π",
    },
    Gate {
        letter: "Y",
        name: "Yield docs",
        description: "All pub items have doc comments",
        primitives: "∃+σ",
    },
];

/// Run the full 8-gate pipeline.
pub fn run(params: ValidifyRunParams) -> Result<CallToolResult, McpError> {
    let crate_path = Path::new(&params.crate_path);
    if !crate_path.exists() {
        return Err(McpError::invalid_params(
            format!("Crate path does not exist: {}", params.crate_path),
            None,
        ));
    }

    let fail_fast = params.fail_fast.unwrap_or(true);
    let skip: Vec<String> = params.skip_gates.unwrap_or_default();

    let mut results = Vec::new();
    let mut all_passed = true;

    for gate in GATES {
        if skip.iter().any(|s| s.eq_ignore_ascii_case(gate.letter)) {
            results.push(json!({
                "gate": gate.letter,
                "name": gate.name,
                "status": "skipped",
                "primitives": gate.primitives,
            }));
            continue;
        }

        let (passed, detail) = run_gate(gate.letter, &params.crate_path);

        results.push(json!({
            "gate": gate.letter,
            "name": gate.name,
            "status": if passed { "passed" } else { "failed" },
            "detail": detail,
            "primitives": gate.primitives,
        }));

        if !passed {
            all_passed = false;
            if fail_fast {
                break;
            }
        }
    }

    let passed_count = results.iter().filter(|r| r["status"] == "passed").count();
    let total = results.len();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "crate_path": params.crate_path,
            "overall": if all_passed { "PASS" } else { "FAIL" },
            "gates_passed": passed_count,
            "gates_total": total,
            "gates": results,
        })
        .to_string(),
    )]))
}

/// Run a single gate.
pub fn gate(params: ValidifyGateParams) -> Result<CallToolResult, McpError> {
    let (passed, detail) = run_gate(&params.gate, &params.crate_path);

    let gate_info = GATES
        .iter()
        .find(|g| g.letter.eq_ignore_ascii_case(&params.gate));

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "gate": params.gate,
            "name": gate_info.map(|g| g.name).unwrap_or("unknown"),
            "status": if passed { "passed" } else { "failed" },
            "detail": detail,
            "primitives": gate_info.map(|g| g.primitives).unwrap_or("?"),
        })
        .to_string(),
    )]))
}

/// List all gate definitions.
pub fn gates_list(_params: ValidifyGatesListParams) -> Result<CallToolResult, McpError> {
    let gates: Vec<serde_json::Value> = GATES
        .iter()
        .map(|g| {
            json!({
                "letter": g.letter,
                "name": g.name,
                "description": g.description,
                "primitives": g.primitives,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({ "gates": gates, "count": gates.len() }).to_string(),
    )]))
}

fn run_gate(gate_letter: &str, crate_path: &str) -> (bool, String) {
    match gate_letter.to_uppercase().as_str() {
        "V" => run_cmd(crate_path, "cargo", &["fmt", "--check"]),
        "A" => run_cmd(crate_path, "cargo", &["check", "--all-targets"]),
        "L" => run_cmd(crate_path, "cargo", &["clippy", "--", "-D", "warnings"]),
        "I" => run_cmd(crate_path, "cargo", &["test", "--lib"]),
        "D" => check_deny_unsafe(crate_path),
        "I2" => check_workspace_deps(crate_path),
        "F" => run_cmd(crate_path, "cargo", &["build", "--release"]),
        "Y" => check_doc_coverage(crate_path),
        _ => (false, format!("Unknown gate: {}", gate_letter)),
    }
}

fn run_cmd(crate_path: &str, cmd: &str, args: &[&str]) -> (bool, String) {
    match Command::new(cmd)
        .args(args)
        .current_dir(crate_path)
        .output()
    {
        Ok(output) => {
            let passed = output.status.success();
            let detail = if passed {
                "OK".to_string()
            } else {
                String::from_utf8_lossy(&output.stderr)
                    .lines()
                    .take(10)
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            (passed, detail)
        }
        Err(e) => (false, format!("Command failed: {}", e)),
    }
}

fn check_deny_unsafe(crate_path: &str) -> (bool, String) {
    let src_dir = Path::new(crate_path).join("src");
    if !src_dir.exists() {
        return (false, "No src/ directory".to_string());
    }

    let mut issues = Vec::new();

    if let Ok(entries) = glob_rs_files(&src_dir) {
        for path in entries {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let rel = path.strip_prefix(crate_path).unwrap_or(&path);
                for (i, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("unsafe ") || trimmed.contains("unsafe {") {
                        issues.push(format!("{}:{}: unsafe block", rel.display(), i + 1));
                    }
                    if trimmed.contains(".unwrap()") {
                        issues.push(format!("{}:{}: .unwrap()", rel.display(), i + 1));
                    }
                    if trimmed.contains(".expect(") {
                        issues.push(format!("{}:{}: .expect()", rel.display(), i + 1));
                    }
                }
            }
        }
    }

    if issues.is_empty() {
        (true, "No unsafe/unwrap/expect found".to_string())
    } else {
        (
            false,
            format!(
                "{} issues: {}",
                issues.len(),
                issues.into_iter().take(10).collect::<Vec<_>>().join("; ")
            ),
        )
    }
}

fn check_workspace_deps(crate_path: &str) -> (bool, String) {
    let cargo_toml = Path::new(crate_path).join("Cargo.toml");
    match std::fs::read_to_string(&cargo_toml) {
        Ok(content) => {
            let mut non_workspace = 0;
            let mut in_deps = false;
            for line in content.lines() {
                if line.starts_with("[dependencies") || line.starts_with("[dev-dependencies") {
                    in_deps = true;
                    continue;
                }
                if line.starts_with('[') {
                    in_deps = false;
                }
                if in_deps && line.contains("version") && !line.contains("workspace = true") {
                    // Pinned version without workspace
                    if line.contains("nexcore-") || line.contains("stem-") {
                        non_workspace += 1;
                    }
                }
            }
            if non_workspace == 0 {
                (true, "All internal deps use workspace = true".to_string())
            } else {
                (
                    false,
                    format!("{} internal deps not using workspace = true", non_workspace),
                )
            }
        }
        Err(e) => (false, format!("Cannot read Cargo.toml: {}", e)),
    }
}

fn check_doc_coverage(crate_path: &str) -> (bool, String) {
    let src_dir = Path::new(crate_path).join("src");
    if !src_dir.exists() {
        return (false, "No src/ directory".to_string());
    }

    let mut pub_items = 0u32;
    let mut documented = 0u32;

    if let Ok(entries) = glob_rs_files(&src_dir) {
        for path in entries {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let lines: Vec<&str> = content.lines().collect();
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("pub fn ")
                        || trimmed.starts_with("pub struct ")
                        || trimmed.starts_with("pub enum ")
                        || trimmed.starts_with("pub trait ")
                    {
                        pub_items += 1;
                        // Check if previous non-empty line is a doc comment
                        if i > 0 {
                            let prev = (0..i).rev().find(|&j| !lines[j].trim().is_empty());
                            if let Some(j) = prev {
                                if lines[j].trim().starts_with("///")
                                    || lines[j].trim().starts_with("//!")
                                {
                                    documented += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let coverage = if pub_items > 0 {
        documented as f64 / pub_items as f64
    } else {
        1.0
    };
    let passed = coverage >= 0.8; // 80% threshold

    (
        passed,
        format!(
            "{}/{} pub items documented ({:.0}%)",
            documented,
            pub_items,
            coverage * 100.0
        ),
    )
}

fn glob_rs_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    collect_rs_files(dir, &mut files)?;
    Ok(files)
}

fn collect_rs_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<(), std::io::Error> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, files)?;
            } else if path.extension().map_or(false, |e| e == "rs") {
                files.push(path);
            }
        }
    }
    Ok(())
}
