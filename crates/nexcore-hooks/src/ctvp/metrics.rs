//! Metrics Collection and Reporting for CTVP

use serde::{Deserialize, Serialize};

/// A point-in-time snapshot of validation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// Unix timestamp
    pub timestamp: f64,
    /// Capability Achievement Rate
    pub car: f64,
    /// Number of observations
    pub observations: u32,
    /// Number of successful observations
    pub successes: u32,
    /// Trend direction
    pub trend: crate::ctvp::TrendDirection,
}

impl MetricSnapshot {
    /// Creates a new snapshot
    ///
    /// # Arguments
    /// * `car` - Capability Achievement Rate
    /// * `observations` - Total observations
    /// * `successes` - Successful observations
    /// * `trend` - Current trend
    ///
    /// # Returns
    /// New MetricSnapshot
    pub fn new(
        car: f64,
        observations: u32,
        successes: u32,
        trend: crate::ctvp::TrendDirection,
    ) -> Self {
        Self {
            timestamp: crate::state::now(),
            car,
            observations,
            successes,
            trend,
        }
    }
}

/// Aggregated validation metrics over time
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationMetrics {
    /// Historical snapshots
    pub snapshots: Vec<MetricSnapshot>,
    /// Maximum snapshots to retain
    max_snapshots: usize,
}

impl ValidationMetrics {
    /// Creates new ValidationMetrics
    ///
    /// # Returns
    /// New ValidationMetrics with default settings
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots: 1000,
        }
    }

    /// Adds a snapshot
    ///
    /// # Arguments
    /// * `snapshot` - The snapshot to add
    pub fn add_snapshot(&mut self, snapshot: MetricSnapshot) {
        self.snapshots.push(snapshot);
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
    }

    /// Returns snapshots for a time period
    ///
    /// # Arguments
    /// * `hours` - Number of hours to look back
    ///
    /// # Returns
    /// Vector of snapshots within the period
    pub fn for_period(&self, hours: f64) -> Vec<&MetricSnapshot> {
        let cutoff = crate::state::now() - (hours * 3600.0);
        self.snapshots
            .iter()
            .filter(|s| s.timestamp >= cutoff)
            .collect()
    }

    /// Returns average CAR for a period
    ///
    /// # Arguments
    /// * `hours` - Number of hours to average over
    ///
    /// # Returns
    /// Average CAR or 0.0 if no data
    pub fn average_car(&self, hours: f64) -> f64 {
        let period = self.for_period(hours);
        if period.is_empty() {
            return 0.0;
        }
        period.iter().map(|s| s.car).sum::<f64>() / period.len() as f64
    }

    /// Returns the most recent snapshot
    ///
    /// # Returns
    /// Most recent snapshot or None
    pub fn latest(&self) -> Option<&MetricSnapshot> {
        self.snapshots.last()
    }

    /// Returns min CAR in period
    ///
    /// # Arguments
    /// * `hours` - Period in hours
    ///
    /// # Returns
    /// Minimum CAR in period or 0.0 if empty
    pub fn min_car(&self, hours: f64) -> f64 {
        self.for_period(hours)
            .iter()
            .map(|s| s.car)
            .reduce(f64::min)
            .unwrap_or(0.0)
    }

    /// Returns max CAR in period
    ///
    /// # Arguments
    /// * `hours` - Period in hours
    ///
    /// # Returns
    /// Maximum CAR in period or 0.0 if empty
    pub fn max_car(&self, hours: f64) -> f64 {
        self.for_period(hours)
            .iter()
            .map(|s| s.car)
            .reduce(f64::max)
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctvp::TrendDirection;

    #[test]
    fn test_metric_snapshot() {
        let snap = MetricSnapshot::new(0.85, 100, 85, TrendDirection::Stable);
        assert!((snap.car - 0.85).abs() < 0.001);
        assert_eq!(snap.observations, 100);
    }

    #[test]
    fn test_validation_metrics() {
        let mut metrics = ValidationMetrics::new();
        metrics.add_snapshot(MetricSnapshot::new(0.80, 10, 8, TrendDirection::Stable));
        metrics.add_snapshot(MetricSnapshot::new(0.85, 20, 17, TrendDirection::Improving));
        metrics.add_snapshot(MetricSnapshot::new(0.90, 30, 27, TrendDirection::Improving));

        assert!(metrics.latest().is_some());
        let latest_car = metrics.latest().map_or(0.0, |s| s.car);
        assert!((latest_car - 0.90).abs() < 0.001);
    }

    #[test]
    fn test_empty_metrics() {
        let metrics = ValidationMetrics::new();
        assert_eq!(metrics.min_car(24.0), 0.0);
        assert_eq!(metrics.max_car(24.0), 0.0);
        assert_eq!(metrics.average_car(24.0), 0.0);
    }
}
