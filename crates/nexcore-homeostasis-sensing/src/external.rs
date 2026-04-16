//! External threat sensors (PAMPs — pathogen-associated molecular patterns).
//!
//! Ports `homeostasis_machine/sensing/external.py` to Rust 1:1.
//!
//! In biology, Pattern Recognition Receptors (PRRs) detect external signatures
//! of infection. In software, external threat sensors detect anomalies arriving
//! from outside the system boundary: error rates from APIs, latency spikes from
//! databases, traffic anomalies from load balancers, queue backpressure.
//!
//! # Structure
//!
//! | Rust type | Python equivalent |
//! |-----------|-------------------|
//! | [`ThreatPattern`] | `ThreatPattern` (abstract base) |
//! | [`ThresholdPattern`] | `ThresholdPattern` |
//! | [`RateOfChangePattern`] | `RateOfChangePattern` |
//! | [`StatisticalAnomalyPattern`] | `StatisticalAnomalyPattern` |
//! | [`ExternalThreatSensor`] | `ExternalThreatSensor` |
//! | [`ErrorRateSensor`] | `ErrorRateSensor` |
//! | [`LatencySensor`] | `LatencySensor` |
//! | [`TrafficSensor`] | `TrafficSensor` |
//! | [`QueueDepthSensor`] | `QueueDepthSensor` |
//!
//! # Example
//!
//! ```rust
//! use nexcore_homeostasis_sensing::external::{ErrorRateSensor, ThresholdPattern};
//!
//! let mut sensor = ErrorRateSensor::new(0.01, 0.05);
//! sensor.record(0.03);
//! let assessment = sensor.assess(0.03);
//! assert!(assessment.is_anomalous);
//! ```

use crate::anomaly::{AnomalyAssessment, AnomalyAssessor};
use serde::{Deserialize, Serialize};

// ─── ThreatPattern trait ─────────────────────────────────────────────────────

/// A recognized threat pattern — the Rust analog of Python's `ThreatPattern`
/// abstract base class.
///
/// Each implementation defines what "looks like a threat" for a particular
/// anomaly type. The `matches` method receives the current value and an
/// immutable view of the recent history slice; it returns `Some(assessment)`
/// when the pattern fires, `None` otherwise.
///
/// The `severity_multiplier` field on each concrete type scales the base
/// severity before it is clamped to `[0.0, 1.0]`.
pub trait ThreatPattern: Send + Sync + std::fmt::Debug {
    /// Return a human-readable name for logging and introspection.
    fn name(&self) -> &str;

    /// Attempt to match the pattern against `value` given `history`.
    ///
    /// Returns `Some(assessment)` if the pattern fires, `None` if it does not.
    /// History is ordered oldest-first; `history.last()` is the most recent
    /// recorded value (which equals `value` when called after `record`).
    fn matches(&self, value: f64, history: &[f64]) -> Option<AnomalyAssessment>;
}

// ─── ThresholdPattern ────────────────────────────────────────────────────────

/// A pattern that fires when the current value meets or exceeds a fixed
/// threshold.
///
/// Maps 1:1 to Python's `ThresholdPattern`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThresholdPattern {
    /// Name for logging.
    name: String,
    /// Absolute threshold; fires when `value >= threshold`.
    pub threshold: f64,
    /// Scales base severity `0.5 × severity_multiplier`, clamped to `[0,1]`.
    pub severity_multiplier: f64,
}

impl ThresholdPattern {
    /// Create a new threshold pattern.
    pub fn new(name: impl Into<String>, threshold: f64, severity_multiplier: f64) -> Self {
        Self {
            name: name.into(),
            threshold,
            severity_multiplier,
        }
    }
}

impl ThreatPattern for ThresholdPattern {
    fn name(&self) -> &str {
        &self.name
    }

    fn matches(&self, value: f64, _history: &[f64]) -> Option<AnomalyAssessment> {
        if value >= self.threshold {
            let severity = (0.5 * self.severity_multiplier).clamp(0.0, 1.0);
            // confidence scales with how far over the threshold we are; floor 0.5
            let confidence = 0.5f64
                .max(if self.threshold > 0.0 {
                    (value / self.threshold - 1.0).min(0.45) + 0.5
                } else {
                    0.5
                })
                .min(0.95);
            Some(AnomalyAssessment::anomalous(severity, confidence))
        } else {
            None
        }
    }
}

// ─── RateOfChangePattern ─────────────────────────────────────────────────────

/// A pattern that fires on rapid changes within a sliding window.
///
/// Maps 1:1 to Python's `RateOfChangePattern`. The absolute difference between
/// the oldest and newest value in the window is compared to `change_threshold`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RateOfChangePattern {
    /// Name for logging.
    name: String,
    /// Minimum absolute change across the window to trigger.
    pub change_threshold: f64,
    /// How many recent history entries to examine (oldest-first slice tail).
    pub window_size: usize,
    /// Scales base severity `0.5 × severity_multiplier`, clamped to `[0,1]`.
    pub severity_multiplier: f64,
}

impl RateOfChangePattern {
    /// Create a new rate-of-change pattern.
    pub fn new(
        name: impl Into<String>,
        change_threshold: f64,
        window_size: usize,
        severity_multiplier: f64,
    ) -> Self {
        Self {
            name: name.into(),
            change_threshold,
            window_size: window_size.max(2),
            severity_multiplier,
        }
    }
}

impl ThreatPattern for RateOfChangePattern {
    fn name(&self) -> &str {
        &self.name
    }

    fn matches(&self, _value: f64, history: &[f64]) -> Option<AnomalyAssessment> {
        // Take the tail slice matching Python's `history[-self.window_size:]`
        let window = if history.len() > self.window_size {
            &history[history.len() - self.window_size..]
        } else {
            history
        };

        if window.len() < 2 {
            return None;
        }

        // Python: `change = abs(history[-1] - history[0])` on the window slice
        let change = (window[window.len() - 1] - window[0]).abs();
        if change >= self.change_threshold {
            let severity = (0.5 * self.severity_multiplier).clamp(0.0, 1.0);
            Some(AnomalyAssessment::anomalous(severity, 0.65))
        } else {
            None
        }
    }
}

// ─── StatisticalAnomalyPattern ───────────────────────────────────────────────

/// A pattern based on statistical deviation (z-score) from the historical mean.
///
/// Maps 1:1 to Python's `StatisticalAnomalyPattern`. Requires at least 10
/// history points before it can fire; returns `None` when history is sparse.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatisticalAnomalyPattern {
    /// Name for logging.
    name: String,
    /// Z-score threshold; fires when `|z| >= z_threshold`.
    pub z_threshold: f64,
    /// Scales base severity `0.5 × severity_multiplier`, clamped to `[0,1]`.
    pub severity_multiplier: f64,
}

impl StatisticalAnomalyPattern {
    /// Create a new statistical anomaly pattern.
    pub fn new(name: impl Into<String>, z_threshold: f64, severity_multiplier: f64) -> Self {
        Self {
            name: name.into(),
            z_threshold,
            severity_multiplier,
        }
    }

    /// Compute sample mean of the slice. Returns `None` if empty.
    fn mean(values: &[f64]) -> Option<f64> {
        if values.is_empty() {
            return None;
        }
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }

    /// Compute sample standard deviation (Bessel-corrected, matching Python's
    /// `statistics.stdev`). Returns `None` if fewer than 2 samples.
    fn stdev(values: &[f64]) -> Option<f64> {
        if values.len() < 2 {
            return None;
        }
        let mean = Self::mean(values)?;
        let variance =
            values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
        Some(variance.sqrt())
    }
}

impl ThreatPattern for StatisticalAnomalyPattern {
    fn name(&self) -> &str {
        &self.name
    }

    fn matches(&self, value: f64, history: &[f64]) -> Option<AnomalyAssessment> {
        // Mirrors Python: requires at least 10 history points
        if history.len() < 10 {
            return None;
        }

        let mean = Self::mean(history)?;
        let stdev = Self::stdev(history)?;

        // Guard: stdev == 0 means no variation — Python returns False
        if stdev < f64::EPSILON {
            return None;
        }

        let z_score = (value - mean).abs() / stdev;
        if z_score >= self.z_threshold {
            let severity = (0.5 * self.severity_multiplier).clamp(0.0, 1.0);
            // Confidence scales with how many sigmas beyond threshold
            let confidence = (0.5 + (z_score - self.z_threshold) * 0.1).min(0.95);
            Some(AnomalyAssessment::anomalous(severity, confidence))
        } else {
            None
        }
    }
}

// ─── ExternalThreatSensor ────────────────────────────────────────────────────

/// A sensor that detects external threats using a registry of [`ThreatPattern`]s.
///
/// This is the software analog of a PRR (Pattern Recognition Receptor).
///
/// Maps 1:1 to Python's `ExternalThreatSensor`. The Python class inherits from
/// `Sensor` and delegates async I/O; in Rust the async bridge lives elsewhere
/// (see `nexcore-homeostasis`). This type handles the synchronous assessment
/// layer only.
///
/// # History cap
///
/// History is capped at `history_size` entries (default 100, matching Python's
/// `deque(maxlen=100)`). Oldest entries are evicted when the cap is reached.
#[derive(Debug)]
pub struct ExternalThreatSensor {
    /// Sensor name (matches Python `self.name`).
    name: String,
    /// Registered threat patterns (the PRR recognition library).
    patterns: Vec<Box<dyn ThreatPattern>>,
    /// Sliding window of recent values, oldest-first.
    history: Vec<f64>,
    /// Maximum number of history entries to retain.
    history_size: usize,
    /// Fallback assessor used when no pattern matches (mirrors Python's
    /// `super()._assess_anomaly(value)`).
    assessor: AnomalyAssessor,
    /// Patterns that matched on the most recent [`assess`] call.
    last_matched: Vec<String>,
}

impl ExternalThreatSensor {
    /// Construct a new sensor with the given patterns and configuration.
    ///
    /// `history_size` defaults to 100 when set to `0` to match Python's
    /// `deque(maxlen=100)`.
    pub fn new(
        name: impl Into<String>,
        patterns: Vec<Box<dyn ThreatPattern>>,
        baseline_value: f64,
        warning_threshold: Option<f64>,
        critical_threshold: Option<f64>,
        history_size: usize,
    ) -> Self {
        let cap = if history_size == 0 { 100 } else { history_size };
        Self {
            name: name.into(),
            patterns,
            history: Vec::with_capacity(cap),
            history_size: cap,
            assessor: AnomalyAssessor::new(baseline_value, warning_threshold, critical_threshold),
            last_matched: Vec::new(),
        }
    }

    /// Sensor name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add a threat pattern to the recognition library.
    pub fn add_pattern(&mut self, pattern: Box<dyn ThreatPattern>) {
        self.patterns.push(pattern);
    }

    /// Record a new value into history (evicting the oldest if at cap).
    ///
    /// Call this before [`assess`] so the pattern matchers see the latest value
    /// in the history slice — mirroring Python's `_read_raw` which appends
    /// before `_assess_anomaly` is called.
    pub fn record(&mut self, value: f64) {
        if self.history.len() >= self.history_size {
            self.history.remove(0);
        }
        self.history.push(value);
    }

    /// Assess a value against all registered patterns.
    ///
    /// Mirrors Python's `_assess_anomaly`:
    /// - Runs all patterns; collects matches.
    /// - If any match: `max_severity` across matches, confidence = `min(0.95, 0.5 + 0.1 × count)`.
    /// - If no match: delegates to the threshold-based [`AnomalyAssessor`].
    ///
    /// The caller is responsible for calling [`record`] first if history
    /// should include the current value.
    pub fn assess(&mut self, value: f64) -> AnomalyAssessment {
        let mut matched_names: Vec<String> = Vec::new();
        let mut max_severity: f64 = 0.0;

        for pattern in &self.patterns {
            if let Some(assessment) = pattern.matches(value, &self.history) {
                matched_names.push(pattern.name().to_owned());
                max_severity = max_severity.max(assessment.severity);
            }
        }

        self.last_matched = matched_names;

        if !self.last_matched.is_empty() {
            let count = self.last_matched.len() as f64;
            // Python: confidence = min(0.95, 0.5 + 0.1 * len(matched))
            let confidence = (0.5 + 0.1 * count).min(0.95);
            AnomalyAssessment::anomalous(max_severity, confidence)
        } else {
            // Fall back to threshold-based assessor (mirrors Python `super()._assess_anomaly`)
            self.assessor.assess(value)
        }
    }

    /// Names of the patterns that fired on the most recent [`assess`] call.
    pub fn matched_pattern_names(&self) -> &[String] {
        &self.last_matched
    }

    /// Read-only view of the current value history.
    pub fn history(&self) -> &[f64] {
        &self.history
    }
}

// ─── Pre-configured sensor constructors ──────────────────────────────────────

/// Pre-configured sensor for monitoring error rates.
///
/// Maps 1:1 to Python's `ErrorRateSensor`. Baseline: 0.001 (0.1%).
///
/// Patterns registered:
/// - `elevated_errors` — threshold at `warning_threshold`, multiplier 0.5
/// - `critical_errors` — threshold at `critical_threshold`, multiplier 1.0
/// - `error_spike` — rate-of-change `>= warning_threshold`, multiplier 0.8
/// - `anomalous_errors` — statistical z-score 3.0, multiplier 0.7
#[derive(Debug)]
pub struct ErrorRateSensor {
    inner: ExternalThreatSensor,
    /// Most-recently recorded value and its assessment.
    last_reading: Option<(f64, AnomalyAssessment)>,
}

impl ErrorRateSensor {
    /// Create with `warning_threshold` (default 1%) and `critical_threshold`
    /// (default 5%), matching Python's defaults.
    pub fn new(warning_threshold: f64, critical_threshold: f64) -> Self {
        let patterns: Vec<Box<dyn ThreatPattern>> = vec![
            Box::new(ThresholdPattern::new(
                "elevated_errors",
                warning_threshold,
                0.5,
            )),
            Box::new(ThresholdPattern::new(
                "critical_errors",
                critical_threshold,
                1.0,
            )),
            Box::new(RateOfChangePattern::new(
                "error_spike",
                warning_threshold,
                5,
                0.8,
            )),
            Box::new(StatisticalAnomalyPattern::new("anomalous_errors", 3.0, 0.7)),
        ];
        Self {
            inner: ExternalThreatSensor::new(
                "error_rate",
                patterns,
                0.001,
                Some(warning_threshold),
                Some(critical_threshold),
                100,
            ),
            last_reading: None,
        }
    }

    /// Sensor name.
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Record a value into history and update the stored last reading.
    pub fn record(&mut self, value: f64) {
        self.inner.record(value);
        let assessment = self.inner.assess(value);
        self.last_reading = Some((value, assessment));
    }

    /// Assess a value (call [`record`] first to include it in history).
    pub fn assess(&mut self, value: f64) -> AnomalyAssessment {
        self.inner.assess(value)
    }

    /// Most-recently recorded value and its assessment, or `None` if
    /// [`record`](Self::record) has not been called yet.
    pub fn last_reading(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading
    }

    /// Inner sensor for advanced access.
    pub fn inner(&self) -> &ExternalThreatSensor {
        &self.inner
    }

    /// Mutable inner sensor.
    pub fn inner_mut(&mut self) -> &mut ExternalThreatSensor {
        &mut self.inner
    }
}

/// Pre-configured sensor for monitoring latency (in milliseconds).
///
/// Maps 1:1 to Python's `LatencySensor`.
///
/// Patterns registered:
/// - `slow_response` — threshold at `baseline_ms × warning_multiplier`, multiplier 0.5
/// - `critical_latency` — threshold at `baseline_ms × critical_multiplier`, multiplier 1.0
/// - `latency_spike` — rate-of-change `>= baseline_ms`, multiplier 0.6
#[derive(Debug)]
pub struct LatencySensor {
    inner: ExternalThreatSensor,
    /// Most-recently recorded value and its assessment.
    last_reading: Option<(f64, AnomalyAssessment)>,
}

impl LatencySensor {
    /// Create with a `baseline_ms` (default 100ms) and multipliers (default 2× warning, 5× critical).
    pub fn new(baseline_ms: f64, warning_multiplier: f64, critical_multiplier: f64) -> Self {
        let warning = baseline_ms * warning_multiplier;
        let critical = baseline_ms * critical_multiplier;

        let patterns: Vec<Box<dyn ThreatPattern>> = vec![
            Box::new(ThresholdPattern::new("slow_response", warning, 0.5)),
            Box::new(ThresholdPattern::new("critical_latency", critical, 1.0)),
            Box::new(RateOfChangePattern::new(
                "latency_spike",
                baseline_ms,
                5,
                0.6,
            )),
        ];
        Self {
            inner: ExternalThreatSensor::new(
                "latency",
                patterns,
                baseline_ms,
                Some(warning),
                Some(critical),
                100,
            ),
            last_reading: None,
        }
    }

    /// Sensor name.
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Record a value into history and update the stored last reading.
    pub fn record(&mut self, value: f64) {
        self.inner.record(value);
        let assessment = self.inner.assess(value);
        self.last_reading = Some((value, assessment));
    }

    /// Assess a value (call [`record`] first to include it in history).
    pub fn assess(&mut self, value: f64) -> AnomalyAssessment {
        self.inner.assess(value)
    }

    /// Most-recently recorded value and its assessment, or `None` if
    /// [`record`](Self::record) has not been called yet.
    pub fn last_reading(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading
    }

    /// Inner sensor for advanced access.
    pub fn inner(&self) -> &ExternalThreatSensor {
        &self.inner
    }
}

/// Pre-configured sensor for monitoring traffic anomalies (requests per second).
///
/// Maps 1:1 to Python's `TrafficSensor`.
///
/// Patterns registered:
/// - `traffic_spike` — threshold at `baseline_rps × spike_multiplier`, multiplier 0.7
/// - `rapid_increase` — rate-of-change `>= baseline_rps × 0.5`, multiplier 0.6
/// - `traffic_anomaly` — statistical z-score 2.5, multiplier 0.5
#[derive(Debug)]
pub struct TrafficSensor {
    inner: ExternalThreatSensor,
    /// Most-recently recorded value and its assessment.
    last_reading: Option<(f64, AnomalyAssessment)>,
}

impl TrafficSensor {
    /// Create with `baseline_rps` (default 1000.0) and `spike_multiplier` (default 3.0).
    pub fn new(baseline_rps: f64, spike_multiplier: f64) -> Self {
        let spike_threshold = baseline_rps * spike_multiplier;

        let patterns: Vec<Box<dyn ThreatPattern>> = vec![
            Box::new(ThresholdPattern::new("traffic_spike", spike_threshold, 0.7)),
            Box::new(RateOfChangePattern::new(
                "rapid_increase",
                baseline_rps * 0.5,
                5,
                0.6,
            )),
            Box::new(StatisticalAnomalyPattern::new("traffic_anomaly", 2.5, 0.5)),
        ];
        Self {
            inner: ExternalThreatSensor::new("traffic", patterns, baseline_rps, None, None, 100),
            last_reading: None,
        }
    }

    /// Sensor name.
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Record a value into history and update the stored last reading.
    pub fn record(&mut self, value: f64) {
        self.inner.record(value);
        let assessment = self.inner.assess(value);
        self.last_reading = Some((value, assessment));
    }

    /// Assess a value (call [`record`] first to include it in history).
    pub fn assess(&mut self, value: f64) -> AnomalyAssessment {
        self.inner.assess(value)
    }

    /// Most-recently recorded value and its assessment, or `None` if
    /// [`record`](Self::record) has not been called yet.
    pub fn last_reading(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading
    }

    /// Inner sensor for advanced access.
    pub fn inner(&self) -> &ExternalThreatSensor {
        &self.inner
    }
}

/// Pre-configured sensor for monitoring queue depths.
///
/// Maps 1:1 to Python's `QueueDepthSensor`.
///
/// Patterns registered:
/// - `queue_building` — threshold at `warning_depth`, multiplier 0.5
/// - `queue_critical` — threshold at `critical_depth`, multiplier 1.0
/// - `queue_growing` — rate-of-change `>= 50`, multiplier 0.6
#[derive(Debug)]
pub struct QueueDepthSensor {
    inner: ExternalThreatSensor,
    /// Most-recently recorded value and its assessment.
    last_reading: Option<(f64, AnomalyAssessment)>,
}

impl QueueDepthSensor {
    /// Create with `warning_depth` (default 100) and `critical_depth` (default 500).
    pub fn new(warning_depth: f64, critical_depth: f64) -> Self {
        let patterns: Vec<Box<dyn ThreatPattern>> = vec![
            Box::new(ThresholdPattern::new("queue_building", warning_depth, 0.5)),
            Box::new(ThresholdPattern::new("queue_critical", critical_depth, 1.0)),
            Box::new(RateOfChangePattern::new("queue_growing", 50.0, 5, 0.6)),
        ];
        Self {
            inner: ExternalThreatSensor::new(
                "queue_depth",
                patterns,
                0.0,
                Some(warning_depth),
                Some(critical_depth),
                100,
            ),
            last_reading: None,
        }
    }

    /// Sensor name.
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Record a value into history and update the stored last reading.
    pub fn record(&mut self, value: f64) {
        self.inner.record(value);
        let assessment = self.inner.assess(value);
        self.last_reading = Some((value, assessment));
    }

    /// Assess a value (call [`record`] first to include it in history).
    pub fn assess(&mut self, value: f64) -> AnomalyAssessment {
        self.inner.assess(value)
    }

    /// Most-recently recorded value and its assessment, or `None` if
    /// [`record`](Self::record) has not been called yet.
    pub fn last_reading(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading
    }

    /// Inner sensor for advanced access.
    pub fn inner(&self) -> &ExternalThreatSensor {
        &self.inner
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── ThresholdPattern ──────────────────────────────────────────────────────

    #[test]
    fn threshold_pattern_fires_at_boundary() {
        let p = ThresholdPattern::new("test", 5.0, 1.0);
        assert!(
            p.matches(5.0, &[]).is_some(),
            "should fire at exact threshold"
        );
        assert!(
            p.matches(4.99, &[]).is_none(),
            "should not fire below threshold"
        );
    }

    #[test]
    fn threshold_pattern_severity_scales_with_multiplier() {
        let half = ThresholdPattern::new("half", 1.0, 0.5);
        let full = ThresholdPattern::new("full", 1.0, 1.0);
        let a_half = half.matches(2.0, &[]).unwrap();
        let a_full = full.matches(2.0, &[]).unwrap();
        assert!(a_half.severity < a_full.severity);
    }

    // ── RateOfChangePattern ───────────────────────────────────────────────────

    #[test]
    fn rate_of_change_fires_on_rapid_increase() {
        let p = RateOfChangePattern::new("spike", 10.0, 5, 1.0);
        // Window of 5; change = 50 - 10 = 40 >= 10 → fires
        let history = [10.0, 20.0, 30.0, 40.0, 50.0];
        assert!(p.matches(50.0, &history).is_some());
    }

    #[test]
    fn rate_of_change_no_fire_on_stable_values() {
        let p = RateOfChangePattern::new("spike", 10.0, 5, 1.0);
        let history = [10.0, 10.1, 9.9, 10.0, 10.05];
        assert!(p.matches(10.05, &history).is_none());
    }

    #[test]
    fn rate_of_change_requires_at_least_two_points() {
        let p = RateOfChangePattern::new("spike", 1.0, 5, 1.0);
        assert!(p.matches(100.0, &[100.0]).is_none());
        assert!(p.matches(100.0, &[]).is_none());
    }

    // ── StatisticalAnomalyPattern ─────────────────────────────────────────────

    #[test]
    fn statistical_pattern_requires_ten_history_points() {
        let p = StatisticalAnomalyPattern::new("stat", 3.0, 1.0);
        let short: Vec<f64> = vec![1.0; 9];
        assert!(
            p.matches(100.0, &short).is_none(),
            "fewer than 10 points → None"
        );
    }

    #[test]
    fn statistical_pattern_fires_on_clear_outlier() {
        let p = StatisticalAnomalyPattern::new("stat", 3.0, 1.0);
        // 10 values around 10.0, then one extreme outlier
        let mut history: Vec<f64> = vec![10.0; 10];
        history.push(100.0);
        assert!(
            p.matches(100.0, &history).is_some(),
            "100.0 is many σ from mean of 10"
        );
    }

    #[test]
    fn statistical_pattern_no_fire_on_normal_value() {
        let p = StatisticalAnomalyPattern::new("stat", 3.0, 1.0);
        let history: Vec<f64> = vec![10.0; 20];
        assert!(
            p.matches(10.0, &history).is_none(),
            "value at mean → no anomaly"
        );
    }

    #[test]
    fn statistical_pattern_no_fire_on_zero_stdev() {
        let p = StatisticalAnomalyPattern::new("stat", 3.0, 1.0);
        // All identical values → stdev = 0 → Python returns False
        let history: Vec<f64> = vec![5.0; 20];
        assert!(p.matches(5.0, &history).is_none());
    }

    // ── ExternalThreatSensor ──────────────────────────────────────────────────

    #[test]
    fn sensor_history_is_capped_at_history_size() {
        let mut sensor = ExternalThreatSensor::new("test", vec![], 0.0, None, None, 3);
        sensor.record(1.0);
        sensor.record(2.0);
        sensor.record(3.0);
        sensor.record(4.0); // evicts 1.0
        assert_eq!(sensor.history(), &[2.0, 3.0, 4.0]);
    }

    #[test]
    fn sensor_no_match_falls_back_to_assessor() {
        // No patterns → falls back to threshold assessor
        let mut sensor = ExternalThreatSensor::new("test", vec![], 1.0, Some(5.0), Some(10.0), 100);
        let result = sensor.assess(8.0);
        // Warning range (5–10) → anomalous via assessor
        assert!(result.is_anomalous);
    }

    #[test]
    fn sensor_matched_pattern_names_populated() {
        let patterns: Vec<Box<dyn ThreatPattern>> =
            vec![Box::new(ThresholdPattern::new("high", 5.0, 1.0))];
        let mut sensor = ExternalThreatSensor::new("test", patterns, 0.0, None, None, 100);
        sensor.assess(10.0);
        assert_eq!(sensor.matched_pattern_names(), &["high"]);
    }

    #[test]
    fn sensor_confidence_scales_with_pattern_count() {
        // Two patterns both fire → confidence > single-pattern confidence
        let patterns: Vec<Box<dyn ThreatPattern>> = vec![
            Box::new(ThresholdPattern::new("p1", 1.0, 0.5)),
            Box::new(ThresholdPattern::new("p2", 1.0, 0.5)),
        ];
        let mut sensor = ExternalThreatSensor::new("test", patterns, 0.0, None, None, 100);
        let result = sensor.assess(5.0);
        // confidence = min(0.95, 0.5 + 0.1 * 2) = 0.70
        assert!(result.is_anomalous);
        assert!(
            (result.confidence - 0.70).abs() < 1e-9,
            "confidence={}",
            result.confidence
        );
    }

    // ── Pre-configured sensors ────────────────────────────────────────────────

    #[test]
    fn error_rate_sensor_fires_above_warning() {
        let mut sensor = ErrorRateSensor::new(0.01, 0.05);
        sensor.record(0.02);
        let result = sensor.assess(0.02);
        assert!(result.is_anomalous, "0.02 > warning 0.01 → anomalous");
    }

    #[test]
    fn error_rate_sensor_normal_below_warning() {
        let mut sensor = ErrorRateSensor::new(0.01, 0.05);
        // baseline=0.001; value must be <2× baseline (0.002) to avoid the
        // ratio-based fallback in AnomalyAssessor. 0.0005 is 0.5× baseline.
        let result = sensor.assess(0.0005);
        assert!(
            !result.is_anomalous,
            "0.0005 < warning 0.01 and < 2× baseline → normal"
        );
    }

    #[test]
    fn latency_sensor_fires_above_warning_multiplier() {
        let mut sensor = LatencySensor::new(100.0, 2.0, 5.0);
        // warning = 200ms, critical = 500ms
        sensor.record(250.0);
        let result = sensor.assess(250.0);
        assert!(result.is_anomalous, "250ms > warning 200ms → anomalous");
    }

    #[test]
    fn latency_sensor_normal_at_baseline() {
        let mut sensor = LatencySensor::new(100.0, 2.0, 5.0);
        // No history — below warning=200ms
        let result = sensor.assess(100.0);
        assert!(!result.is_anomalous, "100ms at baseline → normal");
    }

    #[test]
    fn traffic_sensor_fires_on_spike() {
        let mut sensor = TrafficSensor::new(1000.0, 3.0);
        // spike_threshold = 3000.0
        sensor.record(3500.0);
        let result = sensor.assess(3500.0);
        assert!(result.is_anomalous, "3500 rps > spike 3000 → anomalous");
    }

    #[test]
    fn queue_depth_sensor_fires_at_warning() {
        let mut sensor = QueueDepthSensor::new(100.0, 500.0);
        sensor.record(150.0);
        let result = sensor.assess(150.0);
        assert!(result.is_anomalous, "depth 150 > warning 100 → anomalous");
    }

    #[test]
    fn queue_depth_sensor_normal_when_empty() {
        let mut sensor = QueueDepthSensor::new(100.0, 500.0);
        let result = sensor.assess(0.0);
        // baseline=0.0, value=0 → below warning → normal
        assert!(!result.is_anomalous);
    }
}
