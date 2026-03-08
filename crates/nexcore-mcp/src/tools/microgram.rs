//! Microgram tools: structured access to the microgram decision tree ecosystem
//!
//! Wraps the `rsk mcg` CLI and parses its JSON output into structured MCP responses.
//! The rsk binary lives at ~/Projects/rsk-core/target/release/rsk.

use crate::params::microgram::{
    MgBenchParams, MgCatalogParams, MgChainRunParams, MgChainTestParams, MgCoverageParams,
    MgRunParams, MgTestAllParams, MgTestParams,
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

/// Resolve the default chains directory.
fn resolve_chains_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(&home).join("Projects/rsk-core/rsk/chains");
    }
    PathBuf::from("chains")
}

/// Resolve a chain path by name.
fn resolve_chain_path(name: &str) -> PathBuf {
    let chains_dir = resolve_chains_dir();
    // Try exact name first
    let exact = chains_dir.join(name);
    if exact.exists() {
        return exact;
    }
    // Try with .yaml extension
    let with_ext = chains_dir.join(format!("{name}.yaml"));
    if with_ext.exists() {
        return with_ext;
    }
    // Return as-is; rsk will report the error
    exact
}

/// Parse a chain YAML file and extract the inline chain expression + flags.
///
/// Uses simple line-based parsing to avoid adding serde_yaml dependency.
/// Chain YAML format is stable and simple: `steps:` followed by `  - name` entries.
fn parse_chain_yaml(path: &std::path::Path) -> Result<(String, PathBuf, bool, bool), McpError> {
    let content = std::fs::read_to_string(path).map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Failed to read chain file {}: {e}", path.display()).into(),
        data: None,
    })?;

    let mut steps: Vec<String> = Vec::new();
    let mut accumulate = false;
    let mut resilient = false;
    let mut micrograms_dir: Option<String> = None;
    let mut in_steps = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Top-level key: value pairs
        if !line.starts_with(' ') && !line.starts_with('\t') {
            in_steps = trimmed == "steps:";
            if let Some(rest) = trimmed.strip_prefix("accumulate:") {
                accumulate = rest.trim() == "true";
            }
            if let Some(rest) = trimmed.strip_prefix("resilient:") {
                resilient = rest.trim() == "true";
            }
            if let Some(rest) = trimmed.strip_prefix("micrograms_dir:") {
                micrograms_dir = Some(rest.trim().to_string());
            }
            continue;
        }

        // Inside steps: collect "  - name" entries
        if in_steps {
            if let Some(name) = trimmed.strip_prefix("- ") {
                steps.push(name.trim().to_string());
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                // Non-list-item line ends the steps block
                in_steps = false;
            }
        }
    }

    if steps.is_empty() {
        return Err(McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Chain {} has no steps", path.display()).into(),
            data: None,
        });
    }

    let expr = steps.join(" -> ");

    // Resolve micrograms_dir relative to chain file
    let mcg_dir = if let Some(ref rel) = micrograms_dir {
        let parent = path.parent().unwrap_or(std::path::Path::new("."));
        let resolved = parent.join(rel);
        if resolved.exists() {
            resolved
        } else {
            resolve_mcg_dir(&None)
        }
    } else {
        resolve_mcg_dir(&None)
    };

    Ok((expr, mcg_dir, accumulate, resilient))
}

/// Execute a microgram chain with JSON input.
///
/// Chains compose multiple micrograms into pipelines. The 7 station-* chains
/// bridge NexVigilant Station data extraction into PV decision logic:
/// - station-openvigil-pipeline: disproportionality → signal triage → causality
/// - station-dailymed-pipeline: ADR label data → risk tier → causality
/// - station-trial-pipeline: clinical trial SAE → safety concern → causality
/// - station-drugbank-pipeline: DDI data → interaction severity → causality
/// - station-pubmed-pipeline: literature evidence → signal strength → causality
/// - station-recall-pipeline: FDA recall severity → risk tier → causality
/// - station-rxnav-pipeline: interaction severity → causality
pub fn mcg_chain_run(params: MgChainRunParams) -> Result<CallToolResult, McpError> {
    let chain_path = resolve_chain_path(&params.chain);
    let (chain_expr, mcg_dir, accumulate, resilient) = parse_chain_yaml(&chain_path)?;

    let binary = resolve_rsk_binary();
    let start = Instant::now();

    let mut cmd = Command::new(&binary);
    cmd.arg("mcg")
        .arg("chain")
        .arg(&chain_expr)
        .arg("-d")
        .arg(mcg_dir.to_string_lossy().as_ref())
        .arg("-i")
        .arg(&params.input);
    if accumulate {
        cmd.arg("--accumulate");
    }
    if resilient {
        cmd.arg("--resilient");
    }

    let output = cmd.output().map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Failed to execute chain {}: {e}", params.chain).into(),
        data: None,
    })?;

    let elapsed_ms = start.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    let result = if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let mut enriched = parsed;
        if let Some(obj) = enriched.as_object_mut() {
            obj.insert("elapsed_ms".to_string(), serde_json::json!(elapsed_ms));
            obj.insert("chain".to_string(), serde_json::json!(params.chain));
        }
        serde_json::to_string_pretty(&enriched).unwrap_or_else(|_| stdout.to_string())
    } else {
        serde_json::to_string_pretty(&serde_json::json!({
            "chain": params.chain,
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

/// Self-test microgram chains.
///
/// Tests a single chain by name, or all chains if no name provided.
pub fn mcg_chain_test(params: MgChainTestParams) -> Result<CallToolResult, McpError> {
    match &params.chain {
        Some(name) => {
            let chain_path = resolve_chain_path(name);
            run_mcg("chain-test", &[chain_path.to_string_lossy().to_string()])
        }
        None => {
            let chains_dir = resolve_chains_dir();
            run_mcg("chain-test", &[chains_dir.to_string_lossy().to_string()])
        }
    }
}
