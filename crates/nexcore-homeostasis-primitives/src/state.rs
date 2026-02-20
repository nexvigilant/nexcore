//! System state tracking and trend analysis — where in the response cycle are we?
//!
//! This module answers the central question of homeostasis: *what is the system's
//! current physiological state?* State is not just a snapshot of metrics — it is
//! the integration of current readings, recent history, response level, and
//! trajectory.
//!
//! ## Key types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`MetricHistory`] | Rolling time-series buffer for one metric |
//! | [`SystemState`] | Complete point-in-time snapshot of system health |
//! | [`StateTracker`] | Manages histories, computes state on each control-loop tick |
//!
//! ## Example
//!
//! ```
//! use std::time::Duration;
//! use nexcore_homeostasis_primitives::state::{MetricHistory, StateTracker, SystemState};
//! use nexcore_homeostasis_primitives::enums::{HealthStatus, TrendDirection};
//! use nexcore_homeostasis_primitives::baseline::Baseline;
//!
//! let mut tracker = StateTracker::new(vec!["error_rate".into()], Duration::from_secs(3600), 1000);
//! let baseline = Baseline::default();
//! let metrics = [("error_rate".to_string(), 0.001)].into_iter().collect();
//! let state = tracker.update_state(metrics, 0.0, 0.0, 0.0, &baseline);
//! assert_eq!(state.health_status, HealthStatus::Healthy);
//! ```

use crate::baseline::Baseline;
use crate::data::MetricSnapshot;
use crate::enums::{HealthStatus, ResponsePhase, TrendDirection};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tokio::time::Instant;

// =============================================================================
// DynamicsType
// =============================================================================

/// Annotation for the dominant dynamical regime of a signal lifecycle state.
///
/// Signal systems often mix two fundamentally different dynamics:
///
/// - **Discrete** jumps between macro states (e.g., `Normal → Elevated`).
///   These are Markov chain transitions — instantaneous, memoryless.
///
/// - **Continuous** trajectories within a state — evidence accumulates
///   smoothly over time (e.g., threat score drifting from 0.30 to 0.70
///   while still in the `Elevated` state).
///
/// A system exhibiting both simultaneously is a **Piecewise-Deterministic
/// Markov Process (PDMP)** — the `Hybrid` variant.
///
/// This enum allows the homeostasis engine to select the correct mathematical
/// toolkit for each state: ODE solvers for `Continuous`, rate matrices for
/// `Discrete`, and hybrid simulators for `Hybrid`.
///
/// ## T1 Grounding
///
/// | Variant | Primitives |
/// |---------|------------|
/// | `Discrete` | ς (State) + ∝ (Irreversibility) |
/// | `Continuous` | ν (Frequency) + → (Causality) |
/// | `Hybrid` | ς + ν + → (PDMP composition) |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicsType {
    /// Pure discrete macro-state transition dynamics.
    ///
    /// The system is at a state boundary — it will jump to a new regime.
    /// Mathematical tooling: rate matrices, jump process simulation.
    Discrete,

    /// Pure continuous evidence-accumulation dynamics.
    ///
    /// The system is evolving smoothly within its current macro state.
    /// Mathematical tooling: ODE/SDE solvers, drift-diffusion models.
    Continuous,

    /// Hybrid PDMP dynamics — both discrete jumps and continuous drift active.
    ///
    /// Most common during escalation phases where evidence is accumulating
    /// *and* the system may jump to a new macro state at any moment.
    /// Mathematical tooling: piecewise-deterministic Markov process simulators.
    Hybrid,
}

impl Default for DynamicsType {
    fn default() -> Self {
        Self::Continuous
    }
}

impl std::fmt::Display for DynamicsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Discrete => write!(f, "Discrete"),
            Self::Continuous => write!(f, "Continuous"),
            Self::Hybrid => write!(f, "Hybrid"),
        }
    }
}

// =============================================================================
// MetricHistory
// =============================================================================

/// Rolling time-series buffer for a single named metric.
///
/// Maintains up to `max_samples` entries and automatically evicts entries older
/// than `max_age` on each [`record`](Self::record) call.  Use
/// [`get_trend`](Self::get_trend) to determine whether the metric is
/// improving, stable, degrading, or in accelerating degradation (the storm
/// signature).
///
/// ```
/// use std::time::Duration;
/// use nexcore_homeostasis_primitives::state::MetricHistory;
/// use nexcore_homeostasis_primitives::enums::TrendDirection;
///
/// let mut h = MetricHistory::new("error_rate", 100, Duration::from_secs(1800));
/// for v in [0.001, 0.002, 0.004, 0.008, 0.016, 0.032] {
///     h.record(v);
/// }
/// // Monotonically rising series should be flagged as degrading or accelerating.
/// let trend = h.get_trend(Duration::from_secs(1800));
/// assert!(matches!(trend, TrendDirection::Degrading | TrendDirection::AcceleratingDegradation));
/// ```
#[derive(Debug, Clone)]
pub struct MetricHistory {
    /// Name of the metric being tracked.
    pub name: String,
    /// Maximum number of samples to retain.
    pub max_samples: usize,
    /// Samples older than this are evicted.
    pub max_age: Duration,
    samples: VecDeque<MetricSnapshot>,
}

impl MetricHistory {
    /// Create a new history buffer.
    pub fn new(name: impl Into<String>, max_samples: usize, max_age: Duration) -> Self {
        Self {
            name: name.into(),
            max_samples,
            max_age,
            samples: VecDeque::with_capacity(max_samples.min(256)),
        }
    }

    /// Record a new value, evicting stale samples.
    pub fn record(&mut self, value: f64) {
        self.samples
            .push_back(MetricSnapshot::now(self.name.clone(), value));
        self.evict_old();
        // Hard cap on count.
        while self.samples.len() > self.max_samples {
            self.samples.pop_front();
        }
    }

    /// Most recent recorded value.
    pub fn current(&self) -> Option<f64> {
        self.samples.back().map(|s| s.value)
    }

    /// All samples as a slice view.
    pub fn samples(&self) -> &VecDeque<MetricSnapshot> {
        &self.samples
    }

    /// Samples recorded within the last `duration`.
    pub fn get_recent(&self, duration: Duration) -> Vec<&MetricSnapshot> {
        let cutoff = Instant::now()
            .checked_sub(duration)
            .unwrap_or_else(Instant::now);
        self.samples
            .iter()
            .filter(|s| s.timestamp >= cutoff)
            .collect()
    }

    /// Analyze the trend of this metric over the given `window`.
    ///
    /// Returns [`TrendDirection::Stable`] when fewer than 3 samples exist in the
    /// window (not enough data to determine direction).
    ///
    /// The algorithm splits the window into halves. If the second half mean is
    /// more than 5 % of the first half mean above/below, the metric is considered
    /// degrading/improving. When six or more samples are available the window is
    /// further split into thirds; if the rate of change is accelerating
    /// (rate₂ > 1.5 × rate₁), [`TrendDirection::AcceleratingDegradation`] is
    /// returned — the early storm signature.
    pub fn get_trend(&self, window: Duration) -> TrendDirection {
        let recent = self.get_recent(window);
        if recent.len() < 3 {
            return TrendDirection::Stable;
        }

        let values: Vec<f64> = recent.iter().map(|s| s.value).collect();
        let mid = values.len() / 2;
        let first_half = &values[..mid];
        let second_half = &values[mid..];

        if first_half.is_empty() || second_half.is_empty() {
            return TrendDirection::Stable;
        }

        let avg_first = mean(first_half);
        let avg_second = mean(second_half);

        let threshold = if avg_first.abs() > f64::EPSILON {
            avg_first.abs() * 0.05
        } else {
            0.01
        };

        let change = avg_second - avg_first;

        // Check for acceleration before checking simple direction.
        if values.len() >= 6 {
            let third = values.len() / 3;
            let t1 = &values[..third];
            let t2 = &values[third..2 * third];
            let t3 = &values[2 * third..];

            if !t1.is_empty() && !t2.is_empty() && !t3.is_empty() {
                let rate1 = mean(t2) - mean(t1);
                let rate2 = mean(t3) - mean(t2);
                if rate2 > rate1 * 1.5 && rate2 > threshold {
                    return TrendDirection::AcceleratingDegradation;
                }
            }
        }

        if change.abs() < threshold {
            TrendDirection::Stable
        } else if change > 0.0 {
            TrendDirection::Degrading
        } else {
            TrendDirection::Improving
        }
    }

    /// Rate of change per second (linear regression slope) over `window`.
    ///
    /// Returns `0.0` when fewer than 2 samples are available.
    pub fn calculate_rate_of_change(&self, window: Duration) -> f64 {
        let recent = self.get_recent(window);
        if recent.len() < 2 {
            return 0.0;
        }

        let base = recent[0].timestamp;
        let times: Vec<f64> = recent
            .iter()
            .map(|s| s.timestamp.duration_since(base).as_secs_f64())
            .collect();
        let values: Vec<f64> = recent.iter().map(|s| s.value).collect();

        linear_regression_slope(&times, &values)
    }

    // ── private helpers ────────────────────────────────────────────────────────

    fn evict_old(&mut self) {
        let cutoff = Instant::now()
            .checked_sub(self.max_age)
            .unwrap_or_else(Instant::now);
        while self.samples.front().is_some_and(|s| s.timestamp < cutoff) {
            self.samples.pop_front();
        }
    }
}

// =============================================================================
// SystemState
// =============================================================================

/// Complete point-in-time snapshot of system health.
///
/// `SystemState` is the output of [`StateTracker::update_state`] and is the
/// primary input to the control-loop's decision logic.  It aggregates:
/// - Raw metric values
/// - Derived status (`health_status`, `response_phase`)
/// - Response accounting (`current_response_level`, `response_budget_consumed`)
/// - Threat and damage levels (from the signal manager)
/// - Proportionality ratio and storm risk score
/// - Per-metric trend directions
/// - Baseline deviation summary
#[derive(Clone, Debug)]
pub struct SystemState {
    /// When this state was computed (tokio `Instant`).
    pub timestamp: Instant,
    /// Raw metric values that were provided to [`StateTracker::update_state`].
    pub metrics: HashMap<String, f64>,
    /// Overall health classification.
    pub health_status: HealthStatus,
    /// Phase of the biological response cycle.
    pub response_phase: ResponsePhase,
    /// Current measured response level (units match the baseline).
    pub current_response_level: f64,
    /// How long the system has been in active response (> resting level).
    pub response_duration: Duration,
    /// Cumulative response-level units consumed this hour.
    pub response_budget_consumed: f64,
    /// Sum of all `Threat` signal strengths.
    pub threat_level: f64,
    /// Sum of all `Damage` signal strengths.
    pub damage_level: f64,
    /// `response_level / threat_level` ratio; 1.0 = perfectly proportional.
    ///
    /// Values > 3.0 trigger dampening; values < 0.5 trigger amplification.
    pub proportionality: f64,
    /// 0–1 probability of entering storm state.
    pub storm_risk: f64,
    /// Trend direction per tracked metric name.
    pub trends: HashMap<String, TrendDirection>,
    /// Weighted deviation from baseline across all provided metrics.
    pub overall_deviation: f64,
    /// Name of the metric furthest from its baseline set-point.
    pub most_deviant_metric: Option<String>,
    /// Human-readable annotations added by the control loop.
    pub notes: Vec<String>,

    /// Dominant dynamical regime annotating this state's signal lifecycle.
    ///
    /// - `Discrete` — the system is at a macro state boundary (transition imminent).
    /// - `Continuous` — evidence is accumulating smoothly within the current macro state.
    /// - `Hybrid` — both dynamics are active simultaneously (PDMP regime).
    ///
    /// Used by the guardian engine to select the appropriate mathematical solver.
    pub dynamics: DynamicsType,
}

impl SystemState {
    /// Whether the system is currently at its resting, healthy state.
    pub fn is_healthy(&self) -> bool {
        self.health_status == HealthStatus::Healthy
    }

    /// Whether a cytokine-storm-style cascade is in progress.
    pub fn is_in_storm(&self) -> bool {
        self.health_status == HealthStatus::Storm || self.response_phase == ResponsePhase::Storm
    }

    /// Whether the system should reduce its response level.
    ///
    /// Returns `true` when:
    /// - `proportionality > 3.0` (over-responding)
    /// - Currently in storm state
    /// - Threat is negligible but response is still elevated (lingering activation)
    pub fn needs_dampening(&self) -> bool {
        if self.proportionality > 3.0 {
            return true;
        }
        if self.is_in_storm() {
            return true;
        }
        // Threat has resolved but response is still elevated.
        if self.threat_level < 0.1 && self.current_response_level > 10.0 {
            return true;
        }
        false
    }

    /// Whether the system should increase its response level.
    ///
    /// Returns `true` when a threat exists but the response is proportionally
    /// less than half the threat level.
    pub fn needs_amplification(&self) -> bool {
        self.threat_level > 0.0 && self.proportionality < 0.5
    }
}

// =============================================================================
// StateTracker
// =============================================================================

/// Maintains rolling histories and produces [`SystemState`] on each tick.
///
/// `StateTracker` is the memory of the control loop.  It tracks:
/// - A [`MetricHistory`] for every metric listed in `metrics_to_track`
/// - A bounded ring of past [`SystemState`] snapshots
/// - Response start time (for duration accounting)
/// - Hourly response budget consumption
///
/// ```
/// use std::time::Duration;
/// use nexcore_homeostasis_primitives::state::StateTracker;
/// use nexcore_homeostasis_primitives::baseline::Baseline;
/// use nexcore_homeostasis_primitives::enums::HealthStatus;
///
/// let mut tracker = StateTracker::new(
///     vec!["error_rate".into()],
///     Duration::from_secs(3600),
///     1000,
/// );
/// let baseline = Baseline::default();
/// let metrics = [("error_rate".to_string(), 0.001)].into_iter().collect();
/// let state = tracker.update_state(metrics, 0.0, 0.0, 0.0, &baseline);
/// assert_eq!(state.health_status, HealthStatus::Healthy);
/// ```
#[derive(Debug)]
pub struct StateTracker {
    /// Metric names that will have rolling histories maintained.
    pub metrics_to_track: Vec<String>,
    /// Maximum age of retained history entries.
    pub history_max_age: Duration,
    metric_histories: HashMap<String, MetricHistory>,
    state_history: VecDeque<SystemState>,
    max_state_history: usize,
    current_state: Option<SystemState>,
    // Response timing
    response_start_time: Option<Instant>,
    // Budget accounting (resets each hour)
    total_response_this_hour: f64,
    hour_start: Instant,
}

impl StateTracker {
    /// Create a new `StateTracker`.
    ///
    /// - `metrics_to_track` — names of metrics that will receive rolling histories.
    /// - `history_max_age` — samples older than this are evicted from histories.
    /// - `max_state_history` — maximum number of past [`SystemState`] snapshots retained.
    pub fn new(
        metrics_to_track: Vec<String>,
        history_max_age: Duration,
        max_state_history: usize,
    ) -> Self {
        let metric_histories = metrics_to_track
            .iter()
            .map(|name| {
                let h = MetricHistory::new(name.clone(), 100, history_max_age);
                (name.clone(), h)
            })
            .collect();

        Self {
            metrics_to_track,
            history_max_age,
            metric_histories,
            state_history: VecDeque::with_capacity(max_state_history.min(256)),
            max_state_history,
            current_state: None,
            response_start_time: None,
            total_response_this_hour: 0.0,
            hour_start: Instant::now(),
        }
    }

    /// Record new metric values into their rolling histories.
    ///
    /// Values for metrics not in `metrics_to_track` are silently ignored.
    pub fn record_metrics(&mut self, metrics: &HashMap<String, f64>) {
        for (name, &value) in metrics {
            if let Some(h) = self.metric_histories.get_mut(name) {
                h.record(value);
            }
        }
    }

    /// Compute and return the current [`SystemState`].
    ///
    /// This is the primary entry point called each control-loop iteration.
    /// It records the metrics, recomputes all derived fields, and appends the
    /// result to the internal state history ring.
    pub fn update_state(
        &mut self,
        metrics: HashMap<String, f64>,
        threat_level: f64,
        damage_level: f64,
        response_level: f64,
        baseline: &Baseline,
    ) -> SystemState {
        // Record into histories.
        self.record_metrics(&metrics);

        // Trend per tracked metric.
        let window = Duration::from_secs(300); // 5-minute trend window
        let trends: HashMap<String, TrendDirection> = self
            .metric_histories
            .iter()
            .map(|(name, h)| (name.clone(), h.get_trend(window)))
            .collect();

        // Proportionality: response / threat.
        let proportionality = if threat_level > 0.01 {
            response_level / threat_level
        } else if response_level > 0.0 {
            response_level // no threat but response exists → effectively infinite ratio
        } else {
            1.0
        };

        // Storm risk.
        let storm_risk =
            self.calculate_storm_risk(proportionality, &trends, response_level, baseline);

        // Health status.
        let health_status = Self::determine_health_status(&metrics, baseline, storm_risk);

        // Response phase.
        let response_phase = self.determine_response_phase(
            threat_level,
            response_level,
            proportionality,
            storm_risk,
        );

        // Response duration accounting.
        let response_duration = if response_level > baseline.resting_response_level {
            if self.response_start_time.is_none() {
                self.response_start_time = Some(Instant::now());
            }
            self.response_start_time
                .map(|t| t.elapsed())
                .unwrap_or(Duration::ZERO)
        } else {
            self.response_start_time = None;
            Duration::ZERO
        };

        // Hourly budget.
        self.update_response_budget(response_level);

        // Baseline deviation.
        let overall_deviation = baseline.calculate_overall_deviation(&metrics);
        let most_deviant_metric = baseline
            .get_most_deviant_metric(&metrics)
            .map(|(name, _, _)| name);

        // Annotate dynamics type based on current phase and health.
        let dynamics = match (&response_phase, &health_status) {
            // Storm or emergency: both discrete jumps and continuous escalation.
            (ResponsePhase::Storm, _) | (_, HealthStatus::Storm) | (_, HealthStatus::Emergency) => {
                DynamicsType::Hybrid
            }
            // Active phase transitions: discrete macro-state jumps.
            (ResponsePhase::Escalating, _) | (ResponsePhase::Resolving, _) => {
                DynamicsType::Discrete
            }
            // Default: continuous evidence accumulation within the current state.
            _ => DynamicsType::Continuous,
        };

        let state = SystemState {
            timestamp: Instant::now(),
            metrics,
            health_status,
            response_phase,
            current_response_level: response_level,
            response_duration,
            response_budget_consumed: self.total_response_this_hour,
            threat_level,
            damage_level,
            proportionality,
            storm_risk,
            trends,
            overall_deviation,
            most_deviant_metric,
            notes: Vec::new(),
            dynamics,
        };

        // Store in ring.
        if self.state_history.len() >= self.max_state_history {
            self.state_history.pop_front();
        }
        self.state_history.push_back(state.clone());
        self.current_state = Some(state.clone());

        state
    }

    /// Most recent [`SystemState`], if any iteration has completed.
    pub fn current_state(&self) -> Option<&SystemState> {
        self.current_state.as_ref()
    }

    /// Past states from the last `duration`.
    pub fn get_state_history(&self, duration: Duration) -> Vec<&SystemState> {
        let cutoff = Instant::now()
            .checked_sub(duration)
            .unwrap_or_else(Instant::now);
        self.state_history
            .iter()
            .filter(|s| s.timestamp >= cutoff)
            .collect()
    }

    /// Rolling history for a specific metric, if it is being tracked.
    pub fn get_metric_history(&self, metric_name: &str) -> Option<&MetricHistory> {
        self.metric_histories.get(metric_name)
    }

    // ── private helpers ────────────────────────────────────────────────────────

    fn calculate_storm_risk(
        &self,
        proportionality: f64,
        trends: &HashMap<String, TrendDirection>,
        response_level: f64,
        baseline: &Baseline,
    ) -> f64 {
        let mut risk = 0.0;

        // Factor 1: proportionality (max 0.3).
        if proportionality > 5.0 {
            risk += 0.3;
        } else if proportionality > 3.0 {
            risk += 0.2;
        } else if proportionality > 2.0 {
            risk += 0.1;
        }

        // Factor 2: accelerating degradation signals (max 0.3).
        let accel_count = trends
            .values()
            .filter(|&&t| t == TrendDirection::AcceleratingDegradation)
            .count();
        risk += (accel_count as f64 * 0.1).min(0.3);

        // Factor 3: response level vs max tolerable (max 0.2).
        let response_ratio = response_level / baseline.max_tolerable_response;
        risk += (response_ratio * 0.2).min(0.2);

        // Factor 4: response duration vs max allowed (max 0.2).
        if let Some(start) = self.response_start_time {
            let duration_ratio =
                start.elapsed().as_secs_f64() / baseline.max_response_duration.as_secs_f64();
            risk += (duration_ratio * 0.2).min(0.2);
        }

        risk.min(1.0)
    }

    fn determine_health_status(
        metrics: &HashMap<String, f64>,
        baseline: &Baseline,
        storm_risk: f64,
    ) -> HealthStatus {
        if storm_risk > 0.7 {
            return HealthStatus::Storm;
        }

        let deviation = baseline.calculate_overall_deviation(metrics);

        if deviation < 0.5 {
            HealthStatus::Healthy
        } else if deviation < 1.0 {
            HealthStatus::Elevated
        } else if deviation < 2.0 {
            HealthStatus::Warning
        } else if deviation < 5.0 {
            HealthStatus::Critical
        } else {
            HealthStatus::Emergency
        }
    }

    fn determine_response_phase(
        &self,
        threat_level: f64,
        response_level: f64,
        proportionality: f64,
        storm_risk: f64,
    ) -> ResponsePhase {
        if storm_risk > 0.7 {
            return ResponsePhase::Storm;
        }

        if threat_level < 0.1 && response_level < 0.1 {
            return ResponsePhase::Idle;
        }

        if threat_level > 0.0 && response_level < threat_level * 0.5 {
            return ResponsePhase::Detection;
        }

        // Check trend of the response_level metric specifically.
        if let Some(history) = self.metric_histories.get("response_level") {
            match history.get_trend(Duration::from_secs(300)) {
                TrendDirection::Improving => return ResponsePhase::Resolving,
                TrendDirection::Degrading => return ResponsePhase::Escalating,
                _ => {}
            }
        }

        if proportionality > 2.0 {
            ResponsePhase::Plateau
        } else {
            ResponsePhase::Acute
        }
    }

    fn update_response_budget(&mut self, response_level: f64) {
        // Reset budget each hour.
        if self.hour_start.elapsed() >= Duration::from_secs(3600) {
            self.total_response_this_hour = 0.0;
            self.hour_start = Instant::now();
        }
        self.total_response_this_hour += response_level;
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Ordinary least-squares slope (units per second).
fn linear_regression_slope(times: &[f64], values: &[f64]) -> f64 {
    let n = times.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let sum_t: f64 = times.iter().sum();
    let sum_v: f64 = values.iter().sum();
    let sum_tv: f64 = times.iter().zip(values).map(|(t, v)| t * v).sum();
    let sum_tt: f64 = times.iter().map(|t| t.powi(2)).sum();
    // denom = n * sum_tt - sum_t²
    let denom = n.mul_add(sum_tt, -(sum_t.powi(2)));
    if denom.abs() < f64::EPSILON {
        return 0.0;
    }
    // slope = (n * sum_tv - sum_t * sum_v) / denom
    n.mul_add(sum_tv, -(sum_t * sum_v)) / denom
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::baseline::Baseline;
    use tokio::time;

    fn baseline() -> Baseline {
        Baseline::default()
    }

    // ── MetricHistory ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn metric_history_records_and_retrieves_current() {
        let mut h = MetricHistory::new("err", 50, Duration::from_secs(3600));
        h.record(0.5);
        h.record(1.2);
        assert_eq!(h.current(), Some(1.2));
    }

    #[tokio::test]
    async fn metric_history_stable_trend_with_few_samples() {
        let mut h = MetricHistory::new("cpu", 50, Duration::from_secs(3600));
        h.record(0.4);
        h.record(0.41);
        // Only 2 samples — not enough for trend analysis.
        assert_eq!(
            h.get_trend(Duration::from_secs(3600)),
            TrendDirection::Stable
        );
    }

    #[tokio::test]
    async fn metric_history_degrading_trend() {
        let mut h = MetricHistory::new("latency", 50, Duration::from_secs(3600));
        for v in [100.0, 120.0, 150.0, 200.0, 300.0, 500.0] {
            h.record(v);
        }
        let trend = h.get_trend(Duration::from_secs(3600));
        assert!(
            matches!(
                trend,
                TrendDirection::Degrading | TrendDirection::AcceleratingDegradation
            ),
            "expected degrading trend, got {trend:?}"
        );
    }

    #[tokio::test]
    async fn metric_history_improving_trend() {
        let mut h = MetricHistory::new("errors", 50, Duration::from_secs(3600));
        for v in [100.0, 80.0, 60.0, 40.0, 20.0, 5.0] {
            h.record(v);
        }
        assert_eq!(
            h.get_trend(Duration::from_secs(3600)),
            TrendDirection::Improving
        );
    }

    #[tokio::test]
    async fn metric_history_accelerating_degradation_signature() {
        let mut h = MetricHistory::new("error_rate", 50, Duration::from_secs(3600));
        // Exponentially rising — clearly accelerating.
        for v in [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0] {
            h.record(v);
        }
        let trend = h.get_trend(Duration::from_secs(3600));
        assert_eq!(trend, TrendDirection::AcceleratingDegradation);
    }

    #[tokio::test]
    async fn metric_history_rate_of_change() {
        let mut h = MetricHistory::new("queue", 50, Duration::from_secs(3600));
        h.record(0.0);
        h.record(0.0);
        // Flat series → slope ≈ 0.
        let rate = h.calculate_rate_of_change(Duration::from_secs(3600));
        assert!(rate.abs() < 1e-6, "flat series: rate {rate}");
    }

    // ── SystemState ───────────────────────────────────────────────────────────

    #[test]
    fn system_state_needs_dampening_when_over_proportional() {
        let state = SystemState {
            timestamp: Instant::now(),
            metrics: HashMap::new(),
            health_status: HealthStatus::Warning,
            response_phase: ResponsePhase::Escalating,
            current_response_level: 50.0,
            response_duration: Duration::ZERO,
            response_budget_consumed: 0.0,
            threat_level: 10.0,
            damage_level: 0.0,
            proportionality: 5.0, // over-responding
            storm_risk: 0.1,
            trends: HashMap::new(),
            overall_deviation: 1.0,
            most_deviant_metric: None,
            notes: Vec::new(),
            dynamics: DynamicsType::Discrete,
        };
        assert!(state.needs_dampening());
        assert!(!state.needs_amplification());
    }

    #[test]
    fn system_state_needs_amplification_when_under_responding() {
        let state = SystemState {
            timestamp: Instant::now(),
            metrics: HashMap::new(),
            health_status: HealthStatus::Warning,
            response_phase: ResponsePhase::Detection,
            current_response_level: 5.0,
            response_duration: Duration::ZERO,
            response_budget_consumed: 0.0,
            threat_level: 50.0,
            damage_level: 0.0,
            proportionality: 0.1, // under-responding
            storm_risk: 0.05,
            trends: HashMap::new(),
            overall_deviation: 2.0,
            most_deviant_metric: None,
            notes: Vec::new(),
            dynamics: DynamicsType::Continuous,
        };
        assert!(state.needs_amplification());
        assert!(!state.needs_dampening());
    }

    #[test]
    fn system_state_is_in_storm_via_health_status() {
        let state = SystemState {
            timestamp: Instant::now(),
            metrics: HashMap::new(),
            health_status: HealthStatus::Storm,
            response_phase: ResponsePhase::Plateau,
            current_response_level: 100.0,
            response_duration: Duration::from_secs(600),
            response_budget_consumed: 0.0,
            threat_level: 10.0,
            damage_level: 5.0,
            proportionality: 10.0,
            storm_risk: 0.9,
            trends: HashMap::new(),
            overall_deviation: 8.0,
            most_deviant_metric: None,
            notes: Vec::new(),
            dynamics: DynamicsType::Hybrid,
        };
        assert!(state.is_in_storm());
        assert!(state.needs_dampening());
    }

    // ── StateTracker ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn state_tracker_healthy_at_baseline() {
        let mut tracker =
            StateTracker::new(vec!["error_rate".into()], Duration::from_secs(3600), 100);
        let b = baseline();
        let metrics = [("error_rate".to_string(), 0.001)].into_iter().collect();
        let state = tracker.update_state(metrics, 0.0, 0.0, 0.0, &b);
        assert_eq!(state.health_status, HealthStatus::Healthy);
        assert_eq!(state.response_phase, ResponsePhase::Idle);
    }

    #[tokio::test]
    async fn state_tracker_critical_on_high_deviation() {
        let mut tracker =
            StateTracker::new(vec!["error_rate".into()], Duration::from_secs(3600), 100);
        let b = baseline();
        // error_rate far above absolute_maximum → deviation >> 5 → Emergency.
        let metrics = [("error_rate".to_string(), 0.9)].into_iter().collect();
        let state = tracker.update_state(metrics, 90.0, 0.0, 0.0, &b);
        assert!(
            matches!(
                state.health_status,
                HealthStatus::Critical | HealthStatus::Emergency
            ),
            "got {:?}",
            state.health_status
        );
    }

    #[tokio::test]
    async fn state_tracker_proportionality_calculated() {
        let mut tracker = StateTracker::new(vec![], Duration::from_secs(3600), 100);
        let b = baseline();
        let state = tracker.update_state(HashMap::new(), 10.0, 0.0, 50.0, &b);
        // proportionality = 50 / 10 = 5.
        assert!(
            (state.proportionality - 5.0).abs() < 0.01,
            "{}",
            state.proportionality
        );
    }

    #[tokio::test]
    async fn state_tracker_current_state_persists() {
        let mut tracker = StateTracker::new(vec![], Duration::from_secs(3600), 100);
        let b = baseline();
        assert!(tracker.current_state().is_none());
        tracker.update_state(HashMap::new(), 0.0, 0.0, 0.0, &b);
        assert!(tracker.current_state().is_some());
    }

    #[tokio::test]
    async fn state_tracker_storm_risk_from_high_proportionality() {
        let mut tracker = StateTracker::new(vec![], Duration::from_secs(3600), 100);
        let b = baseline();
        // Extremely high proportionality → storm risk factor kicks in.
        let state = tracker.update_state(HashMap::new(), 1.0, 0.0, 100.0, &b);
        // proportionality = 100 → > 5 → risk += 0.3
        assert!(state.storm_risk >= 0.3, "storm_risk: {}", state.storm_risk);
    }
}
