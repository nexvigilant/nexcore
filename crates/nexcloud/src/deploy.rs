use crate::error::{NexCloudError, Result};
use crate::manifest::CloudManifest;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Stdio;

/// Remote deployment target configuration.
///
/// Tier: T2-C (λ Location + σ Sequence + ∂ Boundary + π Persistence)
/// Maps a local build to a remote location via a sequenced deploy pipeline.
///
/// Constitutional: Enumerated powers — only explicitly declared targets receive deployments.
#[derive(Debug, Clone, Deserialize)]
pub struct DeployTarget {
    /// SSH host (e.g., "user@host" or just "host")
    pub host: String,
    /// Remote directory where binaries are placed
    #[serde(default = "default_remote_dir")]
    pub remote_dir: String,
    /// SSH key path (optional, uses default ssh agent if not set)
    pub ssh_key: Option<PathBuf>,
    /// SSH port
    #[serde(default = "default_ssh_port")]
    pub ssh_port: u16,
}

fn default_remote_dir() -> String {
    "/opt/nexcloud/bin".to_string()
}

fn default_ssh_port() -> u16 {
    22
}

/// Deploy pipeline: build → upload → restart.
///
/// Tier: T3 (σ Sequence + λ Location + ∂ Boundary + ∝ Irreversibility + π Persistence)
/// Full domain type — irreversible deployment sequence across network boundaries.
///
/// Constitutional: Due process — build must succeed before upload, upload before restart.
pub struct DeployPipeline {
    target: DeployTarget,
}

impl DeployPipeline {
    /// Create a new deploy pipeline.
    pub fn new(target: DeployTarget) -> Self {
        Self { target }
    }

    /// Execute the full deploy pipeline for a specific service.
    ///
    /// SEC-001: Validates service_name before it flows into shell commands.
    pub async fn deploy_service(&self, service_name: &str, manifest: &CloudManifest) -> Result<()> {
        // SEC-001: Reject service names with shell metacharacters.
        // Service names flow into SSH command strings — must be alphanumeric + dash/underscore only.
        if service_name.is_empty()
            || service_name.len() > 64
            || !service_name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(NexCloudError::ManifestValidation(format!(
                "service name '{}' contains invalid characters for deploy",
                service_name
            )));
        }

        let svc = manifest.service_by_name(service_name).ok_or_else(|| {
            NexCloudError::ServiceNotFound {
                name: service_name.to_string(),
            }
        })?;

        let binary_path = &svc.binary;

        tracing::info!(
            service = %service_name,
            binary = %binary_path.display(),
            host = %self.target.host,
            "starting deploy"
        );

        // Phase 1: Build
        self.build_release(binary_path).await?;

        // Phase 2: Upload via SCP
        self.upload_binary(binary_path, service_name).await?;

        // Phase 3: Restart service on remote
        self.restart_remote(service_name).await?;

        tracing::info!(service = %service_name, "deploy complete");
        Ok(())
    }

    /// Build the release binary.
    ///
    /// Constitutional: Due process — validate before executing.
    async fn build_release(&self, binary_path: &Path) -> Result<()> {
        // Determine the binary name for cargo build
        let binary_name = binary_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| NexCloudError::ProcessSpawn {
                name: "build".to_string(),
                reason: format!("invalid binary path: {}", binary_path.display()),
            })?;

        tracing::info!(binary = %binary_name, "building release binary");

        let output = tokio::process::Command::new("cargo")
            .args(["build", "--release", "--bin", binary_name])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| NexCloudError::ProcessSpawn {
                name: "cargo build".to_string(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NexCloudError::ProcessSpawn {
                name: "cargo build".to_string(),
                reason: format!("build failed: {stderr}"),
            });
        }

        tracing::info!(binary = %binary_name, "build succeeded");
        Ok(())
    }

    /// Upload the binary via SCP.
    ///
    /// Constitutional: CAP-029 Homeland Security — authenticated transport only.
    async fn upload_binary(&self, binary_path: &Path, service_name: &str) -> Result<()> {
        let remote_path = format!(
            "{}:{}",
            self.target.host,
            format!("{}/{service_name}", self.target.remote_dir)
        );

        let mut args = vec!["-o", "StrictHostKeyChecking=accept-new"];

        // Port
        let port_str = self.target.ssh_port.to_string();
        if self.target.ssh_port != 22 {
            args.extend(["-P", &port_str]);
        }

        // SSH key
        let key_path_str;
        if let Some(ref key) = self.target.ssh_key {
            key_path_str = key.display().to_string();
            args.extend(["-i", &key_path_str]);
        }

        let binary_str = binary_path.display().to_string();
        args.push(&binary_str);
        args.push(&remote_path);

        tracing::info!(
            service = %service_name,
            remote = %remote_path,
            "uploading binary via SCP"
        );

        let output = tokio::process::Command::new("scp")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| NexCloudError::ProcessSpawn {
                name: "scp".to_string(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NexCloudError::ProcessSpawn {
                name: "scp".to_string(),
                reason: format!("upload failed: {stderr}"),
            });
        }

        Ok(())
    }

    /// Restart the service on the remote host via SSH.
    ///
    /// Constitutional: Due process — stop gracefully, then start.
    async fn restart_remote(&self, service_name: &str) -> Result<()> {
        let restart_cmd = format!(
            "nexcloud --manifest /etc/nexcloud/nexcloud.toml restart --service {service_name}"
        );

        let mut args = vec!["-o", "StrictHostKeyChecking=accept-new"];

        let port_str = self.target.ssh_port.to_string();
        if self.target.ssh_port != 22 {
            args.extend(["-p", &port_str]);
        }

        let key_path_str;
        if let Some(ref key) = self.target.ssh_key {
            key_path_str = key.display().to_string();
            args.extend(["-i", &key_path_str]);
        }

        args.push(&self.target.host);
        args.push(&restart_cmd);

        tracing::info!(
            service = %service_name,
            host = %self.target.host,
            "restarting service on remote"
        );

        let output = tokio::process::Command::new("ssh")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| NexCloudError::ProcessSpawn {
                name: "ssh".to_string(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NexCloudError::ProcessSpawn {
                name: "ssh restart".to_string(),
                reason: format!("remote restart failed: {stderr}"),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deploy_target_defaults() {
        let toml_str = r#"host = "user@example.com""#;
        let target: DeployTarget =
            toml::from_str(toml_str).unwrap_or_else(|e| panic!("parse: {e}"));
        assert_eq!(target.host, "user@example.com");
        assert_eq!(target.remote_dir, "/opt/nexcloud/bin");
        assert_eq!(target.ssh_port, 22);
        assert!(target.ssh_key.is_none());
    }

    #[test]
    fn deploy_target_custom() {
        let toml_str = r#"
host = "deploy@prod.nexvigilant.com"
remote_dir = "/srv/nexcloud/bin"
ssh_key = "/home/deploy/.ssh/nexcloud_ed25519"
ssh_port = 2222
"#;
        let target: DeployTarget =
            toml::from_str(toml_str).unwrap_or_else(|e| panic!("parse: {e}"));
        assert_eq!(target.ssh_port, 2222);
        assert!(target.ssh_key.is_some());
    }
}
