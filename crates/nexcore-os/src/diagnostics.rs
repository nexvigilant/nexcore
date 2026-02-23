// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Diagnostic snapshots — point-in-time system state capture.
//!
//! Inspired by Fuchsia Inspect (live component health trees) and Apple sysdiagnose
//! (diagnostic bundle collection). Captures the full OS state for debugging,
//! incident response, and health monitoring.
//!
//! ## Primitive Grounding
//!
//! - Σ Sum: Aggregation of all subsystem states
//! - ς State: Per-service and per-subsystem state capture
//! - N Quantity: Metric counters, trust scores, queue depths
//! - κ Comparison: Health status assessment (OK/Degraded/Critical)
//! - ν Frequency: Snapshot timestamp (tick-aligned)

use crate::security::SecurityLevel;
use crate::service::ServiceState;

use std::fmt;

// ── Health Status ───────────────────────────────────────────────────

/// Health status for a subsystem or the overall system.
///
/// Inspired by Fuchsia's standardized health convention.
///
/// Tier: T2-P (κ Comparison — health assessment)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Subsystem operating normally.
    Ok,
    /// Starting up — not yet fully operational.
    StartingUp,
    /// Degraded — functional but impaired.
    Degraded,
    /// Unhealthy — significant issues detected.
    Unhealthy,
    /// Critical — immediate attention required.
    Critical,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => write!(f, "OK"),
            Self::StartingUp => write!(f, "STARTING_UP"),
            Self::Degraded => write!(f, "DEGRADED"),
            Self::Unhealthy => write!(f, "UNHEALTHY"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

// ── Service Health ──────────────────────────────────────────────────

/// Health snapshot for a single service.
///
/// Tier: T2-C (ς State + N Quantity — service state with metrics)
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    /// Service name.
    pub name: String,
    /// Current lifecycle state.
    pub state: ServiceState,
    /// Computed health status.
    pub health: HealthStatus,
    /// Number of STOS transitions executed (from CountMetrics).
    pub transitions_executed: u64,
}

impl ServiceHealth {
    /// Derive health from service state.
    pub fn from_state(name: impl Into<String>, state: ServiceState, transitions: u64) -> Self {
        let health = match state {
            ServiceState::Running => HealthStatus::Ok,
            ServiceState::Starting => HealthStatus::StartingUp,
            ServiceState::Degraded => HealthStatus::Degraded,
            ServiceState::Failed => HealthStatus::Critical,
            ServiceState::Stopped | ServiceState::Stopping => HealthStatus::Unhealthy,
            ServiceState::Registered => HealthStatus::StartingUp,
        };
        Self {
            name: name.into(),
            state,
            health,
            transitions_executed: transitions,
        }
    }
}

// ── System Diagnostics ──────────────────────────────────────────────

/// Full system diagnostic snapshot.
///
/// Captures the state of all subsystems at a point in time. Used for
/// debugging, incident response, health dashboards, and diagnostic bundles.
///
/// Tier: T3 (Σ + ς + N + κ + ν)
#[derive(Debug, Clone)]
pub struct DiagnosticSnapshot {
    /// Tick when snapshot was taken.
    pub tick: u64,
    /// Overall system health.
    pub system_health: HealthStatus,
    /// Per-service health.
    pub services: Vec<ServiceHealth>,
    /// Security subsystem state.
    pub security_level: SecurityLevel,
    /// Trust engine score.
    pub trust_score: f64,
    /// Active threat count.
    pub active_threats: usize,
    /// STOS aggregate stats.
    pub stos_machines: usize,
    pub stos_total_transitions: u64,
    /// IPC bus stats.
    pub ipc_pending: usize,
    pub ipc_total_emitted: u64,
    /// Journal stats.
    pub journal_entries: usize,
    pub journal_errors: usize,
    pub journal_total_recorded: u64,
    /// Services summary.
    pub services_running: usize,
    pub services_failed: usize,
    pub services_total: usize,
}

impl DiagnosticSnapshot {
    /// Compute overall system health from subsystem states.
    ///
    /// Uses worst-of-all-subsystems logic:
    /// - Any Critical -> Critical
    /// - Any Unhealthy -> Unhealthy
    /// - Any Degraded -> Degraded
    /// - All Ok -> Ok
    pub fn compute_system_health(
        security_level: SecurityLevel,
        services: &[ServiceHealth],
    ) -> HealthStatus {
        // Security lockdown -> Critical
        if security_level == SecurityLevel::Red {
            return HealthStatus::Critical;
        }

        let mut worst = HealthStatus::Ok;
        for svc in services {
            match svc.health {
                HealthStatus::Critical => return HealthStatus::Critical,
                HealthStatus::Unhealthy if worst != HealthStatus::Critical => {
                    worst = HealthStatus::Unhealthy;
                }
                HealthStatus::Degraded
                    if worst != HealthStatus::Critical && worst != HealthStatus::Unhealthy =>
                {
                    worst = HealthStatus::Degraded;
                }
                HealthStatus::StartingUp if worst == HealthStatus::Ok => {
                    worst = HealthStatus::StartingUp;
                }
                _ => {}
            }
        }

        // Elevated security level degrades health
        if security_level == SecurityLevel::Orange && worst == HealthStatus::Ok {
            return HealthStatus::Degraded;
        }

        worst
    }
}

impl fmt::Display for DiagnosticSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== NexCore OS Diagnostic Snapshot (tick {}) ===", self.tick)?;
        writeln!(f, "System Health: {}", self.system_health)?;
        writeln!(f, "Security:     {}", self.security_level)?;
        writeln!(f, "Trust Score:  {:.3}", self.trust_score)?;
        writeln!(
            f,
            "Services:     {}/{} running, {} failed",
            self.services_running, self.services_total, self.services_failed
        )?;
        writeln!(
            f,
            "STOS:         {} machines, {} transitions",
            self.stos_machines, self.stos_total_transitions
        )?;
        writeln!(
            f,
            "IPC:          {} pending, {} total emitted",
            self.ipc_pending, self.ipc_total_emitted
        )?;
        writeln!(
            f,
            "Journal:      {} entries, {} errors, {} total recorded",
            self.journal_entries, self.journal_errors, self.journal_total_recorded
        )?;
        writeln!(f, "--- Service Detail ---")?;
        for svc in &self.services {
            writeln!(
                f,
                "  {:<20} {:?} ({}), {} transitions",
                svc.name, svc.state, svc.health, svc.transitions_executed
            )?;
        }
        Ok(())
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_from_service_state() {
        let running = ServiceHealth::from_state("guardian", ServiceState::Running, 10);
        assert_eq!(running.health, HealthStatus::Ok);

        let failed = ServiceHealth::from_state("vault", ServiceState::Failed, 5);
        assert_eq!(failed.health, HealthStatus::Critical);

        let starting = ServiceHealth::from_state("brain", ServiceState::Starting, 1);
        assert_eq!(starting.health, HealthStatus::StartingUp);

        let degraded = ServiceHealth::from_state("network", ServiceState::Degraded, 8);
        assert_eq!(degraded.health, HealthStatus::Degraded);
    }

    #[test]
    fn system_health_all_ok() {
        let services = vec![
            ServiceHealth::from_state("a", ServiceState::Running, 1),
            ServiceHealth::from_state("b", ServiceState::Running, 1),
        ];
        let health = DiagnosticSnapshot::compute_system_health(SecurityLevel::Green, &services);
        assert_eq!(health, HealthStatus::Ok);
    }

    #[test]
    fn system_health_one_failed_is_critical() {
        let services = vec![
            ServiceHealth::from_state("a", ServiceState::Running, 1),
            ServiceHealth::from_state("b", ServiceState::Failed, 1),
        ];
        let health = DiagnosticSnapshot::compute_system_health(SecurityLevel::Green, &services);
        assert_eq!(health, HealthStatus::Critical);
    }

    #[test]
    fn system_health_security_red_always_critical() {
        let services = vec![
            ServiceHealth::from_state("a", ServiceState::Running, 1),
        ];
        let health = DiagnosticSnapshot::compute_system_health(SecurityLevel::Red, &services);
        assert_eq!(health, HealthStatus::Critical);
    }

    #[test]
    fn system_health_security_orange_degrades_ok() {
        let services = vec![
            ServiceHealth::from_state("a", ServiceState::Running, 1),
        ];
        let health = DiagnosticSnapshot::compute_system_health(SecurityLevel::Orange, &services);
        assert_eq!(health, HealthStatus::Degraded);
    }

    #[test]
    fn system_health_worst_of_all() {
        let services = vec![
            ServiceHealth::from_state("a", ServiceState::Running, 1),
            ServiceHealth::from_state("b", ServiceState::Degraded, 1),
            ServiceHealth::from_state("c", ServiceState::Starting, 1),
        ];
        let health = DiagnosticSnapshot::compute_system_health(SecurityLevel::Green, &services);
        assert_eq!(health, HealthStatus::Degraded);
    }

    #[test]
    fn diagnostic_snapshot_display() {
        let snapshot = DiagnosticSnapshot {
            tick: 42,
            system_health: HealthStatus::Ok,
            services: vec![
                ServiceHealth::from_state("guardian", ServiceState::Running, 10),
            ],
            security_level: SecurityLevel::Green,
            trust_score: 0.85,
            active_threats: 0,
            stos_machines: 11,
            stos_total_transitions: 44,
            ipc_pending: 0,
            ipc_total_emitted: 28,
            journal_entries: 15,
            journal_errors: 0,
            journal_total_recorded: 15,
            services_running: 11,
            services_failed: 0,
            services_total: 11,
        };
        let display = format!("{snapshot}");
        assert!(display.contains("tick 42"));
        assert!(display.contains("OK"));
        assert!(display.contains("guardian"));
    }
}
