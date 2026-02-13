//! # Vigil Runtime
//!
//! Bi-modal runtime abstraction supporting both standalone daemon and
//! embedded-in-MCP execution.
//!
//! ## Runtime Modes
//!
//! - **Daemon**: Standalone process (`friday` binary) with own event loop.
//!   Loads config, initializes sensors/actuators, runs continuously.
//!
//! - **Embedded**: Runs inside another process (e.g., `nexcore-mcp`).
//!   Shares the host's tokio runtime, receives shutdown signal via channel.
//!
//! ## Tier: T2-C (σ + ς + ∂)
//! Dominant: σ Sequence — ordered lifecycle management (start → run → stop).

use std::sync::Arc;

use tokio::sync::watch;
use tracing::{info, warn};

use crate::agentic_loop::{AgenticLoop, LoopConfig};
use crate::bridge::nervous_system::NervousSystem;
use crate::errors::{Result, VigilError};
use crate::events::EventBus;

/// How the Vigil runtime should operate.
#[derive(Clone)]
pub enum RuntimeMode {
    /// Standalone daemon with own tokio runtime.
    Daemon {
        /// Loop configuration
        config: LoopConfig,
        /// Token budget for energy governance
        token_budget: u64,
    },

    /// Embedded in another process, sharing its tokio runtime.
    Embedded {
        /// Shared event bus from the host process
        bus: Arc<EventBus>,
        /// Shutdown signal receiver
        shutdown_rx: watch::Receiver<bool>,
        /// Loop configuration
        config: LoopConfig,
        /// Token budget for energy governance
        token_budget: u64,
    },
}

/// The Vigil runtime controller.
///
/// Manages lifecycle of the agentic loop and nervous system bridges
/// in either daemon or embedded mode.
pub struct VigilRuntime {
    /// Running state
    running: Arc<tokio::sync::RwLock<bool>>,
    /// Shutdown signal sender (for embedded mode, held by creator)
    shutdown_tx: Option<watch::Sender<bool>>,
    /// The nervous system bridges (initialized on start)
    nervous_system: Option<NervousSystem>,
    /// Runtime mode
    mode: RuntimeMode,
}

impl VigilRuntime {
    /// Create a new runtime in the specified mode.
    ///
    /// Does NOT start the loop — call [`start()`] to begin processing.
    pub fn new(mode: RuntimeMode) -> Self {
        Self {
            running: Arc::new(tokio::sync::RwLock::new(false)),
            shutdown_tx: None,
            nervous_system: None,
            mode,
        }
    }

    /// Create a daemon-mode runtime with default configuration.
    pub fn daemon(token_budget: u64) -> Self {
        Self::new(RuntimeMode::Daemon {
            config: LoopConfig::default(),
            token_budget,
        })
    }

    /// Create an embedded-mode runtime.
    ///
    /// Returns `(runtime, shutdown_sender)` — the caller keeps the sender
    /// and calls `shutdown_tx.send(true)` to stop the runtime.
    pub fn embedded(
        bus: Arc<EventBus>,
        config: LoopConfig,
        token_budget: u64,
    ) -> (Self, watch::Sender<bool>) {
        let (tx, rx) = watch::channel(false);

        let runtime = Self::new(RuntimeMode::Embedded {
            bus,
            shutdown_rx: rx,
            config,
            token_budget,
        });

        (runtime, tx)
    }

    /// Start the runtime.
    ///
    /// Initializes the nervous system and begins the agentic loop.
    /// In embedded mode, runs as a background task on the current tokio runtime.
    /// In daemon mode, blocks on the loop.
    pub async fn start(&mut self) -> Result<()> {
        {
            let mut running = self.running.write().await;
            if *running {
                return Err(VigilError::Unknown("runtime already running".to_string()));
            }
            *running = true;
        }

        match &self.mode {
            RuntimeMode::Daemon {
                config,
                token_budget,
            } => {
                let bus = Arc::new(EventBus::default());
                let ns = NervousSystem::init(Arc::clone(&bus), *token_budget);
                self.nervous_system = Some(ns);

                let mut loop_ctrl = AgenticLoop::new(config.clone())
                    .with_pv_sensors()
                    .with_default_actuators();

                info!("vigil_daemon_starting");

                let running = Arc::clone(&self.running);
                // Run loop until stopped
                while *running.read().await {
                    let _result = loop_ctrl.tick().await?;
                    tokio::time::sleep(config.tick_interval).await;
                }

                info!("vigil_daemon_stopped");
            }

            RuntimeMode::Embedded {
                bus,
                shutdown_rx,
                config,
                token_budget,
            } => {
                let ns = NervousSystem::init(Arc::clone(bus), *token_budget);
                self.nervous_system = Some(ns);

                let mut loop_ctrl = AgenticLoop::new(config.clone())
                    .with_pv_sensors()
                    .with_default_actuators();

                let mut shutdown = shutdown_rx.clone();
                let running = Arc::clone(&self.running);
                let tick_interval = config.tick_interval;

                info!("vigil_embedded_starting");

                // Spawn as background task
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            // Check shutdown signal
                            result = shutdown.changed() => {
                                if result.is_ok() && *shutdown.borrow() {
                                    info!("vigil_embedded_shutdown_signal_received");
                                    break;
                                }
                            }
                            // Tick the loop
                            _ = tokio::time::sleep(tick_interval) => {
                                if let Err(e) = loop_ctrl.tick().await {
                                    warn!(error = %e, "vigil_embedded_tick_error");
                                }
                            }
                        }
                    }

                    let mut r = running.write().await;
                    *r = false;
                    info!("vigil_embedded_stopped");
                });
            }
        }

        Ok(())
    }

    /// Stop the runtime gracefully.
    pub async fn stop(&mut self) -> Result<()> {
        let was_running = {
            let mut running = self.running.write().await;
            let was = *running;
            *running = false;
            was
        };

        if !was_running {
            return Ok(());
        }

        // For embedded mode, send shutdown signal
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(true);
        }

        // Persist hormonal state
        if let Some(ns) = &self.nervous_system {
            if let Err(e) = ns.hormonal.save().await {
                warn!(error = %e, "failed_to_save_hormonal_state");
            }
        }

        info!("vigil_runtime_stopped");
        Ok(())
    }

    /// Check if the runtime is currently running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get a reference to the nervous system (if initialized).
    pub fn nervous_system(&self) -> Option<&NervousSystem> {
        self.nervous_system.as_ref()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn daemon_mode_construction() {
        let runtime = VigilRuntime::daemon(100_000);
        assert!(matches!(runtime.mode, RuntimeMode::Daemon { .. }));
    }

    #[test]
    fn embedded_mode_construction() {
        let bus = Arc::new(EventBus::default());
        let config = LoopConfig::default().with_tick_interval(Duration::from_millis(100));
        let (runtime, _tx) = VigilRuntime::embedded(bus, config, 50_000);
        assert!(matches!(runtime.mode, RuntimeMode::Embedded { .. }));
    }

    #[tokio::test]
    async fn runtime_not_running_initially() {
        let runtime = VigilRuntime::daemon(100_000);
        assert!(!runtime.is_running().await);
    }

    #[tokio::test]
    async fn embedded_start_stop() {
        let bus = Arc::new(EventBus::default());
        let config = LoopConfig::default().with_tick_interval(Duration::from_millis(50));
        let (mut runtime, tx) = VigilRuntime::embedded(bus, config, 50_000);

        // Start
        let start_result = runtime.start().await;
        assert!(start_result.is_ok());

        // Let it tick a couple times
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Stop via shutdown signal
        let _ = tx.send(true);
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop the runtime
        let stop_result = runtime.stop().await;
        assert!(stop_result.is_ok());
    }
}
