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

/// Managed state for PTY processes.
pub struct PtyState {
    /// Active PTY processes keyed by session ID.
    pub processes: Arc<Mutex<HashMap<String, PtyProcess>>>,
    /// Counter for generating session IDs.
    counter: Arc<std::sync::atomic::AtomicU64>,
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
    let config = PtyConfig::new(&shell, &working_dir).with_size(PtySize::new(cols, rows));

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
            let data = {
                let mut procs = processes.lock().await;
                match procs.get_mut(&sid) {
                    Some(proc) => {
                        if proc.has_exited() {
                            break;
                        }
                        proc.read(8192).await
                    }
                    None => break,
                }
            };

            match data {
                Ok(bytes) if bytes.is_empty() => {
                    // Process exited
                    break;
                }
                Ok(bytes) => {
                    // Convert to string, preserving raw bytes for terminal
                    let text = String::from_utf8_lossy(&bytes);
                    if let Err(e) = app_handle.emit("pty-output", text.as_ref()) {
                        tracing::debug!(error = %e, "Failed to emit pty-output event");
                        break;
                    }
                }
                Err(e) => {
                    tracing::debug!(error = %e, session = %sid, "PTY read error");
                    // Short sleep on error to avoid busy loop
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
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
