//! Capability Definition and Tracking

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Type of metric
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Rate (0.0 to 1.0)
    Rate,
    /// Count
    Count,
    /// Latency (ms)
    Latency,
    /// Boolean
    Boolean,
}

/// A capability to validate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Unique ID
    pub id: String,
    /// Human name
    pub name: String,
    /// Description
    pub description: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Target threshold
    pub threshold: f64,
    /// Minimum observations
    pub min_observations: u32,
    /// Alert threshold
    pub alert_threshold: f64,
}

impl Capability {
    /// Creates new capability
    ///
    /// # Arguments
    /// * `name` - Capability name
    ///
    /// # Returns
    /// New Capability with defaults
    pub fn new(name: &str) -> Self {
        Self {
            id: name.to_lowercase().replace(' ', "_"),
            name: name.to_string(),
            description: String::new(),
            metric_type: MetricType::Rate,
            threshold: super::DEFAULT_CAR_THRESHOLD,
            min_observations: super::DEFAULT_MIN_OBSERVATIONS,
            alert_threshold: super::DEFAULT_ALERT_THRESHOLD,
        }
    }

    /// Sets metric type
    ///
    /// # Arguments
    /// * `mt` - The metric type
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_metric(mut self, mt: MetricType) -> Self {
        self.metric_type = mt;
        self
    }

    /// Sets threshold
    ///
    /// # Arguments
    /// * `t` - The threshold value
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_threshold(mut self, t: f64) -> Self {
        self.threshold = t;
        self
    }

    /// Sets minimum observations
    ///
    /// # Arguments
    /// * `min` - Minimum count
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_min_observations(mut self, min: u32) -> Self {
        self.min_observations = min;
        self
    }

    /// Sets alert threshold
    ///
    /// # Arguments
    /// * `t` - Alert threshold
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_alert_threshold(mut self, t: f64) -> Self {
        self.alert_threshold = t;
        self
    }

    /// Sets description
    ///
    /// # Arguments
    /// * `desc` - Description text
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }
}

/// A single observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Timestamp
    pub timestamp: f64,
    /// Whether achieved
    pub achieved: bool,
    /// Effect value
    pub value: f64,
    /// Context ID
    pub context_id: Option<String>,
}

/// Tracks capability achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityTracker {
    /// The capability
    pub capability: Capability,
    observations: VecDeque<Observation>,
    window_size: usize,
    total_achieved: u64,
    total_observations: u64,
}

impl CapabilityTracker {
    /// Creates new tracker
    ///
    /// # Arguments
    /// * `capability` - The capability to track
    ///
    /// # Returns
    /// New CapabilityTracker
    pub fn new(capability: Capability) -> Self {
        Self {
            capability,
            observations: VecDeque::new(),
            window_size: 10000,
            total_achieved: 0,
            total_observations: 0,
        }
    }

    /// Records an observation
    ///
    /// # Arguments
    /// * `achieved` - Whether capability was achieved
    /// * `value` - Effect value
    pub fn record(&mut self, achieved: bool, value: f64) {
        let obs = Observation {
            timestamp: super::now(),
            achieved,
            value,
            context_id: None,
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

    /// Returns CAR for windowed observations
    ///
    /// # Returns
    /// Capability Achievement Rate
    pub fn car(&self) -> f64 {
        if self.observations.is_empty() {
            return 0.0;
        }
        let achieved = self.observations.iter().filter(|o| o.achieved).count();
        achieved as f64 / self.observations.len() as f64
    }

    /// Returns all-time CAR
    ///
    /// # Returns
    /// All-time CAR
    pub fn car_all_time(&self) -> f64 {
        if self.total_observations == 0 {
            return 0.0;
        }
        self.total_achieved as f64 / self.total_observations as f64
    }

    /// Returns true if sufficient data exists
    ///
    /// # Returns
    /// True if min_observations met
    pub fn has_sufficient_data(&self) -> bool {
        self.observations.len() >= self.capability.min_observations as usize
    }

    /// Returns true if meets threshold
    ///
    /// # Returns
    /// True if CAR >= threshold
    pub fn meets_threshold(&self) -> bool {
        self.has_sufficient_data() && self.car() >= self.capability.threshold
    }

    /// Returns true if alerting
    ///
    /// # Returns
    /// True if CAR < alert_threshold
    pub fn is_alerting(&self) -> bool {
        self.has_sufficient_data() && self.car() < self.capability.alert_threshold
    }

    /// Returns observation count
    ///
    /// # Returns
    /// Number of observations
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    /// Returns observations needed
    ///
    /// # Returns
    /// Count needed to reach minimum
    pub fn observations_needed(&self) -> u32 {
        let current = self.observations.len() as u32;
        if current >= self.capability.min_observations {
            0
        } else {
            self.capability.min_observations - current
        }
    }

    /// Returns percentile value
    ///
    /// # Arguments
    /// * `p` - Percentile (0-100)
    ///
    /// # Returns
    /// Value at percentile
    pub fn percentile(&self, p: f64) -> f64 {
        if self.observations.is_empty() {
            return 0.0;
        }
        let mut values: Vec<f64> = self.observations.iter().map(|o| o.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((p / 100.0) * (values.len() - 1) as f64).round() as usize;
        values.get(idx).copied().unwrap_or(0.0)
    }

    /// Generates status report
    ///
    /// # Returns
    /// Formatted status string
    pub fn report(&self) -> String {
        let status = if !self.has_sufficient_data() {
            format!("⚠️ Need {} more", self.observations_needed())
        } else if self.meets_threshold() {
            "✅ Met".to_string()
        } else if self.is_alerting() {
            "🚨 ALERT".to_string()
        } else {
            "🟡 Below".to_string()
        };
        format!(
            "{}: CAR={:.1}% (n={}) - {}",
            self.capability.name,
            self.car() * 100.0,
            self.observation_count(),
            status
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability() {
        let cap = Capability::new("Test").with_threshold(0.90);
        assert_eq!(cap.threshold, 0.90);
    }

    #[test]
    fn test_tracker() {
        let cap = Capability::new("Test").with_min_observations(3);
        let mut t = CapabilityTracker::new(cap);
        t.record(true, 1.0);
        t.record(true, 1.0);
        t.record(false, 0.0);
        assert!((t.car() - 0.666).abs() < 0.01);
    }
}
