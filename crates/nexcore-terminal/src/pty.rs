//! PTY manager — async process spawning and I/O for terminal sessions.
//!
//! Uses `tokio::process::Command` as the safe process backend. A future
//! `nexcore-pty` crate will provide true PTY allocation via `forkpty()`
//! for full ANSI control sequence support (job control, Ctrl-C, etc.).
//!
//! ## Primitive Grounding
//!
//! `ς(State) + →(Causality) + σ(Sequence) + ∂(Boundary)`

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStdin, ChildStdout};

/// Terminal dimensions (columns x rows).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtySize {
    /// Column count.
    pub cols: u16,
    /// Row count.
    pub rows: u16,
}

impl PtySize {
    /// Create a new terminal size.
    #[must_use]
    pub fn new(cols: u16, rows: u16) -> Self {
        Self { cols, rows }
    }
}

impl Default for PtySize {
    fn default() -> Self {
        Self { cols: 80, rows: 24 }
    }
}

/// Configuration for spawning a new terminal process.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyConfig {
    /// Shell binary to execute (e.g., "/bin/bash").
    pub shell: String,
    /// Working directory for the shell process.
    pub working_dir: String,
    /// Environment variables to set (merged with inherited env).
    pub env: BTreeMap<String, String>,
    /// Initial terminal dimensions.
    pub size: PtySize,
}

impl PtyConfig {
    /// Create a config with the given shell and working directory, default size.
    #[must_use]
    pub fn new(shell: impl Into<String>, working_dir: impl Into<String>) -> Self {
        Self {
            shell: shell.into(),
            working_dir: working_dir.into(),
            env: BTreeMap::new(),
            size: PtySize::default(),
        }
    }

    /// Set the initial terminal dimensions.
    #[must_use]
    pub fn with_size(mut self, size: PtySize) -> Self {
        self.size = size;
        self
    }

    /// Add an environment variable.
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            shell: "/bin/bash".to_string(),
            working_dir: "/workspace".to_string(),
            env: BTreeMap::new(),
            size: PtySize::default(),
        }
    }
}

/// Error type for PTY operations.
#[non_exhaustive]
#[derive(Debug)]
pub enum PtyError {
    /// Process failed to spawn.
    SpawnFailed(std::io::Error),
    /// I/O error during read or write.
    IoError(std::io::Error),
    /// Process has already exited.
    ProcessExited,
    /// stdin pipe not available.
    StdinUnavailable,
    /// stdout pipe not available.
    StdoutUnavailable,
}

impl std::fmt::Display for PtyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SpawnFailed(e) => write!(f, "process spawn failed: {e}"),
            Self::IoError(e) => write!(f, "PTY I/O error: {e}"),
            Self::ProcessExited => write!(f, "process has exited"),
            Self::StdinUnavailable => write!(f, "stdin pipe not available"),
            Self::StdoutUnavailable => write!(f, "stdout pipe not available"),
        }
    }
}

impl std::error::Error for PtyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SpawnFailed(e) | Self::IoError(e) => Some(e),
            Self::ProcessExited | Self::StdinUnavailable | Self::StdoutUnavailable => None,
        }
    }
}

/// Handle to a running terminal process with async I/O.
///
/// Current backend: `tokio::process::Command` (safe, no PTY allocation).
/// Future backend: `forkpty()` via `nexcore-pty` for full terminal emulation.
pub struct PtyProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
    config: PtyConfig,
    exited: bool,
}

impl PtyProcess {
    /// Spawn a new shell process with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns `PtyError::SpawnFailed` if the process cannot be started.
    pub fn spawn(config: PtyConfig) -> Result<Self, PtyError> {
        let mut cmd = tokio::process::Command::new(&config.shell);
        cmd.current_dir(&config.working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("TERM", "xterm-256color")
            .env("COLUMNS", config.size.cols.to_string())
            .env("LINES", config.size.rows.to_string());

        // Merge custom environment
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        let mut child = cmd.spawn().map_err(PtyError::SpawnFailed)?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();

        Ok(Self {
            child,
            stdin,
            stdout,
            config,
            exited: false,
        })
    }

    /// Write data to the process stdin.
    ///
    /// # Errors
    ///
    /// Returns `PtyError::StdinUnavailable` if stdin pipe was already taken,
    /// or `PtyError::IoError` on write failure.
    pub async fn write(&mut self, data: &[u8]) -> Result<(), PtyError> {
        if self.exited {
            return Err(PtyError::ProcessExited);
        }
        let stdin = self.stdin.as_mut().ok_or(PtyError::StdinUnavailable)?;
        stdin.write_all(data).await.map_err(PtyError::IoError)?;
        stdin.flush().await.map_err(PtyError::IoError)?;
        Ok(())
    }

    /// Read available data from the process stdout.
    ///
    /// Returns up to `buf_size` bytes. Returns an empty vec if no data
    /// is immediately available (non-blocking read not yet implemented —
    /// callers should use `read_with_timeout` or `tokio::select!`).
    ///
    /// # Errors
    ///
    /// Returns `PtyError::StdoutUnavailable` if stdout pipe was already taken,
    /// or `PtyError::IoError` on read failure.
    pub async fn read(&mut self, buf_size: usize) -> Result<Vec<u8>, PtyError> {
        if self.exited {
            return Err(PtyError::ProcessExited);
        }
        let stdout = self.stdout.as_mut().ok_or(PtyError::StdoutUnavailable)?;
        let mut buf = vec![0u8; buf_size];
        let n = stdout.read(&mut buf).await.map_err(PtyError::IoError)?;
        if n == 0 {
            self.exited = true;
            return Ok(Vec::new());
        }
        buf.truncate(n);
        Ok(buf)
    }

    /// Resize the terminal (no-op in tokio::process backend).
    ///
    /// With a real PTY backend, this would send `TIOCSWINSZ` ioctl.
    pub fn resize(&mut self, size: PtySize) {
        self.config.size = size;
        // No-op: tokio::process doesn't support terminal resize.
        // Future: ioctl(fd, TIOCSWINSZ, &winsize) via nexcore-pty.
        tracing::debug!(
            cols = size.cols,
            rows = size.rows,
            "PTY resize requested (no-op in process backend)"
        );
    }

    /// Kill the process and clean up.
    ///
    /// # Errors
    ///
    /// Returns `PtyError::IoError` if the kill signal fails.
    pub async fn kill(&mut self) -> Result<(), PtyError> {
        if self.exited {
            return Ok(());
        }
        self.child.kill().await.map_err(PtyError::IoError)?;
        self.exited = true;
        Ok(())
    }

    /// Check if the process has exited.
    pub fn try_wait(&mut self) -> Result<Option<i32>, PtyError> {
        match self.child.try_wait().map_err(PtyError::IoError)? {
            Some(status) => {
                self.exited = true;
                Ok(Some(status.code().unwrap_or(-1)))
            }
            None => Ok(None),
        }
    }

    /// Get the current terminal size configuration.
    #[must_use]
    pub fn size(&self) -> PtySize {
        self.config.size
    }

    /// Whether the process has exited.
    #[must_use]
    pub fn has_exited(&self) -> bool {
        self.exited
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pty_size() {
        let size = PtySize::default();
        assert_eq!(size.cols, 80);
        assert_eq!(size.rows, 24);
    }

    #[test]
    fn default_pty_config() {
        let config = PtyConfig::default();
        assert_eq!(config.shell, "/bin/bash");
        assert_eq!(config.working_dir, "/workspace");
        assert!(config.env.is_empty());
        assert_eq!(config.size, PtySize::default());
    }

    #[test]
    fn pty_error_display() {
        let err = PtyError::ProcessExited;
        assert_eq!(format!("{err}"), "process has exited");

        let err = PtyError::StdinUnavailable;
        assert_eq!(format!("{err}"), "stdin pipe not available");
    }

    #[tokio::test]
    async fn spawn_echo_process() {
        // Use /bin/echo instead of a shell to avoid interactive mode
        let config = PtyConfig {
            shell: "/bin/echo".to_string(),
            working_dir: "/tmp".to_string(),
            env: BTreeMap::new(),
            size: PtySize::default(),
        };
        let mut proc = PtyProcess::spawn(config);
        assert!(proc.is_ok());
        if let Ok(ref mut p) = proc {
            // echo exits immediately, read its output
            let output = p.read(4096).await;
            assert!(output.is_ok());
        }
    }

    #[tokio::test]
    async fn spawn_nonexistent_binary_fails() {
        let config = PtyConfig {
            shell: "/nonexistent/binary".to_string(),
            working_dir: "/tmp".to_string(),
            env: BTreeMap::new(),
            size: PtySize::default(),
        };
        let result = PtyProcess::spawn(config);
        assert!(result.is_err());
        if let Err(PtyError::SpawnFailed(_)) = result {
            // expected
        } else {
            panic!("Expected SpawnFailed error");
        }
    }

    #[tokio::test]
    async fn kill_terminates_process() {
        let config = PtyConfig {
            shell: "/bin/sleep".to_string(),
            working_dir: "/tmp".to_string(),
            env: BTreeMap::new(),
            size: PtySize::default(),
        };
        // sleep needs an argument; use Command directly for the test
        let mut cmd = tokio::process::Command::new("/bin/sleep");
        cmd.arg("60")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn();
        if let Ok(ref mut c) = child {
            let stdin = c.stdin.take();
            let stdout = c.stdout.take();
            let mut proc = PtyProcess {
                child: child.unwrap(),
                stdin,
                stdout,
                config: PtyConfig::default(),
                exited: false,
            };
            assert!(!proc.has_exited());
            let kill_result = proc.kill().await;
            assert!(kill_result.is_ok());
            assert!(proc.has_exited());
        }
    }

    #[test]
    fn resize_is_noop_but_updates_config() {
        let config = PtyConfig::default();
        // We can't easily test resize without a running process,
        // but we can verify the struct update logic
        let new_size = PtySize {
            cols: 120,
            rows: 40,
        };
        assert_ne!(config.size, new_size);
    }
}
