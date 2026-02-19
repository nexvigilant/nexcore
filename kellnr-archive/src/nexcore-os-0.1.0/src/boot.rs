// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Boot sequence — ordered system initialization.
//!
//! ## Boot Phases (σ Sequence + → Causality)
//!
//! 1. **PAL Init**: Hardware probing and subsystem initialization
//! 2. **Kernel Boot**: STOS state machine runtime startup
//! 3. **Service Start**: System services started in priority order
//! 4. **Shell Launch**: User interface brought up
//!
//! Each phase is causally dependent on the previous (→).

use crate::error::{BootError, OsError};
use crate::service::{ServiceManager, ServicePriority, ServiceState};

/// Boot phase tracking.
///
/// Tier: T2-P (σ Sequence + ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootPhase {
    /// Not started.
    Cold,
    /// PAL hardware initialization.
    PalInit,
    /// STOS kernel startup.
    KernelBoot,
    /// System services starting.
    ServicesStarting,
    /// Shell launching.
    ShellLaunch,
    /// Fully booted and operational.
    Running,
    /// Shutdown initiated.
    ShuttingDown,
    /// System halted.
    Halted,
}

/// Boot sequence orchestrator.
///
/// Tier: T3 (σ Sequence + → Causality + ∂ Boundary)
pub struct BootSequence {
    /// Current boot phase.
    phase: BootPhase,
    /// Boot log entries.
    log: Vec<BootLogEntry>,
    /// Boot start timestamp.
    started_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// A single boot log entry.
///
/// Tier: T2-P (π Persistence — audit trail)
#[derive(Debug, Clone)]
pub struct BootLogEntry {
    /// Phase this entry belongs to.
    pub phase: BootPhase,
    /// Message.
    pub message: String,
    /// Whether this was an error.
    pub is_error: bool,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl BootSequence {
    /// Create a new boot sequence.
    pub fn new() -> Self {
        Self {
            phase: BootPhase::Cold,
            log: Vec::new(),
            started_at: None,
        }
    }

    /// Get current boot phase.
    pub fn phase(&self) -> BootPhase {
        self.phase
    }

    /// Get boot log.
    pub fn log(&self) -> &[BootLogEntry] {
        &self.log
    }

    /// Record a boot log message.
    fn log_message(&mut self, message: impl Into<String>) {
        self.log.push(BootLogEntry {
            phase: self.phase,
            message: message.into(),
            is_error: false,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Record a boot error.
    fn log_error(&mut self, message: impl Into<String>) {
        self.log.push(BootLogEntry {
            phase: self.phase,
            message: message.into(),
            is_error: true,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Execute Phase 1: PAL initialization.
    pub fn init_pal(&mut self, platform_name: &str) -> Result<(), OsError> {
        self.phase = BootPhase::PalInit;
        self.started_at = Some(chrono::Utc::now());
        self.log_message(format!("PAL init: {platform_name}"));
        self.log_message("Hardware subsystems probed");
        Ok(())
    }

    /// Execute Phase 2: STOS kernel boot.
    pub fn boot_kernel(&mut self) -> Result<(), OsError> {
        if self.phase != BootPhase::PalInit {
            return Err(
                BootError::KernelInitFailed("PAL must be initialized first".to_string()).into(),
            );
        }

        self.phase = BootPhase::KernelBoot;
        self.log_message("STOS kernel starting");
        self.log_message("State machine runtime initialized");
        Ok(())
    }

    /// Execute Phase 3: Start system services in priority order.
    pub fn start_services(&mut self, mgr: &mut ServiceManager) -> Result<(), OsError> {
        if self.phase != BootPhase::KernelBoot {
            return Err(
                BootError::CriticalServiceFailed("Kernel must boot first".to_string()).into(),
            );
        }

        self.phase = BootPhase::ServicesStarting;

        // Get services sorted by priority
        let startup_ids: Vec<_> = mgr.startup_order().iter().map(|s| s.id).collect();

        for id in startup_ids {
            if let Some(svc) = mgr.get_mut(id) {
                let name = svc.name.clone();
                let priority = svc.priority;

                match svc.transition(ServiceState::Starting) {
                    Ok(()) => {
                        self.log_message(format!("Starting {name} ({priority:?})"));
                        // Simulate successful start
                        if svc.transition(ServiceState::Running).is_ok() {
                            self.log_message(format!("{name} running"));
                        }
                    }
                    Err(e) => {
                        self.log_error(format!("Failed to start {name}: {e}"));
                        // Critical services failing is a boot error
                        if priority == ServicePriority::Critical {
                            return Err(BootError::CriticalServiceFailed(name).into());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute Phase 4: Launch shell.
    pub fn launch_shell(&mut self) -> Result<(), OsError> {
        if self.phase != BootPhase::ServicesStarting {
            return Err(
                BootError::CriticalServiceFailed("Services must start first".to_string()).into(),
            );
        }

        self.phase = BootPhase::ShellLaunch;
        self.log_message("Shell launching");
        self.phase = BootPhase::Running;
        self.log_message("System fully booted");

        if let Some(start) = self.started_at {
            let elapsed = chrono::Utc::now() - start;
            self.log_message(format!("Boot time: {}ms", elapsed.num_milliseconds()));
        }

        Ok(())
    }

    /// Initiate shutdown.
    pub fn shutdown(&mut self) {
        self.phase = BootPhase::ShuttingDown;
        self.log_message("Shutdown initiated");
    }

    /// Mark system as halted.
    pub fn halt(&mut self) {
        self.phase = BootPhase::Halted;
        self.log_message("System halted");
    }

    /// Whether the system is fully booted.
    pub fn is_running(&self) -> bool {
        self.phase == BootPhase::Running
    }

    /// Boot duration in milliseconds (None if not yet booted).
    pub fn boot_duration_ms(&self) -> Option<i64> {
        let start = self.started_at?;
        if self.phase == BootPhase::Running {
            // Find the "fully booted" log entry timestamp
            self.log
                .iter()
                .rev()
                .find(|e| e.message.contains("fully booted"))
                .map(|e| (e.timestamp - start).num_milliseconds())
        } else {
            None
        }
    }
}

impl Default for BootSequence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_sequence_phases() {
        let mut boot = BootSequence::new();
        let mut mgr = ServiceManager::new();

        assert_eq!(boot.phase(), BootPhase::Cold);

        // Phase 1: PAL
        assert!(boot.init_pal("linux-x86_64-desktop").is_ok());
        assert_eq!(boot.phase(), BootPhase::PalInit);

        // Phase 2: Kernel
        assert!(boot.boot_kernel().is_ok());
        assert_eq!(boot.phase(), BootPhase::KernelBoot);

        // Register services for Phase 3
        mgr.register("stos", ServicePriority::Critical);
        mgr.register("guardian", ServicePriority::Core);

        // Phase 3: Services
        assert!(boot.start_services(&mut mgr).is_ok());
        assert_eq!(boot.phase(), BootPhase::ServicesStarting);

        // Phase 4: Shell
        assert!(boot.launch_shell().is_ok());
        assert!(boot.is_running());
    }

    #[test]
    fn boot_out_of_order_fails() {
        let mut boot = BootSequence::new();

        // Can't boot kernel without PAL init
        assert!(boot.boot_kernel().is_err());
    }

    #[test]
    fn boot_log_entries() {
        let mut boot = BootSequence::new();
        assert!(boot.init_pal("test-platform").is_ok());
        assert!(!boot.log().is_empty());
        assert!(boot.log()[0].message.contains("test-platform"));
    }

    #[test]
    fn shutdown_sequence() {
        let mut boot = BootSequence::new();
        let mut mgr = ServiceManager::new();

        let _ = boot.init_pal("test");
        let _ = boot.boot_kernel();
        let _ = boot.start_services(&mut mgr);
        let _ = boot.launch_shell();

        boot.shutdown();
        assert_eq!(boot.phase(), BootPhase::ShuttingDown);

        boot.halt();
        assert_eq!(boot.phase(), BootPhase::Halted);
    }
}
