//! Microgram tools: structured access to the microgram decision tree ecosystem
//!
//! Wraps the `rsk mcg` CLI and parses its JSON output into structured MCP responses.
//! The rsk binary lives at ~/Projects/rsk-core/target/release/rsk.

use crate::params::microgram::{
    MgBenchParams, MgCatalogParams, MgCoverageParams, MgRunParams, MgTestAllParams, MgTestParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Resolve the rsk binary path.
fn resolve_rsk_binary() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        let binary = PathBuf::from(&home).join("Projects/rsk-core/target/release/rsk");
        if binary.exists() {
            return binary;
        }
    }
    // Fallback: assume it's on PATH
    PathBuf::from("rsk")
}

/// Resolve the default micrograms directory.
fn resolve_mcg_dir(dir: &Option<String>) -> PathBuf {
    if let Some(d) = dir {
        return PathBuf::from(d);
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(&home).join("Projects/rsk-core/rsk/micrograms");
    }
    PathBuf::from("micrograms")
}

/// Resolve a microgram path, prepending the default directory for relative paths.
fn resolve_mcg_path(path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() || p.exists() {
        return p;
    }
    // Try prepending default directory
    let full = resolve_mcg_dir(&None).join(path);
    if full.exists() {
        return full;
    }
    // Try with .yaml extension
    let with_ext = resolve_mcg_dir(&None).join(format!("{path}.yaml"));
    if with_ext.exists() {
        return with_ext;
    }
    // Return as-is; rsk will report the error
    p
}

/// Run an rsk mg command, parse JSON output, return CallToolResult.
fn run_mcg(subcommand: &str, args: &[String]) -> Result<CallToolResult, McpError> {
    let binary = resolve_rsk_binary();
    let mut cmd = Command::new(&binary);
    cmd.arg("mcg").arg(subcommand);
    for arg in args {
        cmd.arg(arg);
    }

    let start = Instant::now();
    let output = cmd.output().map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Failed to execute rsk mg {subcommand}: {e}").into(),
        data: None,
    })?;

    let elapsed_ms = start.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    // rsk outputs JSON on stdout; try to parse and re-format
    let result = if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        // Enrich with metadata
        let mut enriched = parsed;
        if let Some(obj) = enriched.as_object_mut() {
            obj.insert("elapsed_ms".to_string(), serde_json::json!(elapsed_ms));
            obj.insert(
                "command".to_string(),
                serde_json::json!(format!("rsk mcg {subcommand}")),
            );
        }
        serde_json::to_string_pretty(&enriched).unwrap_or_else(|_| stdout.to_string())
    } else {
        // Raw output with metadata wrapper
        serde_json::to_string_pretty(&serde_json::json!({
            "command": format!("rsk mcg {subcommand}"),
            "elapsed_ms": elapsed_ms,
            "success": success,
            "stdout": stdout.trim(),
            "stderr": stderr.trim(),
        }))
        .unwrap_or_else(|_| stdout.to_string())
    };

    let content = vec![Content::text(result)];

    if success {
        Ok(CallToolResult::success(content))
    } else {
        Ok(CallToolResult::error(content))
    }
}

/// Execute a microgram with JSON input.
pub fn mcg_run(params: MgRunParams) -> Result<CallToolResult, McpError> {
    let path = resolve_mcg_path(&params.path);
    run_mcg(
        "run",
        &[
            path.to_string_lossy().to_string(),
            "-i".to_string(),
            params.input,
        ],
    )
}

/// Self-test a single microgram.
pub fn mcg_test(params: MgTestParams) -> Result<CallToolResult, McpError> {
    let path = resolve_mcg_path(&params.path);
    run_mcg("test", &[path.to_string_lossy().to_string()])
}

/// Self-test all micrograms in the ecosystem.
pub fn mcg_test_all(params: MgTestAllParams) -> Result<CallToolResult, McpError> {
    let dir = resolve_mcg_dir(&params.dir);
    run_mcg("test-all", &[dir.to_string_lossy().to_string()])
}

/// Compute decision tree path coverage for all micrograms.
pub fn mcg_coverage(params: MgCoverageParams) -> Result<CallToolResult, McpError> {
    let dir = resolve_mcg_dir(&params.dir);
    run_mcg("coverage", &[dir.to_string_lossy().to_string()])
}

/// Introspect the microgram ecosystem: programs, inputs/outputs, chains.
pub fn mcg_catalog(params: MgCatalogParams) -> Result<CallToolResult, McpError> {
    let dir = resolve_mcg_dir(&params.dir);
    run_mcg("catalog", &[dir.to_string_lossy().to_string()])
}

/// Benchmark microgram execution performance.
pub fn mcg_bench(params: MgBenchParams) -> Result<CallToolResult, McpError> {
    let dir = resolve_mcg_dir(&params.dir);
    let iterations = params.iterations.unwrap_or(100);
    run_mcg(
        "bench",
        &[
            dir.to_string_lossy().to_string(),
            "--iterations".to_string(),
            iterations.to_string(),
        ],
    )
}
