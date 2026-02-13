//! Capability Definition and Tracking
//!
//! Defines capabilities and tracks their achievement rate over time.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Type of metric being tracked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Rate metric (0.0 to 1.0) - e.g., CAR, adoption rate
    Rate,
    /// Count metric - e.g., number of events
    Count,
    /// Latency metric (milliseconds) - e.g., response time
    Latency,
    /// Boolean metric - e.g., pass/fail
    Boolean,
}

/// A capability that can be validated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this capability does
    pub description: String,
    /// Type of primary metric
    pub metric_type: MetricType,
    /// Target threshold for validation
    pub threshold: f64,
    /// Minimum observations before threshold check is valid
    pub min_observations: u32,
    /// Alert threshold (triggers alert when below)
    pub alert_threshold: f64,
}

impl Capability {
    /// Creates a new capability with defaults
    pub fn new(name: &str) -> Self {
        Self {
            id: name.to_lowercase().replace(' ', "_"),
            name: name.to_string(),
            description: String::new(),
            metric_type: MetricType::Rate,
            threshold: crate::ctvp::DEFAULT_CAR_THRESHOLD,
            min_observations: crate::ctvp::DEFAULT_MIN_SESSIONS,
            alert_threshold: crate::ctvp::DEFAULT_ALERT_THRESHOLD,
        }
    }

    /// Sets the metric type
    pub fn with_metric(mut self, metric_type: MetricType) -> Self {
        self.metric_type = metric_type;
        self
    }

    /// Sets the validation threshold
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    /// Sets the minimum observations required
    pub fn with_min_observations(mut self, min: u32) -> Self {
        self.min_observations = min;
        self
    }

    /// Sets the alert threshold
    pub fn with_alert_threshold(mut self, threshold: f64) -> Self {
        self.alert_threshold = threshold;
        self
    }

    /// Sets the description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }
}

/// A single observation of capability achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Unix timestamp
    pub timestamp: f64,
    /// Whether capability was achieved
    pub achieved: bool,
    /// Effect value (latency, count, etc.)
    pub value: f64,
    /// Optional context identifier (session, request, etc.)
    pub context_id: Option<String>,
}

/// Tracks capability achievement over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityTracker {
    /// The capability being tracked
    pub capability: Capability,
    /// Observations (recent first, limited to window_size)
    observations: VecDeque<Observation>,
    /// Maximum observations to keep
    window_size: usize,
    /// Total achieved count (all time)
    total_achieved: u64,
    /// Total observations (all time)
    total_observations: u64,
}

impl CapabilityTracker {
    /// Creates a new tracker for a capability
    pub fn new(capability: Capability) -> Self {
        Self {
            capability,
            observations: VecDeque::new(),
            window_size: 10000,
            total_achieved: 0,
            total_observations: 0,
        }
    }

    /// Sets the window size (number of observations to keep)
    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size;
        self
    }

    /// Records an observation
    pub fn record(&mut self, achieved: bool, value: f64) {
        self.record_with_context(achieved, value, None);
    }

    /// Records an observation with context
    pub fn record_with_context(&mut self, achieved: bool, value: f64, context_id: Option<String>) {
        let obs = Observation {
            timestamp: crate::state::now(),
            achieved,
            value,
            context_id,
        };

        self.observations.push_front(obs);
        if self.observations.len() > self.window_size {
            self.observations.pop_back();
        }

        self.total_observations += 1;
        if achieved {
            self.total_achieved += 1;
        }
    }

    /// Returns the Capability Achievement Rate (CAR) for windowed observations
    pub fn car(&self) -> f64 {
        if self.observations.is_empty() {
            return 0.0;
        }
        let achieved = self.observations.iter().filter(|o| o.achieved).count();
        achieved as f64 / self.observations.len() as f64
    }

    /// Returns all-time CAR
    pub fn car_all_time(&self) -> f64 {
        if self.total_observations == 0 {
            return 0.0;
        }
        self.total_achieved as f64 / self.total_observations as f64
    }

    /// Returns CAR for a specific time window (hours)
    pub fn car_for_period(&self, hours: f64) -> f64 {
        let cutoff = crate::state::now() - (hours * 3600.0);
        let filtered: Vec<_> = self
            .observations
            .iter()
            .filter(|o| o.timestamp >= cutoff)
            .collect();
        if filtered.is_empty() {
            return 0.0;
        }
        let achieved = filtered.iter().filter(|o| o.achieved).count();
        achieved as f64 / filtered.len() as f64
    }

    /// Returns true if enough observations exist for validation
    pub fn has_sufficient_data(&self) -> bool {
        self.observations.len() >= self.capability.min_observations as usize
    }

    /// Returns true if capability meets its threshold
    pub fn meets_threshold(&self) -> bool {
        self.has_sufficient_data() && self.car() >= self.capability.threshold
    }

    /// Returns true if CAR is below alert threshold
    pub fn is_alerting(&self) -> bool {
        self.has_sufficient_data() && self.car() < self.capability.alert_threshold
    }

    /// Returns the number of observations
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    /// Returns observations needed to reach min_observations
    pub fn observations_needed(&self) -> u32 {
        let current = self.observations.len() as u32;
        if current >= self.capability.min_observations {
            0
        } else {
            self.capability.min_observations - current
        }
    }

    /// Returns percentile value for latency metrics
    pub fn percentile(&self, p: f64) -> f64 {
        if self.observations.is_empty() {
            return 0.0;
        }
        let mut values: Vec<f64> = self.observations.iter().map(|o| o.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((p / 100.0) * (values.len() - 1) as f64).round() as usize;
        values.get(idx).copied().unwrap_or(0.0)
    }

    /// Generates a status report
    pub fn report(&self) -> String {
        let status = if !self.has_sufficient_data() {
            format!("⚠️ Need {} more observations", self.observations_needed())
        } else if self.meets_threshold() {
            "✅ Threshold met".to_string()
        } else if self.is_alerting() {
            "🚨 ALERT: Below threshold".to_string()
        } else {
            "🟡 Below target".to_string()
        };

        format!(
            "{}: CAR={:.1}% (n={}) - {}",
            self.capability.name,
            self.car() * 100.0,
            self.observations.len(),
            status
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_creation() {
        let cap = Capability::new("Test Capability")
            .with_threshold(0.90)
            .with_min_observations(5);
        assert_eq!(cap.threshold, 0.90);
        assert_eq!(cap.min_observations, 5);
    }

    #[test]
    fn test_tracker_car() {
        let cap = Capability::new("Test").with_min_observations(3);
        let mut tracker = CapabilityTracker::new(cap);

        tracker.record(true, 1.0);
        tracker.record(true, 1.0);
        tracker.record(false, 0.0);

        assert!((tracker.car() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_tracker_threshold() {
        let cap = Capability::new("Test")
            .with_threshold(0.80)
            .with_min_observations(5);
        let mut tracker = CapabilityTracker::new(cap);

        // Not enough data
        for _ in 0..4 {
            tracker.record(true, 1.0);
        }
        assert!(!tracker.has_sufficient_data());
        assert!(!tracker.meets_threshold());

        // Add one more
        tracker.record(true, 1.0);
        assert!(tracker.has_sufficient_data());
        assert!(tracker.meets_threshold()); // 100% > 80%
    }

    #[test]
    fn test_percentile() {
        let cap = Capability::new("Latency").with_metric(MetricType::Latency);
        let mut tracker = CapabilityTracker::new(cap);

        for v in [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0] {
            tracker.record(true, v);
        }

        // p50 of [10,20,30,40,50,60,70,80,90,100] is around 55
        let p50 = tracker.percentile(50.0);
        assert!(p50 >= 40.0 && p50 <= 60.0, "p50={}", p50);
        // p95 should be around 95-100
        let p95 = tracker.percentile(95.0);
        assert!(p95 >= 90.0 && p95 <= 100.0, "p95={}", p95);
    }
}
