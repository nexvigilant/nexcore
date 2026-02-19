//! Decaying signals and SignalManager — enforces Law 2: every signal must decay.
//!
//! Every signal carries a `half_life` duration. No signal persists forever. This
//! mirrors biological cytokine degradation: without decay, signals accumulate and
//! tip the system into storm.
//!
//! The decay follows one of four mathematical profiles:
//!
//! - **Exponential**: `v(t) = v₀ × 0.5^(t/half_life)` — most biologically realistic
//! - **Linear**: `v(t) = v₀ × max(0, 1 − t/(2×half_life))` — full decay at 2× half-life
//! - **Step**: `v₀` until `t = half_life`, then zero — binary expiration
//! - **Sigmoid**: slow start, fast middle, slow tail — smooth S-curve decay
//!
//! ## Example
//!
//! ```
//! use std::time::Duration;
//! use nexcore_homeostasis_primitives::signals::{DecayingSignal, SignalManager};
//! use nexcore_homeostasis_primitives::enums::{SignalType, DecayFunction};
//!
//! let signal = DecayingSignal::new(
//!     SignalType::Threat,
//!     "error_detector",
//!     100.0,
//!     Duration::from_secs(300),
//!     DecayFunction::Exponential,
//! );
//! // Signal is at full strength immediately after creation.
//! assert!(signal.current_value() > 99.0);
//! assert!(signal.is_significant());
//! ```

use crate::enums::{DecayFunction, SignalType};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::Instant;

// =============================================================================
// DecayingSignal
// =============================================================================

/// A signal that naturally decays over time.
///
/// Every `DecayingSignal` starts at `initial_value` and fades according to
/// `decay_function`. Below `minimum_significant_value` the signal is considered
/// expired and will be pruned by [`SignalManager::tick`].
///
/// Repeated stimulation from the same source is handled by [`boost`](Self::boost),
/// which adds to the current (decayed) value then resets the timestamp — modelling
/// receptor re-stimulation. The ceiling `max_boost_value` prevents runaway
/// accumulation (receptor saturation / downregulation).
///
/// ```
/// use std::time::Duration;
/// use nexcore_homeostasis_primitives::signals::DecayingSignal;
/// use nexcore_homeostasis_primitives::enums::{SignalType, DecayFunction};
///
/// let mut sig = DecayingSignal::new(
///     SignalType::Damage,
///     "db_timeouts",
///     50.0,
///     Duration::from_secs(60),
///     DecayFunction::Exponential,
/// );
///
/// // Boost adds to the current value, capped at max_boost_value (500).
/// sig.boost(30.0);
/// assert!(sig.current_value() <= 500.0);
/// ```
#[derive(Clone, Debug)]
pub struct DecayingSignal {
    /// Category of this signal.
    pub signal_type: SignalType,
    /// Identifier of the component that produced this signal.
    pub source: String,
    /// Strength at the moment the signal was last reset/boosted.
    pub initial_value: f64,
    /// When the signal was last created or boosted.
    pub created_at: Instant,
    /// Duration after which value halves (exponential) or context-specific meaning.
    pub half_life: Duration,
    /// Mathematical shape of the decay curve.
    pub decay_function: DecayFunction,
    /// Signals below this value are considered expired.
    pub minimum_significant_value: f64,
    /// Maximum value reachable via [`boost`](Self::boost) (receptor saturation).
    pub max_boost_value: f64,
    /// Arbitrary metadata passed with the signal.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl DecayingSignal {
    /// Create a new `DecayingSignal` at full strength.
    ///
    /// The `created_at` timestamp is set to [`Instant::now`] at construction,
    /// so in tests you should call this *after* `tokio::time::pause()` and any
    /// initial `tokio::time::advance()` to control the timeline.
    pub fn new(
        signal_type: SignalType,
        source: impl Into<String>,
        initial_value: f64,
        half_life: Duration,
        decay_function: DecayFunction,
    ) -> Self {
        Self {
            signal_type,
            source: source.into(),
            initial_value,
            created_at: Instant::now(),
            half_life,
            decay_function,
            minimum_significant_value: 0.01,
            max_boost_value: 500.0,
            metadata: HashMap::new(),
        }
    }

    /// Age of this signal in seconds (uses tokio time, controllable in tests).
    pub fn age_secs(&self) -> f64 {
        self.created_at.elapsed().as_secs_f64()
    }

    /// Current decayed value.
    ///
    /// Returns a value in `[0, initial_value]`.
    pub fn current_value(&self) -> f64 {
        let age = self.age_secs();
        let half_life_secs = self.half_life.as_secs_f64();

        if half_life_secs <= 0.0 {
            return self.initial_value;
        }

        let decay_factor = match self.decay_function {
            DecayFunction::Exponential => {
                // v(t) = v₀ × 0.5^(t / h)
                0.5_f64.powf(age / half_life_secs)
            }
            DecayFunction::Linear => {
                // Full decay at 2× half_life
                (1.0 - age / (half_life_secs * 2.0)).max(0.0)
            }
            DecayFunction::Step => {
                // Full value until half_life, then zero
                if age < half_life_secs { 1.0 } else { 0.0 }
            }
            DecayFunction::Sigmoid => {
                // S-curve: slow start, fast middle, slow tail
                // x = (t / half_life) − 1  →  factor = 1 / (1 + e^(3x))
                let x = (age / half_life_secs) - 1.0;
                1.0 / (1.0 + (3.0 * x).exp())
            }
        };

        self.initial_value * decay_factor
    }

    /// Whether the signal is still above `minimum_significant_value`.
    pub fn is_significant(&self) -> bool {
        self.current_value() >= self.minimum_significant_value
    }

    /// Whether the signal has decayed below significance.
    pub fn is_expired(&self) -> bool {
        !self.is_significant()
    }

    /// Estimated remaining lifetime before the signal falls below significance.
    ///
    /// For exponential decay this is exact; for other functions it is an
    /// approximation of one additional `half_life`.
    pub fn remaining_lifetime(&self) -> Duration {
        let current = self.current_value();
        if current <= self.minimum_significant_value {
            return Duration::ZERO;
        }
        match self.decay_function {
            DecayFunction::Exponential => {
                let half_life_secs = self.half_life.as_secs_f64();
                let ratio = current / self.minimum_significant_value;
                // t_remaining = half_life × log₂(ratio)
                let remaining_secs = half_life_secs * ratio.log2();
                Duration::from_secs_f64(remaining_secs.max(0.0))
            }
            _ => self.half_life,
        }
    }

    /// Re-stimulate this signal by adding `amount` to the current (decayed) value.
    ///
    /// Models receptor re-stimulation: the signal is refreshed from its current
    /// decayed value rather than from zero. The result is capped at
    /// `max_boost_value` and the timestamp is reset so decay restarts from the
    /// new peak.
    pub fn boost(&mut self, amount: f64) {
        let current = self.current_value();
        let new_value = (current + amount).min(self.max_boost_value);
        self.initial_value = new_value;
        self.created_at = Instant::now();
    }
}

// =============================================================================
// SignalStats
// =============================================================================

/// Summary statistics from [`SignalManager::get_statistics`].
#[derive(Clone, Debug)]
pub struct SignalStats {
    /// Total number of active (not yet expired) signals.
    pub total_signals: usize,
    /// Per-type count of active signals.
    pub signals_by_type: HashMap<String, usize>,
    /// Sum of current decayed values per type.
    pub strength_by_type: HashMap<String, f64>,
    /// Combined decayed value across all signals.
    pub total_strength: f64,
    /// `(threat + damage + response) − dampening`.
    pub net_inflammatory_state: f64,
    /// Source of the currently strongest signal, if any.
    pub strongest_signal_source: Option<String>,
}

// =============================================================================
// SignalManager
// =============================================================================

/// Central registry for all decaying signals.
///
/// `SignalManager` tracks every active signal, prunes expired ones on each
/// [`tick`](Self::tick), and provides aggregate strength queries by type.
///
/// When a new signal arrives from a source that already has an active signal of
/// the same type, the existing signal is *boosted* rather than duplicated.  This
/// mirrors biological receptor re-stimulation and prevents artificial inflation
/// of the signal pool.
///
/// The canonical ID for a signal is `"<type>:<source>"` — returned by
/// [`add_signal`](Self::add_signal) and [`create_signal`](Self::create_signal).
///
/// ```
/// use std::time::Duration;
/// use nexcore_homeostasis_primitives::signals::{DecayingSignal, SignalManager};
/// use nexcore_homeostasis_primitives::enums::{SignalType, DecayFunction};
///
/// let mut mgr = SignalManager::default();
/// let id = mgr.create_signal(SignalType::Threat, "probe", 50.0, None);
/// assert!(mgr.get_threat_level() > 0.0);
/// assert!(mgr.get_signal(&id).is_some());
/// ```
#[derive(Debug)]
pub struct SignalManager {
    /// Default half-life for signals created via [`create_signal`](Self::create_signal).
    pub default_half_life: Duration,
    /// Cleanup is triggered when signals exceed this count.
    pub cleanup_threshold: usize,
    /// How often cleanup runs (minimum elapsed time between cleanup passes).
    pub cleanup_interval: Duration,
    signals: HashMap<String, DecayingSignal>,
    last_cleanup: Instant,
}

impl Default for SignalManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(300), Duration::from_secs(30), 10_000)
    }
}

impl SignalManager {
    /// Create a `SignalManager` with explicit configuration.
    ///
    /// - `default_half_life` — used by [`create_signal`](Self::create_signal) when
    ///   no `half_life` override is provided.
    /// - `cleanup_interval` — minimum time between automatic cleanup passes.
    /// - `max_signals` — triggers an immediate cleanup pass when exceeded.
    pub fn new(
        default_half_life: Duration,
        cleanup_interval: Duration,
        max_signals: usize,
    ) -> Self {
        Self {
            default_half_life,
            cleanup_threshold: max_signals,
            cleanup_interval,
            signals: HashMap::new(),
            last_cleanup: Instant::now(),
        }
    }

    /// Add a pre-built [`DecayingSignal`] to the manager.
    ///
    /// If a signal with the same `"<type>:<source>"` key already exists, the
    /// existing signal is boosted by `signal.initial_value` instead.
    ///
    /// Returns the canonical signal ID.
    pub fn add_signal(&mut self, signal: DecayingSignal) -> String {
        let signal_id = format!("{}:{}", signal.signal_type as u8, signal.source);

        if let Some(existing) = self.signals.get_mut(&signal_id) {
            existing.boost(signal.initial_value);
            tracing::debug!(signal_id = %signal_id, "boosted existing signal");
            return signal_id;
        }

        if self.signals.len() >= self.cleanup_threshold {
            self.cleanup_expired();
        }

        self.signals.insert(signal_id.clone(), signal);
        signal_id
    }

    /// Convenience: create and register a signal in one step.
    ///
    /// Uses `half_life` if provided, otherwise falls back to
    /// `self.default_half_life`. Returns the signal ID.
    pub fn create_signal(
        &mut self,
        signal_type: SignalType,
        source: impl Into<String>,
        value: f64,
        half_life: Option<Duration>,
    ) -> String {
        let hl = half_life.unwrap_or(self.default_half_life);
        let signal = DecayingSignal::new(signal_type, source, value, hl, DecayFunction::Exponential);
        self.add_signal(signal)
    }

    /// Look up a signal by its canonical ID.
    pub fn get_signal(&self, signal_id: &str) -> Option<&DecayingSignal> {
        self.signals.get(signal_id)
    }

    /// Explicitly remove a signal.  Returns `true` if the signal existed.
    pub fn remove_signal(&mut self, signal_id: &str) -> bool {
        self.signals.remove(signal_id).is_some()
    }

    /// Advance the clock: prune expired signals if the cleanup interval has elapsed.
    ///
    /// Call this from the control loop on each iteration.
    pub fn tick(&mut self) {
        if self.last_cleanup.elapsed() >= self.cleanup_interval {
            self.cleanup_expired();
            self.last_cleanup = Instant::now();
        }
    }

    /// Force an immediate cleanup pass, removing all expired signals.
    pub fn cleanup_expired(&mut self) {
        let expired: Vec<String> = self
            .signals
            .iter()
            .filter(|(_, s)| s.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        let count = expired.len();
        for id in expired {
            self.signals.remove(&id);
        }

        if count > 0 {
            tracing::debug!(count, "cleaned up expired signals");
        }
    }

    /// Total decayed strength across all signals, optionally filtered by type.
    ///
    /// Pass `None` to aggregate all types.
    pub fn get_total_signal_strength(&self, signal_type: Option<SignalType>) -> f64 {
        self.signals
            .values()
            .filter(|s| signal_type.is_none_or(|t| s.signal_type == t))
            .map(DecayingSignal::current_value)
            .sum()
    }

    /// All significant signals of the given type.
    pub fn get_signals_by_type(&self, signal_type: SignalType) -> Vec<&DecayingSignal> {
        self.signals
            .values()
            .filter(|s| s.signal_type == signal_type && s.is_significant())
            .collect()
    }

    /// All significant signals from the given source.
    pub fn get_signals_by_source(&self, source: &str) -> Vec<&DecayingSignal> {
        self.signals
            .values()
            .filter(|s| s.source == source && s.is_significant())
            .collect()
    }

    /// The signal with the highest current (decayed) value, if any.
    pub fn get_strongest_signal(&self) -> Option<&DecayingSignal> {
        self.signals
            .values()
            .max_by(|a, b| a.current_value().partial_cmp(&b.current_value()).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Sum of all `Threat` signal strengths.
    pub fn get_threat_level(&self) -> f64 {
        self.get_total_signal_strength(Some(SignalType::Threat))
    }

    /// Sum of all `Damage` signal strengths.
    pub fn get_damage_level(&self) -> f64 {
        self.get_total_signal_strength(Some(SignalType::Damage))
    }

    /// Sum of all `Response` signal strengths.
    pub fn get_response_level(&self) -> f64 {
        self.get_total_signal_strength(Some(SignalType::Response))
    }

    /// Sum of all `Dampening` signal strengths.
    pub fn get_dampening_level(&self) -> f64 {
        self.get_total_signal_strength(Some(SignalType::Dampening))
    }

    /// Net inflammatory state: `(threat + damage + response) − dampening`.
    ///
    /// A positive value means pro-inflammatory signals dominate; approaching
    /// storm as the value climbs.
    pub fn get_net_inflammatory_state(&self) -> f64 {
        let pro = self.get_threat_level() + self.get_damage_level() + self.get_response_level();
        let anti = self.get_dampening_level();
        pro - anti
    }

    /// Remove all signals (emergency reset).
    pub fn clear_all(&mut self) {
        self.signals.clear();
        tracing::warn!("all signals cleared — emergency reset");
    }

    /// Snapshot of current signal statistics.
    pub fn get_statistics(&self) -> SignalStats {
        let mut signals_by_type: HashMap<String, usize> = HashMap::new();
        let mut strength_by_type: HashMap<String, f64> = HashMap::new();
        let mut total_strength = 0.0;

        for signal in self.signals.values() {
            let type_key = format!("{:?}", signal.signal_type).to_lowercase();
            let cv = signal.current_value();
            *signals_by_type.entry(type_key.clone()).or_insert(0) += 1;
            *strength_by_type.entry(type_key).or_insert(0.0) += cv;
            total_strength += cv;
        }

        SignalStats {
            total_signals: self.signals.len(),
            signals_by_type,
            strength_by_type,
            total_strength,
            net_inflammatory_state: self.get_net_inflammatory_state(),
            strongest_signal_source: self
                .get_strongest_signal()
                .map(|s| s.source.clone()),
        }
    }

    /// Number of currently tracked signals (including expired not yet pruned).
    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time;

    // ── DecayingSignal ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn exponential_half_life_halves_value() {
        time::pause();
        let sig = DecayingSignal::new(
            SignalType::Threat,
            "test",
            100.0,
            Duration::from_secs(60),
            DecayFunction::Exponential,
        );
        // At t=0 value should be at full strength.
        assert!((sig.current_value() - 100.0).abs() < 0.01, "t=0: {}", sig.current_value());

        // After exactly one half-life the value should be ≈50.
        time::advance(Duration::from_secs(60)).await;
        let v = sig.current_value();
        assert!((v - 50.0).abs() < 0.5, "t=half_life: expected ~50, got {v}");

        // After two half-lives ≈25.
        time::advance(Duration::from_secs(60)).await;
        let v2 = sig.current_value();
        assert!((v2 - 25.0).abs() < 0.5, "t=2×half_life: expected ~25, got {v2}");
    }

    #[tokio::test]
    async fn linear_decay_reaches_zero_at_double_half_life() {
        time::pause();
        let sig = DecayingSignal::new(
            SignalType::Damage,
            "src",
            100.0,
            Duration::from_secs(30),
            DecayFunction::Linear,
        );
        // At 2× half_life (60 s) value should be 0.
        time::advance(Duration::from_secs(60)).await;
        assert!(sig.current_value() < 0.001, "linear at 2× half_life: {}", sig.current_value());
    }

    #[tokio::test]
    async fn step_decay_full_then_zero() {
        time::pause();
        let sig = DecayingSignal::new(
            SignalType::Response,
            "src",
            80.0,
            Duration::from_secs(10),
            DecayFunction::Step,
        );
        // Before half_life: full value.
        time::advance(Duration::from_secs(9)).await;
        assert!((sig.current_value() - 80.0).abs() < 0.01);

        // After half_life: zero.
        time::advance(Duration::from_secs(2)).await;
        assert!(sig.current_value() < 0.001, "step after half_life: {}", sig.current_value());
    }

    #[tokio::test]
    async fn sigmoid_decay_monotonic() {
        time::pause();
        let sig = DecayingSignal::new(
            SignalType::Threat,
            "src",
            100.0,
            Duration::from_secs(30),
            DecayFunction::Sigmoid,
        );
        let mut prev = sig.current_value();
        for _ in 0..10 {
            time::advance(Duration::from_secs(10)).await;
            let cur = sig.current_value();
            assert!(cur <= prev + 0.001, "sigmoid increased: {cur} > {prev}");
            prev = cur;
        }
    }

    #[tokio::test]
    async fn boost_resets_timestamp_and_adds_value() {
        time::pause();
        let mut sig = DecayingSignal::new(
            SignalType::Threat,
            "probe",
            50.0,
            Duration::from_secs(60),
            DecayFunction::Exponential,
        );
        // Advance so value decays to ≈25.
        time::advance(Duration::from_secs(60)).await;
        let decayed = sig.current_value();
        assert!((decayed - 25.0).abs() < 1.0, "decayed: {decayed}");

        // Boost by 30 → new initial ≈ 55.
        sig.boost(30.0);
        let after_boost = sig.current_value();
        // Immediately after boost, age is ~0, so current_value ≈ new initial.
        assert!((after_boost - 55.0).abs() < 1.0, "after boost: {after_boost}");
    }

    #[tokio::test]
    async fn boost_respects_ceiling() {
        time::pause();
        let mut sig = DecayingSignal::new(
            SignalType::Threat,
            "src",
            400.0,
            Duration::from_secs(60),
            DecayFunction::Exponential,
        );
        sig.boost(200.0); // would be 600 without ceiling
        assert!(sig.current_value() <= 500.0);
    }

    #[tokio::test]
    async fn is_significant_and_expired() {
        time::pause();
        let sig = DecayingSignal::new(
            SignalType::Dampening,
            "src",
            1.0,
            Duration::from_secs(10),
            DecayFunction::Step,
        );
        assert!(sig.is_significant());

        // After half_life the step function drops to zero.
        time::advance(Duration::from_secs(11)).await;
        assert!(sig.is_expired());
    }

    #[tokio::test]
    async fn remaining_lifetime_exponential() {
        time::pause();
        let sig = DecayingSignal::new(
            SignalType::Threat,
            "src",
            100.0,
            Duration::from_secs(60),
            DecayFunction::Exponential,
        );
        // At t=0, remaining lifetime should be a positive, finite duration.
        let remaining = sig.remaining_lifetime();
        assert!(remaining > Duration::ZERO);
        assert!(remaining < Duration::from_secs(10_000));
    }

    // ── SignalManager ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn manager_add_signal_and_retrieve() {
        time::pause();
        let mut mgr = SignalManager::default();
        let sig = DecayingSignal::new(
            SignalType::Threat,
            "web_scanner",
            75.0,
            Duration::from_secs(120),
            DecayFunction::Exponential,
        );
        let id = mgr.add_signal(sig);
        assert!(mgr.get_signal(&id).is_some());
        assert!((mgr.get_threat_level() - 75.0).abs() < 1.0);
    }

    #[tokio::test]
    async fn manager_duplicate_source_boosts_not_duplicates() {
        time::pause();
        let mut mgr = SignalManager::default();
        let id1 = mgr.create_signal(SignalType::Threat, "probe", 50.0, None);
        let id2 = mgr.create_signal(SignalType::Threat, "probe", 30.0, None);
        // Same source → same ID, signal was boosted, not duplicated.
        assert_eq!(id1, id2);
        assert_eq!(mgr.signal_count(), 1);
        // Value should be boosted above 50.
        let level = mgr.get_threat_level();
        assert!(level > 50.0, "expected boosted level > 50, got {level}");
    }

    #[tokio::test]
    async fn manager_remove_signal() {
        time::pause();
        let mut mgr = SignalManager::default();
        let id = mgr.create_signal(SignalType::Damage, "disk", 20.0, None);
        assert!(mgr.remove_signal(&id));
        assert!(!mgr.remove_signal(&id)); // second remove returns false
        assert_eq!(mgr.signal_count(), 0);
    }

    #[tokio::test]
    async fn manager_tick_prunes_expired() {
        time::pause();
        // Very short half-life so signal expires quickly.
        let mut mgr = SignalManager::new(
            Duration::from_secs(1),
            Duration::from_millis(10), // cleanup interval
            10_000,
        );
        mgr.create_signal(
            SignalType::Threat,
            "src",
            100.0,
            Some(Duration::from_millis(1)), // step: expires after 1 ms
        );
        assert_eq!(mgr.signal_count(), 1);

        // Advance past cleanup interval + signal expiry.
        time::advance(Duration::from_millis(50)).await;
        mgr.tick();
        assert_eq!(mgr.signal_count(), 0, "expired signal should have been pruned");
    }

    #[tokio::test]
    async fn manager_net_inflammatory_state() {
        time::pause();
        let mut mgr = SignalManager::default();
        mgr.create_signal(SignalType::Threat, "t1", 60.0, None);
        mgr.create_signal(SignalType::Dampening, "d1", 20.0, None);
        let net = mgr.get_net_inflammatory_state();
        // 60 − 20 = 40 (approximately, before any time passes).
        assert!((net - 40.0).abs() < 1.0, "net inflammatory: {net}");
    }

    #[tokio::test]
    async fn manager_statistics_structure() {
        time::pause();
        let mut mgr = SignalManager::default();
        mgr.create_signal(SignalType::Threat, "s1", 10.0, None);
        mgr.create_signal(SignalType::Damage, "s2", 5.0, None);
        let stats = mgr.get_statistics();
        assert_eq!(stats.total_signals, 2);
        assert!(stats.total_strength > 0.0);
        assert!(stats.strongest_signal_source.is_some());
    }
}
