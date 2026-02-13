//! Managed child process — spawn, line I/O, graceful kill.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: State (ς) | Child process lifecycle |
//! | T1: Sequence (σ) | Line-by-line I/O streaming |
//! | T1: Void (∅) | Process exit / not-yet-spawned |

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc;
use tokio::time::Duration;

use crate::error::{ProxyError, Result};

/// Tier: T2-C — A managed child process with stdin/stdout/stderr access.
pub struct ManagedChild {
    binary: PathBuf,
    args: Vec<String>,
    child: Option<Child>,
    stdin_tx: Option<mpsc::Sender<String>>,
}

/// A line read from the child's stdout or stderr.
#[derive(Debug, Clone)]
pub enum ChildLine {
    Stdout(String),
    Stderr(String),
}

// ── Pure helpers ──────────────────────────────────────────────────

/// Take stdin from child, returning error if unavailable.
fn take_stdin(child: &mut Child) -> Result<ChildStdin> {
    child
        .stdin
        .take()
        .ok_or_else(|| ProxyError::Child("child stdin not available".into()))
}

/// Take stdout from child, returning error if unavailable.
fn take_stdout(child: &mut Child) -> Result<ChildStdout> {
    child
        .stdout
        .take()
        .ok_or_else(|| ProxyError::Child("child stdout not available".into()))
}

/// Take stderr from child, returning error if unavailable.
fn take_stderr(child: &mut Child) -> Result<ChildStderr> {
    child
        .stderr
        .take()
        .ok_or_else(|| ProxyError::Child("child stderr not available".into()))
}

/// Spawn stdin writer task. Returns sender for writing lines.
fn spawn_stdin_writer(child_stdin: ChildStdin) -> mpsc::Sender<String> {
    let (tx, mut rx) = mpsc::channel::<String>(256);
    tokio::spawn(async move {
        let mut writer = child_stdin;
        while let Some(line) = rx.recv().await {
            if writer.write_all(line.as_bytes()).await.is_err() {
                break;
            }
            if writer.flush().await.is_err() {
                break;
            }
        }
    });
    tx
}

/// Spawn stdout reader task, forwarding lines to channel.
fn spawn_stdout_reader(stdout: ChildStdout, tx: mpsc::Sender<ChildLine>) {
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx.send(ChildLine::Stdout(line)).await.is_err() {
                break;
            }
        }
    });
}

/// Spawn stderr reader task, forwarding lines to channel.
fn spawn_stderr_reader(stderr: ChildStderr, tx: mpsc::Sender<ChildLine>) {
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx.send(ChildLine::Stderr(line)).await.is_err() {
                break;
            }
        }
    });
}

/// Ensure string ends with newline.
fn ensure_newline(s: &str) -> String {
    if s.ends_with('\n') {
        s.to_string()
    } else {
        format!("{s}\n")
    }
}

// ── ManagedChild impl ────────────────────────────────────────────

impl ManagedChild {
    /// Create a new managed child (not yet spawned).
    pub fn new(binary: &Path, args: Vec<String>) -> Self {
        Self {
            binary: binary.to_path_buf(),
            args,
            child: None,
            stdin_tx: None,
        }
    }

    /// Spawn the child process. Returns a receiver for stdout/stderr lines.
    pub fn spawn(&mut self) -> Result<mpsc::Receiver<ChildLine>> {
        let mut child = Command::new(&self.binary)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                ProxyError::Child(format!("failed to spawn {}: {e}", self.binary.display()))
            })?;

        let stdin = take_stdin(&mut child)?;
        let stdout = take_stdout(&mut child)?;
        let stderr = take_stderr(&mut child)?;

        self.child = Some(child);
        self.stdin_tx = Some(spawn_stdin_writer(stdin));

        let (line_tx, line_rx) = mpsc::channel::<ChildLine>(1024);
        spawn_stdout_reader(stdout, line_tx.clone());
        spawn_stderr_reader(stderr, line_tx);

        tracing::info!("Spawned child: {}", self.binary.display());
        Ok(line_rx)
    }

    /// Send a line to the child's stdin.
    pub async fn send_line(&self, line: &str) -> Result<()> {
        let tx = self
            .stdin_tx
            .as_ref()
            .ok_or_else(|| ProxyError::Child("child not spawned or stdin closed".into()))?;
        tx.send(ensure_newline(line))
            .await
            .map_err(|_| ProxyError::Child("stdin channel closed".into()))
    }

    /// Close stdin (sends EOF to child).
    pub fn close_stdin(&mut self) {
        self.stdin_tx = None;
    }

    /// Graceful shutdown: close stdin, wait grace period, then SIGKILL.
    pub async fn shutdown(&mut self, grace: Duration) -> Result<()> {
        self.close_stdin();
        self.wait_or_kill(grace).await;
        self.child = None;
        self.stdin_tx = None;
        Ok(())
    }

    /// Wait for child exit or kill after timeout.
    async fn wait_or_kill(&mut self, grace: Duration) {
        let child = match self.child.as_mut() {
            Some(c) => c,
            None => return,
        };
        match tokio::time::timeout(grace, child.wait()).await {
            Ok(Ok(status)) => tracing::info!("Child exited: {status}"),
            Ok(Err(e)) => tracing::warn!("Child wait error: {e}"),
            Err(_) => {
                tracing::warn!("Grace period expired, sending SIGKILL");
                let _ = child.kill().await;
            }
        }
    }

    /// Check if child is still running.
    pub fn is_running(&mut self) -> bool {
        match self.child.as_mut() {
            Some(child) => child.try_wait().ok().flatten().is_none(),
            None => false,
        }
    }
}

/// Verify a binary path exists and is executable.
pub fn verify_binary(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(ProxyError::BinaryNotFound {
            path: path.display().to_string(),
        });
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(path)?;
        if meta.permissions().mode() & 0o111 == 0 {
            return Err(ProxyError::BinaryNotExecutable {
                path: path.display().to_string(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_nonexistent_binary_is_not_found() {
        match verify_binary(Path::new("/nonexistent/binary")) {
            Err(ProxyError::BinaryNotFound { .. }) => {} // expected
            other => panic!("expected BinaryNotFound, got: {other:?}"),
        }
    }

    #[test]
    fn verify_existing_binary() {
        assert!(verify_binary(Path::new("/bin/sh")).is_ok());
    }

    #[test]
    fn managed_child_new_not_running() {
        let mut child = ManagedChild::new(Path::new("/bin/sh"), vec![]);
        assert!(!child.is_running());
    }

    #[test]
    fn ensure_newline_adds_when_missing() {
        assert_eq!(ensure_newline("hello"), "hello\n");
        assert_eq!(ensure_newline("hello\n"), "hello\n");
    }

    #[tokio::test]
    async fn spawn_and_shutdown_cat() {
        let mut child = ManagedChild::new(Path::new("/bin/cat"), vec![]);
        let _rx = child.spawn().unwrap_or_else(|e| panic!("spawn: {e}"));
        assert!(child.is_running());

        assert!(child.shutdown(Duration::from_secs(1)).await.is_ok());
        assert!(!child.is_running());
    }

    #[tokio::test]
    async fn send_line_to_cat() {
        let mut child = ManagedChild::new(Path::new("/bin/cat"), vec![]);
        let mut rx = child.spawn().unwrap_or_else(|e| panic!("spawn: {e}"));

        assert!(child.send_line("hello world").await.is_ok());

        match tokio::time::timeout(Duration::from_secs(2), rx.recv()).await {
            Ok(Some(ChildLine::Stdout(s))) => assert_eq!(s, "hello world"),
            other => panic!("expected stdout line, got: {other:?}"),
        }

        let _ = child.shutdown(Duration::from_secs(1)).await;
    }
}
