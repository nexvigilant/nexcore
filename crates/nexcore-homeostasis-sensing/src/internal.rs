//! Internal health sensors (DAMPs — Damage-Associated Molecular Patterns).
//!
//! In biology, DAMPs are released by damaged or dying cells, signalling "friendly fire" —
//! damage from the body's own immune response. In software, these sensors detect:
//!
//! - Resource exhaustion (memory, CPU)
//! - Thread pool starvation
//! - Connection pool depletion
//! - Self-inflicted damage from our own responses
//!
//! These sensors answer: **"Is our response hurting us?"**
//!
//! # Name Divergence
//!
//! The Python source names one pattern `CascadePattern`. That name is already claimed by
//! `nexcore-homeostasis-storm::detection::CascadePattern` (storm-domain cascade detection).
//! This module therefore uses [`InternalCascadePattern`] to avoid ambiguity — the two
//! concepts are distinct: the storm variant identifies cross-service cascade propagation,
//! while the internal variant identifies rapid localised failures within a single resource.
//!
//! # Python → Rust class map
//!
//! | Python class | Rust type |
//! |---|---|
//! | `DamagePattern` (ABC) | [`DamagePattern`] (trait) |
//! | `ResourceExhaustionPattern` | [`ResourceExhaustionPattern`] |
//! | `CorrelatedDamagePattern` | [`CorrelatedDamagePattern`] |
//! | `CascadePattern` | [`InternalCascadePattern`] (renamed — see above) |
//! | `InternalHealthSensor` | [`InternalHealthSensor`] |
//! | `MemoryPressureSensor` | [`MemoryPressureSensor`] |
//! | `CPUPressureSensor` | [`CpuPressureSensor`] |
//! | `ConnectionPoolSensor` | [`ConnectionPoolSensor`] |
//! | `ThreadPoolSensor` | [`ThreadPoolSensor`] |
//! | `SelfInflictedDamageSensor` | [`SelfInflictedDamageSensor`] |

use crate::anomaly::AnomalyAssessment;
use serde::{Deserialize, Serialize};
use std::time::Instant;

// ─── History cap (matches Python `history_size` default = 100) ────────────────

const HISTORY_CAP: usize = 100;

// ─── DamagePattern trait ──────────────────────────────────────────────────────

/// A pattern that recognises internal damage from a sensor value and its history.
///
/// Ports Python's `DamagePattern` abstract base class. In Python the `matches` method
/// also accepts an arbitrary `context` dict; here the context is split into typed
/// fields on each concrete implementor so the compiler enforces correctness.
pub trait DamagePattern: std::fmt::Debug {
    /// Returns `Some(assessment)` when the pattern is detected, `None` otherwise.
    ///
    /// The `history` slice is the [`HISTORY_CAP`]-bounded value history recorded
    /// so far; most recent value is last. `value` is the *current* reading
    /// (already appended to `history` by the sensor before calling `matches`).
    fn matches(&self, value: f64, history: &[f64]) -> Option<AnomalyAssessment>;

    /// Human-readable pattern name.
    fn name(&self) -> &str;

    /// Whether this pattern suggests storm formation.
    fn indicates_storm(&self) -> bool;
}

// ─── ResourceExhaustionPattern ────────────────────────────────────────────────

/// Detects simple resource exhaustion when utilisation exceeds a threshold.
///
/// Maps to Python's `ResourceExhaustionPattern`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceExhaustionPattern {
    /// Pattern name.
    pub name: String,
    /// Utilisation fraction (0–1) at which the pattern fires.
    pub exhaustion_threshold: f64,
    /// Multiplier applied to base severity (0.5).
    pub severity_multiplier: f64,
}

impl ResourceExhaustionPattern {
    /// Create a new exhaustion pattern with default severity multiplier of 1.0.
    pub fn new(name: impl Into<String>, exhaustion_threshold: f64) -> Self {
        Self {
            name: name.into(),
            exhaustion_threshold,
            severity_multiplier: 1.0,
        }
    }

    /// Builder: override the severity multiplier.
    #[must_use]
    pub fn with_severity_multiplier(mut self, m: f64) -> Self {
        self.severity_multiplier = m;
        self
    }
}

impl DamagePattern for ResourceExhaustionPattern {
    fn matches(&self, value: f64, _history: &[f64]) -> Option<AnomalyAssessment> {
        if value >= self.exhaustion_threshold {
            let severity = (0.5 * self.severity_multiplier).clamp(0.0, 1.0);
            Some(AnomalyAssessment::anomalous(severity, 0.8))
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn indicates_storm(&self) -> bool {
        true
    }
}

// ─── CorrelatedDamagePattern ──────────────────────────────────────────────────

/// Detects damage whose trend is correlated with the system's own response level.
///
/// This is the key storm signature: if damage *increases* while response is high,
/// the response itself may be causing the damage.
///
/// Maps to Python's `CorrelatedDamagePattern`.
///
/// # Divergence from Python
///
/// Python passes histories through an untyped `context` dict. Here the sensor
/// maintains typed buffers and passes them through [`CorrelationContext`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorrelatedDamagePattern {
    /// Pattern name.
    pub name: String,
    /// Minimum Pearson correlation coefficient to trigger detection (0–1).
    pub correlation_threshold: f64,
    /// Multiplier applied to base severity.
    pub severity_multiplier: f64,
}

impl CorrelatedDamagePattern {
    /// Create a new correlated-damage pattern.
    pub fn new(name: impl Into<String>, correlation_threshold: f64) -> Self {
        Self {
            name: name.into(),
            correlation_threshold,
            severity_multiplier: 1.0,
        }
    }

    /// Builder: override the severity multiplier.
    #[must_use]
    pub fn with_severity_multiplier(mut self, m: f64) -> Self {
        self.severity_multiplier = m;
        self
    }

    /// Compute the Pearson correlation coefficient between two slices.
    ///
    /// Returns 0.0 when `n < 2` or either standard deviation is zero.
    pub fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
        let n = x.len().min(y.len());
        if n < 2 {
            return 0.0;
        }
        let x = &x[x.len() - n..];
        let y = &y[y.len() - n..];

        let mean_x = x.iter().sum::<f64>() / n as f64;
        let mean_y = y.iter().sum::<f64>() / n as f64;

        let numerator: f64 = x
            .iter()
            .zip(y.iter())
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
            .sum();

        let denom_x: f64 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum::<f64>().sqrt();
        let denom_y: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum::<f64>().sqrt();

        if denom_x < f64::EPSILON || denom_y < f64::EPSILON {
            return 0.0;
        }
        numerator / (denom_x * denom_y)
    }

    /// Pattern check that consumes explicit typed context.
    ///
    /// Used by [`InternalHealthSensor`], which maintains the buffers separately.
    pub fn matches_with_context(&self, ctx: &CorrelationContext<'_>) -> Option<AnomalyAssessment> {
        // Need at least 5 points on each side — mirrors Python guard.
        if ctx.response_history.len() < 5 || ctx.damage_history.len() < 5 {
            return None;
        }
        let corr = Self::pearson_correlation(ctx.response_history, ctx.damage_history);
        if corr >= self.correlation_threshold {
            let severity = (0.5 * self.severity_multiplier).clamp(0.0, 1.0);
            Some(AnomalyAssessment::anomalous(severity, 0.85))
        } else {
            None
        }
    }
}

impl DamagePattern for CorrelatedDamagePattern {
    /// Fallback with no context — always returns `None`.
    ///
    /// Callers with response history should use
    /// [`CorrelatedDamagePattern::matches_with_context`] directly.
    fn matches(&self, _value: f64, _history: &[f64]) -> Option<AnomalyAssessment> {
        None
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn indicates_storm(&self) -> bool {
        true
    }
}

/// Typed context for [`CorrelatedDamagePattern::matches_with_context`].
pub struct CorrelationContext<'a> {
    /// Response-level history maintained by the sensor.
    pub response_history: &'a [f64],
    /// Damage-value history maintained by the sensor.
    pub damage_history: &'a [f64],
}

// ─── InternalCascadePattern ──────────────────────────────────────────────────

/// Detects cascade failures by counting recent failures within a sliding time window.
///
/// Maps to Python's `CascadePattern`.
///
/// **Renamed** from `CascadePattern` — see module-level doc for reasoning.
///
/// # Divergence from Python
///
/// Python stores `datetime` timestamps directly in the pattern. This Rust port
/// separates concerns: the sensor maintains the timestamp list (via
/// [`InternalHealthSensor::record_failure`]) and passes the pre-computed count to
/// [`InternalCascadePattern::matches_with_count`]. This avoids non-serialisable
/// [`Instant`] values inside a `#[derive(Serialize)]` struct.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InternalCascadePattern {
    /// Pattern name.
    pub name: String,
    /// Minimum failures within `time_window_secs` to trigger.
    pub failure_count_threshold: usize,
    /// Width of the sliding window in seconds.
    pub time_window_secs: f64,
    /// Multiplier applied to base severity.
    pub severity_multiplier: f64,
}

impl InternalCascadePattern {
    /// Create a new cascade pattern.
    pub fn new(
        name: impl Into<String>,
        failure_count_threshold: usize,
        time_window_secs: f64,
    ) -> Self {
        Self {
            name: name.into(),
            failure_count_threshold,
            time_window_secs,
            severity_multiplier: 1.0,
        }
    }

    /// Builder: override the severity multiplier.
    #[must_use]
    pub fn with_severity_multiplier(mut self, m: f64) -> Self {
        self.severity_multiplier = m;
        self
    }

    /// Assess given a pre-computed count of failures within the window.
    pub fn matches_with_count(&self, recent_count: usize) -> Option<AnomalyAssessment> {
        if recent_count >= self.failure_count_threshold {
            let severity = (0.5 * self.severity_multiplier).clamp(0.0, 1.0);
            Some(AnomalyAssessment::anomalous(severity, 0.85))
        } else {
            None
        }
    }
}

impl DamagePattern for InternalCascadePattern {
    /// Fallback — no failure count available through the base trait interface.
    ///
    /// Use [`InternalCascadePattern::matches_with_count`] for cascade detection.
    fn matches(&self, _value: f64, _history: &[f64]) -> Option<AnomalyAssessment> {
        None
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn indicates_storm(&self) -> bool {
        true
    }
}

// ─── Concrete pattern dispatch enum ──────────────────────────────────────────

/// Concrete damage-pattern variants stored inside [`InternalHealthSensor`].
///
/// Using an enum avoids `Box<dyn DamagePattern>` (no heap allocation) while
/// keeping the sensor dispatch exhaustive.
#[derive(Clone, Debug)]
pub enum SensorPattern {
    /// Resource-exhaustion threshold check.
    Exhaustion(ResourceExhaustionPattern),
    /// Response-correlated damage.
    Correlated(CorrelatedDamagePattern),
    /// Rapid local cascade.
    Cascade(InternalCascadePattern),
}

impl SensorPattern {
    fn name(&self) -> &str {
        match self {
            Self::Exhaustion(p) => p.name(),
            Self::Correlated(p) => p.name(),
            Self::Cascade(p) => p.name(),
        }
    }

    fn indicates_storm(&self) -> bool {
        match self {
            Self::Exhaustion(p) => p.indicates_storm(),
            Self::Correlated(p) => p.indicates_storm(),
            Self::Cascade(p) => p.indicates_storm(),
        }
    }
}

// ─── InternalHealthSensor ────────────────────────────────────────────────────

/// Base sensor for monitoring internal system health (the DAMP sensor).
///
/// Maintains a value history (capped at [`HISTORY_CAP`] = 100 entries), an optional
/// response-level history (for [`CorrelatedDamagePattern`]), and a set of
/// [`SensorPattern`]s.
///
/// Maps to Python's `InternalHealthSensor`.
#[derive(Debug)]
pub struct InternalHealthSensor {
    /// Human-readable sensor name.
    pub name: String,
    patterns: Vec<SensorPattern>,
    value_history: Vec<f64>,
    response_history: Vec<f64>,
    /// (Instant, elapsed_secs_since_creation) for each recorded failure.
    failure_timestamps: Vec<(Instant, f64)>,
    /// Sensor creation time — baseline for elapsed-second arithmetic.
    created_at: Instant,
    /// Pattern names matched on the most recent `assess` call.
    last_matched: Vec<String>,
    /// Count of storm-indicating patterns from the most recent `assess`.
    storm_indicator_count: usize,
}

impl InternalHealthSensor {
    /// Create a new internal health sensor with no patterns.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            patterns: Vec::new(),
            value_history: Vec::new(),
            response_history: Vec::new(),
            failure_timestamps: Vec::new(),
            created_at: Instant::now(),
            last_matched: Vec::new(),
            storm_indicator_count: 0,
        }
    }

    /// Add a damage pattern to this sensor.
    pub fn add_pattern(&mut self, pattern: SensorPattern) {
        self.patterns.push(pattern);
    }

    /// Record a failure event for cascade-pattern detection.
    ///
    /// Automatically purges events older than 30 seconds (matches Python).
    pub fn record_failure(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.created_at).as_secs_f64();
        self.failure_timestamps.push((now, elapsed));

        // Purge events older than 30 s.
        let cutoff = elapsed - 30.0;
        self.failure_timestamps.retain(|(_, ts)| *ts >= cutoff);
    }

    /// Count failures within the last `window_secs` seconds.
    pub fn recent_failure_count(&self, window_secs: f64) -> usize {
        let now_elapsed = Instant::now().duration_since(self.created_at).as_secs_f64();
        let cutoff = now_elapsed - window_secs;
        self.failure_timestamps
            .iter()
            .filter(|(_, ts)| *ts >= cutoff)
            .count()
    }

    /// Append a raw value to the history without running assessment.
    ///
    /// History is capped at [`HISTORY_CAP`].
    pub fn record(&mut self, value: f64) {
        if self.value_history.len() >= HISTORY_CAP {
            self.value_history.remove(0);
        }
        self.value_history.push(value);
    }

    /// Record the current response level (used by [`CorrelatedDamagePattern`]).
    ///
    /// History is capped at [`HISTORY_CAP`].
    pub fn record_response(&mut self, response: f64) {
        if self.response_history.len() >= HISTORY_CAP {
            self.response_history.remove(0);
        }
        self.response_history.push(response);
    }

    /// Assess `value`, update history, and return an [`AnomalyAssessment`].
    ///
    /// History is updated *before* pattern evaluation so the current value
    /// participates in trend checks.
    pub fn assess(&mut self, value: f64) -> AnomalyAssessment {
        self.record(value);

        let mut max_severity: f64 = 0.0;
        let mut storm_count: usize = 0;
        let mut matched: Vec<String> = Vec::new();

        // Build correlation context from current buffers.
        let corr_ctx = CorrelationContext {
            response_history: &self.response_history,
            damage_history: &self.value_history,
        };

        // Collect (pattern_name, indicates_storm, assessment) to avoid
        // borrowing self during the loop.
        let mut results: Vec<(String, bool, Option<AnomalyAssessment>)> =
            Vec::with_capacity(self.patterns.len());

        for pattern in &self.patterns {
            let assessment = match pattern {
                SensorPattern::Exhaustion(p) => p.matches(value, &self.value_history),
                SensorPattern::Correlated(p) => p.matches_with_context(&corr_ctx),
                SensorPattern::Cascade(p) => {
                    let count = self.recent_failure_count(p.time_window_secs);
                    p.matches_with_count(count)
                }
            };
            results.push((
                pattern.name().to_owned(),
                pattern.indicates_storm(),
                assessment,
            ));
        }

        for (name, is_storm, maybe_assessment) in results {
            if let Some(a) = maybe_assessment {
                matched.push(name);
                if a.severity > max_severity {
                    max_severity = a.severity;
                }
                if is_storm {
                    storm_count += 1;
                }
            }
        }

        self.last_matched = matched;
        self.storm_indicator_count = storm_count;

        if !self.last_matched.is_empty() {
            // confidence = min(0.95, 0.5 + 0.15 × storm_count) — mirrors Python
            let confidence = (0.5 + 0.15 * storm_count as f64).min(0.95);
            AnomalyAssessment::anomalous(max_severity, confidence)
        } else {
            AnomalyAssessment::NORMAL
        }
    }

    /// Whether the most recent assessment contained storm-indicating patterns.
    pub fn indicates_storm(&self) -> bool {
        self.storm_indicator_count > 0
    }

    /// Pattern names that matched on the most recent [`assess`](Self::assess) call.
    pub fn matched_patterns(&self) -> &[String] {
        &self.last_matched
    }
}

// ─── Pre-configured sensors ───────────────────────────────────────────────────

/// Sensor for monitoring memory pressure.
///
/// Ships with warning (× 0.5) and critical (× 1.0) [`ResourceExhaustionPattern`]s.
///
/// Maps to Python's `MemoryPressureSensor`.
#[derive(Debug)]
pub struct MemoryPressureSensor {
    inner: InternalHealthSensor,
    /// Most-recently recorded value and its assessment.
    last_reading: Option<(f64, AnomalyAssessment)>,
}

impl MemoryPressureSensor {
    /// Create a new memory-pressure sensor.
    pub fn new(warning_threshold: f64, critical_threshold: f64) -> Self {
        let mut inner = InternalHealthSensor::new("memory_pressure");
        inner.add_pattern(SensorPattern::Exhaustion(
            ResourceExhaustionPattern::new("memory_warning", warning_threshold)
                .with_severity_multiplier(0.5),
        ));
        inner.add_pattern(SensorPattern::Exhaustion(
            ResourceExhaustionPattern::new("memory_critical", critical_threshold)
                .with_severity_multiplier(1.0),
        ));
        Self {
            inner,
            last_reading: None,
        }
    }

    /// Sensor name.
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Assess a memory-utilisation fraction (0–1).
    pub fn assess(&mut self, value: f64) -> AnomalyAssessment {
        self.inner.assess(value)
    }

    /// Record a value and update the stored last reading.
    pub fn record(&mut self, value: f64) {
        let assessment = self.inner.assess(value);
        self.last_reading = Some((value, assessment));
    }

    /// Most-recently recorded value and its assessment, or `None` if
    /// [`record`](Self::record) has not been called yet.
    pub fn last_reading(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading
    }

    /// Whether the last reading indicated storm risk.
    pub fn indicates_storm(&self) -> bool {
        self.inner.indicates_storm()
    }
}

/// Sensor for monitoring CPU pressure.
///
/// Ships with warning (× 0.5) and critical (× 1.0) exhaustion patterns.
///
/// Maps to Python's `CPUPressureSensor`.
#[derive(Debug)]
pub struct CpuPressureSensor {
    inner: InternalHealthSensor,
    /// Most-recently recorded value and its assessment.
    last_reading: Option<(f64, AnomalyAssessment)>,
}

impl CpuPressureSensor {
    /// Create a new CPU-pressure sensor with the given warning and critical thresholds.
    pub fn new(warning_threshold: f64, critical_threshold: f64) -> Self {
        let mut inner = InternalHealthSensor::new("cpu_pressure");
        inner.add_pattern(SensorPattern::Exhaustion(
            ResourceExhaustionPattern::new("cpu_warning", warning_threshold)
                .with_severity_multiplier(0.5),
        ));
        inner.add_pattern(SensorPattern::Exhaustion(
            ResourceExhaustionPattern::new("cpu_critical", critical_threshold)
                .with_severity_multiplier(1.0),
        ));
        Self {
            inner,
            last_reading: None,
        }
    }

    /// Sensor name.
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Assess a CPU-utilisation fraction (0–1).
    pub fn assess(&mut self, value: f64) -> AnomalyAssessment {
        self.inner.assess(value)
    }

    /// Record a value and update the stored last reading.
    pub fn record(&mut self, value: f64) {
        let assessment = self.inner.assess(value);
        self.last_reading = Some((value, assessment));
    }

    /// Most-recently recorded value and its assessment, or `None` if
    /// [`record`](Self::record) has not been called yet.
    pub fn last_reading(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading
    }

    /// Whether the last reading indicated storm risk.
    pub fn indicates_storm(&self) -> bool {
        self.inner.indicates_storm()
    }
}

/// Sensor for monitoring connection pool utilisation.
///
/// Maps to Python's `ConnectionPoolSensor`.
#[derive(Debug)]
pub struct ConnectionPoolSensor {
    inner: InternalHealthSensor,
    /// Most-recently recorded value and its assessment.
    last_reading: Option<(f64, AnomalyAssessment)>,
}

impl ConnectionPoolSensor {
    /// Create a new connection-pool sensor.
    pub fn new(warning_threshold: f64, critical_threshold: f64) -> Self {
        let mut inner = InternalHealthSensor::new("connection_pool");
        inner.add_pattern(SensorPattern::Exhaustion(
            ResourceExhaustionPattern::new("pool_pressure", warning_threshold)
                .with_severity_multiplier(0.6),
        ));
        inner.add_pattern(SensorPattern::Exhaustion(
            ResourceExhaustionPattern::new("pool_exhaustion", critical_threshold)
                .with_severity_multiplier(1.0),
        ));
        Self {
            inner,
            last_reading: None,
        }
    }

    /// Sensor name.
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Assess a connection-pool utilisation fraction (0–1).
    pub fn assess(&mut self, value: f64) -> AnomalyAssessment {
        self.inner.assess(value)
    }

    /// Record a value and update the stored last reading.
    pub fn record(&mut self, value: f64) {
        let assessment = self.inner.assess(value);
        self.last_reading = Some((value, assessment));
    }

    /// Most-recently recorded value and its assessment, or `None` if
    /// [`record`](Self::record) has not been called yet.
    pub fn last_reading(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading
    }

    /// Whether the last reading indicated storm risk.
    pub fn indicates_storm(&self) -> bool {
        self.inner.indicates_storm()
    }
}

/// Sensor for monitoring thread pool utilisation and cascade risk.
///
/// Ships with warning (× 0.5), starvation (× 1.0), and cascade (× 1.2) patterns.
/// The cascade pattern fires when ≥ 5 failures occur within 3 seconds.
///
/// Maps to Python's `ThreadPoolSensor`.
#[derive(Debug)]
pub struct ThreadPoolSensor {
    inner: InternalHealthSensor,
    /// Most-recently recorded value and its assessment.
    last_reading: Option<(f64, AnomalyAssessment)>,
}

impl ThreadPoolSensor {
    /// Create a new thread-pool sensor.
    pub fn new(warning_threshold: f64, critical_threshold: f64) -> Self {
        let mut inner = InternalHealthSensor::new("thread_pool");
        inner.add_pattern(SensorPattern::Exhaustion(
            ResourceExhaustionPattern::new("thread_pressure", warning_threshold)
                .with_severity_multiplier(0.5),
        ));
        inner.add_pattern(SensorPattern::Exhaustion(
            ResourceExhaustionPattern::new("thread_starvation", critical_threshold)
                .with_severity_multiplier(1.0),
        ));
        inner.add_pattern(SensorPattern::Cascade(
            InternalCascadePattern::new("thread_cascade", 5, 3.0).with_severity_multiplier(1.2),
        ));
        Self {
            inner,
            last_reading: None,
        }
    }

    /// Sensor name.
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Assess a thread-pool utilisation fraction (0–1).
    pub fn assess(&mut self, value: f64) -> AnomalyAssessment {
        self.inner.assess(value)
    }

    /// Record a value and update the stored last reading.
    pub fn record(&mut self, value: f64) {
        let assessment = self.inner.assess(value);
        self.last_reading = Some((value, assessment));
    }

    /// Most-recently recorded value and its assessment, or `None` if
    /// [`record`](Self::record) has not been called yet.
    pub fn last_reading(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading
    }

    /// Record a thread-pool failure event (for cascade detection).
    pub fn record_failure(&mut self) {
        self.inner.record_failure();
    }

    /// Whether the last reading indicated storm risk.
    pub fn indicates_storm(&self) -> bool {
        self.inner.indicates_storm()
    }
}

/// Sensor that detects damage caused by the system's own response.
///
/// This is the most important DAMP sensor for storm prevention. It monitors
/// whether our response is causing internal harm by watching for positive
/// Pearson correlation between response level and damage metrics.
///
/// Maps to Python's `SelfInflictedDamageSensor`.
#[derive(Debug)]
pub struct SelfInflictedDamageSensor {
    inner: InternalHealthSensor,
    /// Most-recently recorded value and its assessment.
    last_reading: Option<(f64, AnomalyAssessment)>,
}

impl SelfInflictedDamageSensor {
    /// Create a new self-inflicted-damage sensor.
    ///
    /// `correlation_threshold` is the Pearson r value (0–1) above which
    /// response-correlated damage is flagged.
    pub fn new(correlation_threshold: f64) -> Self {
        let mut inner = InternalHealthSensor::new("self_inflicted_damage");
        inner.add_pattern(SensorPattern::Correlated(
            CorrelatedDamagePattern::new("response_correlated_damage", correlation_threshold)
                .with_severity_multiplier(1.5),
        ));
        Self {
            inner,
            last_reading: None,
        }
    }

    /// Sensor name.
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Assess the current damage value.
    ///
    /// Call [`record_response`](Self::record_response) *before* each `assess`
    /// call to keep the response history in sync.
    pub fn assess(&mut self, damage_value: f64) -> AnomalyAssessment {
        self.inner.assess(damage_value)
    }

    /// Record the current response level for correlation tracking.
    pub fn record_response(&mut self, response: f64) {
        self.inner.record_response(response);
    }

    /// Record a damage value and update the stored last reading.
    pub fn record(&mut self, value: f64) {
        let assessment = self.inner.assess(value);
        self.last_reading = Some((value, assessment));
    }

    /// Most-recently recorded value and its assessment, or `None` if
    /// [`record`](Self::record) has not been called yet.
    pub fn last_reading(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading
    }

    /// Whether the last reading indicated storm risk.
    pub fn indicates_storm(&self) -> bool {
        self.inner.indicates_storm()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── ResourceExhaustionPattern ─────────────────────────────────────────────

    #[test]
    fn exhaustion_pattern_fires_at_threshold() {
        let p = ResourceExhaustionPattern::new("test", 0.9);
        assert!(p.matches(0.9, &[]).is_some());
        assert!(p.matches(0.95, &[]).is_some());
    }

    #[test]
    fn exhaustion_pattern_silent_below_threshold() {
        let p = ResourceExhaustionPattern::new("test", 0.9);
        assert!(p.matches(0.89, &[]).is_none());
    }

    #[test]
    fn exhaustion_severity_clamped_to_unit_interval() {
        let p = ResourceExhaustionPattern::new("test", 0.5).with_severity_multiplier(10.0);
        let a = p.matches(1.0, &[]).expect("should match");
        assert!(
            a.severity <= 1.0,
            "severity must not exceed 1.0, got {}",
            a.severity
        );
    }

    #[test]
    fn exhaustion_indicates_storm() {
        let p = ResourceExhaustionPattern::new("test", 0.5);
        assert!(p.indicates_storm());
    }

    // ── CorrelatedDamagePattern ───────────────────────────────────────────────

    #[test]
    fn correlated_pattern_fires_on_perfect_correlation() {
        let p = CorrelatedDamagePattern::new("corr", 0.7).with_severity_multiplier(1.5);
        let ctx = CorrelationContext {
            response_history: &[0.1, 0.2, 0.3, 0.4, 0.5],
            damage_history: &[0.1, 0.2, 0.3, 0.4, 0.5],
        };
        assert!(p.matches_with_context(&ctx).is_some());
    }

    #[test]
    fn correlated_pattern_silent_on_anti_correlation() {
        let p = CorrelatedDamagePattern::new("corr", 0.7);
        let ctx = CorrelationContext {
            response_history: &[0.1, 0.2, 0.3, 0.4, 0.5],
            damage_history: &[0.5, 0.4, 0.3, 0.2, 0.1],
        };
        assert!(p.matches_with_context(&ctx).is_none());
    }

    #[test]
    fn correlated_pattern_requires_min_5_history() {
        let p = CorrelatedDamagePattern::new("corr", 0.7);
        let ctx = CorrelationContext {
            response_history: &[0.1, 0.2], // only 2 points
            damage_history: &[0.1, 0.2],
        };
        assert!(p.matches_with_context(&ctx).is_none());
    }

    #[test]
    fn pearson_returns_zero_for_constant_series() {
        let x = &[1.0_f64; 5];
        let y = &[1.0_f64; 5];
        assert_eq!(CorrelatedDamagePattern::pearson_correlation(x, y), 0.0);
    }

    #[test]
    fn pearson_perfect_positive_correlation() {
        let x = &[1.0, 2.0, 3.0, 4.0, 5.0_f64];
        let y = &[2.0, 4.0, 6.0, 8.0, 10.0_f64];
        let r = CorrelatedDamagePattern::pearson_correlation(x, y);
        assert!((r - 1.0).abs() < 1e-10, "expected r≈1.0, got {r}");
    }

    // ── InternalCascadePattern ────────────────────────────────────────────────

    #[test]
    fn cascade_pattern_fires_at_threshold_count() {
        let p = InternalCascadePattern::new("cas", 3, 5.0);
        assert!(p.matches_with_count(3).is_some());
        assert!(p.matches_with_count(5).is_some());
    }

    #[test]
    fn cascade_pattern_silent_below_count() {
        let p = InternalCascadePattern::new("cas", 3, 5.0);
        assert!(p.matches_with_count(2).is_none());
    }

    #[test]
    fn cascade_indicates_storm() {
        let p = InternalCascadePattern::new("cas", 1, 5.0);
        assert!(p.indicates_storm());
    }

    // ── InternalHealthSensor ──────────────────────────────────────────────────

    #[test]
    fn sensor_normal_when_no_patterns_match() {
        let mut s = InternalHealthSensor::new("test");
        s.add_pattern(SensorPattern::Exhaustion(ResourceExhaustionPattern::new(
            "hi", 0.9,
        )));
        let a = s.assess(0.5);
        assert!(!a.is_anomalous);
        assert!(!s.indicates_storm());
        assert!(s.matched_patterns().is_empty());
    }

    #[test]
    fn sensor_anomalous_when_pattern_matches() {
        let mut s = InternalHealthSensor::new("test");
        s.add_pattern(SensorPattern::Exhaustion(ResourceExhaustionPattern::new(
            "hi", 0.8,
        )));
        let a = s.assess(0.9);
        assert!(a.is_anomalous);
        assert!(s.indicates_storm());
        assert_eq!(s.matched_patterns(), &["hi"]);
    }

    #[test]
    fn sensor_history_capped_at_100() {
        let mut s = InternalHealthSensor::new("test");
        for i in 0..150 {
            s.record(i as f64);
        }
        assert_eq!(s.value_history.len(), HISTORY_CAP);
    }

    #[test]
    fn sensor_record_failure_count() {
        let mut s = InternalHealthSensor::new("test");
        for _ in 0..5 {
            s.record_failure();
        }
        assert_eq!(s.recent_failure_count(30.0), 5);
    }

    // ── MemoryPressureSensor ──────────────────────────────────────────────────

    #[test]
    fn memory_sensor_fires_at_warning_threshold() {
        let mut s = MemoryPressureSensor::new(0.7, 0.85);
        let a = s.assess(0.75);
        assert!(a.is_anomalous);
    }

    #[test]
    fn memory_sensor_normal_below_warning() {
        let mut s = MemoryPressureSensor::new(0.7, 0.85);
        let a = s.assess(0.65);
        assert!(!a.is_anomalous);
    }

    // ── CpuPressureSensor ─────────────────────────────────────────────────────

    #[test]
    fn cpu_sensor_fires_at_critical_threshold() {
        let mut s = CpuPressureSensor::new(0.7, 0.9);
        let a = s.assess(0.95);
        assert!(a.is_anomalous);
        assert!(s.indicates_storm());
    }

    #[test]
    fn cpu_sensor_normal_below_warning() {
        let mut s = CpuPressureSensor::new(0.7, 0.9);
        let a = s.assess(0.5);
        assert!(!a.is_anomalous);
    }

    // ── ConnectionPoolSensor ──────────────────────────────────────────────────

    #[test]
    fn pool_sensor_fires_at_pressure_threshold() {
        let mut s = ConnectionPoolSensor::new(0.7, 0.9);
        let a = s.assess(0.8);
        assert!(a.is_anomalous);
    }

    #[test]
    fn pool_sensor_normal_below_warning() {
        let mut s = ConnectionPoolSensor::new(0.7, 0.9);
        let a = s.assess(0.5);
        assert!(!a.is_anomalous);
    }

    // ── ThreadPoolSensor ──────────────────────────────────────────────────────

    #[test]
    fn thread_sensor_fires_at_starvation() {
        let mut s = ThreadPoolSensor::new(0.6, 0.85);
        let a = s.assess(0.9);
        assert!(a.is_anomalous);
    }

    #[test]
    fn thread_sensor_normal_below_warning() {
        let mut s = ThreadPoolSensor::new(0.6, 0.85);
        let a = s.assess(0.4);
        assert!(!a.is_anomalous);
    }

    // ── SelfInflictedDamageSensor ─────────────────────────────────────────────

    #[test]
    fn self_inflicted_detects_correlated_damage() {
        let mut s = SelfInflictedDamageSensor::new(0.7);
        // Seed 5 correlated pairs into history before assessing.
        for i in 1..=5_u32 {
            let v = i as f64 * 0.1;
            s.record_response(v);
            s.record(v);
        }
        // 6th point — both histories now have ≥ 5 entries.
        s.record_response(0.6);
        let a = s.assess(0.6);
        assert!(a.is_anomalous, "self-inflicted pattern should fire");
        assert!(s.indicates_storm());
    }

    #[test]
    fn self_inflicted_quiet_with_no_history() {
        let mut s = SelfInflictedDamageSensor::new(0.7);
        let a = s.assess(0.9);
        assert!(!a.is_anomalous, "should be quiet with no response history");
    }
}
