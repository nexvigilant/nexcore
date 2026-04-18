//! Process sandbox — Linux namespace isolation for terminal sessions.
//!
//! Uses `clone(CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWNET | CLONE_NEWUSER)`
//! with seccomp BPF and cgroup v2 limits. No Docker dependency.
//!
//! ## Phase 3 Contract
//!
//! Types are defined now so that `nexcore-api` WebSocket handlers can compile
//! against the sandbox interface. Implementation bodies return
//! `SandboxError::NotImplemented` until the namespace + seccomp work lands.
//!
//! ## Primitive Grounding
//!
//! `∂(Boundary: process isolation) + ς(State: sandbox lifecycle) + N(Quantity: resource limits)`

use serde::{Deserialize, Serialize};

use crate::config::SandboxConfig;
use crate::pty::{PtyConfig, PtyError, PtyProcess};

/// Sandbox lifecycle status.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxStatus {
    /// Namespace and cgroup being provisioned.
    Provisioning,
    /// Sandbox running, PTY attached.
    Running,
    /// Resource limit exceeded (OOM, CPU, disk).
    ResourceExceeded,
    /// Sandbox torn down.
    Terminated,
}

/// Error type for sandbox operations.
#[non_exhaustive]
#[derive(Debug)]
pub enum SandboxError {
    /// Sandbox subsystem not yet implemented (Phase 3).
    NotImplemented,
    /// PTY error within the sandbox.
    Pty(PtyError),
    /// Namespace creation failed.
    NamespaceFailed(String),
    /// Cgroup setup failed.
    CgroupFailed(String),
    /// Seccomp filter installation failed.
    SeccompFailed(String),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented => write!(f, "sandbox not yet implemented (Phase 3)"),
            Self::Pty(e) => write!(f, "sandbox PTY error: {e}"),
            Self::NamespaceFailed(e) => write!(f, "namespace creation failed: {e}"),
            Self::CgroupFailed(e) => write!(f, "cgroup setup failed: {e}"),
            Self::SeccompFailed(e) => write!(f, "seccomp filter failed: {e}"),
        }
    }
}

impl std::error::Error for SandboxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Pty(e) => Some(e),
            Self::NotImplemented
            | Self::NamespaceFailed(_)
            | Self::CgroupFailed(_)
            | Self::SeccompFailed(_) => None,
        }
    }
}

impl From<PtyError> for SandboxError {
    fn from(e: PtyError) -> Self {
        Self::Pty(e)
    }
}

/// A sandboxed terminal process — PTY running inside Linux namespace isolation.
///
/// ## Phase 3 Implementation Plan
///
/// 1. `clone(CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWNET | CLONE_NEWUSER)`
/// 2. Mount tmpfs at `/workspace` inside the namespace
/// 3. Apply cgroup v2 limits from [`SandboxConfig`]
/// 4. Install seccomp BPF filter (allowlist based on tier)
/// 5. Spawn PTY inside the namespace
pub struct SandboxedProcess {
    /// The underlying PTY process (runs inside the sandbox once implemented).
    pty: PtyProcess,
    /// Sandbox configuration (resource limits, network policy).
    config: SandboxConfig,
    /// Current lifecycle status.
    status: SandboxStatus,
}

impl SandboxedProcess {
    /// Spawn a sandboxed terminal process.
    ///
    /// Phase 3: This will create namespaces, apply cgroup limits, install
    /// seccomp filters, then spawn the PTY inside the isolated environment.
    ///
    /// Current: Spawns an unsandboxed PTY process (passthrough mode).
    ///
    /// # Errors
    ///
    /// Returns `SandboxError::Pty` if the PTY spawn fails.
    pub fn spawn(
        pty_config: PtyConfig,
        sandbox_config: SandboxConfig,
    ) -> Result<Self, SandboxError> {
        // Phase 3: namespace + cgroup + seccomp setup goes here.
        // For now, direct PTY spawn (no isolation).
        let pty = PtyProcess::spawn(pty_config)?;
        Ok(Self {
            pty,
            config: sandbox_config,
            status: SandboxStatus::Running,
        })
    }

    /// Write data to the sandboxed PTY.
    ///
    /// # Errors
    ///
    /// Returns `SandboxError::Pty` on write failure.
    pub async fn write(&mut self, data: &[u8]) -> Result<(), SandboxError> {
        self.pty.write(data).await.map_err(SandboxError::from)
    }

    /// Read data from the sandboxed PTY.
    ///
    /// # Errors
    ///
    /// Returns `SandboxError::Pty` on read failure.
    pub async fn read(&mut self, buf_size: usize) -> Result<Vec<u8>, SandboxError> {
        self.pty.read(buf_size).await.map_err(SandboxError::from)
    }

    /// Kill the sandboxed process and tear down the sandbox.
    ///
    /// # Errors
    ///
    /// Returns `SandboxError::Pty` if the kill signal fails.
    pub async fn kill(&mut self) -> Result<(), SandboxError> {
        self.pty.kill().await?;
        self.status = SandboxStatus::Terminated;
        // Phase 3: tear down cgroup + namespace here.
        Ok(())
    }

    /// Current sandbox status.
    #[must_use]
    pub fn status(&self) -> SandboxStatus {
        self.status
    }

    /// Reference to the sandbox resource configuration.
    #[must_use]
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    /// Whether the sandbox has been terminated.
    #[must_use]
    pub fn is_terminated(&self) -> bool {
        self.status == SandboxStatus::Terminated
    }

    /// Access the inner PTY for resize and status checks.
    #[must_use]
    pub fn pty(&self) -> &PtyProcess {
        &self.pty
    }

    /// Mutable access to the inner PTY for resize operations.
    pub fn pty_mut(&mut self) -> &mut PtyProcess {
        &mut self.pty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vr_core::tenant::SubscriptionTier;

    #[test]
    fn sandbox_error_display() {
        let err = SandboxError::NotImplemented;
        assert_eq!(format!("{err}"), "sandbox not yet implemented (Phase 3)");

        let err = SandboxError::NamespaceFailed("EPERM".to_string());
        assert!(format!("{err}").contains("EPERM"));
    }

    #[test]
    fn sandbox_status_serde_roundtrip() {
        let status = SandboxStatus::Running;
        let json = serde_json::to_string(&status).unwrap_or_default();
        assert_eq!(json, "\"running\"");
    }

    #[tokio::test]
    async fn spawn_passthrough_mode() {
        let pty_config = PtyConfig::new("/bin/bash", "/tmp");
        let sandbox_config = SandboxConfig::from_tier(&SubscriptionTier::Explorer);

        let result = SandboxedProcess::spawn(pty_config, sandbox_config);
        assert!(result.is_ok(), "passthrough sandbox spawn should succeed");

        if let Ok(mut proc) = result {
            assert_eq!(proc.status(), SandboxStatus::Running);
            assert!(!proc.is_terminated());
            let kill_result = proc.kill().await;
            assert!(kill_result.is_ok());
            assert!(proc.is_terminated());
            assert_eq!(proc.status(), SandboxStatus::Terminated);
        }
    }
}
