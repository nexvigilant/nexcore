// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # PeriodicMonitor
//!
//! **Tier**: T2-C (nu + exists + partial + causality)
//! **Dominant**: nu (Frequency)
//!
//! Heartbeat/liveness monitoring with configurable check intervals
//! and failure detection thresholds.

use core::fmt;

/// Health status of a monitored target.
///
/// ## Tier: T2-P (exists + kappa)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Target is responding normally.
    Healthy,
    /// Target has missed some heartbeats but within tolerance.
    Degraded,
    /// Target has exceeded failure threshold — considered dead.
    Dead,
    /// No heartbeats received yet.
    Unknown,
}

/// A heartbeat record.
#[derive(Debug, Clone, Copy)]
struct Heartbeat {
    /// Monotonic timestamp of the heartbeat.
    timestamp: u64,
    /// Whether the check succeeded.
    success: bool,
}

/// Periodic liveness monitor.
///
/// ## Tier: T2-C (nu + exists + partial + causality)
/// Dominant: nu (Frequency)
///
/// Tracks periodic heartbeats and determines health status.
/// A target is "dead" if it misses `failure_threshold` consecutive checks.
#[derive(Debug, Clone)]
pub struct PeriodicMonitor {
    /// Target name/identifier.
    name: String,
    /// Expected check interval (ms).
    check_interval_ms: u64,
    /// Number of consecutive failures before declaring dead.
    failure_threshold: u32,
    /// Number of consecutive successes to recover from degraded.
    recovery_threshold: u32,
    /// Consecutive failures.
    consecutive_failures: u32,
    /// Consecutive successes.
    consecutive_successes: u32,
    /// Current status.
    status: HealthStatus,
    /// Total checks performed.
    total_checks: u64,
    /// Total successful checks.
    total_successes: u64,
    /// Last heartbeat timestamp.
    last_heartbeat: Option<u64>,
}

impl PeriodicMonitor {
    /// Create a new monitor for a target.
    #[must_use]
    pub fn new(name: impl Into<String>, check_interval_ms: u64) -> Self {
        Self {
            name: name.into(),
            check_interval_ms,
            failure_threshold: 3,
            recovery_threshold: 2,
            consecutive_failures: 0,
            consecutive_successes: 0,
            status: HealthStatus::Unknown,
            total_checks: 0,
            total_successes: 0,
            last_heartbeat: None,
        }
    }

    /// Set the failure threshold.
    #[must_use]
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold.max(1);
        self
    }

    /// Set the recovery threshold.
    #[must_use]
    pub fn with_recovery_threshold(mut self, threshold: u32) -> Self {
        self.recovery_threshold = threshold.max(1);
        self
    }

    /// Record a heartbeat (success or failure).
    pub fn record_heartbeat(&mut self, timestamp: u64, success: bool) {
        self.total_checks += 1;
        self.last_heartbeat = Some(timestamp);

        if success {
            self.total_successes += 1;
            self.consecutive_successes += 1;
            self.consecutive_failures = 0;

            match self.status {
                HealthStatus::Unknown | HealthStatus::Dead | HealthStatus::Degraded => {
                    if self.consecutive_successes >= self.recovery_threshold {
                        self.status = HealthStatus::Healthy;
                    } else if self.status == HealthStatus::Dead {
                        self.status = HealthStatus::Degraded;
                    } else if self.status == HealthStatus::Unknown {
                        self.status = HealthStatus::Healthy;
                    }
                }
                HealthStatus::Healthy => {} // stay healthy
            }
        } else {
            self.consecutive_failures += 1;
            self.consecutive_successes = 0;

            if self.consecutive_failures >= self.failure_threshold {
                self.status = HealthStatus::Dead;
            } else if self.status == HealthStatus::Healthy {
                self.status = HealthStatus::Degraded;
            }
        }
    }

    /// Current health status.
    #[must_use]
    pub fn status(&self) -> HealthStatus {
        self.status
    }

    /// Whether the target is considered alive.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Target name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Uptime ratio (0.0 to 1.0).
    #[must_use]
    pub fn uptime_ratio(&self) -> f64 {
        if self.total_checks == 0 {
            return 0.0;
        }
        self.total_successes as f64 / self.total_checks as f64
    }

    /// Expected check frequency in Hz.
    #[must_use]
    pub fn expected_frequency_hz(&self) -> f64 {
        if self.check_interval_ms == 0 {
            return 0.0;
        }
        1000.0 / self.check_interval_ms as f64
    }

    /// Consecutive failures count.
    #[must_use]
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }

    /// Total checks performed.
    #[must_use]
    pub fn total_checks(&self) -> u64 {
        self.total_checks
    }
}

impl fmt::Display for PeriodicMonitor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Monitor[{}]: {:?} (uptime {:.1}%, {}ms interval)",
            self.name,
            self.status,
            self.uptime_ratio() * 100.0,
            self.check_interval_ms,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let mon = PeriodicMonitor::new("test", 1000);
        assert_eq!(mon.status(), HealthStatus::Unknown);
        assert!(!mon.is_alive());
    }

    #[test]
    fn test_healthy_on_first_success() {
        let mut mon = PeriodicMonitor::new("test", 1000);
        mon.record_heartbeat(1000, true);
        assert_eq!(mon.status(), HealthStatus::Healthy);
        assert!(mon.is_alive());
    }

    #[test]
    fn test_degraded_on_failure() {
        let mut mon = PeriodicMonitor::new("test", 1000).with_failure_threshold(3);

        mon.record_heartbeat(1000, true); // Healthy
        mon.record_heartbeat(2000, false); // Degraded (1 failure)
        assert_eq!(mon.status(), HealthStatus::Degraded);
        assert!(mon.is_alive()); // still alive
    }

    #[test]
    fn test_dead_after_threshold() {
        let mut mon = PeriodicMonitor::new("test", 1000).with_failure_threshold(3);

        mon.record_heartbeat(1000, true);
        mon.record_heartbeat(2000, false);
        mon.record_heartbeat(3000, false);
        mon.record_heartbeat(4000, false); // 3rd consecutive failure
        assert_eq!(mon.status(), HealthStatus::Dead);
        assert!(!mon.is_alive());
    }

    #[test]
    fn test_recovery() {
        let mut mon = PeriodicMonitor::new("test", 1000)
            .with_failure_threshold(2)
            .with_recovery_threshold(2);

        mon.record_heartbeat(1000, true);
        mon.record_heartbeat(2000, false);
        mon.record_heartbeat(3000, false); // Dead
        assert_eq!(mon.status(), HealthStatus::Dead);

        mon.record_heartbeat(4000, true); // Degraded
        assert_eq!(mon.status(), HealthStatus::Degraded);

        mon.record_heartbeat(5000, true); // Healthy (2 consecutive)
        assert_eq!(mon.status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_uptime_ratio() {
        let mut mon = PeriodicMonitor::new("test", 1000);
        mon.record_heartbeat(1000, true);
        mon.record_heartbeat(2000, true);
        mon.record_heartbeat(3000, false);
        mon.record_heartbeat(4000, true);

        // 3 out of 4 = 75%
        assert!((mon.uptime_ratio() - 0.75).abs() < 1e-10);
    }
}
