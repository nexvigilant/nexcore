//! Per-task and per-subsystem timing metrics.
//!
//! Tracks completion latency, throughput, and active task counts
//! for the parallel task execution engine.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;

/// Rolling window of task completion times for latency tracking.
#[derive(Debug)]
pub struct LatencyTracker {
    /// Completed task durations (most recent first).
    window: Mutex<VecDeque<Duration>>,
    /// Maximum window size.
    max_samples: usize,
}

impl LatencyTracker {
    /// Create a tracker with the given window size.
    #[must_use]
    pub fn new(max_samples: usize) -> Self {
        Self {
            window: Mutex::new(VecDeque::with_capacity(max_samples)),
            max_samples,
        }
    }

    /// Record a task completion duration.
    pub fn record(&self, duration: Duration) {
        if let Ok(mut window) = self.window.lock() {
            if window.len() >= self.max_samples {
                window.pop_back();
            }
            window.push_front(duration);
        }
    }

    /// Compute percentile latency (0.0..=1.0). Returns `None` if no samples.
    #[must_use]
    pub fn percentile(&self, p: f64) -> Option<Duration> {
        let window = self.window.lock().ok()?;
        if window.is_empty() {
            return None;
        }
        let mut sorted: Vec<Duration> = window.iter().copied().collect();
        sorted.sort();
        let idx = ((p * (sorted.len() as f64 - 1.0)).round() as usize).min(sorted.len() - 1);
        Some(sorted[idx])
    }

    /// Average latency across the window. Returns `None` if no samples.
    #[must_use]
    pub fn mean(&self) -> Option<Duration> {
        let window = self.window.lock().ok()?;
        if window.is_empty() {
            return None;
        }
        let total: Duration = window.iter().sum();
        Some(total / window.len() as u32)
    }

    /// Number of samples in the window.
    #[must_use]
    pub fn sample_count(&self) -> usize {
        self.window.lock().map(|w| w.len()).unwrap_or(0)
    }
}

/// Snapshot of engine-wide metrics at a point in time.
#[derive(Debug, Clone, Serialize)]
pub struct EngineSnapshot {
    /// Total tasks submitted since engine start.
    pub total_submitted: u64,
    /// Total tasks completed (success).
    pub total_completed: u64,
    /// Total tasks failed.
    pub total_failed: u64,
    /// Total tasks shed (dropped under load).
    pub total_shed: u64,
    /// Currently active (executing) tasks.
    pub active_tasks: usize,
    /// Current queue depth across all subsystems.
    pub queue_depth: usize,
    /// Mean latency in microseconds (if available).
    pub mean_latency_us: Option<u64>,
    /// P95 latency in microseconds (if available).
    pub p95_latency_us: Option<u64>,
    /// P99 latency in microseconds (if available).
    pub p99_latency_us: Option<u64>,
    /// Current degradation level name.
    pub degradation_level: String,
    /// Subsystem-level summaries.
    pub subsystems: Vec<SubsystemSnapshot>,
}

/// Per-subsystem metric snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct SubsystemSnapshot {
    /// Subsystem name.
    pub name: String,
    /// Active tasks in this subsystem.
    pub active: usize,
    /// Queued tasks in this subsystem.
    pub queued: usize,
    /// Completed tasks in this subsystem.
    pub completed: u64,
    /// Failed tasks in this subsystem.
    pub failed: u64,
    /// Whether this subsystem is currently degraded.
    pub degraded: bool,
}

/// A stopwatch for measuring a single task's execution time.
#[derive(Debug)]
pub struct TaskTimer {
    start: Instant,
}

impl TaskTimer {
    /// Start a new timer.
    #[must_use]
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Elapsed time since start.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Stop and return the elapsed duration.
    #[must_use]
    pub fn stop(self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latency_tracker_records_and_computes() {
        let tracker = LatencyTracker::new(100);
        tracker.record(Duration::from_millis(10));
        tracker.record(Duration::from_millis(20));
        tracker.record(Duration::from_millis(30));

        assert_eq!(tracker.sample_count(), 3);

        let mean = tracker.mean();
        assert!(mean.is_some());
        let mean = mean.map(|d| d.as_millis());
        assert_eq!(mean, Some(20));
    }

    #[test]
    fn latency_tracker_percentiles() {
        let tracker = LatencyTracker::new(100);
        for i in 1..=100 {
            tracker.record(Duration::from_millis(i));
        }

        // Nearest-rank: p50 of [1..=100] = index 50 (0-based) = 51ms
        let p50 = tracker.percentile(0.5).map(|d| d.as_millis());
        assert_eq!(p50, Some(51));

        let p99 = tracker.percentile(0.99).map(|d| d.as_millis());
        assert_eq!(p99, Some(99));
    }

    #[test]
    fn latency_tracker_window_eviction() {
        let tracker = LatencyTracker::new(3);
        tracker.record(Duration::from_millis(1));
        tracker.record(Duration::from_millis(2));
        tracker.record(Duration::from_millis(3));
        tracker.record(Duration::from_millis(100)); // evicts oldest

        assert_eq!(tracker.sample_count(), 3);
        // Window now: [100, 3, 2] — oldest (1ms) evicted
        let mean = tracker.mean().map(|d| d.as_millis());
        assert_eq!(mean, Some(35)); // (100+3+2)/3 = 35
    }

    #[test]
    fn latency_tracker_empty() {
        let tracker = LatencyTracker::new(10);
        assert!(tracker.mean().is_none());
        assert!(tracker.percentile(0.5).is_none());
        assert_eq!(tracker.sample_count(), 0);
    }

    #[test]
    fn task_timer_measures_elapsed() {
        let timer = TaskTimer::start();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.stop();
        assert!(elapsed >= Duration::from_millis(5));
    }
}
