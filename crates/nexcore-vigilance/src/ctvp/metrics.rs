//! Metrics Collection and Reporting

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A point-in-time metric snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: f64,
    /// Timestamp
    pub timestamp: f64,
    /// Unit of measurement
    pub unit: String,
    /// Additional tags
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

impl MetricSnapshot {
    /// Creates new metric snapshot
    ///
    /// # Arguments
    /// * `name` - Metric name
    /// * `value` - Metric value
    ///
    /// # Returns
    /// New MetricSnapshot
    pub fn new(name: &str, value: f64) -> Self {
        Self {
            name: name.to_string(),
            value,
            timestamp: super::now(),
            unit: String::new(),
            tags: HashMap::new(),
        }
    }

    /// Sets unit
    ///
    /// # Arguments
    /// * `unit` - Unit string
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = unit.to_string();
        self
    }

    /// Adds tag
    ///
    /// # Arguments
    /// * `key` - Tag key
    /// * `value` - Tag value
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }
}

/// Collection of validation metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationMetrics {
    /// Metrics by name
    metrics: HashMap<String, Vec<MetricSnapshot>>,
    /// Maximum history per metric
    max_history: usize,
}

impl ValidationMetrics {
    /// Creates new metrics collection
    ///
    /// # Returns
    /// New ValidationMetrics
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            max_history: 10000,
        }
    }

    /// Sets max history
    ///
    /// # Arguments
    /// * `max` - Maximum snapshots per metric
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Records a metric
    ///
    /// # Arguments
    /// * `snapshot` - The metric snapshot
    pub fn record(&mut self, snapshot: MetricSnapshot) {
        let history = self.metrics.entry(snapshot.name.clone()).or_default();
        history.push(snapshot);
        if history.len() > self.max_history {
            history.remove(0);
        }
    }

    /// Records a simple value
    ///
    /// # Arguments
    /// * `name` - Metric name
    /// * `value` - Metric value
    pub fn record_value(&mut self, name: &str, value: f64) {
        self.record(MetricSnapshot::new(name, value));
    }

    /// Gets latest value for a metric
    ///
    /// # Arguments
    /// * `name` - Metric name
    ///
    /// # Returns
    /// Latest value or None
    pub fn latest(&self, name: &str) -> Option<f64> {
        self.metrics
            .get(name)
            .and_then(|h| h.last())
            .map(|s| s.value)
    }

    /// Gets history for a metric
    ///
    /// # Arguments
    /// * `name` - Metric name
    ///
    /// # Returns
    /// Slice of snapshots
    pub fn history(&self, name: &str) -> &[MetricSnapshot] {
        self.metrics.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Gets all metric names
    ///
    /// # Returns
    /// Iterator over metric names
    pub fn metric_names(&self) -> impl Iterator<Item = &String> {
        self.metrics.keys()
    }

    /// Calculates mean for a metric
    ///
    /// # Arguments
    /// * `name` - Metric name
    ///
    /// # Returns
    /// Mean value or 0.0
    pub fn mean(&self, name: &str) -> f64 {
        let history = self.history(name);
        if history.is_empty() {
            return 0.0;
        }
        let sum: f64 = history.iter().map(|s| s.value).sum();
        sum / history.len() as f64
    }

    /// Calculates percentile for a metric
    ///
    /// # Arguments
    /// * `name` - Metric name
    /// * `p` - Percentile (0-100)
    ///
    /// # Returns
    /// Value at percentile
    pub fn percentile(&self, name: &str, p: f64) -> f64 {
        let history = self.history(name);
        if history.is_empty() {
            return 0.0;
        }
        let mut values: Vec<f64> = history.iter().map(|s| s.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((p / 100.0) * (values.len() - 1) as f64).round() as usize;
        values.get(idx).copied().unwrap_or(0.0)
    }

    /// Generates summary report
    ///
    /// # Returns
    /// Formatted report string
    pub fn report(&self) -> String {
        let mut r = String::new();
        r.push_str("╔═══════════════════════════════════════════╗\n");
        r.push_str("║  📊 CTVP Metrics Summary                  ║\n");
        r.push_str("╠═══════════════════════════════════════════╣\n");

        for name in self.metric_names() {
            let latest = self.latest(name).unwrap_or(0.0);
            let mean = self.mean(name);
            let count = self.history(name).len();
            r.push_str(&format!(
                "║  {:<15} latest={:.2} mean={:.2} n={:<4}║\n",
                name, latest, mean, count
            ));
        }

        r.push_str("╚═══════════════════════════════════════════╝\n");
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics() {
        let mut m = ValidationMetrics::new();
        m.record_value("car", 0.85);
        m.record_value("car", 0.90);
        assert!((m.mean("car") - 0.875).abs() < 0.001);
    }

    #[test]
    fn test_percentile() {
        let mut m = ValidationMetrics::new();
        for v in [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0] {
            m.record_value("latency", v);
        }
        // Nearest-rank method: idx = round(0.5 * 9) = 5, values[5] = 60
        assert!((m.percentile("latency", 50.0) - 60.0).abs() < 1.0);
        // p90 should be 100.0: idx = round(0.9 * 9) = 8, values[8] = 90
        assert!((m.percentile("latency", 90.0) - 90.0).abs() < 1.0);
    }
}
