//! # Experiment Framework: Metrics and Measures for Hook-Skill Molecules
//!
//! This module provides an experimental measurement framework for tracking
//! the efficacy of hook-skill bonding and primitive extraction.
//!
//! ## Metrics Captured
//!
//! - **Reaction Time**: How long hooks take to execute
//! - **Bond Strength Decay**: How bond effectiveness degrades over time
//! - **Cascade Depth**: How many molecules activate in chain reactions
//! - **Extraction Yield**: Ratio of primitives found to expected
//! - **Decomposition Accuracy**: How well primitives match T1/T2/T3 tiers
//!
//! ## Experiment Types
//!
//! - **A/B Testing**: Compare hook configurations
//! - **Time Series**: Track metrics over sessions
//! - **Stress Testing**: Push bonding limits

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Unique experiment identifier
pub type ExperimentId = String;

/// A single measurement point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    /// Timestamp (unix millis)
    pub timestamp: u64,
    /// Metric name
    pub metric: String,
    /// Measured value
    pub value: f64,
    /// Unit of measurement
    pub unit: String,
    /// Associated tags
    pub tags: HashMap<String, String>,
}

impl Measurement {
    /// Create a new measurement
    pub fn new(metric: &str, value: f64) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            metric: metric.to_string(),
            value,
            unit: "unit".to_string(),
            tags: HashMap::new(),
        }
    }

    /// Set unit
    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = unit.to_string();
        self
    }

    /// Add tag
    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }
}

/// Experiment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentStatus {
    /// Not yet started
    Pending,
    /// Currently running
    Running,
    /// Paused
    Paused,
    /// Completed successfully
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

/// Experiment type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentType {
    /// A/B test between two configurations
    ABTest { control: String, variant: String },
    /// Time series tracking
    TimeSeries {
        interval_secs: u64,
        duration_secs: u64,
    },
    /// Stress test
    StressTest { load_factor: f64, ramp_up_secs: u64 },
    /// Primitive extraction validation
    PrimitiveExtraction {
        domain: String,
        expected_t1: usize,
        expected_t2: usize,
        expected_t3: usize,
    },
    /// Bond strength analysis
    BondAnalysis { molecules: Vec<String> },
    /// Custom experiment
    Custom {
        name: String,
        parameters: HashMap<String, String>,
    },
}

/// Hypothesis for the experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    /// Hypothesis statement
    pub statement: String,
    /// Expected outcome
    pub expected: String,
    /// Null hypothesis (what we're disproving)
    pub null_hypothesis: String,
    /// Confidence level required (0.0-1.0)
    pub confidence_required: f64,
}

impl Hypothesis {
    /// Create a new hypothesis
    pub fn new(statement: &str) -> Self {
        Self {
            statement: statement.to_string(),
            expected: String::new(),
            null_hypothesis: String::new(),
            confidence_required: 0.95,
        }
    }

    /// Set expected outcome
    pub fn expecting(mut self, expected: &str) -> Self {
        self.expected = expected.to_string();
        self
    }

    /// Set null hypothesis
    pub fn null(mut self, null: &str) -> Self {
        self.null_hypothesis = null.to_string();
        self
    }

    /// Set confidence level
    pub fn with_confidence(mut self, level: f64) -> Self {
        self.confidence_required = level.clamp(0.0, 1.0);
        self
    }
}

/// An experiment definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    /// Unique ID
    pub id: ExperimentId,
    /// Experiment name
    pub name: String,
    /// Description
    pub description: String,
    /// Experiment type
    pub experiment_type: ExperimentType,
    /// Hypothesis being tested
    pub hypothesis: Option<Hypothesis>,
    /// Status
    pub status: ExperimentStatus,
    /// Start time (unix millis)
    pub started_at: Option<u64>,
    /// End time (unix millis)
    pub ended_at: Option<u64>,
    /// Collected measurements
    pub measurements: Vec<Measurement>,
    /// Derived metrics (computed from measurements)
    pub derived_metrics: HashMap<String, f64>,
    /// Configuration parameters
    pub config: HashMap<String, String>,
    /// Results summary
    pub results: Option<ExperimentResults>,
}

/// Experiment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResults {
    /// Was the hypothesis confirmed?
    pub hypothesis_confirmed: Option<bool>,
    /// Confidence level achieved
    pub confidence: f64,
    /// P-value (if statistical test performed)
    pub p_value: Option<f64>,
    /// Summary statistics
    pub statistics: HashMap<String, f64>,
    /// Key findings
    pub findings: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

impl Experiment {
    /// Create a new experiment
    pub fn new(name: &str, experiment_type: ExperimentType) -> Self {
        Self {
            id: format!(
                "exp-{}-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0),
                std::process::id()
            ),
            name: name.to_string(),
            description: String::new(),
            experiment_type,
            hypothesis: None,
            status: ExperimentStatus::Pending,
            started_at: None,
            ended_at: None,
            measurements: Vec::new(),
            derived_metrics: HashMap::new(),
            config: HashMap::new(),
            results: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Set hypothesis
    pub fn with_hypothesis(mut self, hypothesis: Hypothesis) -> Self {
        self.hypothesis = Some(hypothesis);
        self
    }

    /// Add configuration parameter
    pub fn with_config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.to_string(), value.to_string());
        self
    }

    /// Start the experiment
    pub fn start(&mut self) {
        self.status = ExperimentStatus::Running;
        self.started_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        );
    }

    /// Record a measurement
    pub fn record(&mut self, measurement: Measurement) {
        self.measurements.push(measurement);
    }

    /// Record a simple metric
    pub fn record_metric(&mut self, name: &str, value: f64) {
        self.record(Measurement::new(name, value));
    }

    /// End the experiment
    pub fn end(&mut self, status: ExperimentStatus) {
        self.status = status;
        self.ended_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        );
        self.compute_derived_metrics();
    }

    /// Compute derived metrics from measurements
    fn compute_derived_metrics(&mut self) {
        // Group measurements by metric name
        let mut grouped: HashMap<String, Vec<f64>> = HashMap::new();
        for m in &self.measurements {
            grouped.entry(m.metric.clone()).or_default().push(m.value);
        }

        // Compute statistics for each metric
        for (metric, values) in grouped {
            if values.is_empty() {
                continue;
            }

            let n = values.len() as f64;
            let sum: f64 = values.iter().sum();
            let mean = sum / n;

            let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
            let std_dev = variance.sqrt();

            let min = values.iter().copied().fold(f64::INFINITY, f64::min);
            let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

            self.derived_metrics
                .insert(format!("{}_mean", metric), mean);
            self.derived_metrics
                .insert(format!("{}_std", metric), std_dev);
            self.derived_metrics.insert(format!("{}_min", metric), min);
            self.derived_metrics.insert(format!("{}_max", metric), max);
            self.derived_metrics.insert(format!("{}_count", metric), n);
        }
    }

    /// Duration of the experiment
    pub fn duration(&self) -> Option<Duration> {
        match (self.started_at, self.ended_at) {
            (Some(start), Some(end)) => Some(Duration::from_millis(end - start)),
            _ => None,
        }
    }

    /// Finalize with results
    pub fn finalize(&mut self, results: ExperimentResults) {
        self.results = Some(results);
        if self.status == ExperimentStatus::Running {
            self.end(ExperimentStatus::Completed);
        }
    }
}

/// Timer for measuring durations
pub struct Timer {
    start: Instant,
    label: String,
}

impl Timer {
    /// Start a new timer
    pub fn start(label: &str) -> Self {
        Self {
            start: Instant::now(),
            label: label.to_string(),
        }
    }

    /// Stop and get duration in milliseconds
    pub fn stop(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    /// Stop and create measurement
    pub fn to_measurement(&self) -> Measurement {
        Measurement::new(&self.label, self.stop()).with_unit("ms")
    }
}

/// Primitive extraction metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractionMetrics {
    /// Total primitives found
    pub total_found: usize,
    /// T1 (universal) primitives
    pub t1_count: usize,
    /// T2P (cross-domain primitive) count
    pub t2p_count: usize,
    /// T2C (cross-domain composite) count
    pub t2c_count: usize,
    /// T3 (domain-specific) count
    pub t3_count: usize,
    /// Extraction time in ms
    pub extraction_time_ms: f64,
    /// Dependency graph depth
    pub graph_depth: usize,
    /// Transfer confidence average
    pub avg_transfer_confidence: f64,
}

impl ExtractionMetrics {
    /// Calculate extraction yield
    pub fn yield_ratio(&self, expected_total: usize) -> f64 {
        if expected_total == 0 {
            return 0.0;
        }
        self.total_found as f64 / expected_total as f64
    }

    /// Calculate tier distribution
    pub fn tier_distribution(&self) -> HashMap<String, f64> {
        let total = self.total_found as f64;
        if total == 0.0 {
            return HashMap::new();
        }

        let mut dist = HashMap::new();
        dist.insert("T1".to_string(), self.t1_count as f64 / total);
        dist.insert("T2P".to_string(), self.t2p_count as f64 / total);
        dist.insert("T2C".to_string(), self.t2c_count as f64 / total);
        dist.insert("T3".to_string(), self.t3_count as f64 / total);
        dist
    }

    /// Convert to measurements
    pub fn to_measurements(&self, domain: &str) -> Vec<Measurement> {
        vec![
            Measurement::new("extraction.total", self.total_found as f64)
                .with_tag("domain", domain),
            Measurement::new("extraction.t1_count", self.t1_count as f64)
                .with_tag("domain", domain),
            Measurement::new("extraction.t2p_count", self.t2p_count as f64)
                .with_tag("domain", domain),
            Measurement::new("extraction.t2c_count", self.t2c_count as f64)
                .with_tag("domain", domain),
            Measurement::new("extraction.t3_count", self.t3_count as f64)
                .with_tag("domain", domain),
            Measurement::new("extraction.time_ms", self.extraction_time_ms)
                .with_unit("ms")
                .with_tag("domain", domain),
            Measurement::new("extraction.graph_depth", self.graph_depth as f64)
                .with_tag("domain", domain),
            Measurement::new(
                "extraction.transfer_confidence",
                self.avg_transfer_confidence,
            )
            .with_tag("domain", domain),
        ]
    }
}

/// Bond strength metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BondMetrics {
    /// Average bond strength
    pub avg_strength: f64,
    /// Bond count by type
    pub bond_counts: HashMap<String, usize>,
    /// Broken bonds count
    pub broken_count: usize,
    /// Cascade depth achieved
    pub cascade_depth: usize,
    /// Reaction success rate
    pub reaction_success_rate: f64,
}

impl BondMetrics {
    /// Convert to measurements
    pub fn to_measurements(&self, molecule: &str) -> Vec<Measurement> {
        let mut measurements = vec![
            Measurement::new("bond.avg_strength", self.avg_strength).with_tag("molecule", molecule),
            Measurement::new("bond.broken_count", self.broken_count as f64)
                .with_tag("molecule", molecule),
            Measurement::new("bond.cascade_depth", self.cascade_depth as f64)
                .with_tag("molecule", molecule),
            Measurement::new("bond.reaction_success_rate", self.reaction_success_rate)
                .with_tag("molecule", molecule),
        ];

        for (bond_type, count) in &self.bond_counts {
            measurements.push(
                Measurement::new(&format!("bond.count.{}", bond_type), *count as f64)
                    .with_tag("molecule", molecule),
            );
        }

        measurements
    }
}

/// Experiment registry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentRegistry {
    /// All experiments
    pub experiments: HashMap<ExperimentId, Experiment>,
    /// Active experiment IDs
    pub active: Vec<ExperimentId>,
}

impl ExperimentRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an experiment
    pub fn register(&mut self, experiment: Experiment) -> ExperimentId {
        let id = experiment.id.clone();
        self.experiments.insert(id.clone(), experiment);
        id
    }

    /// Start an experiment
    pub fn start(&mut self, id: &str) -> Option<&mut Experiment> {
        if let Some(exp) = self.experiments.get_mut(id) {
            exp.start();
            self.active.push(id.to_string());
            Some(exp)
        } else {
            None
        }
    }

    /// Get active experiments
    pub fn active_experiments(&self) -> Vec<&Experiment> {
        self.active
            .iter()
            .filter_map(|id| self.experiments.get(id))
            .filter(|e| e.status == ExperimentStatus::Running)
            .collect()
    }

    /// Record to all active experiments
    pub fn record_to_active(&mut self, measurement: Measurement) {
        for id in &self.active.clone() {
            if let Some(exp) = self.experiments.get_mut(id) {
                if exp.status == ExperimentStatus::Running {
                    exp.record(measurement.clone());
                }
            }
        }
    }

    /// Get completed experiments
    pub fn completed(&self) -> Vec<&Experiment> {
        self.experiments
            .values()
            .filter(|e| e.status == ExperimentStatus::Completed)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measurement_creation() {
        let m = Measurement::new("test_metric", 42.0)
            .with_unit("ms")
            .with_tag("hook", "pretool_panic");

        assert_eq!(m.metric, "test_metric");
        assert_eq!(m.value, 42.0);
        assert_eq!(m.unit, "ms");
        assert_eq!(m.tags.get("hook"), Some(&"pretool_panic".to_string()));
    }

    #[test]
    fn test_experiment_lifecycle() {
        let mut exp = Experiment::new(
            "Hook Performance",
            ExperimentType::TimeSeries {
                interval_secs: 60,
                duration_secs: 3600,
            },
        )
        .with_description("Test hook execution times")
        .with_hypothesis(
            Hypothesis::new("Hooks execute under 10ms on average")
                .expecting("mean < 10ms")
                .with_confidence(0.95),
        );

        assert_eq!(exp.status, ExperimentStatus::Pending);

        exp.start();
        assert_eq!(exp.status, ExperimentStatus::Running);
        assert!(exp.started_at.is_some());

        exp.record_metric("execution_time", 5.0);
        exp.record_metric("execution_time", 8.0);
        exp.record_metric("execution_time", 3.0);

        exp.end(ExperimentStatus::Completed);
        assert_eq!(exp.status, ExperimentStatus::Completed);
        assert!(exp.ended_at.is_some());

        // Check derived metrics
        assert!(exp.derived_metrics.contains_key("execution_time_mean"));
        let mean = exp.derived_metrics.get("execution_time_mean").unwrap();
        assert!((mean - 5.33).abs() < 0.1);
    }

    #[test]
    fn test_extraction_metrics() {
        let metrics = ExtractionMetrics {
            total_found: 10,
            t1_count: 3,
            t2p_count: 2,
            t2c_count: 2,
            t3_count: 3,
            extraction_time_ms: 150.0,
            graph_depth: 4,
            avg_transfer_confidence: 0.85,
        };

        assert_eq!(metrics.yield_ratio(10), 1.0);
        assert_eq!(metrics.yield_ratio(20), 0.5);

        let dist = metrics.tier_distribution();
        assert_eq!(dist.get("T1"), Some(&0.3));
    }

    #[test]
    fn test_timer() {
        let timer = Timer::start("test_op");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.stop();
        assert!(elapsed >= 10.0);

        let measurement = timer.to_measurement();
        assert_eq!(measurement.metric, "test_op");
        assert_eq!(measurement.unit, "ms");
    }

    #[test]
    fn test_experiment_registry() {
        let mut registry = ExperimentRegistry::new();

        let exp1 = Experiment::new(
            "Exp1",
            ExperimentType::ABTest {
                control: "default".to_string(),
                variant: "new".to_string(),
            },
        );

        let id = registry.register(exp1);
        registry.start(&id);

        assert_eq!(registry.active_experiments().len(), 1);

        registry.record_to_active(Measurement::new("test", 1.0));

        let exp = registry.experiments.get(&id).unwrap();
        assert_eq!(exp.measurements.len(), 1);
    }
}
