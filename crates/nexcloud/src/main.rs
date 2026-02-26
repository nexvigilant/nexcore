use clap::{Parser, Subcommand};
use nexcloud::CloudSupervisor;
use nexcloud::deploy::{DeployPipeline, DeployTarget};
use nexcloud::manifest::CloudManifest;
use nexcloud::supervisor::registry::ProcessState;
use std::path::PathBuf;

/// NexCloud: Rust-native cloud platform.
///
/// Process supervisor + reverse proxy for bare metal and VPS deployments.
/// Replaces GCP Cloud Run with a single binary.
///
/// Constitutional: We the People — a platform of, by, and for the operator.
#[derive(Parser)]
#[command(name = "nexcloud", version, about)]
struct Cli {
    /// Path to the nexcloud.toml manifest file.
    #[arg(short, long, default_value = "nexcloud.toml")]
    manifest: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the platform (all services + reverse proxy).
    Start,

    /// Stop all running services gracefully (SIGTERM → SIGKILL).
    Stop,

    /// Restart a specific service or all services.
    Restart {
        /// Service name to restart. If omitted, restarts all.
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Show status of all services.
    Status,

    /// Validate the manifest without starting services.
    Validate,

    /// Tail service logs.
    Logs {
        /// Service name to tail logs for.
        service: String,

        /// Number of lines to show (default: 50).
        #[arg(short = 'n', long, default_value = "50")]
        lines: usize,

        /// Show stderr instead of stdout.
        #[arg(long)]
        stderr: bool,
    },

    /// Deploy a service to a remote host (build → SCP → restart).
    Deploy {
        /// Service name to deploy.
        service: String,

        /// Remote host (e.g., "user@host" or "host").
        #[arg(long)]
        host: String,

        /// Remote directory for binaries.
        #[arg(long, default_value = "/opt/nexcloud/bin")]
        remote_dir: String,

        /// SSH key path.
        #[arg(long)]
        ssh_key: Option<PathBuf>,

        /// SSH port.
        #[arg(long, default_value = "22")]
        ssh_port: u16,
    },
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Start => cmd_start(&cli.manifest).await,
        Commands::Stop => cmd_stop(&cli.manifest).await,
        Commands::Restart { service } => cmd_restart(&cli.manifest, service.as_deref()).await,
        Commands::Status => cmd_status(&cli.manifest),
        Commands::Validate => cmd_validate(&cli.manifest),
        Commands::Logs {
            service,
            lines,
            stderr,
        } => cmd_logs(&cli.manifest, &service, lines, stderr),
        Commands::Deploy {
            service,
            host,
            remote_dir,
            ssh_key,
            ssh_port,
        } => {
            cmd_deploy(
                &cli.manifest,
                &service,
                &host,
                &remote_dir,
                ssh_key.as_ref(),
                ssh_port,
            )
            .await
        }
    };

    if let Err(e) = result {
        tracing::error!("fatal: {e}");
        std::process::exit(1);
    }
}

/// Start the platform: spawn services + reverse proxy.
///
/// Constitutional: Due process — Ctrl+C triggers graceful shutdown (SIGTERM before SIGKILL).
async fn cmd_start(manifest_path: &PathBuf) -> nexcloud::Result<()> {
    let manifest = CloudManifest::from_file(manifest_path)?;

    tracing::info!(
        platform = %manifest.platform.name,
        services = manifest.services.len(),
        routes = manifest.routes.len(),
        tls = manifest.proxy.tls.is_some(),
        "loaded manifest"
    );

    let mut supervisor = CloudSupervisor::new(manifest)?;

    // Write PID file for detached stop/restart
    let pid_path = pid_file_path(manifest_path);
    write_pid_file(&pid_path)?;

    // SEC-005: Graceful shutdown on SIGINT (Ctrl+C) or SIGTERM (`nexcloud stop`).
    // Without this, child processes become orphans still consuming resources.
    // BUG FIX: Previously only handled SIGINT — SIGTERM from `nexcloud stop` left
    // children running. Now handles both signals identically.
    let registry = supervisor.registry().clone();
    let pid_path_clone = pid_path.clone();
    tokio::spawn(async move {
        // Listen for BOTH SIGINT and SIGTERM — whichever arrives first triggers shutdown
        let sigterm_result =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate());

        match sigterm_result {
            Ok(mut sigterm) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        tracing::info!("received SIGINT (Ctrl+C)");
                    }
                    _ = sigterm.recv() => {
                        tracing::info!("received SIGTERM");
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "SIGTERM handler registration failed ({e}), falling back to SIGINT only"
                );
                if let Err(e) = tokio::signal::ctrl_c().await {
                    tracing::error!(
                        "SIGINT handler also failed: {e} — shutdown requires manual kill"
                    );
                    return;
                }
                tracing::info!("received SIGINT (Ctrl+C)");
            }
        }

        // Shutdown sequence: SIGTERM all children → wait → SIGKILL stragglers
        tracing::info!("initiating graceful shutdown of child processes");
        for record in registry.snapshot() {
            registry.update_state(&record.name, ProcessState::Stopping);
            if let Some(pid) = record.pid {
                let nix_pid = nix::unistd::Pid::from_raw(pid as i32);
                let _ = nix::sys::signal::kill(nix_pid, nix::sys::signal::Signal::SIGTERM);
                tracing::debug!(service = %record.name, pid = pid, "sent SIGTERM");
            }
        }

        // Constitutional: Due process — SIGTERM before SIGKILL
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        // SIGKILL any remaining
        for record in registry.snapshot() {
            if let Some(pid) = record.pid {
                let nix_pid = nix::unistd::Pid::from_raw(pid as i32);
                if nix::sys::signal::kill(nix_pid, None).is_ok() {
                    let _ = nix::sys::signal::kill(nix_pid, nix::sys::signal::Signal::SIGKILL);
                    tracing::warn!(service = %record.name, pid = pid, "sent SIGKILL");
                }
            }
        }

        // Remove PID file
        let _ = std::fs::remove_file(&pid_path_clone);
        std::process::exit(0);
    });

    supervisor.start().await
}

/// Stop all services by sending SIGTERM to the running nexcloud process.
///
/// Constitutional: Due process — graceful termination before forced exit.
async fn cmd_stop(manifest_path: &PathBuf) -> nexcloud::Result<()> {
    let pid_path = pid_file_path(manifest_path);

    let pid = read_pid_file(&pid_path)?;

    tracing::info!(pid = pid, "sending SIGTERM to nexcloud process");

    let nix_pid = nix::unistd::Pid::from_raw(pid);
    nix::sys::signal::kill(nix_pid, nix::sys::signal::Signal::SIGTERM).map_err(|e| {
        nexcloud::NexCloudError::Shutdown(format!("failed to send SIGTERM to PID {pid}: {e}"))
    })?;

    // Wait briefly for process to exit, then verify
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Check if process is still alive
    match nix::sys::signal::kill(nix_pid, None) {
        Ok(()) => {
            tracing::warn!(
                pid = pid,
                "process still running after SIGTERM, sending SIGKILL"
            );
            let _ = nix::sys::signal::kill(nix_pid, nix::sys::signal::Signal::SIGKILL);
        }
        Err(_) => {
            tracing::info!(pid = pid, "process stopped");
        }
    }

    // Clean up PID file
    let _ = std::fs::remove_file(&pid_path);

    Ok(())
}

/// Restart a specific service or all services.
///
/// Constitutional: Continuity of government — services restart in dependency order.
async fn cmd_restart(manifest_path: &PathBuf, service_name: Option<&str>) -> nexcloud::Result<()> {
    let manifest = CloudManifest::from_file(manifest_path)?;
    let mut supervisor = CloudSupervisor::new(manifest)?;

    match service_name {
        Some(name) => {
            tracing::info!(service = %name, "restarting service");
            supervisor.stop_service_by_name(name).await?;
            supervisor.start_service_by_name(name).await?;
            tracing::info!(service = %name, "service restarted");
        }
        None => {
            tracing::info!("restarting all services");
            supervisor.stop().await?;
            supervisor.start().await?;
        }
    }

    Ok(())
}

/// Show status of all services.
fn cmd_status(manifest_path: &PathBuf) -> nexcloud::Result<()> {
    let manifest = CloudManifest::from_file(manifest_path)?;
    let platform_name = manifest.platform.name.clone();
    let supervisor = CloudSupervisor::new(manifest)?;

    // Check if nexcloud is running
    let pid_path = pid_file_path(manifest_path);
    let running = read_pid_file(&pid_path).is_ok();

    println!(
        "Platform: {} ({})",
        platform_name,
        if running { "RUNNING" } else { "STOPPED" }
    );
    println!();
    println!(
        "{:<20} {:<12} {:<8} {:<10} {:<6}",
        "SERVICE", "STATE", "PID", "RESTARTS", "PORT"
    );
    println!("{}", "-".repeat(58));

    for record in supervisor.status() {
        println!(
            "{:<20} {:<12} {:<8} {:<10} {:<6}",
            record.name,
            record.state,
            record
                .pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string()),
            record.restarts,
            record.port,
        );
    }

    Ok(())
}

/// Validate the manifest without starting services.
fn cmd_validate(manifest_path: &PathBuf) -> nexcloud::Result<()> {
    let manifest = CloudManifest::from_file(manifest_path)?;

    println!("Manifest valid.");
    println!("  Platform:  {}", manifest.platform.name);
    println!("  Services:  {}", manifest.services.len());
    println!("  Routes:    {}", manifest.routes.len());
    println!(
        "  TLS:       {}",
        if manifest.proxy.tls.is_some() {
            "configured"
        } else {
            "none"
        }
    );

    let order = manifest.topo_sort()?;
    println!("  Start order: {}", order.join(" -> "));

    Ok(())
}

/// Tail service logs.
///
/// Constitutional: Habeas corpus — every process must be accountable; logs provide the record.
fn cmd_logs(
    manifest_path: &PathBuf,
    service_name: &str,
    lines: usize,
    stderr: bool,
) -> nexcloud::Result<()> {
    let manifest = CloudManifest::from_file(manifest_path)?;

    // Verify service exists
    if manifest.service_by_name(service_name).is_none() {
        return Err(nexcloud::NexCloudError::ServiceNotFound {
            name: service_name.to_string(),
        });
    }

    let suffix = if stderr { "stderr" } else { "stdout" };
    let log_path = manifest
        .platform
        .log_dir
        .join(format!("{service_name}.{suffix}.log"));

    if !log_path.exists() {
        println!("No log file found at: {}", log_path.display());
        return Ok(());
    }

    let content = std::fs::read_to_string(&log_path)?;
    let total_lines: Vec<&str> = content.lines().collect();
    let start = total_lines.len().saturating_sub(lines);

    for line in &total_lines[start..] {
        println!("{line}");
    }

    Ok(())
}

/// Deploy a service: build → SCP → SSH restart.
///
/// Constitutional: Due process — build must succeed before upload, upload before restart.
/// CAP-029 Homeland Security — authenticated transport (SSH) only.
async fn cmd_deploy(
    manifest_path: &PathBuf,
    service_name: &str,
    host: &str,
    remote_dir: &str,
    ssh_key: Option<&PathBuf>,
    ssh_port: u16,
) -> nexcloud::Result<()> {
    let manifest = CloudManifest::from_file(manifest_path)?;

    let target = DeployTarget {
        host: host.to_string(),
        remote_dir: remote_dir.to_string(),
        ssh_key: ssh_key.cloned(),
        ssh_port,
    };

    let pipeline = DeployPipeline::new(target);
    pipeline.deploy_service(service_name, &manifest).await
}

// --- PID file helpers ---

/// Derive PID file path from manifest path.
fn pid_file_path(manifest_path: &PathBuf) -> PathBuf {
    let stem = manifest_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("nexcloud");
    let dir = manifest_path.parent().unwrap_or(std::path::Path::new("."));
    dir.join(format!(".{stem}.pid"))
}

/// Write the current process PID to a file.
///
/// SEC-004: Uses restrictive permissions (owner-only read/write) to prevent
/// tampering with PID file contents.
fn write_pid_file(path: &PathBuf) -> nexcloud::Result<()> {
    let pid = std::process::id();
    std::fs::write(path, pid.to_string())?;

    // Set restrictive permissions (0600 = owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        let _ = std::fs::set_permissions(path, perms);
    }

    tracing::debug!(pid = pid, path = %path.display(), "wrote PID file");
    Ok(())
}

/// Read a PID from a PID file.
///
/// SEC-004: Validates PID is a positive integer within valid range.
fn read_pid_file(path: &PathBuf) -> nexcloud::Result<i32> {
    let content = std::fs::read_to_string(path).map_err(|_| {
        nexcloud::NexCloudError::Shutdown("nexcloud is not running (no PID file found)".to_string())
    })?;
    let pid = content.trim().parse::<i32>().map_err(|_| {
        nexcloud::NexCloudError::Shutdown(format!("invalid PID in file: {}", path.display()))
    })?;

    // SEC-004: Validate PID range (must be positive, Linux max is ~4M)
    if pid <= 0 || pid > 4_194_304 {
        return Err(nexcloud::NexCloudError::Shutdown(format!(
            "PID {pid} out of valid range"
        )));
    }

    // SEC-004: Verify process is actually a nexcloud instance via /proc/PID/cmdline
    #[cfg(target_os = "linux")]
    {
        let cmdline_path = format!("/proc/{pid}/cmdline");
        if let Ok(cmdline) = std::fs::read_to_string(&cmdline_path) {
            if !cmdline.contains("nexcloud") {
                return Err(nexcloud::NexCloudError::Shutdown(format!(
                    "PID {pid} is not a nexcloud process (possible PID reuse)"
                )));
            }
        }
    }

    Ok(pid)
}
