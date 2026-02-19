//! Cargo tools: structured build, check, test, clippy, fmt, tree
//!
//! Wraps cargo CLI commands and parses output into structured JSON responses.
//! Replaces raw Bash cargo invocations with typed MCP tools that return
//! parsed diagnostics, test results, and dependency graphs.

use crate::params::cargo::{
    CargoBuildParams, CargoCheckParams, CargoClippyParams, CargoFmtParams, CargoTestParams,
    CargoTreeParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Resolve the working directory for cargo commands.
///
/// When `path` is None, tries $NEXCORE_ROOT, then ~/nexcore, then ~/Projects/nexcore.
/// This fixes the 0% success rate when MCP server CWD lacks a Cargo.toml.
fn resolve_cargo_path(path: &Option<String>) -> Option<PathBuf> {
    if let Some(p) = path {
        return Some(PathBuf::from(p));
    }

    // Try NEXCORE_ROOT env var
    if let Ok(root) = std::env::var("NEXCORE_ROOT") {
        let p = PathBuf::from(&root);
        if p.join("Cargo.toml").exists() {
            return Some(p);
        }
    }

    // Try ~/nexcore (symlink)
    if let Ok(home) = std::env::var("HOME") {
        let nexcore = PathBuf::from(&home).join("nexcore");
        if nexcore.join("Cargo.toml").exists() {
            return Some(nexcore);
        }

        // Try ~/Projects/nexcore (real path)
        let projects = PathBuf::from(&home).join("Projects/nexcore");
        if projects.join("Cargo.toml").exists() {
            return Some(projects);
        }
    }

    None
}

/// Run cargo check and return structured diagnostics.
pub fn cargo_check(params: CargoCheckParams) -> Result<CallToolResult, McpError> {
    let mut cmd = Command::new("cargo");
    cmd.arg("check").arg("--message-format=json");

    if let Some(pkg) = &params.package {
        cmd.arg("-p").arg(pkg);
    }
    if params.release {
        cmd.arg("--release");
    }
    for arg in &params.extra_args {
        cmd.arg(arg);
    }
    if let Some(path) = resolve_cargo_path(&params.path) {
        cmd.current_dir(path);
    }

    run_cargo_diagnostics(cmd, "check")
}

/// Run cargo build and return structured diagnostics.
pub fn cargo_build(params: CargoBuildParams) -> Result<CallToolResult, McpError> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--message-format=json");

    if let Some(pkg) = &params.package {
        cmd.arg("-p").arg(pkg);
    }
    if params.release {
        cmd.arg("--release");
    }
    for arg in &params.extra_args {
        cmd.arg(arg);
    }
    if let Some(path) = resolve_cargo_path(&params.path) {
        cmd.current_dir(path);
    }

    run_cargo_diagnostics(cmd, "build")
}

/// Run cargo test and return structured test results.
pub fn cargo_test(params: CargoTestParams) -> Result<CallToolResult, McpError> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");

    if let Some(pkg) = &params.package {
        cmd.arg("-p").arg(pkg);
    }
    if params.lib_only {
        cmd.arg("--lib");
    }
    for arg in &params.extra_args {
        cmd.arg(arg);
    }

    // Test filter and skip go after --
    let has_test_args = params.test_filter.is_some() || !params.skip.is_empty();
    if has_test_args {
        cmd.arg("--");
        if let Some(filter) = &params.test_filter {
            cmd.arg(filter);
        }
        for skip in &params.skip {
            cmd.arg("--skip").arg(skip);
        }
    }

    if let Some(path) = resolve_cargo_path(&params.path) {
        cmd.current_dir(path);
    }

    let start = Instant::now();
    let output = cmd.output().map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Failed to execute cargo test: {e}").into(),
        data: None,
    })?;

    let elapsed_ms = start.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    // Parse test results from stdout/stderr
    let mut tests: Vec<serde_json::Value> = Vec::new();
    let mut passed: u32 = 0;
    let mut failed: u32 = 0;
    let mut ignored: u32 = 0;

    for line in stdout.lines().chain(stderr.lines()) {
        let trimmed = line.trim();
        if trimmed.starts_with("test ") {
            if trimmed.ends_with("... ok") {
                let name = extract_test_name(trimmed);
                passed += 1;
                tests.push(json!({"name": name, "status": "ok"}));
            } else if trimmed.ends_with("... FAILED") {
                let name = extract_test_name(trimmed);
                failed += 1;
                tests.push(json!({"name": name, "status": "FAILED"}));
            } else if trimmed.ends_with("... ignored") {
                let name = extract_test_name(trimmed);
                ignored += 1;
                tests.push(json!({"name": name, "status": "ignored"}));
            }
        }
    }

    // Extract compile errors if the build phase failed
    let diagnostics = parse_diagnostics_from_stderr(&stderr);

    let result = json!({
        "command": "cargo test",
        "success": success,
        "elapsed_ms": elapsed_ms,
        "summary": {
            "total": passed + failed + ignored,
            "passed": passed,
            "failed": failed,
            "ignored": ignored,
        },
        "tests": tests,
        "diagnostics": diagnostics,
        "exit_code": output.status.code(),
    });

    let content = vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )];

    if success {
        Ok(CallToolResult::success(content))
    } else {
        Ok(CallToolResult::error(content))
    }
}

/// Run cargo clippy and return structured lint diagnostics.
pub fn cargo_clippy(params: CargoClippyParams) -> Result<CallToolResult, McpError> {
    let mut cmd = Command::new("cargo");
    cmd.arg("clippy").arg("--message-format=json");

    if let Some(pkg) = &params.package {
        cmd.arg("-p").arg(pkg);
    }
    for arg in &params.extra_args {
        cmd.arg(arg);
    }
    if params.deny_warnings {
        cmd.arg("--").arg("-D").arg("warnings");
    }
    if let Some(path) = resolve_cargo_path(&params.path) {
        cmd.current_dir(path);
    }

    run_cargo_diagnostics(cmd, "clippy")
}

/// Run cargo fmt and return format check results.
pub fn cargo_fmt(params: CargoFmtParams) -> Result<CallToolResult, McpError> {
    let mut cmd = Command::new("cargo");
    cmd.arg("fmt");

    if let Some(pkg) = &params.package {
        cmd.arg("-p").arg(pkg);
    }
    if params.check_only {
        cmd.arg("--check");
    }
    if let Some(path) = resolve_cargo_path(&params.path) {
        cmd.current_dir(path);
    }

    let start = Instant::now();
    let output = cmd.output().map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Failed to execute cargo fmt: {e}").into(),
        data: None,
    })?;

    let elapsed_ms = start.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    // Parse files that need formatting from stdout (--check mode)
    let mut unformatted_files: Vec<String> = Vec::new();
    if params.check_only {
        for line in stdout.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && (trimmed.starts_with("Diff in") || trimmed.ends_with(".rs")) {
                unformatted_files.push(trimmed.to_string());
            }
        }
    }

    let result = json!({
        "command": "cargo fmt",
        "success": success,
        "elapsed_ms": elapsed_ms,
        "check_only": params.check_only,
        "unformatted_files": unformatted_files,
        "unformatted_count": unformatted_files.len(),
        "stderr": if stderr.is_empty() { serde_json::Value::Null } else { json!(stderr.to_string()) },
        "exit_code": output.status.code(),
    });

    let content = vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )];

    if success {
        Ok(CallToolResult::success(content))
    } else {
        Ok(CallToolResult::error(content))
    }
}

/// Run cargo tree and return dependency information.
pub fn cargo_tree(params: CargoTreeParams) -> Result<CallToolResult, McpError> {
    let mut cmd = Command::new("cargo");
    cmd.arg("tree");

    if let Some(pkg) = &params.package {
        cmd.arg("-p").arg(pkg);
    }
    if let Some(inv) = &params.invert {
        cmd.arg("--invert").arg(inv);
    }
    if let Some(depth) = params.depth {
        cmd.arg("--depth").arg(depth.to_string());
    }
    if params.duplicates {
        cmd.arg("--duplicates");
    }
    if let Some(path) = resolve_cargo_path(&params.path) {
        cmd.current_dir(path);
    }

    let start = Instant::now();
    let output = cmd.output().map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Failed to execute cargo tree: {e}").into(),
        data: None,
    })?;

    let elapsed_ms = start.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    let dep_count = stdout.lines().count();

    let result = json!({
        "command": "cargo tree",
        "success": success,
        "elapsed_ms": elapsed_ms,
        "dependency_count": dep_count,
        "tree": stdout.to_string(),
        "stderr": if stderr.is_empty() { serde_json::Value::Null } else { json!(stderr.to_string()) },
        "exit_code": output.status.code(),
    });

    let content = vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )];

    if success {
        Ok(CallToolResult::success(content))
    } else {
        Ok(CallToolResult::error(content))
    }
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Extract test name from "test path::to::test ... ok" line
fn extract_test_name(line: &str) -> String {
    let without_prefix = line.trim().strip_prefix("test ").unwrap_or(line);
    // Find the " ... " separator
    if let Some(idx) = without_prefix.find(" ...") {
        without_prefix[..idx].to_string()
    } else {
        without_prefix.to_string()
    }
}

/// Run a cargo command with --message-format=json and parse diagnostics.
fn run_cargo_diagnostics(mut cmd: Command, command_name: &str) -> Result<CallToolResult, McpError> {
    let start = Instant::now();
    let output = cmd.output().map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Failed to execute cargo {command_name}: {e}").into(),
        data: None,
    })?;

    let elapsed_ms = start.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    // Parse JSON message lines from stdout
    let mut diagnostics: Vec<serde_json::Value> = Vec::new();
    let mut errors: u32 = 0;
    let mut warnings: u32 = 0;

    for line in stdout.lines() {
        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) {
            if msg.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                if let Some(message) = msg.get("message") {
                    let level = message
                        .get("level")
                        .and_then(|l| l.as_str())
                        .unwrap_or("unknown");
                    let text = message
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("");
                    let code = message
                        .get("code")
                        .and_then(|c| c.get("code"))
                        .and_then(|c| c.as_str());

                    // Extract primary span
                    let span = message
                        .get("spans")
                        .and_then(|s| s.as_array())
                        .and_then(|spans| {
                            spans.iter().find(|s| {
                                s.get("is_primary")
                                    .and_then(|p| p.as_bool())
                                    .unwrap_or(false)
                            })
                        });

                    let file = span
                        .and_then(|s| s.get("file_name"))
                        .and_then(|f| f.as_str());
                    let line_num = span
                        .and_then(|s| s.get("line_start"))
                        .and_then(|l| l.as_u64());
                    let col = span
                        .and_then(|s| s.get("column_start"))
                        .and_then(|c| c.as_u64());

                    match level {
                        "error" => errors += 1,
                        "warning" => warnings += 1,
                        _ => {}
                    }

                    diagnostics.push(json!({
                        "severity": level,
                        "message": text,
                        "file": file,
                        "line": line_num,
                        "column": col,
                        "code": code,
                    }));
                }
            }
        }
    }

    // Also capture any non-JSON stderr
    let stderr_lines: Vec<&str> = stderr.lines().filter(|l| !l.trim().is_empty()).collect();

    let result = json!({
        "command": format!("cargo {command_name}"),
        "success": success,
        "elapsed_ms": elapsed_ms,
        "summary": {
            "errors": errors,
            "warnings": warnings,
            "total_diagnostics": diagnostics.len(),
        },
        "diagnostics": diagnostics,
        "stderr": if stderr_lines.is_empty() { serde_json::Value::Null } else { json!(stderr_lines) },
        "exit_code": output.status.code(),
    });

    let content = vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )];

    if success {
        Ok(CallToolResult::success(content))
    } else {
        Ok(CallToolResult::error(content))
    }
}

/// Parse diagnostics from raw stderr (for commands like cargo test that
/// print compiler errors to stderr without --message-format=json).
fn parse_diagnostics_from_stderr(stderr: &str) -> Vec<serde_json::Value> {
    let mut diagnostics: Vec<serde_json::Value> = Vec::new();

    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("error") {
            let (code, message) = parse_error_line(rest);
            diagnostics.push(json!({
                "severity": "error",
                "message": message,
                "code": code,
            }));
        } else if let Some(rest) = trimmed.strip_prefix("warning") {
            let (code, message) = parse_error_line(rest);
            diagnostics.push(json!({
                "severity": "warning",
                "message": message,
                "code": code,
            }));
        }
    }
    diagnostics
}

/// Parse "error[E0308]: message" or "error: message" patterns
fn parse_error_line(rest: &str) -> (Option<String>, String) {
    let rest = rest.trim();
    if let Some(bracket_content) = rest.strip_prefix('[') {
        if let Some(bracket_end) = bracket_content.find(']') {
            let code = bracket_content[..bracket_end].to_string();
            let message = bracket_content[bracket_end + 1..]
                .trim_start_matches(':')
                .trim()
                .to_string();
            return (Some(code), message);
        }
    }
    let message = rest.trim_start_matches(':').trim().to_string();
    (None, message)
}
