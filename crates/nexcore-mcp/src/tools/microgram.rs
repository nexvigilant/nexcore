//! Microgram tools — in-process dispatch into the RSK kernel.
//!
//! Previously this file shelled out to `~/Projects/rsk-core/target/release/rsk`
//! via `std::process::Command::new()` and parsed the JSON stdout. Every
//! Claude-side tool call paid for a process fork + rsk binary startup + YAML
//! reparse + JSON parse-and-reserialise round-trip.
//!
//! With `rsk` declared as a workspace dependency, the same logic runs
//! in-process through the crate API. A static `MicrogramIndex` cache — backed
//! by `rsk-core`'s `Arc`-indexed name lookup — scans the microgram fleet once
//! per server lifetime instead of once per request.
//!
//! The public function surface is unchanged: every `mcg_*` fn still returns
//! `Result<CallToolResult, McpError>` and emits a JSON-encoded `Content::text`
//! with a shape downstream consumers can treat as before (an `elapsed_ms`
//! metadata field is preserved for parity with the old subprocess output).

use crate::params::microgram::{
    MgBenchParams, MgCatalogParams, MgChainRunParams, MgChainTestParams, MgCoverageParams,
    MgRunParams, MgTestAllParams, MgTestParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Instant;

use rsk::modules::microgram::{Microgram, MicrogramIndex};

// ─────────────────────────────────────────────────────────────────────────────
// Path resolution (preserved verbatim from the prior subprocess implementation —
// Claude Code users depend on these fallback conventions).
// ─────────────────────────────────────────────────────────────────────────────

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
    let full = resolve_mcg_dir(&None).join(path);
    if full.exists() {
        return full;
    }
    let with_ext = resolve_mcg_dir(&None).join(format!("{path}.yaml"));
    if with_ext.exists() {
        return with_ext;
    }
    p
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
    let exact = chains_dir.join(name);
    if exact.exists() {
        return exact;
    }
    let with_ext = chains_dir.join(format!("{name}.yaml"));
    if with_ext.exists() {
        return with_ext;
    }
    exact
}

// ─────────────────────────────────────────────────────────────────────────────
// Process-wide index cache
// ─────────────────────────────────────────────────────────────────────────────

fn index_cache() -> &'static RwLock<HashMap<PathBuf, Arc<MicrogramIndex>>> {
    static CACHE: OnceLock<RwLock<HashMap<PathBuf, Arc<MicrogramIndex>>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn index_for(dir: &Path) -> Result<Arc<MicrogramIndex>, String> {
    let key = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    if let Ok(guard) = index_cache().read()
        && let Some(idx) = guard.get(&key)
    {
        return Ok(Arc::clone(idx));
    }
    let built = MicrogramIndex::load_lossy(dir)
        .map_err(|e| format!("index {} failed: {e}", dir.display()))?;
    let arc = Arc::new(built);
    if let Ok(mut guard) = index_cache().write() {
        guard.insert(key, Arc::clone(&arc));
    }
    Ok(arc)
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers for wrapping rsk results as MCP CallToolResult payloads
// ─────────────────────────────────────────────────────────────────────────────

/// Build an MCP error for a failed operation with a consistent shape.
fn err(msg: impl Into<String>) -> McpError {
    McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: msg.into().into(),
        data: None,
    }
}

/// Serialize a value as pretty JSON with an embedded `elapsed_ms` field for
/// parity with the prior subprocess output, and wrap as MCP content.
fn wrap_json(
    payload: serde_json::Value,
    elapsed_ms: u128,
    success: bool,
) -> Result<CallToolResult, McpError> {
    let enriched = match payload {
        serde_json::Value::Object(mut map) => {
            map.insert("elapsed_ms".to_string(), serde_json::json!(elapsed_ms));
            serde_json::Value::Object(map)
        }
        other => serde_json::json!({
            "result": other,
            "elapsed_ms": elapsed_ms,
        }),
    };

    let text = serde_json::to_string_pretty(&enriched).unwrap_or_else(|_| enriched.to_string());
    let content = vec![Content::text(text)];

    if success {
        Ok(CallToolResult::success(content))
    } else {
        Ok(CallToolResult::error(content))
    }
}

/// Parse the JSON input string that MCP clients pass for `mcg_run`/`mcg_chain_run`.
fn parse_input_json(input: &str) -> Result<HashMap<String, rsk::Value>, McpError> {
    if input.is_empty() {
        return Ok(HashMap::new());
    }
    let v: serde_json::Value =
        serde_json::from_str(input).map_err(|e| err(format!("Invalid input JSON: {e}")))?;
    match v {
        serde_json::Value::Null => Ok(HashMap::new()),
        serde_json::Value::Object(map) => {
            let mut out = HashMap::with_capacity(map.len());
            for (k, val) in map {
                let rv: rsk::Value = serde_json::from_value(val)
                    .map_err(|e| err(format!("input field '{k}': {e}")))?;
                out.insert(k, rv);
            }
            Ok(out)
        }
        other => Err(err(format!(
            "expected input to be a JSON object, got {}",
            match other {
                serde_json::Value::Array(_) => "array",
                serde_json::Value::String(_) => "string",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::Bool(_) => "bool",
                _ => "unknown",
            }
        ))),
    }
}

/// Load a standalone microgram from a resolved path (used by `mcg_run`/`mcg_test`
/// where the caller provides a YAML path rather than a bare name).
fn load_from_path(path: &Path) -> Result<Microgram, McpError> {
    Microgram::load(path).map_err(|e| err(format!("Failed to load {}: {e}", path.display())))
}

// ─────────────────────────────────────────────────────────────────────────────
// Public tool functions — same signatures as the prior subprocess version
// ─────────────────────────────────────────────────────────────────────────────

/// Execute a microgram with JSON input.
pub fn mcg_run(params: MgRunParams) -> Result<CallToolResult, McpError> {
    let start = Instant::now();
    let path = resolve_mcg_path(&params.path);
    let mg = load_from_path(&path)?;
    let input = parse_input_json(&params.input)?;
    let result = mg.run(input);

    let payload = serde_json::to_value(&result)
        .map_err(|e| err(format!("Failed to serialize run result: {e}")))?;
    wrap_json(payload, start.elapsed().as_millis(), result.success)
}

/// Self-test a single microgram.
pub fn mcg_test(params: MgTestParams) -> Result<CallToolResult, McpError> {
    let start = Instant::now();
    let path = resolve_mcg_path(&params.path);
    let mg = load_from_path(&path)?;
    let result = mg.test();
    let success = result.failed == 0;

    let payload = serde_json::to_value(&result)
        .map_err(|e| err(format!("Failed to serialize test result: {e}")))?;
    wrap_json(payload, start.elapsed().as_millis(), success)
}

/// Self-test all micrograms in the ecosystem.
pub fn mcg_test_all(params: MgTestAllParams) -> Result<CallToolResult, McpError> {
    let start = Instant::now();
    let dir = resolve_mcg_dir(&params.dir);

    let results = rsk::modules::microgram::test_all(&dir)
        .map_err(|e| err(format!("test_all {} failed: {e}", dir.display())))?;

    let total: usize = results.iter().map(|r| r.total).sum();
    let passed: usize = results.iter().map(|r| r.passed).sum();
    let failed: usize = results.iter().map(|r| r.failed).sum();
    let success = failed == 0;

    let payload = serde_json::json!({
        "programs": results.len(),
        "total_tests": total,
        "passed": passed,
        "failed": failed,
        "failures": results
            .iter()
            .filter(|r| r.failed > 0)
            .map(|r| serde_json::json!({
                "name": r.name,
                "failed": r.failed,
                "total": r.total,
            }))
            .collect::<Vec<_>>(),
    });
    wrap_json(payload, start.elapsed().as_millis(), success)
}

/// Compute decision tree path coverage for all micrograms.
pub fn mcg_coverage(params: MgCoverageParams) -> Result<CallToolResult, McpError> {
    let start = Instant::now();
    let dir = resolve_mcg_dir(&params.dir);
    let results = rsk::modules::microgram::coverage_all(&dir)
        .map_err(|e| err(format!("coverage_all {} failed: {e}", dir.display())))?;
    let payload = serde_json::to_value(&results)
        .map_err(|e| err(format!("Failed to serialize coverage: {e}")))?;
    wrap_json(payload, start.elapsed().as_millis(), true)
}

/// Introspect the microgram ecosystem: programs, inputs/outputs, chains.
pub fn mcg_catalog(params: MgCatalogParams) -> Result<CallToolResult, McpError> {
    let start = Instant::now();
    let dir = resolve_mcg_dir(&params.dir);
    let cat = rsk::modules::microgram::catalog(&dir)
        .map_err(|e| err(format!("catalog {} failed: {e}", dir.display())))?;
    let payload =
        serde_json::to_value(&cat).map_err(|e| err(format!("Failed to serialize catalog: {e}")))?;
    wrap_json(payload, start.elapsed().as_millis(), true)
}

/// Benchmark microgram execution performance.
///
/// `iterations` is no longer forwarded to a CLI flag (the in-process API uses
/// a fixed schedule). Callers that depended on the old CLI behaviour get the
/// same relative numbers — only the absolute iteration count differs.
pub fn mcg_bench(params: MgBenchParams) -> Result<CallToolResult, McpError> {
    let start = Instant::now();
    let dir = resolve_mcg_dir(&params.dir);
    let iterations = params.iterations.unwrap_or(100);
    let results = rsk::modules::microgram::bench_all(&dir, iterations as usize)
        .map_err(|e| err(format!("bench_all {} failed: {e}", dir.display())))?;
    let payload = serde_json::json!({
        "iterations": iterations,
        "results": results,
    });
    wrap_json(payload, start.elapsed().as_millis(), true)
}

// ─────────────────────────────────────────────────────────────────────────────
// Chain tools — parse chain YAML once (line-based, same as the prior impl),
// then dispatch through the appropriate rsk chain function.
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a chain YAML file and extract `(steps, mcg_dir, accumulate, resilient)`.
///
/// Kept from the prior subprocess implementation to avoid the dependency cost
/// of a full YAML parser inside this tool surface. Chain YAMLs follow a
/// stable `steps:`/`accumulate:`/`resilient:`/`micrograms_dir:` shape.
fn parse_chain_yaml(path: &Path) -> Result<(Vec<String>, PathBuf, bool, bool), McpError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| err(format!("Failed to read chain file {}: {e}", path.display())))?;

    let mut steps: Vec<String> = Vec::new();
    let mut accumulate = false;
    let mut resilient = false;
    let mut micrograms_dir: Option<String> = None;
    let mut in_steps = false;

    for line in content.lines() {
        let trimmed = line.trim();
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

        if in_steps {
            if let Some(name) = trimmed.strip_prefix("- ") {
                steps.push(name.trim().to_string());
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                in_steps = false;
            }
        }
    }

    if steps.is_empty() {
        return Err(err(format!("Chain {} has no steps", path.display())));
    }

    let mcg_dir = if let Some(ref rel) = micrograms_dir {
        let parent = path.parent().unwrap_or(Path::new("."));
        let resolved = parent.join(rel);
        if resolved.exists() {
            resolved
        } else {
            resolve_mcg_dir(&None)
        }
    } else {
        resolve_mcg_dir(&None)
    };

    Ok((steps, mcg_dir, accumulate, resilient))
}

/// Execute a microgram chain with JSON input.
///
/// Chains compose multiple micrograms into pipelines. The 7 station-* chains
/// bridge NexVigilant Station data extraction into PV decision logic.
pub fn mcg_chain_run(params: MgChainRunParams) -> Result<CallToolResult, McpError> {
    let start = Instant::now();
    let chain_path = resolve_chain_path(&params.chain);
    let (steps, mcg_dir, accumulate, resilient) = parse_chain_yaml(&chain_path)?;

    let index = index_for(&mcg_dir).map_err(err)?;
    let input = parse_input_json(&params.input)?;

    let step_refs: Vec<&str> = steps.iter().map(String::as_str).collect();

    let (success, payload) = if resilient {
        let r = rsk::modules::microgram::chain_resilient_with_index(&index, &step_refs, input)
            .map_err(err)?;
        let succ = matches!(r.status, rsk::modules::microgram::ChainStatus::Complete);
        let v = serde_json::to_value(&r)
            .map_err(|e| err(format!("Failed to serialize chain result: {e}")))?;
        (succ, v)
    } else if accumulate {
        let r = rsk::modules::microgram::chain_accumulate_with_index(&index, &step_refs, input)
            .map_err(err)?;
        let v = serde_json::to_value(&r)
            .map_err(|e| err(format!("Failed to serialize chain result: {e}")))?;
        (r.success, v)
    } else {
        let r =
            rsk::modules::microgram::chain_with_index(&index, &step_refs, input).map_err(err)?;
        let v = serde_json::to_value(&r)
            .map_err(|e| err(format!("Failed to serialize chain result: {e}")))?;
        (r.success, v)
    };

    let enriched = match payload {
        serde_json::Value::Object(mut map) => {
            map.insert("chain".to_string(), serde_json::json!(params.chain));
            serde_json::Value::Object(map)
        }
        other => serde_json::json!({"chain": params.chain, "result": other}),
    };

    wrap_json(enriched, start.elapsed().as_millis(), success)
}

/// Self-test microgram chains — one by name, or all if no name provided.
pub fn mcg_chain_test(params: MgChainTestParams) -> Result<CallToolResult, McpError> {
    let start = Instant::now();
    let chains_dir = resolve_chains_dir();

    let all = rsk::modules::microgram::test_chains(&chains_dir)
        .map_err(|e| err(format!("test_chains {} failed: {e}", chains_dir.display())))?;

    // Filter to a single chain if the caller specified one — rsk::test_chains
    // runs every chain in the directory, and the CLI-style subcommand used to
    // accept a path. We post-filter by name so both behaviours stay available.
    let filtered: Vec<_> = match &params.chain {
        Some(name) => all
            .into_iter()
            .filter(|r| {
                let candidate = format!("{name}.yaml");
                r.chain_name == *name || r.chain_name == candidate
            })
            .collect(),
        None => all,
    };

    let total = filtered.len();
    let passed = filtered.iter().filter(|r| r.passed == r.total).count();
    let success = filtered.iter().all(|r| r.passed == r.total);

    let payload = serde_json::json!({
        "chains_tested": total,
        "chains_passed": passed,
        "results": filtered,
    });
    wrap_json(payload, start.elapsed().as_millis(), success)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests — unit coverage for path resolution + JSON conversion.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_mcg_dir_honours_override() {
        let out = resolve_mcg_dir(&Some("/tmp/x".to_string()));
        assert_eq!(out, PathBuf::from("/tmp/x"));
    }

    #[test]
    fn resolve_mcg_dir_defaults_under_home() {
        let out = resolve_mcg_dir(&None);
        assert!(out.to_string_lossy().contains("micrograms"));
    }

    #[test]
    fn parse_input_json_empty_string_is_empty_map() {
        let out = parse_input_json("").unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn parse_input_json_rejects_array() {
        let err = parse_input_json("[1,2,3]").unwrap_err();
        assert!(
            err.message.contains("array"),
            "error message was: {}",
            err.message
        );
    }

    #[test]
    fn parse_input_json_accepts_object() {
        let out = parse_input_json(r#"{"a": 1, "b": "x"}"#).unwrap();
        assert_eq!(out.len(), 2);
    }
}
