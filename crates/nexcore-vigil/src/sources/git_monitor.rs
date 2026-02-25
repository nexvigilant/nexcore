use crate::events::EventBus;
use crate::models::{Event, Urgency};
use crate::sources::Source;
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Monitors Git repositories for activity and staleness.
pub struct GitMonitor {
    bus: EventBus,
    watch_paths: Vec<PathBuf>,
    poll_interval: Duration,
}

impl GitMonitor {
    pub fn new(bus: EventBus, watch_paths: Vec<PathBuf>, poll_interval_secs: u64) -> Self {
        Self {
            bus,
            watch_paths,
            poll_interval: Duration::from_secs(poll_interval_secs),
        }
    }

    async fn check_repository(&self, path: &PathBuf) -> nexcore_error::Result<()> {
        debug!(?path, "checking_git_repo");

        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("log")
            .arg("-1")
            .arg("--format=%ct")
            .output()
            .await?;

        if !output.status.success() {
            return Err(nexcore_error::nexerror!(
                "Not a git repository or git error"
            ));
        }

        let timestamp_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let timestamp: i64 = timestamp_str.parse().unwrap_or(0);
        let now = nexcore_chrono::DateTime::now().timestamp();

        let days_since = (now - timestamp) / (24 * 3600);

        if days_since > 7 {
            warn!(%days_since, ?path, "stale_repository_detected");
            self.bus
                .emit(Event {
                    source: "git_monitor".to_string(),
                    event_type: "git_stale".to_string(),
                    payload: serde_json::json!({
                        "path": path.to_string_lossy(),
                        "days_since_last_commit": days_since,
                    }),
                    priority: Urgency::Normal,
                    ..Event::default()
                })
                .await;
        }

        Ok(())
    }
}

#[async_trait]
impl Source for GitMonitor {
    fn name(&self) -> &'static str {
        "git_monitor"
    }

    async fn run(&self) -> nexcore_error::Result<()> {
        info!(interval = ?self.poll_interval, "git_monitor_starting");

        loop {
            for path in &self.watch_paths {
                if let Err(e) = self.check_repository(path).await {
                    error!(error = %e, ?path, "git_check_failed");
                }
            }
            tokio::time::sleep(self.poll_interval).await;
        }
    }
}
