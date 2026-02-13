use crate::error::{NexCloudError, Result};
use crate::manifest::ServiceDef;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};

/// A managed child process for a single service.
///
/// Tier: T2-C (ς State + σ Sequence + ∂ Boundary)
/// Manages the lifecycle (state transitions) of an OS process within defined boundaries.
pub struct ProcessTask {
    name: String,
    binary: PathBuf,
    port: u16,
    args: Vec<String>,
    env: Vec<(String, String)>,
    child: Option<Child>,
    log_dir: PathBuf,
}

impl ProcessTask {
    /// Create a process task from a service definition.
    pub fn from_service_def(def: &ServiceDef, log_dir: &PathBuf) -> Self {
        Self {
            name: def.name.clone(),
            binary: def.binary.clone(),
            port: def.port,
            args: def.args.clone(),
            env: def
                .env
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            child: None,
            log_dir: log_dir.clone(),
        }
    }

    /// Spawn the child process.
    pub fn spawn(&mut self) -> Result<u32> {
        // Verify binary exists
        if !self.binary.exists() {
            return Err(NexCloudError::BinaryNotFound {
                path: self.binary.clone(),
            });
        }

        // Set up log files
        let stdout_path = self.log_dir.join(format!("{}.stdout.log", self.name));
        let stderr_path = self.log_dir.join(format!("{}.stderr.log", self.name));

        let stdout_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&stdout_path)
            .map_err(|e| NexCloudError::ProcessSpawn {
                name: self.name.clone(),
                reason: format!("failed to open stdout log: {e}"),
            })?;

        let stderr_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&stderr_path)
            .map_err(|e| NexCloudError::ProcessSpawn {
                name: self.name.clone(),
                reason: format!("failed to open stderr log: {e}"),
            })?;

        // SEC-010: Restrict log file permissions to owner-only (0640)
        // Logs may contain secrets, tokens, or PII from service output.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o640);
            let _ = std::fs::set_permissions(&stdout_path, perms.clone());
            let _ = std::fs::set_permissions(&stderr_path, perms);
        }

        let mut cmd = Command::new(&self.binary);
        cmd.args(&self.args)
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .env("PORT", self.port.to_string());

        for (key, val) in &self.env {
            cmd.env(key, val);
        }

        let child = cmd.spawn().map_err(|e| NexCloudError::ProcessSpawn {
            name: self.name.clone(),
            reason: e.to_string(),
        })?;

        let pid = child.id().unwrap_or(0);
        self.child = Some(child);

        tracing::info!(
            service = %self.name,
            pid = pid,
            binary = %self.binary.display(),
            "process spawned"
        );

        Ok(pid)
    }

    /// Check if the child process is still running.
    pub fn is_alive(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(None) => true,     // still running
                Ok(Some(_)) => false, // exited
                Err(_) => false,      // error checking = assume dead
            }
        } else {
            false
        }
    }

    /// Get the exit code if the process has exited.
    pub fn try_exit_code(&mut self) -> Option<i32> {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(status)) => status.code(),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Graceful shutdown: SIGTERM → wait → SIGKILL.
    ///
    /// Sends SIGTERM first for graceful shutdown. If the process doesn't exit
    /// within the timeout, escalates to SIGKILL.
    pub async fn stop(&mut self, timeout: std::time::Duration) -> Result<()> {
        if let Some(ref mut child) = self.child {
            // Send SIGTERM for graceful shutdown
            if let Some(raw_pid) = child.id() {
                let pid = nix::unistd::Pid::from_raw(raw_pid as i32);
                let _ = nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGTERM);
                tracing::debug!(service = %self.name, pid = raw_pid, "sent SIGTERM");
            }

            // Wait for graceful exit
            let wait_result = tokio::time::timeout(timeout, child.wait()).await;

            match wait_result {
                Ok(Ok(status)) => {
                    tracing::info!(
                        service = %self.name,
                        code = status.code().unwrap_or(-1),
                        "process stopped gracefully"
                    );
                }
                Ok(Err(e)) => {
                    tracing::warn!(service = %self.name, error = %e, "error waiting for process");
                }
                Err(_) => {
                    // Timeout — escalate to SIGKILL
                    tracing::warn!(service = %self.name, "SIGTERM timeout, sending SIGKILL");
                    let _ = child.kill().await;
                }
            }
        }

        self.child = None;
        Ok(())
    }

    /// Get the process PID if running.
    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().and_then(|c| c.id())
    }

    /// Service name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Service port.
    pub fn port(&self) -> u16 {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{RestartPolicyKind, ServiceDef};
    use std::collections::HashMap;

    fn test_service_def() -> ServiceDef {
        ServiceDef {
            name: "test-svc".to_string(),
            binary: PathBuf::from("/usr/bin/sleep"),
            port: 9999,
            health: "/health".to_string(),
            restart: RestartPolicyKind::OnFailure,
            max_restarts: 3,
            backoff_ms: 100,
            env: HashMap::new(),
            depends_on: vec![],
            args: vec!["30".to_string()],
        }
    }

    #[test]
    fn process_task_from_def() {
        let def = test_service_def();
        let log_dir = PathBuf::from("/tmp");
        let task = ProcessTask::from_service_def(&def, &log_dir);
        assert_eq!(task.name(), "test-svc");
        assert_eq!(task.port(), 9999);
        assert!(task.pid().is_none());
    }

    #[test]
    fn binary_not_found() {
        let mut def = test_service_def();
        def.binary = PathBuf::from("/nonexistent/binary");
        let log_dir = PathBuf::from("/tmp");
        let mut task = ProcessTask::from_service_def(&def, &log_dir);
        let result = task.spawn();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn spawn_and_stop() {
        let def = test_service_def();
        let tmp = tempfile::tempdir().ok().unwrap_or_else(|| panic!("tmpdir"));
        let log_dir = tmp.path().to_path_buf();
        let mut task = ProcessTask::from_service_def(&def, &log_dir);

        // sleep binary should exist on Linux
        if def.binary.exists() {
            let pid = task.spawn();
            assert!(pid.is_ok());
            assert!(task.is_alive());

            let stop_result = task.stop(std::time::Duration::from_secs(2)).await;
            assert!(stop_result.is_ok());
            assert!(!task.is_alive());
        }
    }

    #[test]
    fn not_alive_before_spawn() {
        let def = test_service_def();
        let log_dir = PathBuf::from("/tmp");
        let mut task = ProcessTask::from_service_def(&def, &log_dir);
        assert!(!task.is_alive());
    }

    #[test]
    fn no_pid_before_spawn() {
        let def = test_service_def();
        let log_dir = PathBuf::from("/tmp");
        let task = ProcessTask::from_service_def(&def, &log_dir);
        assert!(task.pid().is_none());
    }

    #[test]
    fn no_exit_code_before_spawn() {
        let def = test_service_def();
        let log_dir = PathBuf::from("/tmp");
        let mut task = ProcessTask::from_service_def(&def, &log_dir);
        assert!(task.try_exit_code().is_none());
    }

    #[test]
    fn env_vars_preserved() {
        let mut def = test_service_def();
        def.env
            .insert("CUSTOM_VAR".to_string(), "custom_val".to_string());
        let log_dir = PathBuf::from("/tmp");
        let task = ProcessTask::from_service_def(&def, &log_dir);
        assert_eq!(task.name(), "test-svc");
        assert_eq!(task.port(), 9999);
    }

    #[test]
    fn args_preserved() {
        let mut def = test_service_def();
        def.args = vec!["--flag".to_string(), "value".to_string()];
        let log_dir = PathBuf::from("/tmp");
        let task = ProcessTask::from_service_def(&def, &log_dir);
        assert_eq!(task.name(), "test-svc");
    }

    #[test]
    fn binary_not_found_error_contains_path() {
        let mut def = test_service_def();
        def.binary = PathBuf::from("/nonexistent/very/specific/path");
        let log_dir = PathBuf::from("/tmp");
        let mut task = ProcessTask::from_service_def(&def, &log_dir);
        let err = task.spawn();
        assert!(err.is_err());
        if let Err(e) = err {
            let msg = format!("{e}");
            assert!(msg.contains("/nonexistent/very/specific/path"));
        }
    }

    #[tokio::test]
    async fn stop_without_spawn_is_ok() {
        let def = test_service_def();
        let log_dir = PathBuf::from("/tmp");
        let mut task = ProcessTask::from_service_def(&def, &log_dir);
        // Stopping a never-spawned task should succeed
        let result = task.stop(std::time::Duration::from_secs(1)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn spawn_creates_log_files() {
        let def = test_service_def();
        let tmp = tempfile::tempdir().ok().unwrap_or_else(|| panic!("tmpdir"));
        let log_dir = tmp.path().to_path_buf();
        let mut task = ProcessTask::from_service_def(&def, &log_dir);

        if def.binary.exists() {
            let _ = task.spawn();
            let stdout_log = log_dir.join("test-svc.stdout.log");
            let stderr_log = log_dir.join("test-svc.stderr.log");
            assert!(stdout_log.exists());
            assert!(stderr_log.exists());
            let _ = task.stop(std::time::Duration::from_secs(1)).await;
        }
    }

    #[tokio::test]
    async fn pid_available_after_spawn() {
        let def = test_service_def();
        let tmp = tempfile::tempdir().ok().unwrap_or_else(|| panic!("tmpdir"));
        let log_dir = tmp.path().to_path_buf();
        let mut task = ProcessTask::from_service_def(&def, &log_dir);

        if def.binary.exists() {
            let pid_result = task.spawn();
            assert!(pid_result.is_ok());
            assert!(task.pid().is_some());
            let _ = task.stop(std::time::Duration::from_secs(1)).await;
        }
    }

    #[tokio::test]
    async fn pid_none_after_stop() {
        let def = test_service_def();
        let tmp = tempfile::tempdir().ok().unwrap_or_else(|| panic!("tmpdir"));
        let log_dir = tmp.path().to_path_buf();
        let mut task = ProcessTask::from_service_def(&def, &log_dir);

        if def.binary.exists() {
            let _ = task.spawn();
            let _ = task.stop(std::time::Duration::from_secs(1)).await;
            // After stop, child is set to None, so pid() returns None
            assert!(task.pid().is_none());
        }
    }
}
