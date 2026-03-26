// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Tauri IPC commands for PTY process management.
//!
//! Spawns real POSIX PTY processes via `nexcore-terminal::pty::PtyProcess`
//! and streams output to the frontend via Tauri events.

use nexcore_terminal::pty::{PtyConfig, PtyProcess, PtySize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

/// Maximum number of concurrent PTY sessions allowed.
/// Guards against fork bombs and resource exhaustion.
const MAX_PTY_SESSIONS: usize = 16;

/// Managed state for PTY processes.
pub struct PtyState {
    /// Active PTY processes keyed by session ID.
    pub processes: Arc<Mutex<HashMap<String, PtyProcess>>>,
    /// Counter for generating session IDs.
    counter: Arc<std::sync::atomic::AtomicU64>,
}

impl Default for PtyState {
    fn default() -> Self {
        Self::new()
    }
}

impl PtyState {
    /// Create a new PTY state manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            counter: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }

    fn next_id(&self) -> String {
        let n = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("pty-{n:04}")
    }
}

/// Response from spawning a PTY.
#[derive(Debug, Serialize, Deserialize)]
pub struct PtySpawnResult {
    /// Session identifier for this PTY.
    pub session_id: String,
    /// PID of the child process.
    pub pid: u32,
}

/// Spawn a new PTY process and begin streaming output.
#[tauri::command]
pub async fn pty_spawn(
    state: tauri::State<'_, PtyState>,
    app: tauri::AppHandle,
    shell: String,
    working_dir: String,
    cols: u16,
    rows: u16,
) -> Result<PtySpawnResult, String> {
    // Resource guard: prevent fork bombs and OOM
    {
        let procs = state.processes.lock().await;
        if procs.len() >= MAX_PTY_SESSIONS {
            return Err(format!(
                "Maximum PTY sessions reached ({MAX_PTY_SESSIONS}). Kill an existing session first."
            ));
        }
    }

    // Ensure ~/.cargo/bin and common tool paths are in PATH.
    // When launched from a desktop environment (COSMIC DE, GNOME, etc.),
    // the Tauri app doesn't inherit shell profile PATH additions.
    let enriched_path = {
        let current = std::env::var("PATH").unwrap_or_default();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        let cargo_bin = format!("{home}/.cargo/bin");
        let local_bin = format!("{home}/.local/bin");
        let mut paths: Vec<&str> = current.split(':').collect();
        if !paths.contains(&cargo_bin.as_str()) {
            paths.insert(0, &cargo_bin);
        }
        if !paths.contains(&local_bin.as_str()) {
            paths.insert(1, &local_bin);
        }
        paths.join(":")
    };

    let config = PtyConfig::new(&shell, &working_dir)
        .with_size(PtySize::new(cols, rows))
        .with_env("PATH", &enriched_path);

    let process = PtyProcess::spawn(config).map_err(|e| format!("PTY spawn failed: {e}"))?;

    let session_id = state.next_id();
    let pid = 0u32; // PtyProcess doesn't expose PID directly in public API

    let mut procs = state.processes.lock().await;
    procs.insert(session_id.clone(), process);

    // Start output reader task
    let processes = Arc::clone(&state.processes);
    let sid = session_id.clone();
    let app_handle = app.clone();

    tokio::spawn(async move {
        loop {
            // Acquire lock with a bounded read timeout so writes can interleave.
            // Without this, the reader holds the lock during the entire async read,
            // deadlocking any pty_write calls.
            let read_result = {
                let mut procs = processes.lock().await;
                match procs.get_mut(&sid) {
                    Some(proc) => {
                        if proc.has_exited() {
                            None // signals break
                        } else {
                            Some(
                                tokio::time::timeout(
                                    std::time::Duration::from_millis(50),
                                    proc.read(8192),
                                )
                                .await,
                            )
                        }
                    }
                    None => None, // signals break
                }
            }; // Lock released here — writers can proceed

            match read_result {
                None => break, // Process exited or removed
                Some(Err(_timeout)) => {
                    // No data within 50ms — yield so pty_write can acquire lock
                    tokio::task::yield_now().await;
                    continue;
                }
                Some(Ok(Ok(bytes))) if bytes.is_empty() => {
                    // Real EOF — process closed its stdout.
                    break;
                }
                Some(Ok(Ok(bytes))) => {
                    let text = String::from_utf8_lossy(&bytes);
                    if let Err(e) = app_handle.emit("pty-output", text.as_ref()) {
                        tracing::debug!(error = %e, "Failed to emit pty-output event");
                        break;
                    }
                }
                Some(Ok(Err(e))) => {
                    let is_would_block = e.to_string().contains("no data available");
                    if !is_would_block {
                        tracing::debug!(error = %e, session = %sid, "PTY read error");
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }
        }

        // Emit exit event
        let exit_code = {
            let mut procs = processes.lock().await;
            if let Some(mut proc) = procs.remove(&sid) {
                match proc.try_wait() {
                    Ok(Some(code)) => Some(code),
                    _ => None,
                }
            } else {
                None
            }
        };

        #[derive(Serialize, Clone)]
        struct PtyExitPayload {
            session_id: String,
            code: Option<i32>,
        }

        let _ = app_handle.emit(
            "pty-exit",
            PtyExitPayload {
                session_id: sid,
                code: exit_code,
            },
        );
    });

    Ok(PtySpawnResult { session_id, pid })
}

/// Write data to a PTY process.
#[tauri::command]
pub async fn pty_write(
    state: tauri::State<'_, PtyState>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    let mut procs = state.processes.lock().await;
    let process = procs
        .get_mut(&session_id)
        .ok_or_else(|| format!("No PTY session: {session_id}"))?;

    process
        .write(data.as_bytes())
        .await
        .map_err(|e| format!("PTY write failed: {e}"))
}

/// Resize a PTY process.
#[tauri::command]
pub async fn pty_resize(
    state: tauri::State<'_, PtyState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let mut procs = state.processes.lock().await;
    let process = procs
        .get_mut(&session_id)
        .ok_or_else(|| format!("No PTY session: {session_id}"))?;

    process.resize(PtySize::new(cols, rows));
    Ok(())
}

/// Kill a PTY process.
#[tauri::command]
pub async fn pty_kill(state: tauri::State<'_, PtyState>, session_id: String) -> Result<(), String> {
    let mut procs = state.processes.lock().await;
    if let Some(mut process) = procs.remove(&session_id) {
        process
            .kill()
            .await
            .map_err(|e| format!("PTY kill failed: {e}"))?;
    }
    Ok(())
}

/// Reconnect to an existing PTY session (reattach output stream).
#[tauri::command]
pub async fn pty_reconnect(
    state: tauri::State<'_, PtyState>,
    session_id: String,
) -> Result<(), String> {
    let procs = state.processes.lock().await;
    if procs.contains_key(&session_id) {
        Ok(())
    } else {
        Err(format!("No PTY session to reconnect: {session_id}"))
    }
}
