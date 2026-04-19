//! PTY manager — async process spawning and I/O for terminal sessions.
//!
//! Uses `nexcore-pty` for real POSIX PTY allocation (`openpty`, `forkpty`,
//! `TIOCSWINSZ` ioctl). Provides full terminal emulation: job control,
//! Ctrl-C/Ctrl-Z signal delivery, ICRNL line discipline, and ANSI support.
//!
//! ## Primitive Grounding
//!
//! `ς(State) + →(Causality) + σ(Sequence) + ∂(Boundary)`

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::os::fd::{AsRawFd, OwnedFd};
use tokio::io::unix::AsyncFd;

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
    /// Extra argv entries passed after argv[0]. Each entry is forwarded verbatim
    /// as a single argument — the spawn layer does not split on whitespace, so
    /// shell-injection via embedded spaces is structurally impossible.
    pub args: Vec<String>,
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
            args: Vec::new(),
            working_dir: working_dir.into(),
            env: BTreeMap::new(),
            size: PtySize::default(),
        }
    }

    /// Append a single argument to argv (after argv[0]).
    #[must_use]
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
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
            args: Vec::new(),
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
    /// No data available (spurious readiness, not EOF).
    WouldBlock,
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
            Self::WouldBlock => write!(f, "no data available"),
            Self::StdinUnavailable => write!(f, "stdin pipe not available"),
            Self::StdoutUnavailable => write!(f, "stdout pipe not available"),
        }
    }
}

impl std::error::Error for PtyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SpawnFailed(e) | Self::IoError(e) => Some(e),
            Self::ProcessExited
            | Self::WouldBlock
            | Self::StdinUnavailable
            | Self::StdoutUnavailable => None,
        }
    }
}

/// Handle to a running terminal process with async I/O over a real PTY.
///
/// Backend: `nexcore-pty` POSIX PTY allocation (`openpty` + `forkpty`).
/// The master fd is wrapped in `tokio::io::unix::AsyncFd` for async reads
/// and writes via the tokio reactor.
pub struct PtyProcess {
    master: AsyncFd<OwnedFd>,
    child_pid: u32,
    config: PtyConfig,
    exited: bool,
}

impl PtyProcess {
    /// Spawn a new shell process with the given configuration.
    ///
    /// Allocates a real POSIX PTY, forks, and execs the shell. The child
    /// gets a controlling terminal with full line discipline (ICRNL, echo,
    /// signal generation).
    ///
    /// # Errors
    ///
    /// Returns `PtyError::SpawnFailed` if PTY allocation or fork/exec fails.
    pub fn spawn(config: PtyConfig) -> Result<Self, PtyError> {
        let ws = nexcore_pty::WinSize {
            rows: config.size.rows,
            cols: config.size.cols,
        };

        let pair = nexcore_pty::open_pty(ws).map_err(|e| PtyError::SpawnFailed(e.into_io()))?;

        nexcore_pty::set_nonblocking(&pair.master)
            .map_err(|e| PtyError::SpawnFailed(e.into_io()))?;

        let env_pairs: Vec<(String, String)> = vec![
            ("TERM".to_string(), "xterm-256color".to_string()),
            ("COLUMNS".to_string(), config.size.cols.to_string()),
            ("LINES".to_string(), config.size.rows.to_string()),
        ]
        .into_iter()
        .chain(config.env.iter().map(|(k, v)| (k.clone(), v.clone())))
        .collect();

        // argv[0] is the program path by convention; any user-provided args
        // follow as separate argv entries (never tokenized).
        let mut argv: Vec<&str> = Vec::with_capacity(1 + config.args.len());
        argv.push(&config.shell);
        for a in &config.args {
            argv.push(a.as_str());
        }

        let spawn_config = nexcore_pty::SpawnConfig {
            program: &config.shell,
            args: &argv,
            working_dir: &config.working_dir,
            env: &env_pairs,
        };

        let master_raw = pair.master.as_raw_fd();
        let child_pid = nexcore_pty::fork_exec(pair.slave, master_raw, &spawn_config)
            .map_err(|e| PtyError::SpawnFailed(e.into_io()))?;

        let async_master = AsyncFd::new(pair.master).map_err(|e| PtyError::SpawnFailed(e))?;

        Ok(Self {
            master: async_master,
            child_pid,
            config,
            exited: false,
        })
    }

    /// Write data to the PTY master (goes to child's stdin).
    ///
    /// The PTY line discipline translates CR to LF (ICRNL) and generates
    /// signals from control characters (Ctrl-C → SIGINT, Ctrl-Z → SIGTSTP).
    ///
    /// # Errors
    ///
    /// Returns `PtyError::ProcessExited` if the child has exited,
    /// or `PtyError::IoError` on write failure.
    pub async fn write(&mut self, data: &[u8]) -> Result<(), PtyError> {
        if self.exited {
            return Err(PtyError::ProcessExited);
        }

        let mut guard = self.master.writable().await.map_err(PtyError::IoError)?;

        match guard.try_io(|inner| nexcore_pty::write_master(inner.get_ref(), data)) {
            Ok(Ok(_n)) => Ok(()),
            Ok(Err(e)) => Err(PtyError::IoError(e)),
            Err(_would_block) => {
                // Spurious readiness — retry once.
                drop(guard);
                let mut guard2 = self.master.writable().await.map_err(PtyError::IoError)?;
                match guard2.try_io(|inner| nexcore_pty::write_master(inner.get_ref(), data)) {
                    Ok(Ok(_n)) => Ok(()),
                    Ok(Err(e)) => Err(PtyError::IoError(e)),
                    Err(_) => Err(PtyError::IoError(std::io::Error::new(
                        std::io::ErrorKind::WouldBlock,
                        "PTY write would block after retry",
                    ))),
                }
            }
        }
    }

    /// Read available data from the PTY master (child's stdout/stderr).
    ///
    /// Returns up to `buf_size` bytes. Output includes terminal control
    /// sequences, echo, and program output — all multiplexed through the
    /// PTY line discipline.
    ///
    /// # Errors
    ///
    /// Returns `PtyError::ProcessExited` if the child has exited,
    /// or `PtyError::IoError` on read failure.
    pub async fn read(&mut self, buf_size: usize) -> Result<Vec<u8>, PtyError> {
        if self.exited {
            return Err(PtyError::ProcessExited);
        }

        let mut guard = self.master.readable().await.map_err(PtyError::IoError)?;

        let mut buf = vec![0u8; buf_size];

        match guard.try_io(|inner| nexcore_pty::read_master(inner.get_ref(), &mut buf)) {
            Ok(Ok(0)) => {
                self.exited = true;
                Ok(Vec::new())
            }
            Ok(Ok(n)) => {
                buf.truncate(n);
                Ok(buf)
            }
            Ok(Err(e)) => Err(PtyError::IoError(e)),
            Err(_would_block) => {
                // Spurious readiness — no data yet. Distinct from EOF.
                Err(PtyError::WouldBlock)
            }
        }
    }

    /// Resize the terminal via TIOCSWINSZ ioctl.
    ///
    /// Sends the new dimensions to the terminal driver, which delivers
    /// SIGWINCH to the child's foreground process group.
    pub fn resize(&mut self, size: PtySize) {
        self.config.size = size;
        let ws = nexcore_pty::WinSize {
            rows: size.rows,
            cols: size.cols,
        };
        if let Err(e) = nexcore_pty::resize(self.master.get_ref(), ws) {
            tracing::debug!(
                cols = size.cols,
                rows = size.rows,
                error = %e,
                "PTY resize failed"
            );
        }
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
        // SIGKILL = 9
        if let Err(e) = nexcore_pty::signal_process(self.child_pid, 9) {
            tracing::debug!(pid = self.child_pid, error = %e, "PTY kill signal failed");
            return Err(PtyError::IoError(e.into_io()));
        }
        // Wait for child to exit (bounded spin).
        for _ in 0..50 {
            if let Ok(Some(_)) = nexcore_pty::try_wait_pid(self.child_pid) {
                self.exited = true;
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        self.exited = true;
        Ok(())
    }

    /// Check if the process has exited.
    pub fn try_wait(&mut self) -> Result<Option<i32>, PtyError> {
        match nexcore_pty::try_wait_pid(self.child_pid) {
            Ok(Some(code)) => {
                self.exited = true;
                Ok(Some(code))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(PtyError::IoError(e.into_io())),
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

impl Drop for PtyProcess {
    fn drop(&mut self) {
        if !self.exited {
            // Best-effort kill on drop.
            if let Err(e) = nexcore_pty::signal_process(self.child_pid, 9) {
                tracing::debug!(pid = self.child_pid, error = %e, "PTY drop: kill failed");
                return;
            }
            // Best-effort reap (non-blocking).
            for _ in 0..10 {
                if let Ok(Some(_)) = nexcore_pty::try_wait_pid(self.child_pid) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
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

        let err = PtyError::WouldBlock;
        assert_eq!(format!("{err}"), "no data available");

        let err = PtyError::StdinUnavailable;
        assert_eq!(format!("{err}"), "stdin pipe not available");
    }

    #[tokio::test]
    async fn spawn_and_read() {
        let config = PtyConfig::new("/bin/bash", "/tmp");
        let mut proc = PtyProcess::spawn(config);
        assert!(proc.is_ok(), "spawn should succeed with real PTY");
        if let Ok(ref mut p) = proc {
            // Shell should produce some output (prompt, etc.)
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            let output = p.read(4096).await;
            assert!(output.is_ok());
            // Kill to clean up.
            let kill_result = p.kill().await;
            assert!(kill_result.is_ok());
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
        let config = PtyConfig::new("/bin/bash", "/tmp");
        let result = PtyProcess::spawn(config);
        assert!(result.is_ok());
        if let Ok(mut proc) = result {
            assert!(!proc.has_exited());
            let kill_result = proc.kill().await;
            assert!(kill_result.is_ok());
            assert!(proc.has_exited());
        }
    }

    #[tokio::test]
    async fn resize_sends_ioctl() {
        // Spawn a shell to get a valid PtyProcess, then resize.
        let config = PtyConfig::new("/bin/bash", "/tmp");
        if let Ok(mut proc) = PtyProcess::spawn(config) {
            let new_size = PtySize {
                cols: 120,
                rows: 40,
            };
            // Should not panic — sends real TIOCSWINSZ ioctl.
            proc.resize(new_size);
            assert_eq!(proc.size(), new_size);
            let kill_result = proc.kill().await;
            assert!(kill_result.is_ok());
        }
    }

    #[tokio::test]
    async fn write_and_read_through_pty() {
        let config = PtyConfig::new("/bin/bash", "/tmp");
        if let Ok(mut proc) = PtyProcess::spawn(config) {
            // Wait for shell startup.
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            // Drain initial output.
            let _initial = proc.read(4096).await;

            // Write a command through PTY (CR triggers line discipline).
            let write_result = proc.write(b"echo PTY_TEST_OK\r").await;
            assert!(write_result.is_ok(), "write through PTY should succeed");

            // Read output — should contain our marker.
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            let output = proc.read(4096).await;
            assert!(output.is_ok());
            if let Ok(data) = output {
                let text = String::from_utf8_lossy(&data);
                assert!(
                    text.contains("PTY_TEST_OK"),
                    "PTY output should contain marker, got: {text:?}"
                );
            }

            let kill_result = proc.kill().await;
            assert!(kill_result.is_ok());
        }
    }
}
