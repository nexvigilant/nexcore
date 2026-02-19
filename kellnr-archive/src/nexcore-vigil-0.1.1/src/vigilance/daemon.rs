//! # Vigil Daemon — The Unified Vigilance Engine
//!
//! Ties all 4 layers together:
//! - ν Watcher → collects events
//! - ∂ BoundaryGate → evaluates boundaries
//! - ∝ ConsequencePipeline → escalates violations
//! - π VigilanceLedger → records everything
//!
//! ## Tier: T3 (π + ∂ + ν + ∝)
//! The daemon IS vigilance: π(∂·ν)|∝

use crate::vigilance::boundary::{BoundaryGate, BoundarySpec};
use crate::vigilance::consequence::ConsequencePipeline;
use crate::vigilance::error::{VigilError, VigilResult};
use crate::vigilance::event::WatchEvent;
use crate::vigilance::ledger::{LedgerEntryType, VigilanceLedger};
use crate::vigilance::vigil_config::VigilConfig;
use crate::vigilance::watcher::{WatchSource, Watcher};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc, watch};

/// Health status of the daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VigilHealth {
    pub running: bool,
    pub uptime_ms: u64,
    pub sources: usize,
    pub boundaries: usize,
    pub consequences: usize,
    pub ledger_entries: usize,
    pub chain_verified: bool,
}

/// Runtime statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VigilStats {
    pub total_events: u64,
    pub total_violations: u64,
    pub total_consequences: u64,
    pub uptime_ms: u64,
    pub ledger_head_hash: String,
}

/// Handle for requesting daemon shutdown from outside the run loop.
///
/// Tier: T2-P (∂ + ς) — boundary + state
#[derive(Clone)]
pub struct ShutdownHandle {
    tx: watch::Sender<bool>,
}

impl ShutdownHandle {
    /// Signal the daemon to shut down gracefully.
    pub fn shutdown(&self) {
        let _ = self.tx.send(true);
    }
}

/// The unified vigilance daemon.
///
/// Tier: T3 (π + ∂ + ν + ∝), dominant π
/// This IS the vigilance formula: π(∂·ν)|∝
pub struct VigilDaemon {
    watcher: Watcher,
    gate: BoundaryGate,
    pipeline: ConsequencePipeline,
    ledger: Arc<Mutex<VigilanceLedger>>,
    config: VigilConfig,
    event_rx: mpsc::Receiver<WatchEvent>,
    running: bool,
    start_time: u64,
    total_violations: u64,
    total_consequences: u64,
    shutdown_rx: watch::Receiver<bool>,
    shutdown_tx: watch::Sender<bool>,
}

impl VigilDaemon {
    /// Create a new daemon with default configuration.
    pub fn new(config: VigilConfig) -> Self {
        let ledger = Arc::new(Mutex::new(VigilanceLedger::new(config.wal_path.clone())));
        let (event_tx, event_rx) = mpsc::channel(config.max_event_buffer);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        Self {
            watcher: Watcher::new(event_tx),
            gate: BoundaryGate::new(),
            pipeline: ConsequencePipeline::new(Arc::clone(&ledger)),
            ledger,
            config,
            event_rx,
            running: false,
            start_time: 0,
            total_violations: 0,
            total_consequences: 0,
            shutdown_rx,
            shutdown_tx,
        }
    }

    /// Get a shutdown handle for signaling the daemon from another task.
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle {
            tx: self.shutdown_tx.clone(),
        }
    }

    /// Build a fully configured daemon from a VigilConfig.
    ///
    /// Materializes sources, boundaries, and consequences from config structs
    /// into live trait objects. Falls back to defaults if config sections are empty.
    pub fn from_config(config: VigilConfig) -> Self {
        use crate::vigilance::boundary::BoundarySpec;
        use crate::vigilance::consequence::{
            EscalationLevel, LogConsequence, NotifyConsequence, ShellConsequence,
            WebhookConsequence,
        };
        use crate::vigilance::event::{EventKind, EventSeverity};
        use crate::vigilance::sources::TimerSource;
        use crate::vigilance::vigil_config::{ConsequenceConfig, SourceConfig, ThresholdConfig};

        let mut daemon = Self::new(config.clone());

        // ν Layer: Materialize sources
        if config.sources.is_empty() {
            // Default: 60s heartbeat
            daemon.add_source(Box::new(TimerSource::new(
                "heartbeat",
                Duration::from_secs(60),
            )));
        } else {
            for src in &config.sources {
                match src {
                    SourceConfig::Timer { name, interval_ms } => {
                        daemon.add_source(Box::new(TimerSource::new(
                            name,
                            Duration::from_millis(*interval_ms),
                        )));
                    }
                    SourceConfig::FileSystem { name, paths } => {
                        let path_bufs: Vec<std::path::PathBuf> =
                            paths.iter().map(std::path::PathBuf::from).collect();
                        if let Ok(fs_source) = crate::vigilance::sources::FileSystemSource::new(
                            name,
                            &path_bufs,
                            true, // recursive
                            Duration::from_millis(config.poll_interval_ms),
                        ) {
                            daemon.add_source(Box::new(fs_source));
                        } else {
                            tracing::warn!(name, "failed_to_create_filesystem_source");
                        }
                    }
                }
            }
        }

        // ∂ Layer: Materialize boundaries
        for bc in &config.boundaries {
            let threshold = match &bc.threshold {
                ThresholdConfig::Always => crate::vigilance::boundary::ThresholdCheck::Always,
                ThresholdConfig::SeverityAtLeast { level } => {
                    let sev = match level.to_lowercase().as_str() {
                        "critical" => EventSeverity::Critical,
                        "high" => EventSeverity::High,
                        "medium" => EventSeverity::Medium,
                        "low" => EventSeverity::Low,
                        _ => EventSeverity::Info,
                    };
                    crate::vigilance::boundary::ThresholdCheck::SeverityAtLeast(sev)
                }
                ThresholdConfig::CountExceeds { count, window_ms } => {
                    crate::vigilance::boundary::ThresholdCheck::CountExceeds {
                        count: *count,
                        window: Duration::from_millis(*window_ms),
                    }
                }
                ThresholdConfig::PayloadMatch { json_path, pattern } => {
                    crate::vigilance::boundary::ThresholdCheck::PayloadMatch {
                        json_path: json_path.clone(),
                        pattern: pattern.clone(),
                    }
                }
            };

            let kind_filter = bc.kind_filter.as_deref().map(|k| match k {
                "timer" => EventKind::Timer,
                "file_change" => EventKind::FileChange,
                "signal" => EventKind::Signal,
                "channel" => EventKind::Channel,
                other => EventKind::Custom(other.to_string()),
            });

            daemon.add_boundary(BoundarySpec {
                name: bc.name.clone(),
                source_filter: bc.source_filter.clone(),
                kind_filter,
                threshold,
                cooldown: Duration::from_millis(bc.cooldown_ms),
            });
        }

        // ∝ Layer: Materialize consequences
        if config.consequences.is_empty() {
            // Default: log consequence
            daemon.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Observe)));
        } else {
            for cc in &config.consequences {
                match cc {
                    ConsequenceConfig::Log { level } => {
                        let el = parse_escalation_level(level);
                        daemon.add_consequence(Box::new(LogConsequence::new(el)));
                    }
                    ConsequenceConfig::Shell {
                        command,
                        timeout_ms,
                    } => {
                        daemon.add_consequence(Box::new(ShellConsequence::new(
                            "shell",
                            command.clone(),
                            *timeout_ms,
                        )));
                    }
                    ConsequenceConfig::Notify { dir } => {
                        daemon.add_consequence(Box::new(NotifyConsequence::new(
                            std::path::PathBuf::from(dir),
                        )));
                    }
                    ConsequenceConfig::Webhook { url, timeout_ms: _ } => {
                        daemon.add_consequence(Box::new(WebhookConsequence::new(
                            "webhook",
                            url.clone(),
                        )));
                    }
                }
            }
        }

        daemon
    }

    /// Add a watch source.
    pub fn add_source(&mut self, source: Box<dyn WatchSource>) {
        self.watcher.add_source(source);
    }

    /// Add a boundary specification.
    pub fn add_boundary(&mut self, spec: BoundarySpec) {
        self.gate.add_spec(spec);
    }

    /// Add a consequence to the escalation chain.
    pub fn add_consequence(
        &mut self,
        consequence: Box<dyn crate::vigilance::consequence::Consequence>,
    ) {
        self.pipeline.add_consequence(consequence);
    }

    /// Run one cycle of the vigilance loop: poll → evaluate → escalate.
    ///
    /// Returns true if the daemon should continue, false if shutdown.
    pub async fn select_once(&mut self) -> VigilResult<bool> {
        // Poll all sources
        let poll_count = self.watcher.poll_once().await?;

        // Process events from channel
        let mut events = Vec::with_capacity(poll_count);
        while let Ok(event) = self.event_rx.try_recv() {
            events.push(event);
        }

        // Handle each event
        for event in &events {
            self.handle_event(event)?;
        }

        Ok(true)
    }

    /// Run the daemon continuously until shutdown is signaled.
    ///
    /// This is the production entry point. It:
    /// 1. Calls `start()` to record DaemonStarted
    /// 2. Loops: `select_once()` → sleep(poll_interval) → check shutdown
    /// 3. On shutdown signal: calls `stop()` to flush WAL
    ///
    /// Returns the total number of cycles executed.
    pub async fn run(&mut self) -> VigilResult<u64> {
        self.start()?;
        let poll_interval = Duration::from_millis(self.config.poll_interval_ms);
        let mut cycles: u64 = 0;

        loop {
            // Check shutdown before each cycle
            if *self.shutdown_rx.borrow() {
                tracing::info!(cycles, "vigil_daemon_shutdown_requested");
                break;
            }

            // Run one vigilance cycle
            if let Err(e) = self.select_once().await {
                tracing::warn!(error = %e, "vigil_cycle_error");
            }
            cycles += 1;

            // Sleep with shutdown check — wake early if shutdown signaled
            tokio::select! {
                biased;
                result = self.shutdown_rx.changed() => {
                    if result.is_ok() && *self.shutdown_rx.borrow() {
                        tracing::info!(cycles, "vigil_daemon_shutdown_during_sleep");
                        break;
                    }
                }
                _ = tokio::time::sleep(poll_interval) => {}
            }
        }

        self.stop()?;
        Ok(cycles)
    }

    /// Get the configured poll interval.
    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.config.poll_interval_ms)
    }

    /// Handle a single event through the boundary gate and consequence pipeline.
    fn handle_event(&mut self, event: &WatchEvent) -> VigilResult<()> {
        // Record event in ledger
        {
            let mut ledger = self
                .ledger
                .lock()
                .map_err(|e| VigilError::Daemon(format!("Ledger lock failed: {e}")))?;
            let _ = ledger.append(
                LedgerEntryType::EventObserved,
                serde_json::json!({
                    "event_id": event.id.0,
                    "source": event.source,
                    "kind": format!("{}", event.kind),
                    "severity": format!("{}", event.severity),
                }),
            )?;
        }

        // Evaluate boundaries
        let violations = self.gate.evaluate(event);

        // Process violations through consequence pipeline
        for violation in &violations {
            self.total_violations += 1;

            // Record violation in ledger
            {
                let mut ledger = self
                    .ledger
                    .lock()
                    .map_err(|e| VigilError::Daemon(format!("Ledger lock failed: {e}")))?;
                let _ = ledger.append(
                    LedgerEntryType::BoundaryViolation,
                    serde_json::json!({
                        "boundary": violation.boundary,
                        "violation_count": violation.violation_count,
                    }),
                )?;
            }

            // Execute consequences
            let receipts = self.pipeline.execute(violation)?;
            self.total_consequences += receipts.len() as u64;
        }

        Ok(())
    }

    /// Start the daemon (records DaemonStarted in ledger).
    pub fn start(&mut self) -> VigilResult<()> {
        self.running = true;
        self.start_time = crate::vigilance::event::now_millis();

        let mut ledger = self
            .ledger
            .lock()
            .map_err(|e| VigilError::Daemon(format!("Ledger lock failed: {e}")))?;
        let _ = ledger.append(
            LedgerEntryType::DaemonStarted,
            serde_json::json!({
                "sources": self.watcher.source_count(),
                "boundaries": self.gate.spec_count(),
                "consequences": self.pipeline.chain_length(),
            }),
        )?;

        tracing::info!("vigil_daemon_started");
        Ok(())
    }

    /// Stop the daemon gracefully (records DaemonStopped, flushes WAL).
    pub fn stop(&mut self) -> VigilResult<()> {
        self.running = false;

        let mut ledger = self
            .ledger
            .lock()
            .map_err(|e| VigilError::Daemon(format!("Ledger lock failed: {e}")))?;
        let _ = ledger.append(
            LedgerEntryType::DaemonStopped,
            serde_json::json!({
                "uptime_ms": crate::vigilance::event::now_millis().saturating_sub(self.start_time),
                "total_events": self.watcher.total_events(),
                "total_violations": self.total_violations,
                "total_consequences": self.total_consequences,
            }),
        )?;

        ledger.flush_wal()?;
        tracing::info!("vigil_daemon_stopped");
        Ok(())
    }

    /// Get daemon health.
    pub fn health(&self) -> VigilHealth {
        let ledger = self.ledger.lock();
        let (entries, verified) = match ledger {
            Ok(l) => (l.len(), l.verify_chain().unwrap_or(false)),
            Err(_) => (0, false),
        };

        VigilHealth {
            running: self.running,
            uptime_ms: if self.running {
                crate::vigilance::event::now_millis().saturating_sub(self.start_time)
            } else {
                0
            },
            sources: self.watcher.source_count(),
            boundaries: self.gate.spec_count(),
            consequences: self.pipeline.chain_length(),
            ledger_entries: entries,
            chain_verified: verified,
        }
    }

    /// Get runtime statistics.
    pub fn stats(&self) -> VigilStats {
        let ledger = self.ledger.lock();
        let head_hash = match &ledger {
            Ok(l) => hex::encode(l.head_hash()),
            Err(_) => "error".to_string(),
        };

        VigilStats {
            total_events: self.watcher.total_events(),
            total_violations: self.total_violations,
            total_consequences: self.total_consequences,
            uptime_ms: if self.running {
                crate::vigilance::event::now_millis().saturating_sub(self.start_time)
            } else {
                0
            },
            ledger_head_hash: head_hash,
        }
    }

    /// Get a reference to the ledger (for external queries).
    pub fn ledger(&self) -> &Arc<Mutex<VigilanceLedger>> {
        &self.ledger
    }

    /// Whether the daemon is currently running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Access boundary spec names.
    pub fn boundary_names(&self) -> Vec<&str> {
        self.gate.spec_names()
    }
}

fn parse_escalation_level(s: &str) -> crate::vigilance::consequence::EscalationLevel {
    use crate::vigilance::consequence::EscalationLevel;
    match s.to_lowercase().as_str() {
        "observe" => EscalationLevel::Observe,
        "warn" => EscalationLevel::Warn,
        "alert" => EscalationLevel::Alert,
        "act" => EscalationLevel::Act,
        "audit" => EscalationLevel::Audit,
        _ => EscalationLevel::Observe,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vigilance::boundary::ThresholdCheck;
    use crate::vigilance::consequence::{EscalationLevel, LogConsequence};
    use crate::vigilance::event::EventSeverity;
    use crate::vigilance::sources::timer::TimerSource;
    use std::time::Duration;

    fn test_config() -> VigilConfig {
        VigilConfig {
            wal_path: std::path::PathBuf::from("/tmp/vigil-daemon-test.wal"),
            poll_interval_ms: 100,
            max_event_buffer: 1000,
            sources: Vec::new(),
            boundaries: Vec::new(),
            consequences: Vec::new(),
            sync_wal: false,
        }
    }

    #[test]
    fn daemon_lifecycle() {
        let mut daemon = VigilDaemon::new(test_config());
        assert!(!daemon.is_running());

        let start = daemon.start();
        assert!(start.is_ok());
        assert!(daemon.is_running());

        let health = daemon.health();
        assert!(health.running);
        assert_eq!(health.ledger_entries, 1); // DaemonStarted

        let stop = daemon.stop();
        assert!(stop.is_ok());
        assert!(!daemon.is_running());
    }

    #[tokio::test]
    async fn daemon_event_flow() {
        let mut daemon = VigilDaemon::new(test_config());

        // Add a timer source
        daemon.add_source(Box::new(TimerSource::new(
            "heartbeat",
            Duration::from_millis(100),
        )));

        // Add an always-fire boundary
        daemon.add_boundary(BoundarySpec {
            name: "catch-all".to_string(),
            source_filter: None,
            kind_filter: None,
            threshold: ThresholdCheck::Always,
            cooldown: Duration::from_millis(0),
        });

        // Add a log consequence
        daemon.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Observe)));

        let _ = daemon.start();

        // Run one cycle
        let result = daemon.select_once().await;
        assert!(result.is_ok());

        let stats = daemon.stats();
        assert!(stats.total_events >= 1);

        let _ = daemon.stop();
    }

    #[tokio::test]
    async fn daemon_run_loop_with_shutdown() {
        let mut daemon = VigilDaemon::new(test_config());

        // Add a timer source
        daemon.add_source(Box::new(TimerSource::new(
            "heartbeat",
            Duration::from_millis(50),
        )));

        // Add a log consequence
        daemon.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Observe)));

        // Get shutdown handle before run
        let handle = daemon.shutdown_handle();

        // Signal shutdown after a short delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(250)).await;
            handle.shutdown();
        });

        // Run the daemon — should return after shutdown signal
        let result = daemon.run().await;
        assert!(result.is_ok());

        let cycles = result.unwrap_or(0);
        assert!(cycles >= 1, "Should have completed at least 1 cycle");
        assert!(!daemon.is_running());
    }

    #[tokio::test]
    async fn shutdown_handle_is_cloneable() {
        let daemon = VigilDaemon::new(test_config());
        let h1 = daemon.shutdown_handle();
        let h2 = h1.clone();
        // Both handles should work
        h2.shutdown();
        drop(h1);
    }

    #[test]
    fn daemon_poll_interval() {
        let mut cfg = test_config();
        cfg.poll_interval_ms = 500;
        let daemon = VigilDaemon::new(cfg);
        assert_eq!(daemon.poll_interval(), Duration::from_millis(500));
    }

    #[test]
    fn daemon_health_and_stats() {
        let daemon = VigilDaemon::new(test_config());
        let health = daemon.health();
        assert!(!health.running);
        assert_eq!(health.sources, 0);

        let stats = daemon.stats();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.total_violations, 0);
    }

    #[test]
    fn daemon_boundary_management() {
        let mut daemon = VigilDaemon::new(test_config());

        daemon.add_boundary(BoundarySpec {
            name: "alpha".to_string(),
            source_filter: None,
            kind_filter: None,
            threshold: ThresholdCheck::SeverityAtLeast(EventSeverity::High),
            cooldown: Duration::from_secs(1),
        });

        let names = daemon.boundary_names();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"alpha"));
    }

    #[test]
    fn from_config_defaults() {
        // Empty config → default heartbeat source + default log consequence
        let daemon = VigilDaemon::from_config(test_config());
        let health = daemon.health();
        assert_eq!(health.sources, 1, "Default heartbeat source");
        assert_eq!(health.consequences, 1, "Default log consequence");
    }

    #[test]
    fn from_config_with_sources_and_boundaries() {
        use crate::vigilance::vigil_config::{
            BoundaryConfig, ConsequenceConfig, SourceConfig, ThresholdConfig,
        };

        let config = VigilConfig {
            sources: vec![
                SourceConfig::Timer {
                    name: "fast".to_string(),
                    interval_ms: 100,
                },
                SourceConfig::Timer {
                    name: "slow".to_string(),
                    interval_ms: 5000,
                },
            ],
            boundaries: vec![BoundaryConfig {
                name: "high-severity".to_string(),
                source_filter: None,
                kind_filter: None,
                threshold: ThresholdConfig::SeverityAtLeast {
                    level: "high".to_string(),
                },
                cooldown_ms: 1000,
            }],
            consequences: vec![
                ConsequenceConfig::Log {
                    level: "observe".to_string(),
                },
                ConsequenceConfig::Notify {
                    dir: "/tmp/vigil-test-notify".to_string(),
                },
            ],
            ..test_config()
        };

        let daemon = VigilDaemon::from_config(config);
        let health = daemon.health();
        assert_eq!(health.sources, 2, "Two timer sources from config");
        assert_eq!(health.boundaries, 1, "One boundary from config");
        assert_eq!(health.consequences, 2, "Log + Notify from config");

        let names = daemon.boundary_names();
        assert!(names.contains(&"high-severity"));
    }

    #[test]
    fn from_config_toml_roundtrip() {
        use crate::vigilance::vigil_config::{
            BoundaryConfig, ConsequenceConfig, SourceConfig, ThresholdConfig,
        };

        let config = VigilConfig {
            poll_interval_ms: 500,
            sources: vec![SourceConfig::Timer {
                name: "tick".to_string(),
                interval_ms: 200,
            }],
            boundaries: vec![BoundaryConfig {
                name: "catch-all".to_string(),
                source_filter: None,
                kind_filter: None,
                threshold: ThresholdConfig::Always,
                cooldown_ms: 0,
            }],
            consequences: vec![ConsequenceConfig::Log {
                level: "alert".to_string(),
            }],
            ..test_config()
        };

        // Serialize to TOML and back
        let toml_str = toml::to_string(&config);
        assert!(toml_str.is_ok(), "Config should serialize to TOML");

        let parsed: Result<VigilConfig, _> = toml::from_str(&toml_str.unwrap_or_default());
        assert!(parsed.is_ok(), "TOML should deserialize back");

        let parsed = parsed.unwrap_or_default();
        assert_eq!(parsed.poll_interval_ms, 500);
        assert_eq!(parsed.sources.len(), 1);
        assert_eq!(parsed.boundaries.len(), 1);
        assert_eq!(parsed.consequences.len(), 1);
    }
}
