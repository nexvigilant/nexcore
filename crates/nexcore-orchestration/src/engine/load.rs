//! Load monitoring and graceful degradation.
//!
//! Tracks system load across subsystems and determines when to shed
//! low-priority work. JARVIS never crashes — it degrades gracefully,
//! dropping background intel before ever losing suit control or vitals.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;

/// Degradation levels — determines which priority tiers are shed.
///
/// Maps to JARVIS behavior:
/// - Normal: all systems active, full throughput
/// - Elevated: background analytics deferred
/// - High: only High + Critical tasks accepted
/// - Critical: only Critical tasks (suit control, vitals)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum DegradationLevel {
    /// All systems nominal. All priorities accepted.
    Normal,
    /// Load rising. Low-priority tasks shed.
    Elevated,
    /// Significant load. Only High and Critical accepted.
    High,
    /// Maximum load. Only Critical tasks execute.
    Critical,
}

impl DegradationLevel {
    /// Minimum priority accepted at this degradation level.
    #[must_use]
    pub fn min_priority(self) -> crate::types::Priority {
        match self {
            Self::Normal => crate::types::Priority::Low,
            Self::Elevated => crate::types::Priority::Normal,
            Self::High => crate::types::Priority::High,
            Self::Critical => crate::types::Priority::Critical,
        }
    }

    /// Whether a given priority should be shed at this level.
    #[must_use]
    pub fn should_shed(self, priority: crate::types::Priority) -> bool {
        priority < self.min_priority()
    }

    /// Human-readable name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Elevated => "elevated",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

impl std::fmt::Display for DegradationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Thresholds that govern degradation transitions.
#[derive(Debug, Clone)]
pub struct LoadThresholds {
    /// Queue depth ratio (queue_depth / capacity) that triggers Elevated.
    pub elevated_ratio: f64,
    /// Queue depth ratio that triggers High.
    pub high_ratio: f64,
    /// Queue depth ratio that triggers Critical.
    pub critical_ratio: f64,
    /// Latency (p95) that triggers Elevated, independent of queue depth.
    pub latency_elevated: Duration,
    /// Latency (p95) that triggers High.
    pub latency_high: Duration,
    /// Minimum time to hold a degradation level before recovering.
    /// Prevents flapping between levels.
    pub hold_duration: Duration,
}

impl Default for LoadThresholds {
    fn default() -> Self {
        Self {
            elevated_ratio: 0.5,
            high_ratio: 0.75,
            critical_ratio: 0.9,
            latency_elevated: Duration::from_millis(500),
            latency_high: Duration::from_secs(2),
            hold_duration: Duration::from_secs(5),
        }
    }
}

/// Monitors system load and computes degradation level.
#[derive(Debug)]
pub struct LoadMonitor {
    thresholds: LoadThresholds,
    /// Current degradation level.
    level: Mutex<DegradationLevel>,
    /// When the current level was entered.
    level_entered_at: Mutex<Instant>,
    /// Cumulative tasks shed.
    tasks_shed: AtomicU64,
}

impl LoadMonitor {
    /// Create a monitor with the given thresholds.
    #[must_use]
    pub fn new(thresholds: LoadThresholds) -> Self {
        Self {
            thresholds,
            level: Mutex::new(DegradationLevel::Normal),
            level_entered_at: Mutex::new(Instant::now()),
            tasks_shed: AtomicU64::new(0),
        }
    }

    /// Create a monitor with default thresholds.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(LoadThresholds::default())
    }

    /// Current degradation level.
    #[must_use]
    pub fn level(&self) -> DegradationLevel {
        self.level
            .lock()
            .map(|guard| *guard)
            .unwrap_or(DegradationLevel::Normal)
    }

    /// Total tasks shed since monitor creation.
    #[must_use]
    pub fn total_shed(&self) -> u64 {
        self.tasks_shed.load(Ordering::Relaxed)
    }

    /// Record that a task was shed.
    pub fn record_shed(&self) {
        self.tasks_shed.fetch_add(1, Ordering::Relaxed);
    }

    /// Evaluate current load signals and update degradation level.
    ///
    /// Call this periodically or on each task submission.
    /// Returns the new level.
    pub fn evaluate(
        &self,
        queue_depth: usize,
        capacity: usize,
        p95_latency: Option<Duration>,
    ) -> DegradationLevel {
        let ratio = if capacity == 0 {
            0.0
        } else {
            queue_depth as f64 / capacity as f64
        };

        // Compute target level from queue ratio
        let queue_level = if ratio >= self.thresholds.critical_ratio {
            DegradationLevel::Critical
        } else if ratio >= self.thresholds.high_ratio {
            DegradationLevel::High
        } else if ratio >= self.thresholds.elevated_ratio {
            DegradationLevel::Elevated
        } else {
            DegradationLevel::Normal
        };

        // Compute target level from latency (if available)
        let latency_level = p95_latency.map_or(DegradationLevel::Normal, |lat| {
            if lat >= self.thresholds.latency_high {
                DegradationLevel::High
            } else if lat >= self.thresholds.latency_elevated {
                DegradationLevel::Elevated
            } else {
                DegradationLevel::Normal
            }
        });

        // Take the worse of queue-based and latency-based signals
        let target = latency_level.max(queue_level);

        // Apply hold duration to prevent flapping on recovery
        let current = self.level();
        if target < current {
            // Recovering — check hold
            let held_long_enough = self
                .level_entered_at
                .lock()
                .map(|entered| entered.elapsed() >= self.thresholds.hold_duration)
                .unwrap_or(true);
            if !held_long_enough {
                return current; // Hold current level
            }
        }

        // Transition
        if target != current {
            if let Ok(mut level) = self.level.lock() {
                *level = target;
            }
            if let Ok(mut entered) = self.level_entered_at.lock() {
                *entered = Instant::now();
            }
        }

        target
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Priority;

    #[test]
    fn degradation_shedding_rules() {
        // Normal: accept everything
        assert!(!DegradationLevel::Normal.should_shed(Priority::Low));
        assert!(!DegradationLevel::Normal.should_shed(Priority::Critical));

        // Elevated: shed Low
        assert!(DegradationLevel::Elevated.should_shed(Priority::Low));
        assert!(!DegradationLevel::Elevated.should_shed(Priority::Normal));

        // High: shed Low + Normal
        assert!(DegradationLevel::High.should_shed(Priority::Low));
        assert!(DegradationLevel::High.should_shed(Priority::Normal));
        assert!(!DegradationLevel::High.should_shed(Priority::High));

        // Critical: shed everything except Critical
        assert!(DegradationLevel::Critical.should_shed(Priority::Low));
        assert!(DegradationLevel::Critical.should_shed(Priority::Normal));
        assert!(DegradationLevel::Critical.should_shed(Priority::High));
        assert!(!DegradationLevel::Critical.should_shed(Priority::Critical));
    }

    #[test]
    fn load_monitor_evaluates_queue_ratio() {
        let monitor = LoadMonitor::new(LoadThresholds {
            hold_duration: Duration::ZERO, // disable hold for testing
            ..LoadThresholds::default()
        });

        // Empty queue
        let level = monitor.evaluate(0, 100, None);
        assert_eq!(level, DegradationLevel::Normal);

        // 60% full
        let level = monitor.evaluate(60, 100, None);
        assert_eq!(level, DegradationLevel::Elevated);

        // 80% full
        let level = monitor.evaluate(80, 100, None);
        assert_eq!(level, DegradationLevel::High);

        // 95% full
        let level = monitor.evaluate(95, 100, None);
        assert_eq!(level, DegradationLevel::Critical);
    }

    #[test]
    fn load_monitor_latency_override() {
        let monitor = LoadMonitor::new(LoadThresholds {
            hold_duration: Duration::ZERO,
            ..LoadThresholds::default()
        });

        // Low queue but high latency → escalate
        let level = monitor.evaluate(10, 100, Some(Duration::from_secs(3)));
        assert_eq!(level, DegradationLevel::High);
    }

    #[test]
    fn load_monitor_hold_prevents_flapping() {
        let monitor = LoadMonitor::new(LoadThresholds {
            hold_duration: Duration::from_secs(60), // long hold
            ..LoadThresholds::default()
        });

        // Escalate to High
        monitor.evaluate(80, 100, None);
        assert_eq!(monitor.level(), DegradationLevel::High);

        // Try to recover immediately — should be held
        let level = monitor.evaluate(10, 100, None);
        assert_eq!(level, DegradationLevel::High); // held, not Normal
    }

    #[test]
    fn load_monitor_shed_counter() {
        let monitor = LoadMonitor::with_defaults();
        assert_eq!(monitor.total_shed(), 0);
        monitor.record_shed();
        monitor.record_shed();
        assert_eq!(monitor.total_shed(), 2);
    }

    #[test]
    fn zero_capacity_doesnt_panic() {
        let monitor = LoadMonitor::with_defaults();
        let level = monitor.evaluate(5, 0, None);
        assert_eq!(level, DegradationLevel::Normal);
    }
}
