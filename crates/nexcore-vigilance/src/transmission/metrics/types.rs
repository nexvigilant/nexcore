//! Metrics data types.
//!
//! Pure types for representing metrics, without collection logic.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    /// Monotonically increasing counter.
    Counter,
    /// Value that can go up or down.
    Gauge,
    /// Distribution of values.
    Histogram,
    /// Statistical summary.
    Summary,
}

/// A single metric data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name.
    pub name: String,
    /// Type of metric.
    pub metric_type: MetricType,
    /// Help text describing the metric.
    pub help: String,
    /// Current value.
    pub value: f64,
    /// Labels for the metric.
    pub labels: HashMap<String, String>,
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: u64,
}

impl Metric {
    /// Create a new counter metric.
    #[must_use]
    pub fn counter(name: impl Into<String>, help: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Counter,
            help: help.into(),
            value,
            labels: HashMap::new(),
            timestamp_ms: current_timestamp_ms(),
        }
    }

    /// Create a new gauge metric.
    #[must_use]
    pub fn gauge(name: impl Into<String>, help: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Gauge,
            help: help.into(),
            value,
            labels: HashMap::new(),
            timestamp_ms: current_timestamp_ms(),
        }
    }

    /// Add a label.
    #[must_use]
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Add multiple labels.
    #[must_use]
    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels.extend(labels);
        self
    }

    /// Format as Prometheus exposition format.
    #[must_use]
    pub fn to_prometheus(&self) -> String {
        let labels_str = if self.labels.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = self
                .labels
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect();
            format!("{{{}}}", pairs.join(","))
        };

        format!(
            "# HELP {} {}\n# TYPE {} {:?}\n{}{} {} {}\n",
            self.name,
            self.help,
            self.name,
            self.metric_type,
            self.name,
            labels_str,
            self.value,
            self.timestamp_ms
        )
    }
}

/// Workflow metrics summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowMetricsSummary {
    /// Workflow name.
    pub workflow_name: String,
    /// Total number of executions.
    pub total_executions: u64,
    /// Number of successful executions.
    pub successful_executions: u64,
    /// Number of failed executions.
    pub failed_executions: u64,
    /// Average duration in milliseconds.
    pub average_duration_ms: f64,
    /// Minimum duration in milliseconds.
    pub min_duration_ms: u64,
    /// Maximum duration in milliseconds.
    pub max_duration_ms: u64,
    /// Step-level metrics.
    pub step_metrics: HashMap<usize, StepMetricsSummary>,
    /// Last execution timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_executed_at: Option<String>,
    /// Error type counts.
    pub error_counts: HashMap<String, u64>,
}

impl WorkflowMetricsSummary {
    /// Create a new metrics summary for a workflow.
    #[must_use]
    pub fn new(workflow_name: impl Into<String>) -> Self {
        Self {
            workflow_name: workflow_name.into(),
            ..Default::default()
        }
    }

    /// Calculate success rate.
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.successful_executions as f64 / self.total_executions as f64
        }
    }

    /// Record a successful execution.
    pub fn record_success(&mut self, duration_ms: u64) {
        self.total_executions += 1;
        self.successful_executions += 1;
        self.update_duration_stats(duration_ms);
    }

    /// Record a failed execution.
    pub fn record_failure(&mut self, duration_ms: u64, error_type: &str) {
        self.total_executions += 1;
        self.failed_executions += 1;
        self.update_duration_stats(duration_ms);
        *self.error_counts.entry(error_type.to_string()).or_insert(0) += 1;
    }

    fn update_duration_stats(&mut self, duration_ms: u64) {
        // Update min/max
        if self.total_executions == 1 {
            self.min_duration_ms = duration_ms;
            self.max_duration_ms = duration_ms;
            self.average_duration_ms = duration_ms as f64;
        } else {
            self.min_duration_ms = self.min_duration_ms.min(duration_ms);
            self.max_duration_ms = self.max_duration_ms.max(duration_ms);
            // Running average
            let n = self.total_executions as f64;
            self.average_duration_ms =
                self.average_duration_ms * (n - 1.0) / n + duration_ms as f64 / n;
        }
    }
}

/// Step-level metrics summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepMetricsSummary {
    /// Engine name.
    pub engine: String,
    /// Action name.
    pub action: String,
    /// Total executions.
    pub total_executions: u64,
    /// Successful executions.
    pub successful_executions: u64,
    /// Failed executions.
    pub failed_executions: u64,
    /// Average duration in milliseconds.
    pub average_duration_ms: f64,
    /// Average retry count.
    pub average_retry_count: f64,
}

impl StepMetricsSummary {
    /// Create a new step metrics summary.
    #[must_use]
    pub fn new(engine: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            engine: engine.into(),
            action: action.into(),
            ..Default::default()
        }
    }

    /// Record a step execution.
    pub fn record(&mut self, success: bool, duration_ms: u64, retry_count: u32) {
        self.total_executions += 1;
        if success {
            self.successful_executions += 1;
        } else {
            self.failed_executions += 1;
        }

        // Update running averages
        let n = self.total_executions as f64;
        self.average_duration_ms =
            self.average_duration_ms * (n - 1.0) / n + duration_ms as f64 / n;
        self.average_retry_count =
            self.average_retry_count * (n - 1.0) / n + retry_count as f64 / n;
    }
}

/// System metrics summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetricsSummary {
    /// Total metrics count.
    pub total_metrics: usize,
    /// Workflow metrics.
    pub workflow_metrics: WorkflowMetricsOverview,
    /// Engine metrics.
    pub engine_metrics: EngineMetricsOverview,
    /// System resource metrics.
    pub system_metrics: ResourceMetrics,
}

/// Overview of workflow metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowMetricsOverview {
    /// Total workflow executions.
    pub total_executions: u64,
    /// Successful executions.
    pub successful_executions: u64,
    /// Failed executions.
    pub failed_executions: u64,
    /// Currently active executions.
    pub active_executions: u64,
}

/// Overview of engine metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineMetricsOverview {
    /// Total engine calls.
    pub total_calls: u64,
    /// Successful calls.
    pub successful_calls: u64,
    /// Failed calls.
    pub failed_calls: u64,
}

/// System resource metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetrics {
    /// HTTP requests count.
    pub http_requests: u64,
    /// Active timers count.
    pub active_timers: usize,
}

/// Get current timestamp in milliseconds.
fn current_timestamp_ms() -> u64 {
    nexcore_chrono::DateTime::now().timestamp_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_counter() {
        let metric =
            Metric::counter("test_counter", "A test counter", 42.0).with_label("env", "test");

        assert_eq!(metric.name, "test_counter");
        assert_eq!(metric.metric_type, MetricType::Counter);
        assert_eq!(metric.value, 42.0);
        assert_eq!(metric.labels.get("env"), Some(&"test".to_string()));
    }

    #[test]
    fn test_metric_prometheus_format() {
        let metric = Metric::gauge("memory_usage", "Memory usage in bytes", 1024.0)
            .with_label("type", "heap");

        let prometheus = metric.to_prometheus();
        assert!(prometheus.contains("# HELP memory_usage"));
        assert!(prometheus.contains("type=\"heap\""));
    }

    #[test]
    fn test_workflow_metrics_summary() {
        let mut summary = WorkflowMetricsSummary::new("test-workflow");

        summary.record_success(100);
        summary.record_success(200);
        summary.record_failure(150, "TimeoutError");

        assert_eq!(summary.total_executions, 3);
        assert_eq!(summary.successful_executions, 2);
        assert_eq!(summary.failed_executions, 1);
        assert_eq!(summary.min_duration_ms, 100);
        assert_eq!(summary.max_duration_ms, 200);
        assert!((summary.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_step_metrics_summary() {
        let mut step = StepMetricsSummary::new("content-engine", "create");

        step.record(true, 100, 0);
        step.record(false, 200, 2);

        assert_eq!(step.total_executions, 2);
        assert_eq!(step.successful_executions, 1);
        assert_eq!(step.failed_executions, 1);
        assert_eq!(step.average_duration_ms, 150.0);
        assert_eq!(step.average_retry_count, 1.0);
    }
}
