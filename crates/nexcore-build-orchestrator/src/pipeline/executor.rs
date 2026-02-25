//! Pipeline executor — runs stages via Command with output capture.
//!
//! ## Primitive Foundation
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: σ (Sequence) | Stages execute in dependency order |
//! | T1: → (Causality) | Stage completion triggers next |
//! | T1: ∂ (Boundary) | Timeouts, build locks |
//! | T1: ς (State) | RunStatus transitions |

use crate::error::{BuildOrcError, BuildOrcResult};
use crate::pipeline::definition::PipelineDefinition;
use crate::pipeline::stage::StageConfig;
use crate::pipeline::state::{PipelineRunState, RunStatus};
use crate::types::{BuildDuration, LogChunk, StageId};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// Execute a single stage via cargo Command.
///
/// Returns (exit_code, stdout, stderr, duration).
fn execute_stage_command(
    config: &StageConfig,
    workspace: &Path,
) -> BuildOrcResult<(i32, String, String, BuildDuration)> {
    let args = config.effective_args();
    if args.is_empty() {
        return Err(BuildOrcError::Definition(format!(
            "Stage '{}' has no arguments",
            config.id
        )));
    }

    tracing::info!("Executing stage '{}': cargo {}", config.id, args.join(" "));

    let start = Instant::now();

    let output = Command::new("cargo")
        .args(&args)
        .current_dir(workspace)
        .output()
        .map_err(|e| BuildOrcError::Io {
            path: workspace.to_path_buf(),
            source: e,
        })?;

    let duration = BuildDuration::from_duration(start.elapsed());
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Check timeout
    if start.elapsed() > config.timeout {
        return Err(BuildOrcError::StageTimeout {
            stage: config.id.to_string(),
            timeout_secs: config.timeout.as_secs(),
        });
    }

    Ok((exit_code, stdout, stderr, duration))
}

/// Result from executing a single stage in a parallel wave.
///
/// Tier: T2-C (captures stage execution outcome for deferred state update)
struct WaveResult {
    stage_id: StageId,
    allow_failure: bool,
    exit_code: i32,
    stdout: String,
    stderr: String,
    duration: BuildDuration,
    error: Option<String>,
}

/// Execute a single stage and return a `WaveResult`.
/// Designed for use inside `std::thread::scope` for parallel wave execution.
fn execute_wave_stage(config: &StageConfig, workspace: &Path) -> WaveResult {
    match execute_stage_command(config, workspace) {
        Ok((exit_code, stdout, stderr, duration)) => WaveResult {
            stage_id: config.id.clone(),
            allow_failure: config.allow_failure,
            exit_code,
            stdout,
            stderr,
            duration,
            error: None,
        },
        Err(e) => WaveResult {
            stage_id: config.id.clone(),
            allow_failure: config.allow_failure,
            exit_code: 1,
            stdout: String::new(),
            stderr: e.to_string(),
            duration: BuildDuration::from_duration(std::time::Duration::ZERO),
            error: Some(e.to_string()),
        },
    }
}

/// Execute a full pipeline synchronously.
///
/// Walks the dependency DAG, executing stages as their deps are satisfied.
/// On failure, remaining stages are either cancelled or skipped based on
/// the `allow_failure` flag.
pub fn execute_pipeline(
    definition: &PipelineDefinition,
    workspace: &Path,
) -> BuildOrcResult<PipelineRunState> {
    let stage_ids = definition.stage_ids();
    let mut state = PipelineRunState::new(
        &definition.name,
        &stage_ids,
        &workspace.display().to_string(),
    );

    // Compute source hash
    match nexcore_build_gate::hash_source_dir(workspace) {
        Ok(hash) => state.source_hash = Some(hash),
        Err(e) => tracing::warn!("Could not hash workspace: {e}"),
    }

    // Start pipeline
    state.status = RunStatus::Running;

    let mut completed: Vec<StageId> = Vec::new();
    let mut had_failure = false;

    loop {
        let runnable = definition.next_runnable(&completed);
        if runnable.is_empty() {
            break;
        }

        // If a previous required stage failed, cancel all remaining
        if had_failure {
            for config in &runnable {
                if let Some(stage_state) = state.stage_mut(&config.id) {
                    let _ = stage_state.cancel();
                }
                completed.push(config.id.clone());
            }
            if completed.len() >= stage_ids.len() {
                break;
            }
            continue;
        }

        // Mark all runnable stages as started
        for config in &runnable {
            if let Some(stage_state) = state.stage_mut(&config.id) {
                let _ = stage_state.start();
            }
        }

        // Execute stages in this wave concurrently using scoped threads.
        // Stages with no dependency relationship run in parallel.
        let wave_results: Vec<WaveResult> = if runnable.len() == 1 {
            // Single stage — no thread overhead needed
            let config = runnable[0];
            vec![execute_wave_stage(config, workspace)]
        } else {
            tracing::info!(
                "Executing {} stages in parallel: {}",
                runnable.len(),
                runnable
                    .iter()
                    .map(|s| s.id.0.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            std::thread::scope(|s| {
                let handles: Vec<_> = runnable
                    .iter()
                    .map(|config| s.spawn(|| execute_wave_stage(config, workspace)))
                    .collect();
                handles
                    .into_iter()
                    .map(|h| {
                        h.join().unwrap_or_else(|_| WaveResult {
                            stage_id: StageId("unknown".into()),
                            allow_failure: false,
                            exit_code: 1,
                            stdout: String::new(),
                            stderr: "Thread panicked".to_string(),
                            duration: BuildDuration::from_duration(std::time::Duration::ZERO),
                            error: Some("Thread panicked during execution".to_string()),
                        })
                    })
                    .collect()
            })
        };

        // Process results sequentially (state updates are single-threaded)
        for result in wave_results {
            if let Some(stage_state) = state.stage_mut(&result.stage_id) {
                if !result.stdout.is_empty() {
                    stage_state.logs.push(LogChunk {
                        stage_id: result.stage_id.clone(),
                        content: result.stdout,
                        is_stderr: false,
                        timestamp: nexcore_chrono::DateTime::now(),
                    });
                }
                if !result.stderr.is_empty() {
                    stage_state.logs.push(LogChunk {
                        stage_id: result.stage_id.clone(),
                        content: result.stderr,
                        is_stderr: true,
                        timestamp: nexcore_chrono::DateTime::now(),
                    });
                }

                if let Some(err_msg) = &result.error {
                    tracing::error!("Stage '{}' error: {}", result.stage_id, err_msg);
                    let _ = stage_state.complete(result.exit_code);
                    if !result.allow_failure {
                        had_failure = true;
                    }
                } else {
                    let _ = stage_state.complete(result.exit_code);
                    stage_state.duration = Some(result.duration);

                    if result.exit_code != 0 {
                        tracing::error!(
                            "Stage '{}' failed (exit {})",
                            result.stage_id,
                            result.exit_code
                        );
                        if !result.allow_failure {
                            had_failure = true;
                        }
                    } else {
                        tracing::info!(
                            "Stage '{}' completed in {}",
                            result.stage_id,
                            result.duration
                        );
                    }
                }
            }

            completed.push(result.stage_id);
        }

        // Safety check: if we made no progress, break to prevent infinite loop
        if completed.len() >= stage_ids.len() {
            break;
        }
    }

    state.finalize();
    Ok(state)
}

/// Dry run — returns the execution plan without actually running stages.
#[must_use]
pub fn dry_run(definition: &PipelineDefinition) -> Vec<Vec<StageId>> {
    let mut waves = Vec::new();
    let mut completed: Vec<StageId> = Vec::new();

    loop {
        let runnable: Vec<StageId> = definition
            .next_runnable(&completed)
            .into_iter()
            .map(|c| c.id.clone())
            .collect();

        if runnable.is_empty() {
            break;
        }

        completed.extend(runnable.clone());
        waves.push(runnable);
    }

    waves
}
