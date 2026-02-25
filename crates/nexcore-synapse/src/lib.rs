//! # NexVigilant Core — Synapse - Amplitude Growth Learning Model
//!
//! Mathematical foundation for learning systems using quantum-grounded amplitude growth.
//!
//! ## Core Formula
//!
//! ```text
//! α(t+1) = α(t)·e^(-λt) + η·ν·π
//!          ───────────   ────────
//!             decay       growth
//! ```
//!
//! Where:
//! - `α` = amplitude (T1: Quantity)
//! - `λ` = decay constant (T1: Irreversibility)
//! - `t` = time since observation
//! - `η` = learning rate (T1: Comparison threshold)
//! - `ν` = observation frequency (T1: Frequency)
//! - `π` = persistence factor (T1: Persistence)
//!
//! ## T1 Primitive Grounding
//!
//! ```text
//! Amplitude Growth = ν × Σ × π with ∂ gating and ∝ decay
//!                    ↑   ↑   ↑     ↑            ↑
//!               Frequency Sum Persistence Boundary Irreversibility
//! ```
//!
//! ## Features
//!
//! - **Exponential Decay**: Time-based amplitude reduction with configurable half-life
//! - **Saturation Kinetics**: Michaelis-Menten bounded growth (no unbounded accumulation)
//! - **Threshold Gating**: Consolidation only when amplitude exceeds boundary
//! - **Persistence Modes**: Volatile (session) vs durable (cross-session) learning
//!
//! ## Usage
//!
//! ```rust
//! use nexcore_synapse::{Amplitude, AmplitudeConfig, LearningSignal, Synapse};
//!
//! // Create synapse with default configuration
//! let mut synapse = Synapse::new("pattern_recognition", AmplitudeConfig::default());
//!
//! // Observe signals
//! synapse.observe(LearningSignal::new(0.8, 1.0)); // confidence=0.8, relevance=1.0
//! synapse.observe(LearningSignal::new(0.9, 0.9));
//! synapse.observe(LearningSignal::new(0.7, 1.0));
//!
//! // Check current amplitude (with decay applied)
//! let current = synapse.current_amplitude();
//!
//! // Check if consolidated (crossed threshold)
//! if synapse.is_consolidated() {
//!     println!("Pattern consolidated with amplitude {}", current.value());
//! }
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![warn(missing_docs)]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::disallowed_types,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    reason = "Synapse domain types are intentionally closed and map structures preserve compatibility with existing serialized state"
)]

pub mod gate_control;
pub mod grounding;
pub mod referred_pain;

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// T1 PRIMITIVE TYPES
// ============================================================================

/// Amplitude value representing learning strength.
///
/// Tier: T2-P (grounded to N: Quantity)
///
/// Amplitudes are bounded [0.0, 1.0] through Michaelis-Menten saturation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Amplitude(f64);

impl Amplitude {
    /// Zero amplitude (no learning signal).
    pub const ZERO: Self = Self(0.0);

    /// Maximum amplitude (full saturation).
    pub const MAX: Self = Self(1.0);

    /// Create a new amplitude, clamped to [0.0, 1.0].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get the raw amplitude value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }

    /// Check if amplitude is effectively zero (below epsilon).
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0 < f64::EPSILON
    }

    /// Check if amplitude is at saturation.
    #[must_use]
    pub fn is_saturated(self) -> bool {
        self.0 >= 0.99
    }

    /// Apply exponential decay.
    ///
    /// Formula: `α × e^(-λt)` where `λ = ln(2) / half_life`
    #[must_use]
    pub fn decay(self, elapsed_seconds: f64, half_life_seconds: f64) -> Self {
        if elapsed_seconds <= 0.0 || half_life_seconds <= 0.0 {
            return self;
        }
        let lambda = std::f64::consts::LN_2 / half_life_seconds;
        let decay_factor = (-lambda * elapsed_seconds).exp();
        Self::new(self.0 * decay_factor)
    }
}

impl Default for Amplitude {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Amplitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

impl std::ops::Add for Amplitude {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}

impl std::ops::Mul<f64> for Amplitude {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.0 * rhs)
    }
}

// ============================================================================
// LEARNING SIGNAL
// ============================================================================

/// A single observation that contributes to amplitude growth.
///
/// Tier: T2-C (grounded to ∃: Existence + N: Quantity)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LearningSignal {
    /// Confidence in this observation [0.0, 1.0].
    pub confidence: f64,

    /// Relevance to the learning target [0.0, 1.0].
    pub relevance: f64,

    /// Timestamp of observation.
    pub observed_at: DateTime,
}

impl LearningSignal {
    /// Create a new learning signal with current timestamp.
    #[must_use]
    pub fn new(confidence: f64, relevance: f64) -> Self {
        Self {
            confidence: confidence.clamp(0.0, 1.0),
            relevance: relevance.clamp(0.0, 1.0),
            observed_at: DateTime::now(),
        }
    }

    /// Create a signal with specific timestamp.
    #[must_use]
    pub fn with_timestamp(confidence: f64, relevance: f64, timestamp: DateTime) -> Self {
        Self {
            confidence: confidence.clamp(0.0, 1.0),
            relevance: relevance.clamp(0.0, 1.0),
            observed_at: timestamp,
        }
    }

    /// Compute the boost contribution: η × confidence × relevance.
    #[must_use]
    pub fn boost(&self, learning_rate: f64) -> f64 {
        learning_rate * self.confidence * self.relevance
    }
}

impl Default for LearningSignal {
    fn default() -> Self {
        Self::new(0.5, 0.5)
    }
}

// ============================================================================
// SATURATION KINETICS
// ============================================================================

/// Michaelis-Menten saturation parameters.
///
/// Tier: T2-C (grounded to ∂: Boundary + N: Quantity + ρ: Recursion)
///
/// Models bounded growth: rate = Vmax × [S] / (Km + [S])
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SaturationKinetics {
    /// Maximum velocity (asymptotic limit).
    pub v_max: f64,

    /// Michaelis constant (half-saturation point).
    pub k_m: f64,
}

impl SaturationKinetics {
    /// Default saturation: Vmax=1.0, Km=5.0 (50% at 5 observations).
    pub const DEFAULT: Self = Self {
        v_max: 1.0,
        k_m: 5.0,
    };

    /// Compute saturated amplitude given observation count.
    ///
    /// Formula: Vmax × n / (Km + n)
    #[must_use]
    pub fn saturate(&self, observation_count: u32) -> Amplitude {
        let n = f64::from(observation_count);
        if n <= 0.0 {
            return Amplitude::ZERO;
        }
        let value = self.v_max * n / (self.k_m + n);
        Amplitude::new(value)
    }

    /// Compute gradient (instantaneous rate of change).
    ///
    /// d/dn [Vmax × n / (Km + n)] = Vmax × Km / (Km + n)²
    #[must_use]
    pub fn gradient(&self, observation_count: u32) -> f64 {
        let n = f64::from(observation_count);
        let denominator = self.k_m + n;
        self.v_max * self.k_m / (denominator * denominator)
    }
}

impl Default for SaturationKinetics {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// ============================================================================
// AMPLITUDE CONFIGURATION
// ============================================================================

/// Configuration for amplitude growth behavior.
///
/// Tier: T2-C (composition of multiple T1 boundaries)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmplitudeConfig {
    /// Learning rate η (signal boost multiplier).
    pub learning_rate: f64,

    /// Decay half-life in seconds.
    pub half_life_seconds: f64,

    /// Consolidation threshold (amplitude required to mark as "learned").
    pub consolidation_threshold: f64,

    /// Saturation kinetics parameters.
    pub saturation: SaturationKinetics,

    /// Whether to persist across sessions.
    pub persistent: bool,
}

impl AmplitudeConfig {
    /// Default: η=0.1, half-life=30 days, threshold=0.7.
    pub const DEFAULT: Self = Self {
        learning_rate: 0.1,
        half_life_seconds: 30.0 * 24.0 * 60.0 * 60.0, // 30 days
        consolidation_threshold: 0.7,
        saturation: SaturationKinetics::DEFAULT,
        persistent: true,
    };

    /// Fast learning: η=0.3, half-life=7 days, threshold=0.5.
    pub const FAST: Self = Self {
        learning_rate: 0.3,
        half_life_seconds: 7.0 * 24.0 * 60.0 * 60.0,
        consolidation_threshold: 0.5,
        saturation: SaturationKinetics {
            v_max: 1.0,
            k_m: 3.0,
        },
        persistent: true,
    };

    /// Volatile (session-only): η=0.2, half-life=1 hour, threshold=0.6.
    pub const VOLATILE: Self = Self {
        learning_rate: 0.2,
        half_life_seconds: 60.0 * 60.0, // 1 hour
        consolidation_threshold: 0.6,
        saturation: SaturationKinetics {
            v_max: 1.0,
            k_m: 2.0,
        },
        persistent: false,
    };
}

impl Default for AmplitudeConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// ============================================================================
// CONSOLIDATION STATUS
// ============================================================================

/// Consolidation state of a synapse.
///
/// Tier: T2-P (grounded to ς: State + ∂: Boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsolidationStatus {
    /// Below threshold, still accumulating.
    Accumulating,

    /// Crossed threshold, pattern consolidated.
    Consolidated,

    /// Was consolidated but decayed below threshold.
    Decayed,
}

impl fmt::Display for ConsolidationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Accumulating => "accumulating",
            Self::Consolidated => "consolidated",
            Self::Decayed => "decayed",
        };
        write!(f, "{s}")
    }
}

// ============================================================================
// SYNAPSE
// ============================================================================

/// A learning synapse that grows amplitude through observations.
///
/// Tier: T2-C (grounded to ν + Σ + π + ∂ + ∝)
///
/// Implements the unified amplitude growth equation:
/// ```text
/// α(t+1) = α(t)·e^(-λt) + η·confidence·relevance
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synapse {
    /// Unique identifier for this synapse.
    pub id: String,

    /// Configuration parameters.
    pub config: AmplitudeConfig,

    /// Raw amplitude (before decay).
    amplitude: Amplitude,

    /// Total observation count (ν: Frequency).
    observation_count: u32,

    /// Timestamp of last observation.
    last_observed: DateTime,

    /// Timestamp when created.
    created_at: DateTime,

    /// Peak amplitude ever reached.
    peak_amplitude: Amplitude,

    /// Whether consolidation threshold was ever crossed.
    ever_consolidated: bool,
}

impl Synapse {
    /// Create a new synapse with given ID and configuration.
    #[must_use]
    pub fn new(id: impl Into<String>, config: AmplitudeConfig) -> Self {
        let now = DateTime::now();
        Self {
            id: id.into(),
            config,
            amplitude: Amplitude::ZERO,
            observation_count: 0,
            last_observed: now,
            created_at: now,
            peak_amplitude: Amplitude::ZERO,
            ever_consolidated: false,
        }
    }

    /// Observe a learning signal, updating amplitude.
    ///
    /// Applies the growth equation:
    /// ```text
    /// α(t+1) = α(t)·decay + η·confidence·relevance
    /// ```
    pub fn observe(&mut self, signal: LearningSignal) {
        // Calculate elapsed time for decay
        #[allow(
            clippy::cast_precision_loss,
            reason = "Count-to-f64 conversion for bounded runtime metrics"
        )]
        // Elapsed seconds are typically small; exact precision not needed for decay
        let elapsed = (signal.observed_at - self.last_observed)
            .num_seconds()
            .max(0) as f64;

        // Apply decay to existing amplitude
        let decayed = self.amplitude.decay(elapsed, self.config.half_life_seconds);

        // Calculate boost from new signal
        let boost = signal.boost(self.config.learning_rate);

        // Compute raw sum (before saturation)
        let raw_sum = decayed.value() + boost;

        // Apply Michaelis-Menten saturation
        self.observation_count += 1;
        let saturated_max = self.config.saturation.saturate(self.observation_count);

        // Final amplitude is min(raw_sum, saturated_max)
        self.amplitude = Amplitude::new(raw_sum.min(saturated_max.value()));

        // Update peak tracking
        if self.amplitude.value() > self.peak_amplitude.value() {
            self.peak_amplitude = self.amplitude;
        }

        // Check consolidation
        if self.amplitude.value() >= self.config.consolidation_threshold {
            self.ever_consolidated = true;
        }

        self.last_observed = signal.observed_at;
    }

    /// Get current amplitude with decay applied.
    #[must_use]
    pub fn current_amplitude(&self) -> Amplitude {
        #[allow(
            clippy::cast_precision_loss,
            reason = "Count-to-f64 conversion for bounded runtime metrics"
        )]
        // Elapsed seconds are typically small; exact precision not needed for decay
        let elapsed = (DateTime::now() - self.last_observed).num_seconds().max(0) as f64;
        self.amplitude.decay(elapsed, self.config.half_life_seconds)
    }

    /// Get raw amplitude (without decay).
    #[must_use]
    pub fn raw_amplitude(&self) -> Amplitude {
        self.amplitude
    }

    /// Get observation count (ν: Frequency).
    #[must_use]
    pub fn observation_count(&self) -> u32 {
        self.observation_count
    }

    /// Get peak amplitude ever reached.
    #[must_use]
    pub fn peak_amplitude(&self) -> Amplitude {
        self.peak_amplitude
    }

    /// Check if currently consolidated (amplitude >= threshold).
    #[must_use]
    pub fn is_consolidated(&self) -> bool {
        self.current_amplitude().value() >= self.config.consolidation_threshold
    }

    /// Get consolidation status.
    #[must_use]
    pub fn status(&self) -> ConsolidationStatus {
        let current = self.current_amplitude();
        if current.value() >= self.config.consolidation_threshold {
            ConsolidationStatus::Consolidated
        } else if self.ever_consolidated {
            ConsolidationStatus::Decayed
        } else {
            ConsolidationStatus::Accumulating
        }
    }

    /// Check if this synapse should persist across sessions.
    #[must_use]
    pub fn is_persistent(&self) -> bool {
        self.config.persistent
    }

    /// Time until amplitude decays to threshold (None if already below).
    #[must_use]
    pub fn time_to_decay(&self) -> Option<std::time::Duration> {
        let current = self.current_amplitude().value();
        let threshold = self.config.consolidation_threshold;

        if current <= threshold {
            return None;
        }

        // Solve: current × e^(-λt) = threshold
        // t = -ln(threshold/current) / λ
        let lambda = std::f64::consts::LN_2 / self.config.half_life_seconds;
        let t_seconds = -(threshold / current).ln() / lambda;

        if t_seconds > 0.0 && t_seconds.is_finite() {
            Some(std::time::Duration::from_secs_f64(t_seconds))
        } else {
            None
        }
    }

    /// Reset the synapse to initial state.
    pub fn reset(&mut self) {
        self.amplitude = Amplitude::ZERO;
        self.observation_count = 0;
        self.last_observed = DateTime::now();
        self.peak_amplitude = Amplitude::ZERO;
        self.ever_consolidated = false;
    }
}

impl fmt::Display for Synapse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Synapse[{}]: α={}, ν={}, status={}",
            self.id,
            self.current_amplitude(),
            self.observation_count,
            self.status()
        )
    }
}

// ============================================================================
// SYNAPSE BANK (MULTI-SYNAPSE MANAGEMENT)
// ============================================================================

/// A collection of synapses for managing multiple learning targets.
///
/// Tier: T3 (domain-specific composition)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SynapseBank {
    synapses: std::collections::HashMap<String, Synapse>,
}

impl SynapseBank {
    /// Create a new empty synapse bank.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a synapse with given ID.
    pub fn get_or_create(&mut self, id: &str, config: AmplitudeConfig) -> &mut Synapse {
        self.synapses
            .entry(id.to_string())
            .or_insert_with(|| Synapse::new(id, config))
    }

    /// Get a synapse by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Synapse> {
        self.synapses.get(id)
    }

    /// Get a mutable synapse by ID.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Synapse> {
        self.synapses.get_mut(id)
    }

    /// List all consolidated synapses.
    pub fn consolidated(&self) -> impl Iterator<Item = &Synapse> {
        self.synapses.values().filter(|s| s.is_consolidated())
    }

    /// List all accumulating (not yet consolidated) synapses.
    pub fn accumulating(&self) -> impl Iterator<Item = &Synapse> {
        self.synapses.values().filter(|s| !s.is_consolidated())
    }

    /// Prune decayed synapses (amplitude below epsilon).
    pub fn prune_decayed(&mut self) -> usize {
        let before = self.synapses.len();
        self.synapses
            .retain(|_, s| !s.current_amplitude().is_zero());
        before - self.synapses.len()
    }

    /// Total synapse count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.synapses.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.synapses.is_empty()
    }

    /// Iterate over all synapses.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Synapse)> {
        self.synapses.iter()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_chrono::Duration;

    #[test]
    fn test_amplitude_clamping() {
        assert!((Amplitude::new(1.5).value() - 1.0).abs() < f64::EPSILON);
        assert!((Amplitude::new(-0.5).value()).abs() < f64::EPSILON);
        assert!((Amplitude::new(0.5).value() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_amplitude_decay() {
        let amp = Amplitude::new(1.0);
        let half_life = 100.0; // seconds

        // After one half-life, should be ~0.5
        let decayed = amp.decay(100.0, half_life);
        assert!((decayed.value() - 0.5).abs() < 0.01);

        // After two half-lives, should be ~0.25
        let decayed2 = amp.decay(200.0, half_life);
        assert!((decayed2.value() - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_saturation_kinetics() {
        let kinetics = SaturationKinetics::DEFAULT; // Vmax=1.0, Km=5.0

        // At n=5 (Km), should be 50% of Vmax
        let at_km = kinetics.saturate(5);
        assert!((at_km.value() - 0.5).abs() < 0.01);

        // At n=0, should be 0
        let at_zero = kinetics.saturate(0);
        assert!(at_zero.is_zero());

        // At large n, should approach Vmax
        let at_large = kinetics.saturate(1000);
        assert!(at_large.value() > 0.99);
    }

    #[test]
    fn test_synapse_growth() {
        let mut synapse = Synapse::new("test", AmplitudeConfig::FAST);

        // Initial state
        assert!(synapse.current_amplitude().is_zero());
        assert_eq!(synapse.observation_count(), 0);

        // Add observations
        for _ in 0..10 {
            synapse.observe(LearningSignal::new(0.9, 1.0));
        }

        // Should have grown
        assert!(synapse.current_amplitude().value() > 0.0);
        assert_eq!(synapse.observation_count(), 10);
    }

    #[test]
    fn test_consolidation_status() {
        let config = AmplitudeConfig {
            consolidation_threshold: 0.3,
            learning_rate: 0.5,
            ..AmplitudeConfig::FAST
        };
        let mut synapse = Synapse::new("test", config);

        assert_eq!(synapse.status(), ConsolidationStatus::Accumulating);

        // Add high-confidence signals
        for _ in 0..5 {
            synapse.observe(LearningSignal::new(1.0, 1.0));
        }

        assert_eq!(synapse.status(), ConsolidationStatus::Consolidated);
    }

    #[test]
    fn test_synapse_bank() {
        let mut bank = SynapseBank::new();

        bank.get_or_create("pattern_a", AmplitudeConfig::default());
        bank.get_or_create("pattern_b", AmplitudeConfig::default());

        assert_eq!(bank.len(), 2);

        if let Some(synapse) = bank.get_mut("pattern_a") {
            synapse.observe(LearningSignal::new(0.9, 1.0));
        }

        assert!(bank.get("pattern_a").is_some());
    }

    #[test]
    fn test_learning_signal_boost() {
        let signal = LearningSignal::new(0.8, 0.5);
        let boost = signal.boost(0.1);

        // boost = 0.1 × 0.8 × 0.5 = 0.04
        assert!((boost - 0.04).abs() < f64::EPSILON);
    }
}
