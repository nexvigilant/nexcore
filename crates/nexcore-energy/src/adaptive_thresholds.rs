//! # Adaptive Signal Thresholds — Energy + Density Governed PV Thresholds
//!
//! Signal detection thresholds that self-regulate based on:
//! 1. **Energy regime** — Crisis raises thresholds (conserve), Anabolic lowers them (explore)
//! 2. **Signal density** — High signal rate raises thresholds to reduce noise
//!
//! This creates a homeostatic feedback loop: more signals → higher threshold →
//! fewer low-quality signals → threshold relaxes → system stabilizes.
//!
//! ## Innovation Scan 001 — Goal 3 (Score: 7.95)
//!
//! ```text
//! base_threshold × energy_factor(EC) × density_factor(history) = adaptive_threshold
//! ```
//!
//! ## ToV Alignment: V4 Safety Manifold
//! d(s) > 0 — thresholds never drop below safety floor regardless of energy state.
//! Patient safety is the invariant membrane that adaptation cannot cross.
//!
//! ## Tier: T2-C (N + ∂ + ς + ν + →)

use crate::Regime;
use serde::{Deserialize, Serialize};
use std::fmt;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Absolute minimum PRR threshold — safety floor (ToV V4).
/// No adaptation can push the threshold below this.
pub const PRR_SAFETY_FLOOR: f64 = 1.0;

/// Absolute minimum ROR CI lower bound threshold.
pub const ROR_CI_SAFETY_FLOOR: f64 = 0.5;

/// Default signal density window (number of recent signals to consider).
pub const DEFAULT_HISTORY_WINDOW: usize = 100;

/// Density scaling factor — controls how aggressively density raises thresholds.
/// Higher values = more aggressive noise suppression.
pub const DENSITY_SCALING: f64 = 0.1;

/// Maximum density multiplier — caps how high density can push a threshold.
pub const MAX_DENSITY_FACTOR: f64 = 2.0;

// ─── Threshold Policy ────────────────────────────────────────────────────────

/// How thresholds adapt to runtime conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThresholdPolicy {
    /// Fixed threshold — no adaptation.
    Fixed,
    /// Adapts based on energy regime only.
    EnergyAdaptive,
    /// Adapts based on signal density only.
    DensityAdaptive,
    /// Full adaptation: energy + density combined.
    FullAdaptive,
}

impl Default for ThresholdPolicy {
    fn default() -> Self {
        Self::FullAdaptive
    }
}

impl fmt::Display for ThresholdPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed => write!(f, "Fixed"),
            Self::EnergyAdaptive => write!(f, "Energy-Adaptive"),
            Self::DensityAdaptive => write!(f, "Density-Adaptive"),
            Self::FullAdaptive => write!(f, "Full-Adaptive"),
        }
    }
}

// ─── Signal History ──────────────────────────────────────────────────────────

/// Tracks recent signal detection rates for density-based adaptation.
///
/// Uses a bounded ring buffer of signal outcomes (detected/not-detected)
/// to compute a rolling density rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalHistory {
    /// Recent signal outcomes: true = signal detected, false = no signal.
    outcomes: Vec<bool>,
    /// Maximum window size.
    window: usize,
    /// Write position in the ring buffer.
    position: usize,
    /// Whether the buffer has wrapped (all slots used at least once).
    wrapped: bool,
}

impl SignalHistory {
    /// Create a new empty signal history with the given window size.
    #[must_use]
    pub fn new(window: usize) -> Self {
        let window = window.max(1);
        Self {
            outcomes: vec![false; window],
            window,
            position: 0,
            wrapped: false,
        }
    }

    /// Create with default window size.
    #[must_use]
    pub fn default_window() -> Self {
        Self::new(DEFAULT_HISTORY_WINDOW)
    }

    /// Record a signal detection outcome.
    pub fn record(&mut self, signal_detected: bool) {
        self.outcomes[self.position] = signal_detected;
        self.position += 1;
        if self.position >= self.window {
            self.position = 0;
            self.wrapped = true;
        }
    }

    /// Record multiple outcomes at once.
    pub fn record_batch(&mut self, detected_count: usize, total_count: usize) {
        for i in 0..total_count {
            self.record(i < detected_count);
        }
    }

    /// How many observations are in the active window.
    #[must_use]
    pub fn active_count(&self) -> usize {
        if self.wrapped {
            self.window
        } else {
            self.position
        }
    }

    /// Signal density rate: proportion of recent observations that were signals.
    /// Returns 0.0 if no observations yet.
    #[must_use]
    pub fn density_rate(&self) -> f64 {
        let count = self.active_count();
        if count == 0 {
            return 0.0;
        }

        let signal_count = if self.wrapped {
            self.outcomes.iter().filter(|&&s| s).count()
        } else {
            self.outcomes[..self.position].iter().filter(|&&s| s).count()
        };

        signal_count as f64 / count as f64
    }

    /// Reset all history.
    pub fn reset(&mut self) {
        self.outcomes.fill(false);
        self.position = 0;
        self.wrapped = false;
    }
}

// ─── Energy Factor ───────────────────────────────────────────────────────────

/// Compute the energy-based threshold multiplier.
///
/// - **Crisis** → 1.50x (raise threshold, only strong signals pass)
/// - **Catabolic** → 1.20x (slightly raised, conserve analysis budget)
/// - **Homeostatic** → 1.00x (nominal — no adjustment)
/// - **Anabolic** → 0.75x (lower threshold, explore more signals)
#[must_use]
pub fn energy_factor(regime: Regime) -> f64 {
    match regime {
        Regime::Crisis => 1.50,
        Regime::Catabolic => 1.20,
        Regime::Homeostatic => 1.00,
        Regime::Anabolic => 0.75,
    }
}

/// Compute energy factor directly from energy charge.
#[must_use]
pub fn energy_factor_from_ec(ec: f64) -> f64 {
    energy_factor(Regime::from_ec(ec))
}

// ─── Density Factor ──────────────────────────────────────────────────────────

/// Compute the density-based threshold multiplier.
///
/// Higher signal density → higher threshold to suppress noise.
/// `factor = 1.0 + (density_rate × DENSITY_SCALING × 10)`, capped at MAX_DENSITY_FACTOR.
///
/// At 50% density: factor = 1.0 + (0.5 × 0.1 × 10) = 1.5
/// At 0% density: factor = 1.0 (no adjustment)
/// At 100% density: factor = 2.0 (maximum suppression)
#[must_use]
pub fn density_factor(history: &SignalHistory) -> f64 {
    let rate = history.density_rate();
    let factor = 1.0 + (rate * DENSITY_SCALING * 10.0);
    factor.min(MAX_DENSITY_FACTOR)
}

// ─── Adaptive Threshold ──────────────────────────────────────────────────────

/// Configuration for an adaptive threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveThresholdConfig {
    /// Base threshold value (e.g., PRR = 2.0).
    pub base: f64,
    /// Safety floor — threshold never drops below this (ToV V4).
    pub safety_floor: f64,
    /// Adaptation policy.
    pub policy: ThresholdPolicy,
    /// Algorithm name (for display/logging).
    pub algorithm: String,
}

impl AdaptiveThresholdConfig {
    /// Create a PRR threshold config with standard defaults.
    #[must_use]
    pub fn prr() -> Self {
        Self {
            base: 2.0,
            safety_floor: PRR_SAFETY_FLOOR,
            policy: ThresholdPolicy::FullAdaptive,
            algorithm: "PRR".to_string(),
        }
    }

    /// Create an ROR CI lower bound threshold config.
    #[must_use]
    pub fn ror_ci() -> Self {
        Self {
            base: 1.0,
            safety_floor: ROR_CI_SAFETY_FLOOR,
            policy: ThresholdPolicy::FullAdaptive,
            algorithm: "ROR_CI_Lower".to_string(),
        }
    }

    /// Create a Chi-square threshold config.
    #[must_use]
    pub fn chi_square() -> Self {
        Self {
            base: 3.841,
            safety_floor: 2.706,
            policy: ThresholdPolicy::FullAdaptive,
            algorithm: "ChiSquare".to_string(),
        }
    }

    /// Create a custom threshold config.
    #[must_use]
    pub fn custom(algorithm: impl Into<String>, base: f64, floor: f64) -> Self {
        Self {
            base,
            safety_floor: floor,
            policy: ThresholdPolicy::FullAdaptive,
            algorithm: algorithm.into(),
        }
    }

    /// Override the policy.
    #[must_use]
    pub fn with_policy(mut self, policy: ThresholdPolicy) -> Self {
        self.policy = policy;
        self
    }
}

// ─── Adaptation Result ───────────────────────────────────────────────────────

/// Result of a threshold adaptation computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptedThreshold {
    /// The adapted threshold value.
    pub value: f64,
    /// Base threshold before adaptation.
    pub base: f64,
    /// Energy factor applied.
    pub energy_factor: f64,
    /// Density factor applied.
    pub density_factor: f64,
    /// Whether the safety floor was hit.
    pub floor_applied: bool,
    /// The regime that drove the energy factor.
    pub regime: Regime,
    /// Current signal density rate.
    pub density_rate: f64,
    /// Algorithm name.
    pub algorithm: String,
    /// Policy used.
    pub policy: ThresholdPolicy,
}

impl fmt::Display for AdaptedThreshold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {:.3} (base={:.3}, energy={:.2}x, density={:.2}x, floor={})",
            self.algorithm,
            self.value,
            self.base,
            self.energy_factor,
            self.density_factor,
            self.floor_applied,
        )
    }
}

// ─── Core Adaptation Function ────────────────────────────────────────────────

/// Compute an adapted threshold based on config, energy charge, and signal history.
///
/// ```text
/// adapted = max(base × energy_factor × density_factor, safety_floor)
/// ```
///
/// The safety floor is the ToV V4 invariant — patient safety membrane
/// that no adaptation can penetrate.
///
/// # Example
///
/// ```
/// use nexcore_energy::adaptive_thresholds::*;
///
/// let config = AdaptiveThresholdConfig::prr();
/// let history = SignalHistory::default_window();
///
/// let result = adapt(&config, 0.75, &history);
/// assert!((result.value - 2.0).abs() < f64::EPSILON); // Homeostatic = no change
/// assert!(!result.floor_applied);
/// ```
#[must_use]
pub fn adapt(
    config: &AdaptiveThresholdConfig,
    energy_charge: f64,
    history: &SignalHistory,
) -> AdaptedThreshold {
    let regime = Regime::from_ec(energy_charge);
    let e_factor = match config.policy {
        ThresholdPolicy::Fixed | ThresholdPolicy::DensityAdaptive => 1.0,
        ThresholdPolicy::EnergyAdaptive | ThresholdPolicy::FullAdaptive => energy_factor(regime),
    };
    let d_factor = match config.policy {
        ThresholdPolicy::Fixed | ThresholdPolicy::EnergyAdaptive => 1.0,
        ThresholdPolicy::DensityAdaptive | ThresholdPolicy::FullAdaptive => density_factor(history),
    };

    let raw = config.base * e_factor * d_factor;
    let floor_applied = raw < config.safety_floor;
    let value = raw.max(config.safety_floor);

    AdaptedThreshold {
        value,
        base: config.base,
        energy_factor: e_factor,
        density_factor: d_factor,
        floor_applied,
        regime,
        density_rate: history.density_rate(),
        algorithm: config.algorithm.clone(),
        policy: config.policy,
    }
}

/// Adapt all standard PV thresholds at once.
///
/// Returns adapted thresholds for PRR, ROR CI, and Chi-square.
#[must_use]
pub fn adapt_all_pv(
    energy_charge: f64,
    history: &SignalHistory,
) -> Vec<AdaptedThreshold> {
    let configs = [
        AdaptiveThresholdConfig::prr(),
        AdaptiveThresholdConfig::ror_ci(),
        AdaptiveThresholdConfig::chi_square(),
    ];

    configs
        .iter()
        .map(|c| adapt(c, energy_charge, history))
        .collect()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Signal History ────────────────────────────────────────────────────

    #[test]
    fn test_empty_history_zero_density() {
        let h = SignalHistory::new(10);
        assert!((h.density_rate() - 0.0).abs() < f64::EPSILON);
        assert_eq!(h.active_count(), 0);
    }

    #[test]
    fn test_history_records_outcomes() {
        let mut h = SignalHistory::new(10);
        h.record(true);
        h.record(false);
        h.record(true);
        assert_eq!(h.active_count(), 3);
        assert!((h.density_rate() - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_history_wraps_correctly() {
        let mut h = SignalHistory::new(4);
        // Fill: [T, T, F, F]
        h.record(true);
        h.record(true);
        h.record(false);
        h.record(false);
        assert!(h.wrapped);
        assert_eq!(h.active_count(), 4);
        assert!((h.density_rate() - 0.5).abs() < f64::EPSILON);

        // Overwrite first slot: [F, T, F, F] → density = 1/4
        h.record(false);
        assert!((h.density_rate() - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_history_batch_record() {
        let mut h = SignalHistory::new(20);
        h.record_batch(3, 10); // 3 detected out of 10
        assert_eq!(h.active_count(), 10);
        assert!((h.density_rate() - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_history_reset() {
        let mut h = SignalHistory::new(10);
        h.record(true);
        h.record(true);
        h.reset();
        assert_eq!(h.active_count(), 0);
        assert!((h.density_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_history_minimum_window_is_one() {
        let h = SignalHistory::new(0); // Should be clamped to 1
        assert_eq!(h.window, 1);
    }

    // ── Energy Factor ─────────────────────────────────────────────────────

    #[test]
    fn test_crisis_raises_threshold() {
        assert!((energy_factor(Regime::Crisis) - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_homeostatic_neutral() {
        assert!((energy_factor(Regime::Homeostatic) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_anabolic_lowers_threshold() {
        assert!(energy_factor(Regime::Anabolic) < 1.0);
    }

    #[test]
    fn test_energy_factor_ordering() {
        assert!(energy_factor(Regime::Crisis) > energy_factor(Regime::Catabolic));
        assert!(energy_factor(Regime::Catabolic) > energy_factor(Regime::Homeostatic));
        assert!(energy_factor(Regime::Homeostatic) > energy_factor(Regime::Anabolic));
    }

    // ── Density Factor ────────────────────────────────────────────────────

    #[test]
    fn test_zero_density_no_adjustment() {
        let h = SignalHistory::new(10);
        assert!((density_factor(&h) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_high_density_raises_factor() {
        let mut h = SignalHistory::new(10);
        for _ in 0..10 {
            h.record(true); // 100% density
        }
        let factor = density_factor(&h);
        assert!((factor - MAX_DENSITY_FACTOR).abs() < f64::EPSILON);
    }

    #[test]
    fn test_density_factor_capped() {
        let mut h = SignalHistory::new(10);
        for _ in 0..10 {
            h.record(true);
        }
        assert!(density_factor(&h) <= MAX_DENSITY_FACTOR);
    }

    // ── Adaptive Threshold ────────────────────────────────────────────────

    #[test]
    fn test_fixed_policy_no_adaptation() {
        let config = AdaptiveThresholdConfig::prr().with_policy(ThresholdPolicy::Fixed);
        let mut h = SignalHistory::new(10);
        for _ in 0..10 {
            h.record(true);
        }

        let result = adapt(&config, 0.30, &h); // Crisis + high density
        assert!((result.value - 2.0).abs() < f64::EPSILON); // No change
        assert!((result.energy_factor - 1.0).abs() < f64::EPSILON);
        assert!((result.density_factor - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_energy_only_adaptation() {
        let config = AdaptiveThresholdConfig::prr().with_policy(ThresholdPolicy::EnergyAdaptive);
        let h = SignalHistory::new(10);

        let crisis = adapt(&config, 0.30, &h);
        assert!((crisis.value - 3.0).abs() < f64::EPSILON); // 2.0 × 1.5

        let anabolic = adapt(&config, 0.90, &h);
        assert!((anabolic.value - 1.5).abs() < f64::EPSILON); // 2.0 × 0.75
    }

    #[test]
    fn test_density_only_adaptation() {
        let config = AdaptiveThresholdConfig::prr().with_policy(ThresholdPolicy::DensityAdaptive);
        let mut h = SignalHistory::new(10);
        h.record_batch(5, 10); // 50% density → factor = 1.5

        let result = adapt(&config, 0.30, &h); // Crisis ignored
        assert!((result.value - 3.0).abs() < f64::EPSILON); // 2.0 × 1.5
        assert!((result.energy_factor - 1.0).abs() < f64::EPSILON); // No energy
    }

    #[test]
    fn test_full_adaptive_combines_factors() {
        let config = AdaptiveThresholdConfig::prr();
        let mut h = SignalHistory::new(10);
        h.record_batch(5, 10); // 50% density → factor 1.5

        let result = adapt(&config, 0.30, &h); // Crisis → energy 1.5
        // 2.0 × 1.5 × 1.5 = 4.5
        assert!((result.value - 4.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_safety_floor_enforced() {
        let config = AdaptiveThresholdConfig::prr(); // floor = 1.0
        let h = SignalHistory::new(10);

        // Anabolic → 2.0 × 0.75 = 1.5 — above floor
        let result = adapt(&config, 0.90, &h);
        assert!(!result.floor_applied);

        // Custom with high base but low floor
        let config_low = AdaptiveThresholdConfig::custom("test", 0.5, 1.0);
        let result_low = adapt(&config_low, 0.90, &h);
        // 0.5 × 0.75 = 0.375 → floor to 1.0
        assert!(result_low.floor_applied);
        assert!((result_low.value - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_safety_floor_is_invariant() {
        // ToV V4: No combination of energy + density can breach the floor
        let config = AdaptiveThresholdConfig::prr();
        let h = SignalHistory::new(10); // Empty = density factor 1.0

        // Even at maximum anabolic (0.75x), PRR floor is 1.0
        let result = adapt(&config, 0.99, &h);
        assert!(result.value >= PRR_SAFETY_FLOOR);
    }

    // ── Config Constructors ───────────────────────────────────────────────

    #[test]
    fn test_prr_config() {
        let c = AdaptiveThresholdConfig::prr();
        assert!((c.base - 2.0).abs() < f64::EPSILON);
        assert!((c.safety_floor - PRR_SAFETY_FLOOR).abs() < f64::EPSILON);
        assert_eq!(c.algorithm, "PRR");
    }

    #[test]
    fn test_ror_ci_config() {
        let c = AdaptiveThresholdConfig::ror_ci();
        assert!((c.base - 1.0).abs() < f64::EPSILON);
        assert_eq!(c.algorithm, "ROR_CI_Lower");
    }

    #[test]
    fn test_chi_square_config() {
        let c = AdaptiveThresholdConfig::chi_square();
        assert!((c.base - 3.841).abs() < f64::EPSILON);
        assert!((c.safety_floor - 2.706).abs() < f64::EPSILON);
    }

    // ── Batch Adaptation ──────────────────────────────────────────────────

    #[test]
    fn test_adapt_all_pv() {
        let h = SignalHistory::default_window();
        let thresholds = adapt_all_pv(0.75, &h);
        assert_eq!(thresholds.len(), 3);

        let names: Vec<&str> = thresholds.iter().map(|t| t.algorithm.as_str()).collect();
        assert!(names.contains(&"PRR"));
        assert!(names.contains(&"ROR_CI_Lower"));
        assert!(names.contains(&"ChiSquare"));
    }

    #[test]
    fn test_adapt_all_pv_crisis_raises_all() {
        let h = SignalHistory::default_window();
        let normal = adapt_all_pv(0.75, &h);
        let crisis = adapt_all_pv(0.30, &h);

        for (n, c) in normal.iter().zip(crisis.iter()) {
            assert!(
                c.value >= n.value,
                "{} crisis ({}) should be >= normal ({})",
                n.algorithm,
                c.value,
                n.value,
            );
        }
    }

    // ── Display ───────────────────────────────────────────────────────────

    #[test]
    fn test_adapted_threshold_display() {
        let config = AdaptiveThresholdConfig::prr();
        let h = SignalHistory::default_window();
        let result = adapt(&config, 0.75, &h);
        let display = format!("{result}");
        assert!(display.contains("PRR"));
        assert!(display.contains("2.000"));
    }

    #[test]
    fn test_policy_display() {
        assert_eq!(ThresholdPolicy::FullAdaptive.to_string(), "Full-Adaptive");
        assert_eq!(ThresholdPolicy::Fixed.to_string(), "Fixed");
    }

    // ── From EC ───────────────────────────────────────────────────────────

    #[test]
    fn test_energy_factor_from_ec() {
        assert!((energy_factor_from_ec(0.90) - 0.75).abs() < f64::EPSILON);
        assert!((energy_factor_from_ec(0.30) - 1.50).abs() < f64::EPSILON);
    }

    // ── Homeostatic feedback ──────────────────────────────────────────────

    #[test]
    fn test_feedback_loop_stabilizes() {
        // Simulate: high signal density → threshold rises → fewer signals detected
        let config = AdaptiveThresholdConfig::prr();
        let mut h = SignalHistory::new(20);

        // Phase 1: lots of signals (10/10 = 100%)
        h.record_batch(10, 10);
        let t1 = adapt(&config, 0.75, &h);

        // Phase 2: threshold rose → fewer signals (2/10 = signals down)
        h.record_batch(2, 10);
        let t2 = adapt(&config, 0.75, &h);

        // Threshold should have relaxed as density dropped
        assert!(
            t2.value < t1.value,
            "Threshold should drop as density drops: t1={}, t2={}",
            t1.value,
            t2.value,
        );
    }
}
