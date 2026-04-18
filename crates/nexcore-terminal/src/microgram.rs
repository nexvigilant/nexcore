//! Microgram Bridge — in-process dispatch into the RSK kernel.
//!
//! Calls the `rsk` crate directly (no subprocess fork + exec). Variables arrive
//! as `serde_json::Value`; we project them onto `rsk::Value`, hand the
//! microgram its input, and return the structured output.
//!
//! ## Why this file is tiny now
//!
//! The previous implementation shelled out to `rsk mcg run <file>` via
//! `tokio::process::Command`. Every call paid for a process fork + rsk binary
//! startup + YAML reparse. Path-depending `rsk` as a library flattens all
//! of that into a single function call and lets us share a process-wide
//! `MicrogramIndex` cache — the scan of the ~1.5K microgram fleet happens
//! once per server lifetime instead of once per request.
//!
//! ## Grounding
//!
//! `σ(Sequence: resolve → run → map) + μ(Mapping: name → Microgram) +
//!  →(Causality: variables → decision → output)`

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use rsk::modules::microgram::MicrogramIndex;

/// Default path to the micrograms directory, relative to `$RSK_CORE_DIR`.
const DEFAULT_MCG_DIR: &str = "rsk/micrograms";

/// Microgram execution configuration.
///
/// `rsk_binary` is retained only for backwards compatibility with callers that
/// still construct `MicrogramConfig { rsk_binary, .. }`. The in-process runtime
/// does not consult it.
#[derive(Debug, Clone)]
pub struct MicrogramConfig {
    /// Historical: path to the `rsk` CLI binary. **Ignored** by the in-process
    /// runtime — field kept to preserve the struct shape for existing callers.
    pub rsk_binary: PathBuf,
    /// Path to the micrograms directory.
    pub mcg_dir: PathBuf,
    /// Execution timeout. Applies to a blocking run wrapped in `spawn_blocking`.
    pub timeout: Duration,
}

impl Default for MicrogramConfig {
    fn default() -> Self {
        let rsk_dir = std::env::var("RSK_CORE_DIR").unwrap_or_else(|_| dirs_fallback());
        let rsk_path = PathBuf::from(&rsk_dir);

        Self {
            rsk_binary: rsk_path.join("target/release/rsk"),
            mcg_dir: rsk_path.join(DEFAULT_MCG_DIR),
            timeout: Duration::from_secs(10),
        }
    }
}

/// Fallback RSK directory path derived from HOME environment variable.
fn dirs_fallback() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{home}/Projects/rsk-core")
}

/// Result of a microgram execution.
#[derive(Debug, Serialize, Deserialize)]
pub struct MicrogramResult {
    /// The microgram name that was executed.
    pub name: String,
    /// The decision/output from the microgram.
    pub output: serde_json::Value,
    /// Execution time.
    pub elapsed_ms: u64,
    /// Whether execution succeeded.
    pub success: bool,
    /// Error message if execution failed.
    pub error: Option<String>,
}

/// Process-wide cache of loaded microgram directories.
///
/// First call per directory pays the full scan; subsequent calls reuse the
/// cached [`MicrogramIndex`]. Cache never invalidates — restart the server to
/// pick up on-disk changes.
fn index_cache() -> &'static RwLock<HashMap<PathBuf, Arc<MicrogramIndex>>> {
    static CACHE: std::sync::OnceLock<RwLock<HashMap<PathBuf, Arc<MicrogramIndex>>>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Fetch (or build, on miss) a cached index for `dir`.
fn index_for(dir: &std::path::Path) -> Result<Arc<MicrogramIndex>, String> {
    let key = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());

    if let Ok(guard) = index_cache().read()
        && let Some(idx) = guard.get(&key)
    {
        return Ok(Arc::clone(idx));
    }

    let built = MicrogramIndex::load_lossy(dir)
        .map_err(|e| format!("Failed to index {}: {e}", dir.display()))?;
    let arc = Arc::new(built);
    if let Ok(mut guard) = index_cache().write() {
        guard.insert(key, Arc::clone(&arc));
    }
    Ok(arc)
}

/// Convert an incoming `serde_json::Value` (expected to be an object) into the
/// `HashMap<String, rsk::Value>` shape the microgram runtime expects. Numeric,
/// string, boolean, and nested values pass through via the shared serde
/// representation.
fn json_input_to_rsk(variables: &serde_json::Value) -> Result<HashMap<String, rsk::Value>, String> {
    match variables {
        serde_json::Value::Object(map) => {
            let mut out = HashMap::with_capacity(map.len());
            for (k, v) in map {
                let rv: rsk::Value = serde_json::from_value(v.clone())
                    .map_err(|e| format!("input field '{k}': {e}"))?;
                out.insert(k.clone(), rv);
            }
            Ok(out)
        }
        serde_json::Value::Null => Ok(HashMap::new()),
        other => Err(format!(
            "expected variables to be a JSON object, got {}",
            other_type_name(other)
        )),
    }
}

fn other_type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Convert the rsk output `HashMap<String, rsk::Value>` back into JSON for
/// callers that speak `serde_json::Value` (WebSocket frames, REST responses,
/// MCP content blocks).
fn rsk_output_to_json(output: HashMap<String, rsk::Value>) -> serde_json::Value {
    // Both `rsk::Value` and `serde_json::Value` derive serde; round-trip through
    // JSON strings is the least surprising conversion (handles the full
    // Null/Bool/Int/Float/String/Array/Object space identically).
    serde_json::to_value(output).unwrap_or(serde_json::Value::Null)
}

/// Execute a microgram decision tree in-process.
///
/// Preserves the original `async fn` signature so every existing caller (WS
/// terminal, MCP bridge, tests) compiles unchanged. Internally:
///
/// 1. Resolves a cached [`MicrogramIndex`] for `config.mcg_dir`.
/// 2. Looks up the microgram by `name` — O(1), cheap `Arc` clone.
/// 3. Converts inputs to `rsk::Value`, runs the tree, converts back.
///
/// The blocking `mg.run()` is parked on `spawn_blocking` so we honour the
/// async contract without starving the tokio runtime on pathologically deep
/// trees (bounded by rsk's own 1 000-step guard).
pub async fn run_microgram(
    name: &str,
    variables: &serde_json::Value,
    config: &MicrogramConfig,
) -> MicrogramResult {
    let start = Instant::now();

    // Resolve the index. Cache miss = full dir scan; hit = Arc bump.
    let index = match index_for(&config.mcg_dir) {
        Ok(idx) => idx,
        Err(e) => {
            return MicrogramResult {
                name: name.to_string(),
                output: serde_json::Value::Null,
                elapsed_ms: start.elapsed().as_millis() as u64,
                success: false,
                error: Some(e),
            };
        }
    };

    // Look up the microgram by name.
    let Some(mg) = index.get(name) else {
        return MicrogramResult {
            name: name.to_string(),
            output: serde_json::Value::Null,
            elapsed_ms: start.elapsed().as_millis() as u64,
            success: false,
            error: Some(format!(
                "Microgram '{name}' not found in {}",
                config.mcg_dir.display()
            )),
        };
    };

    // Map inputs.
    let input = match json_input_to_rsk(variables) {
        Ok(m) => m,
        Err(e) => {
            return MicrogramResult {
                name: name.to_string(),
                output: serde_json::Value::Null,
                elapsed_ms: start.elapsed().as_millis() as u64,
                success: false,
                error: Some(format!("Input conversion failed: {e}")),
            };
        }
    };

    // Move the run to a blocking pool — individual runs are μs-scale but we
    // respect the async contract regardless. Bound by `config.timeout`.
    let mg_for_task = Arc::clone(&mg);
    let name_owned = name.to_string();
    let handle = tokio::task::spawn_blocking(move || mg_for_task.run(input));

    let elapsed_ms_fn = || -> u64 { start.elapsed().as_millis() as u64 };

    let run_result = match tokio::time::timeout(config.timeout, handle).await {
        Ok(Ok(r)) => r,
        Ok(Err(join_err)) => {
            return MicrogramResult {
                name: name_owned,
                output: serde_json::Value::Null,
                elapsed_ms: elapsed_ms_fn(),
                success: false,
                error: Some(format!("Blocking task failed: {join_err}")),
            };
        }
        Err(_) => {
            return MicrogramResult {
                name: name_owned,
                output: serde_json::Value::Null,
                elapsed_ms: elapsed_ms_fn(),
                success: false,
                error: Some(format!(
                    "Microgram timed out after {}s",
                    config.timeout.as_secs()
                )),
            };
        }
    };

    let success = run_result.success;
    let error_msg = if success {
        None
    } else {
        // Pull any `_error` field the tree produced for the caller.
        run_result.output.get("_error").and_then(|v| match v {
            rsk::Value::String(s) => Some(s.clone()),
            other => serde_json::to_string(other).ok(),
        })
    };

    MicrogramResult {
        name: run_result.name,
        output: rsk_output_to_json(run_result.output),
        elapsed_ms: elapsed_ms_fn(),
        success,
        error: error_msg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_resolves_paths() {
        let config = MicrogramConfig::default();
        assert!(config.rsk_binary.to_string_lossy().contains("rsk"));
        assert!(config.mcg_dir.to_string_lossy().contains("micrograms"));
    }

    #[test]
    fn microgram_result_serializes() {
        let result = MicrogramResult {
            name: "prr-signal".to_string(),
            output: serde_json::json!({"decision": "signal_detected", "prr": 2.5}),
            elapsed_ms: 42,
            success: true,
            error: None,
        };
        let json = serde_json::to_string(&result);
        assert!(json.is_ok());
    }

    #[test]
    fn json_input_to_rsk_rejects_non_object() {
        let err = json_input_to_rsk(&serde_json::json!([1, 2, 3])).unwrap_err();
        assert!(
            err.contains("array"),
            "error should name the wrong type: {err}"
        );
    }

    #[test]
    fn json_input_to_rsk_accepts_null_as_empty() {
        // A null input is treated as "no variables" rather than an error —
        // matches the permissive behaviour of the prior subprocess bridge,
        // which passed no --var flags for a null payload.
        let out = json_input_to_rsk(&serde_json::Value::Null).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn json_input_to_rsk_roundtrips_scalars() {
        let input = serde_json::json!({
            "int_field": 42,
            "float_field": 3.14,
            "bool_field": true,
            "string_field": "hi",
        });
        let out = json_input_to_rsk(&input).unwrap();
        assert_eq!(out.len(), 4);
        assert!(out.contains_key("int_field"));
        assert!(out.contains_key("string_field"));
    }

    #[tokio::test]
    async fn unknown_microgram_reports_error_without_panic() {
        // Use a non-existent directory — cache build will succeed with an
        // empty index, lookup will miss, we get a clean error (not a panic).
        let tmp = std::env::temp_dir().join("nexcore-terminal-test-empty-mcg");
        std::fs::create_dir_all(&tmp).ok();

        let config = MicrogramConfig {
            rsk_binary: PathBuf::from("/unused"),
            mcg_dir: tmp,
            timeout: Duration::from_secs(5),
        };
        let variables = serde_json::json!({});
        let result = run_microgram("does-not-exist", &variables, &config).await;

        assert!(!result.success);
        assert_eq!(result.name, "does-not-exist");
        assert!(
            result
                .error
                .as_ref()
                .is_some_and(|e| e.contains("not found"))
        );
    }
}
