//! Microgram Bridge — subprocess dispatch to RSK kernel decision trees.
//!
//! Invokes `rsk mcg run <name>.yaml` with variables passed as `--var key=value`
//! flags. Parses the YAML/JSON output into a structured result.
//!
//! ## Grounding
//!
//! `σ(Sequence: invoke → parse → return) + μ(Mapping: name → file) +
//!  →(Causality: variables → decision → output)`

use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Default path to the RSK binary.
const DEFAULT_RSK_BINARY: &str = "rsk";

/// Default path to the microgram directory.
const DEFAULT_MCG_DIR: &str = "rsk/micrograms";

/// Microgram execution configuration.
#[derive(Debug, Clone)]
pub struct MicrogramConfig {
    /// Path to the `rsk` binary.
    pub rsk_binary: PathBuf,
    /// Path to the micrograms directory.
    pub mcg_dir: PathBuf,
    /// Execution timeout.
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

/// Fallback RSK directory path.
fn dirs_fallback() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
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

/// Execute a microgram decision tree via the RSK subprocess.
pub async fn run_microgram(
    name: &str,
    variables: &serde_json::Value,
    config: &MicrogramConfig,
) -> MicrogramResult {
    let start = Instant::now();

    // Resolve microgram file path
    let mcg_file = config.mcg_dir.join(format!("{name}.yaml"));
    if !mcg_file.exists() {
        return MicrogramResult {
            name: name.to_string(),
            output: serde_json::Value::Null,
            elapsed_ms: start.elapsed().as_millis() as u64,
            success: false,
            error: Some(format!("Microgram not found: {}", mcg_file.display())),
        };
    }

    // Build command: rsk mcg run <file> --var key=value ...
    let mut cmd = tokio::process::Command::new(&config.rsk_binary);
    cmd.arg("mcg").arg("run").arg(&mcg_file);

    // Add variables as --var flags
    if let Some(obj) = variables.as_object() {
        for (key, value) in obj {
            let val_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                other => other.to_string(),
            };
            cmd.arg("--var").arg(format!("{key}={val_str}"));
        }
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    // Execute with timeout
    let result = tokio::time::timeout(config.timeout, cmd.output()).await;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if output.status.success() {
                // Try to parse as JSON first, then wrap as string
                let parsed = serde_json::from_str::<serde_json::Value>(&stdout)
                    .unwrap_or_else(|_| serde_json::Value::String(stdout.trim().to_string()));

                MicrogramResult {
                    name: name.to_string(),
                    output: parsed,
                    elapsed_ms,
                    success: true,
                    error: None,
                }
            } else {
                MicrogramResult {
                    name: name.to_string(),
                    output: serde_json::Value::String(stdout),
                    elapsed_ms,
                    success: false,
                    error: Some(if stderr.is_empty() {
                        format!("Exit code: {}", output.status)
                    } else {
                        stderr.trim().to_string()
                    }),
                }
            }
        }
        Ok(Err(e)) => MicrogramResult {
            name: name.to_string(),
            output: serde_json::Value::Null,
            elapsed_ms,
            success: false,
            error: Some(format!("Failed to spawn rsk: {e}")),
        },
        Err(_) => MicrogramResult {
            name: name.to_string(),
            output: serde_json::Value::Null,
            elapsed_ms,
            success: false,
            error: Some(format!(
                "Microgram timed out after {}s",
                config.timeout.as_secs()
            )),
        },
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
}
