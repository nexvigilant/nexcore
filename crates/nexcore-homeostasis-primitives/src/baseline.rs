//! Baseline definition system — the reference for healthy system operation.
//!
//! The baseline is the set of physiological set points against which ALL
//! measurements are compared. Without a baseline the system cannot calculate
//! deviation, cannot determine proportionality, and cannot know when to return
//! to rest.

use crate::enums::BaselineMetricType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// =============================================================================
// AllostasisTracker
// =============================================================================

/// Tracks path-dependent drift of the homeostatic set-point.
///
/// Recovery is 2–5× slower than adaptation (asymmetric hysteresis).
/// Each exposure cycle leaves a permanent residue via `ratchet_factor`.
///
/// ## Biological analog
///
/// Analogous to allostatic load in the HPA axis: chronic stress shifts the
/// cortisol set-point upward. Full recovery takes 3× longer than the original
/// adaptation, and each stress cycle permanently raises the resting floor by
/// `ratchet_factor`.
///
/// ## Usage
///
/// ```
/// use nexcore_homeostasis_primitives::baseline::AllostasisTracker;
///
/// let mut tracker = AllostasisTracker::new(50.0, 300.0);
/// let now_ms: i64 = 0;
/// tracker.begin_exposure(now_ms);
///
/// // 300 s later — should have drifted toward stressed target.
/// let sp = tracker.current_setpoint(now_ms + 300_000);
/// assert!(sp > 50.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllostasisTracker {
    /// Stored set-point snapshot at the moment of the last state transition.
    ///
    /// This is the *starting value* for the next exponential approach.
    /// To read the *live* value at a given instant, call [`current_setpoint`](Self::current_setpoint).
    pub setpoint: f64,

    /// Original homeostatic baseline before any allostatic load.
    pub original_baseline: f64,

    /// Adaptation time constant in seconds — how fast the set-point drifts under load.
    pub tau_on_secs: f64,

    /// Recovery time constant in seconds — 2–5× `tau_on_secs`.
    pub tau_off_secs: f64,

    /// Ratio `tau_off / tau_on` — default 3.0.
    pub asymmetry: f64,

    /// Permanent upward ratchet applied to `original_baseline` after each cycle.
    ///
    /// Default 0.01 (1 % per cycle).
    pub ratchet_factor: f64,

    /// Number of completed exposure–recovery cycles.
    pub cycle_count: u32,

    /// `true` while the system is under load (adapting); `false` during recovery.
    pub is_exposed: bool,

    /// UTC milliseconds at the most recent state transition.
    pub last_transition_ms: i64,
}

impl AllostasisTracker {
    /// Create a tracker with default asymmetry (3.0) and ratchet factor (0.01).
    ///
    /// `tau_off_secs` is derived as `tau_on_secs × 3.0`.
    ///
    /// ```
    /// use nexcore_homeostasis_primitives::baseline::AllostasisTracker;
    ///
    /// let tracker = AllostasisTracker::new(50.0, 300.0);
    /// assert!((tracker.original_baseline - 50.0).abs() < f64::EPSILON);
    /// assert!((tracker.tau_off_secs - 900.0).abs() < f64::EPSILON);
    /// assert!((tracker.asymmetry - 3.0).abs() < f64::EPSILON);
    /// ```
    pub fn new(baseline: f64, tau_on_secs: f64) -> Self {
        let asymmetry = 3.0_f64;
        Self {
            setpoint: baseline,
            original_baseline: baseline,
            tau_on_secs,
            tau_off_secs: tau_on_secs * asymmetry,
            asymmetry,
            ratchet_factor: 0.01,
            cycle_count: 0,
            is_exposed: false,
            last_transition_ms: 0,
        }
    }

    /// Compute the dynamic set-point at `now_ms` (UTC milliseconds).
    ///
    /// - **Exposed**: set-point approaches the stressed target using `tau_on`.
    ///   Stressed target = `original_baseline × (1.2 + ratchet × cycles)`.
    /// - **Recovering**: set-point returns toward the ratcheted floor using `tau_off`.
    ///   Ratcheted floor = `original_baseline × (1.0 + ratchet × cycles)`.
    ///
    /// ```
    /// use nexcore_homeostasis_primitives::baseline::AllostasisTracker;
    ///
    /// let mut tracker = AllostasisTracker::new(100.0, 60.0);
    /// let t0: i64 = 1_000_000;
    /// tracker.begin_exposure(t0);
    ///
    /// // Right after start — close to baseline.
    /// let sp0 = tracker.current_setpoint(t0);
    /// assert!((sp0 - 100.0).abs() < 1.0);
    ///
    /// // After 5 × tau_on — nearly at stressed target (~95% converged).
    /// let sp_late = tracker.current_setpoint(t0 + 300_000);
    /// assert!(sp_late > 100.0);
    /// ```
    pub fn current_setpoint(&self, now_ms: i64) -> f64 {
        let dt_secs = if self.last_transition_ms == 0 {
            0.0_f64
        } else {
            (now_ms - self.last_transition_ms).max(0) as f64 / 1000.0
        };

        if self.is_exposed {
            // Adapting upward: set-point converges to the stressed target.
            let target =
                self.original_baseline * (1.2 + self.ratchet_factor * f64::from(self.cycle_count));
            target + (self.setpoint - target) * (-dt_secs / self.tau_on_secs).exp()
        } else {
            // Recovering: set-point converges back to the ratcheted floor.
            let floor =
                self.original_baseline * (1.0 + self.ratchet_factor * f64::from(self.cycle_count));
            floor + (self.setpoint - floor) * (-dt_secs / self.tau_off_secs).exp()
        }
    }

    /// Begin an exposure cycle at `now_ms`.
    ///
    /// Snapshots the current dynamic set-point as the new starting value, then
    /// starts the adaptation clock. No-op when already exposed.
    pub fn begin_exposure(&mut self, now_ms: i64) {
        if !self.is_exposed {
            self.setpoint = self.current_setpoint(now_ms);
            self.is_exposed = true;
            self.last_transition_ms = now_ms;
        }
    }

    /// End an exposure cycle at `now_ms`.
    ///
    /// Snapshots the current dynamic set-point, increments the cycle count,
    /// and applies the ratchet (permanently raises `original_baseline`).
    /// No-op when not exposed.
    pub fn end_exposure(&mut self, now_ms: i64) {
        if self.is_exposed {
            self.setpoint = self.current_setpoint(now_ms);
            self.is_exposed = false;
            self.last_transition_ms = now_ms;
            self.cycle_count = self.cycle_count.saturating_add(1);
            // Ratchet: permanently raise the baseline floor so the next recovery
            // converges to a higher value than before.
            self.original_baseline *= 1.0 + self.ratchet_factor;
        }
    }

    /// Normalized drift fraction from the original baseline.
    ///
    /// - `0.0` → at or below original baseline (fully recovered).
    /// - `1.0` → at the stressed target (fully adapted, 20% above original).
    ///
    /// Values are clamped to `[0.0, 1.0]`.  The reference maximum drift is
    /// defined as 20% above `original_baseline`.
    pub fn drift_fraction(&self, now_ms: i64) -> f64 {
        let current = self.current_setpoint(now_ms);
        let max_drift = self.original_baseline * 0.2;
        if max_drift.abs() < f64::EPSILON {
            return 0.0;
        }
        ((current - self.original_baseline) / max_drift).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod allostasis_tests {
    use super::AllostasisTracker;

    #[test]
    fn new_sets_correct_defaults() {
        let t = AllostasisTracker::new(100.0, 60.0);
        assert!((t.original_baseline - 100.0).abs() < f64::EPSILON);
        assert!((t.tau_off_secs - 180.0).abs() < f64::EPSILON);
        assert!((t.asymmetry - 3.0).abs() < f64::EPSILON);
        assert!((t.ratchet_factor - 0.01).abs() < f64::EPSILON);
        assert_eq!(t.cycle_count, 0);
        assert!(!t.is_exposed);
    }

    #[test]
    fn setpoint_at_time_zero_returns_baseline() {
        let t = AllostasisTracker::new(100.0, 60.0);
        // last_transition_ms = 0, dt = 0 → no decay
        let sp = t.current_setpoint(0);
        assert!((sp - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn begin_exposure_starts_drift() {
        let mut t = AllostasisTracker::new(100.0, 60.0);
        let t0: i64 = 1_000_000;
        t.begin_exposure(t0);
        assert!(t.is_exposed);
        // After 5 × tau_on (5 min), should be close to stressed target (120).
        let sp = t.current_setpoint(t0 + 5 * 60_000);
        assert!(sp > 100.0, "setpoint {sp} should be above baseline");
        assert!(
            sp < 121.0,
            "setpoint {sp} should not exceed stressed target"
        );
    }

    #[test]
    fn end_exposure_applies_ratchet() {
        let mut t = AllostasisTracker::new(100.0, 60.0);
        let t0: i64 = 1_000_000;
        t.begin_exposure(t0);
        let original_floor = t.original_baseline;
        t.end_exposure(t0 + 300_000); // 5 min later
        // Ratchet raises original_baseline by 1%.
        assert!(
            t.original_baseline > original_floor,
            "ratchet should raise baseline"
        );
        assert_eq!(t.cycle_count, 1);
        assert!(!t.is_exposed);
    }

    #[test]
    fn recovery_is_slower_than_adaptation() {
        // With asymmetry=3 the recovery constant is 3x the adaptation constant.
        let t = AllostasisTracker::new(100.0, 60.0);
        assert!((t.tau_off_secs / t.tau_on_secs - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drift_fraction_clamps_to_zero_at_rest() {
        let t = AllostasisTracker::new(100.0, 60.0);
        let frac = t.drift_fraction(0);
        assert!((frac - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drift_fraction_approaches_one_when_fully_adapted() {
        let mut t = AllostasisTracker::new(100.0, 60.0);
        let t0: i64 = 1_000_000;
        t.begin_exposure(t0);
        // After 20 × tau_on, very close to stressed target → drift ≈ 1.0.
        let frac = t.drift_fraction(t0 + 20 * 60_000);
        assert!(
            frac > 0.9,
            "drift_fraction {frac} should be near 1.0 after 20×tau_on"
        );
    }
}

// =============================================================================
// BaselineMetric
// =============================================================================

/// A single metric within the baseline, with thresholds for each severity tier.
///
/// The `deviation_from_baseline` method returns a normalized score:
/// - 0.0 = at baseline
/// - 1.0 = at warning threshold
/// - 2.0+ = beyond critical
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaselineMetric {
    /// Metric name.
    pub name: String,
    /// Category of this metric.
    pub metric_type: BaselineMetricType,
    /// Normal / resting value.
    pub resting_value: f64,
    /// Threshold at which the metric is elevated but manageable.
    pub warning_threshold: f64,
    /// Threshold requiring immediate response.
    pub critical_threshold: f64,
    /// Hard ceiling triggering emergency actions.
    pub absolute_maximum: f64,
    /// Unit string for display.
    pub unit: String,
    /// Human-readable description.
    pub description: String,
    /// `true` if higher values are worse (e.g. error rate, latency).
    /// `false` if lower values are worse (e.g. throughput).
    pub higher_is_worse: bool,
}

impl BaselineMetric {
    /// Calculate normalized deviation from baseline.
    ///
    /// Returns 0.0 when at or better than baseline, 1.0 at warning threshold,
    /// and values > 2.0 beyond critical.
    pub fn deviation_from_baseline(&self, current: f64) -> f64 {
        if self.higher_is_worse {
            if current <= self.resting_value {
                return 0.0;
            }
            let range = self.warning_threshold - self.resting_value;
            if range <= 0.0 {
                return 0.0;
            }
            (current - self.resting_value) / range
        } else {
            if current >= self.resting_value {
                return 0.0;
            }
            let range = self.resting_value - self.warning_threshold;
            if range <= 0.0 {
                return 0.0;
            }
            (self.resting_value - current) / range
        }
    }

    /// Determine severity level for a current value.
    ///
    /// Returns one of: `"normal"`, `"elevated"`, `"warning"`, `"critical"`,
    /// or `"emergency"`.
    pub fn severity_level(&self, current: f64) -> &'static str {
        if self.higher_is_worse {
            if current <= self.resting_value {
                "normal"
            } else if current <= self.warning_threshold {
                "elevated"
            } else if current <= self.critical_threshold {
                "warning"
            } else if current <= self.absolute_maximum {
                "critical"
            } else {
                "emergency"
            }
        } else if current >= self.resting_value {
            "normal"
        } else if current >= self.warning_threshold {
            "elevated"
        } else if current >= self.critical_threshold {
            "warning"
        } else if current >= self.absolute_maximum {
            "critical"
        } else {
            "emergency"
        }
    }
}

// =============================================================================
// BaselineConfig
// =============================================================================

/// Configuration for constructing a [`Baseline`] from external data sources
/// (YAML, JSON, env-vars).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BaselineConfig {
    /// Raw metric definitions keyed by name.
    #[serde(default)]
    pub metrics: HashMap<String, serde_json::Value>,
    /// Control loop poll interval in seconds.
    pub control_loop_interval_secs: Option<f64>,
    /// Maximum continuous response duration in minutes.
    pub max_response_duration_mins: Option<f64>,
    /// Response budget per hour (total response-level units).
    pub response_budget_per_hour: Option<f64>,
}

impl BaselineConfig {
    /// Parse from a JSON string.
    pub fn from_json(json: &str) -> nexcore_error::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

// =============================================================================
// Baseline
// =============================================================================

/// Complete baseline definition — the system's "healthy" set points.
///
/// Contains built-in metrics for the most common observable signals plus a
/// `custom_metrics` map for application-specific signals.
///
/// ```
/// use nexcore_homeostasis_primitives::baseline::Baseline;
///
/// let baseline = Baseline::default();
/// let deviation = baseline.calculate_overall_deviation(
///     &[("error_rate".to_string(), 0.001)].into_iter().collect()
/// );
/// assert!(deviation < 0.1); // at baseline
/// ```
#[derive(Clone, Debug)]
pub struct Baseline {
    // ── Core metric set points ──────────────────────────────────────────────
    /// Normal error rate (default 0.1%).
    pub error_rate: f64,
    /// Normal P99 latency in ms (default 200ms).
    pub latency_p99_ms: f64,
    /// Normal resource utilization fraction 0–1 (default 40%).
    pub resource_utilization: f64,
    /// Normal queue depth (default 0).
    pub queue_depth: f64,

    // ── Response state ──────────────────────────────────────────────────────
    /// Resting response level (usually 0).
    pub resting_response_level: f64,
    /// Maximum tolerable response before dampening is triggered.
    pub max_tolerable_response: f64,
    /// Hard ceiling — triggers emergency shutdown.
    pub absolute_max_response: f64,

    // ── Timing ─────────────────────────────────────────────────────────────
    /// Control loop poll interval.
    pub control_loop_interval: Duration,
    /// Maximum time the system is allowed to stay in active response.
    pub max_response_duration: Duration,

    // ── Budget ──────────────────────────────────────────────────────────────
    /// Total response budget per hour.
    pub response_budget_per_hour: f64,

    // ── Derived metrics (built by new()) ────────────────────────────────────
    metrics: HashMap<String, BaselineMetric>,
}

impl Default for Baseline {
    fn default() -> Self {
        Self::new()
    }
}

impl Baseline {
    /// Create a baseline with sensible defaults.
    pub fn new() -> Self {
        let mut b = Self {
            error_rate: 0.001,
            latency_p99_ms: 200.0,
            resource_utilization: 0.4,
            queue_depth: 0.0,
            resting_response_level: 0.0,
            max_tolerable_response: 100.0,
            absolute_max_response: 150.0,
            control_loop_interval: Duration::from_secs(10),
            max_response_duration: Duration::from_secs(3600),
            response_budget_per_hour: 200.0,
            metrics: HashMap::new(),
        };
        b.rebuild_metrics();
        b
    }

    /// Create a baseline from a [`BaselineConfig`].
    pub fn from_config(config: &BaselineConfig) -> nexcore_error::Result<Self> {
        let mut b = Self::new();
        if let Some(secs) = config.control_loop_interval_secs {
            b.control_loop_interval = Duration::from_secs_f64(secs);
        }
        if let Some(mins) = config.max_response_duration_mins {
            b.max_response_duration = Duration::from_secs_f64(mins * 60.0);
        }
        if let Some(budget) = config.response_budget_per_hour {
            b.response_budget_per_hour = budget;
        }
        for (name, raw) in &config.metrics {
            let resting = raw["resting_value"].as_f64().unwrap_or(0.0);
            let warning = raw["warning_threshold"].as_f64().unwrap_or(resting * 5.0);
            let critical = raw["critical_threshold"].as_f64().unwrap_or(resting * 20.0);
            let abs_max = raw["absolute_maximum"].as_f64().unwrap_or(1.0);
            let higher_is_worse = raw["higher_is_worse"].as_bool().unwrap_or(true);
            let metric = BaselineMetric {
                name: name.clone(),
                metric_type: BaselineMetricType::Custom,
                resting_value: resting,
                warning_threshold: warning,
                critical_threshold: critical,
                absolute_maximum: abs_max,
                unit: raw["unit"].as_str().unwrap_or("").to_string(),
                description: raw["description"].as_str().unwrap_or("").to_string(),
                higher_is_worse,
            };
            b.metrics.insert(name.clone(), metric);
        }
        Ok(b)
    }

    fn rebuild_metrics(&mut self) {
        let er = self.error_rate;
        let lat = self.latency_p99_ms;
        let ru = self.resource_utilization;
        let qd = self.queue_depth;
        let rl = self.resting_response_level;
        let mtr = self.max_tolerable_response;
        let amr = self.absolute_max_response;

        self.metrics.insert(
            "error_rate".into(),
            BaselineMetric {
                name: "error_rate".into(),
                metric_type: BaselineMetricType::ErrorRate,
                resting_value: er,
                warning_threshold: er * 5.0,
                critical_threshold: er * 20.0,
                absolute_maximum: 0.5,
                unit: "ratio".into(),
                description: "HTTP/RPC error rate".into(),
                higher_is_worse: true,
            },
        );
        self.metrics.insert(
            "latency_p99_ms".into(),
            BaselineMetric {
                name: "latency_p99_ms".into(),
                metric_type: BaselineMetricType::Latency,
                resting_value: lat,
                warning_threshold: lat * 2.0,
                critical_threshold: lat * 5.0,
                absolute_maximum: lat * 20.0,
                unit: "milliseconds".into(),
                description: "P99 request latency".into(),
                higher_is_worse: true,
            },
        );
        self.metrics.insert(
            "resource_utilization".into(),
            BaselineMetric {
                name: "resource_utilization".into(),
                metric_type: BaselineMetricType::ResourceUtilization,
                resting_value: ru,
                warning_threshold: 0.7,
                critical_threshold: 0.85,
                absolute_maximum: 0.95,
                unit: "ratio".into(),
                description: "CPU/memory utilization".into(),
                higher_is_worse: true,
            },
        );
        self.metrics.insert(
            "queue_depth".into(),
            BaselineMetric {
                name: "queue_depth".into(),
                metric_type: BaselineMetricType::QueueDepth,
                resting_value: qd,
                warning_threshold: 100.0,
                critical_threshold: 500.0,
                absolute_maximum: 2000.0,
                unit: "items".into(),
                description: "Pending queue depth".into(),
                higher_is_worse: true,
            },
        );
        self.metrics.insert(
            "response_level".into(),
            BaselineMetric {
                name: "response_level".into(),
                metric_type: BaselineMetricType::ResponseLevel,
                resting_value: rl,
                warning_threshold: mtr * 0.5,
                critical_threshold: mtr * 0.8,
                absolute_maximum: amr,
                unit: "units".into(),
                description: "Self-measured response level".into(),
                higher_is_worse: true,
            },
        );
    }

    /// Add a custom metric.
    pub fn add_metric(&mut self, metric: BaselineMetric) {
        self.metrics.insert(metric.name.clone(), metric);
    }

    /// Get a metric definition by name.
    pub fn get_metric(&self, name: &str) -> Option<&BaselineMetric> {
        self.metrics.get(name)
    }

    /// Calculate the weighted overall deviation from baseline.
    ///
    /// Returns 0.0 when at baseline, higher when deviating.
    pub fn calculate_overall_deviation(&self, current_values: &HashMap<String, f64>) -> f64 {
        if current_values.is_empty() {
            return 0.0;
        }
        let weights: HashMap<&str, f64> = [
            ("error_rate", 3.0),
            ("latency_p99_ms", 2.0),
            ("resource_utilization", 1.0),
            ("queue_depth", 1.5),
            ("response_level", 2.0),
        ]
        .into_iter()
        .collect();

        let mut total = 0.0;
        let mut weight_sum = 0.0;
        for (name, &value) in current_values {
            if let Some(metric) = self.metrics.get(name.as_str()) {
                let w = weights.get(name.as_str()).copied().unwrap_or(1.0);
                total += metric.deviation_from_baseline(value) * w;
                weight_sum += w;
            }
        }
        if weight_sum > 0.0 {
            total / weight_sum
        } else {
            0.0
        }
    }

    /// Whether the system is effectively at baseline (within `tolerance`).
    pub fn is_at_baseline(&self, current_values: &HashMap<String, f64>, tolerance: f64) -> bool {
        self.calculate_overall_deviation(current_values) < tolerance
    }

    /// Find which metric is furthest from baseline.
    ///
    /// Returns `Some((name, deviation, severity))` or `None` if no known metrics.
    pub fn get_most_deviant_metric(
        &self,
        current_values: &HashMap<String, f64>,
    ) -> Option<(String, f64, &'static str)> {
        let mut max_dev = 0.0;
        let mut result: Option<(String, f64, &'static str)> = None;

        for (name, &value) in current_values {
            if let Some(metric) = self.metrics.get(name.as_str()) {
                let dev = metric.deviation_from_baseline(value);
                if dev > max_dev {
                    max_dev = dev;
                    result = Some((name.clone(), dev, metric.severity_level(value)));
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_baseline() -> Baseline {
        Baseline::new()
    }

    #[test]
    fn deviation_at_baseline_is_zero() {
        let b = sample_baseline();
        let values: HashMap<String, f64> = [
            ("error_rate".to_string(), 0.001), // exactly at resting
            ("latency_p99_ms".to_string(), 200.0),
        ]
        .into_iter()
        .collect();
        assert!(b.calculate_overall_deviation(&values) < 0.01);
    }

    #[test]
    fn deviation_at_warning_is_one() {
        let b = sample_baseline();
        let metric = b.get_metric("error_rate").unwrap();
        // warning_threshold = resting * 5 = 0.005
        let dev = metric.deviation_from_baseline(0.005);
        assert!((dev - 1.0).abs() < 0.01, "expected ~1.0, got {dev}");
    }

    #[test]
    fn deviation_beyond_critical_exceeds_two() {
        let b = sample_baseline();
        let metric = b.get_metric("error_rate").unwrap();
        // critical_threshold = resting * 20 = 0.02
        let dev = metric.deviation_from_baseline(0.02);
        assert!(dev >= 2.0, "expected ≥2.0, got {dev}");
    }

    #[test]
    fn severity_levels() {
        let b = sample_baseline();
        let metric = b.get_metric("error_rate").unwrap(); // higher_is_worse
        assert_eq!(metric.severity_level(0.001), "normal");
        assert_eq!(metric.severity_level(0.003), "elevated"); // between resting and warning
        assert_eq!(metric.severity_level(0.01), "warning"); // between warning and critical
        assert_eq!(metric.severity_level(0.1), "critical"); // between critical and abs_max
        assert_eq!(metric.severity_level(0.6), "emergency"); // beyond abs_max
    }

    #[test]
    fn is_at_baseline() {
        let b = sample_baseline();
        let at_rest: HashMap<String, f64> =
            [("error_rate".to_string(), 0.001)].into_iter().collect();
        assert!(b.is_at_baseline(&at_rest, 0.1));

        let elevated: HashMap<String, f64> =
            [("error_rate".to_string(), 0.01)].into_iter().collect();
        assert!(!b.is_at_baseline(&elevated, 0.1));
    }

    #[test]
    fn get_most_deviant_metric() {
        let b = sample_baseline();
        let values: HashMap<String, f64> = [
            ("error_rate".to_string(), 0.001),      // at baseline
            ("latency_p99_ms".to_string(), 1000.0), // very elevated
        ]
        .into_iter()
        .collect();
        let worst = b.get_most_deviant_metric(&values).unwrap();
        assert_eq!(worst.0, "latency_p99_ms");
        assert!(worst.1 > 1.0);
    }

    #[test]
    fn add_custom_metric() {
        let mut b = Baseline::new();
        b.add_metric(BaselineMetric {
            name: "custom_rps".into(),
            metric_type: BaselineMetricType::Custom,
            resting_value: 1000.0,
            warning_threshold: 500.0, // lower is worse
            critical_threshold: 200.0,
            absolute_maximum: 50.0,
            unit: "rps".into(),
            description: "Requests per second".into(),
            higher_is_worse: false,
        });
        let metric = b.get_metric("custom_rps").unwrap();
        assert_eq!(metric.severity_level(1000.0), "normal");
        assert_eq!(metric.severity_level(300.0), "warning");
    }
}
