//! # NexVigilant Core — Hormones - Endocrine System for .claude
//!
//! Persistent state modulators affecting system behavior across sessions.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//!
//! ## Hormone Types
//!
//! | Hormone | Function |
//! |---------|----------|
//! | Cortisol | Stress response - risk aversion |
//! | Dopamine | Reward - pattern reinforcement |
//! | Serotonin | Stability - consistency |
//! | Adrenaline | Crisis mode - capability unlock |
//! | Oxytocin | Trust - partnership strength |
//! | Melatonin | Rest - session pacing |
//!
//! ## Tier Classification
//!
//! - `HormoneLevel`: T2-P (Grounded in T1 Quantity N)
//! - `HormoneType`: T2-P (Grounded in T1 Classification)
//! - `EndocrineState`: T2-C (Grounded in T1 State ς and Persistence π)

pub mod grounding;

use core::fmt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Errors during endocrine operations
/// Tier: T2-C (Grounded in T1 Recursion ρ)
#[derive(Debug)]
pub enum EndocrineError {
    /// Failed to read state file
    ReadError(std::io::Error),
    /// Failed to parse JSON
    ParseError(serde_json::Error),
    /// HOME not set
    NoHomeDir,
}

impl fmt::Display for EndocrineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadError(e) => write!(f, "failed to read hormone state: {e}"),
            Self::ParseError(e) => write!(f, "corrupted hormone state JSON: {e}"),
            Self::NoHomeDir => write!(f, "HOME environment variable not set"),
        }
    }
}

impl std::error::Error for EndocrineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadError(e) => Some(e),
            Self::ParseError(e) => Some(e),
            Self::NoHomeDir => None,
        }
    }
}

impl From<std::io::Error> for EndocrineError {
    fn from(e: std::io::Error) -> Self {
        Self::ReadError(e)
    }
}

impl From<serde_json::Error> for EndocrineError {
    fn from(e: serde_json::Error) -> Self {
        Self::ParseError(e)
    }
}

/// Result type for endocrine operations
pub type EndocrineResult<T> = Result<T, EndocrineError>;

/// Hormone level - bounded 0.0 to 1.0
/// Tier: T2-P (Cross-Domain Primitive)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct HormoneLevel(f64);

impl HormoneLevel {
    /// Create a new hormone level, clamped to valid range
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get the raw value
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }

    /// Baseline level (homeostatic)
    #[must_use]
    pub const fn baseline() -> Self {
        Self(0.5)
    }

    /// Minimum level
    #[must_use]
    pub const fn min() -> Self {
        Self(0.0)
    }

    /// Maximum level
    #[must_use]
    pub const fn max() -> Self {
        Self(1.0)
    }

    /// Apply decay toward baseline
    #[must_use]
    pub fn decay_toward_baseline(self, rate: f64) -> Self {
        let diff = 0.5 - self.0;
        Self::new(self.0 + diff * rate.clamp(0.0, 1.0))
    }

    /// Increase level
    #[must_use]
    pub fn increase(self, amount: f64) -> Self {
        Self::new(self.0 + amount)
    }

    /// Decrease level
    #[must_use]
    pub fn decrease(self, amount: f64) -> Self {
        Self::new(self.0 - amount)
    }
}

impl Default for HormoneLevel {
    fn default() -> Self {
        Self::baseline()
    }
}

/// Hormone type enumeration
/// Tier: T2-C (Grounded in T1 Recursion ρ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HormoneType {
    /// Stress response
    Cortisol,
    /// Reward/motivation
    Dopamine,
    /// Mood stability
    Serotonin,
    /// Fight/flight
    Adrenaline,
    /// Trust/bonding
    Oxytocin,
    /// Rest/recovery
    Melatonin,
}

impl HormoneType {
    /// All hormone types
    pub const ALL: [HormoneType; 6] = [
        Self::Cortisol,
        Self::Dopamine,
        Self::Serotonin,
        Self::Adrenaline,
        Self::Oxytocin,
        Self::Melatonin,
    ];

    /// Decay rate per session
    #[must_use]
    pub fn decay_rate(self) -> f64 {
        match self {
            Self::Cortisol => 0.3,
            Self::Dopamine => 0.5,
            Self::Serotonin => 0.1,
            Self::Adrenaline => 0.8,
            Self::Oxytocin => 0.05,
            Self::Melatonin => 0.7,
        }
    }

    /// Human-readable name
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Cortisol => "Cortisol (Stress)",
            Self::Dopamine => "Dopamine (Reward)",
            Self::Serotonin => "Serotonin (Stability)",
            Self::Adrenaline => "Adrenaline (Crisis)",
            Self::Oxytocin => "Oxytocin (Trust)",
            Self::Melatonin => "Melatonin (Rest)",
        }
    }
}

/// Complete endocrine state
/// Tier: T2-C (Grounded in T1 Recursion ρ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndocrineState {
    /// Stress response
    pub cortisol: HormoneLevel,
    /// Reward/motivation
    pub dopamine: HormoneLevel,
    /// Mood stability
    pub serotonin: HormoneLevel,
    /// Fight/flight
    pub adrenaline: HormoneLevel,
    /// Trust/bonding
    pub oxytocin: HormoneLevel,
    /// Rest/recovery
    pub melatonin: HormoneLevel,
    /// Last update time
    pub last_updated: chrono::DateTime<chrono::Utc>,
    /// Session count
    pub session_count: u64,
}

impl Default for EndocrineState {
    fn default() -> Self {
        Self {
            cortisol: HormoneLevel::baseline(),
            dopamine: HormoneLevel::baseline(),
            serotonin: HormoneLevel::baseline(),
            adrenaline: HormoneLevel::min(),
            oxytocin: HormoneLevel::baseline(),
            melatonin: HormoneLevel::min(),
            last_updated: chrono::Utc::now(),
            session_count: 0,
        }
    }
}

impl EndocrineState {
    /// Load from persistent storage
    #[must_use]
    pub fn load() -> Self {
        Self::load_with_result().unwrap_or_default()
    }

    /// Load with explicit error handling
    pub fn load_with_result() -> EndocrineResult<Self> {
        let path = Self::storage_path()?;
        Self::load_from_path(&path)
    }

    /// Load from a specific path
    pub fn load_from_path(path: &std::path::Path) -> EndocrineResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let state: Self = serde_json::from_str(&content)?;
        Ok(state)
    }

    /// Save to persistent storage
    pub fn save(&self) -> EndocrineResult<()> {
        let path = Self::storage_path()?;
        self.save_to_path(&path)
    }

    /// Save to a specific path
    pub fn save_to_path(&self, path: &std::path::Path) -> EndocrineResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    fn storage_path() -> EndocrineResult<PathBuf> {
        let home = std::env::var("HOME").map_err(|_| EndocrineError::NoHomeDir)?;
        Ok(PathBuf::from(home)
            .join(".claude")
            .join("hormones")
            .join("state.json"))
    }

    /// Get hormone level by type
    #[must_use]
    pub fn get(&self, hormone: HormoneType) -> HormoneLevel {
        match hormone {
            HormoneType::Cortisol => self.cortisol,
            HormoneType::Dopamine => self.dopamine,
            HormoneType::Serotonin => self.serotonin,
            HormoneType::Adrenaline => self.adrenaline,
            HormoneType::Oxytocin => self.oxytocin,
            HormoneType::Melatonin => self.melatonin,
        }
    }

    /// Set hormone level by type
    pub fn set(&mut self, hormone: HormoneType, level: HormoneLevel) {
        match hormone {
            HormoneType::Cortisol => self.cortisol = level,
            HormoneType::Dopamine => self.dopamine = level,
            HormoneType::Serotonin => self.serotonin = level,
            HormoneType::Adrenaline => self.adrenaline = level,
            HormoneType::Oxytocin => self.oxytocin = level,
            HormoneType::Melatonin => self.melatonin = level,
        }
        self.last_updated = chrono::Utc::now();
    }

    /// Apply decay to all hormones
    pub fn apply_decay(&mut self) {
        for hormone in HormoneType::ALL {
            let current = self.get(hormone);
            let decayed = current.decay_toward_baseline(hormone.decay_rate());
            self.set(hormone, decayed);
        }
        self.session_count += 1;
    }

    /// Calculate overall mood score (0.0-1.0)
    #[must_use]
    pub fn mood_score(&self) -> f64 {
        let positive = self.dopamine.value() + self.serotonin.value() + self.oxytocin.value();
        let negative = self.cortisol.value() + self.adrenaline.value();
        (positive - negative + 2.0) / 5.0
    }

    /// Get risk tolerance
    #[must_use]
    pub fn risk_tolerance(&self) -> f64 {
        let base = 0.5;
        let cortisol_effect = -0.3 * (self.cortisol.value() - 0.5);
        let dopamine_effect = 0.3 * (self.dopamine.value() - 0.5);
        let adrenaline_effect = 0.1 * (self.adrenaline.value() - 0.5);
        (base + cortisol_effect + dopamine_effect + adrenaline_effect).clamp(0.0, 1.0)
    }

    /// Check if in crisis mode
    #[must_use]
    pub fn is_crisis_mode(&self) -> bool {
        self.adrenaline.value() > 0.7
    }

    /// Check if trust is established
    #[must_use]
    pub fn is_trusted_partnership(&self) -> bool {
        self.oxytocin.value() > 0.6
    }

    /// Check if rest recommended
    #[must_use]
    pub fn should_rest(&self) -> bool {
        self.melatonin.value() > 0.7
    }
}

/// Stimuli that trigger hormone changes
/// Tier: T3 (Grounded in T1 Mapping μ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stimulus {
    /// Cortisol triggers
    ErrorEncountered {
        severity: f64,
    },
    DeadlinePressure {
        urgency: f64,
    },
    UncertaintyDetected {
        confidence_gap: f64,
    },

    /// Dopamine triggers
    TaskCompleted {
        complexity: f64,
    },
    PositiveFeedback {
        intensity: f64,
    },
    PatternSuccess {
        reuse_count: u32,
    },

    /// Serotonin triggers
    ConsistentSession {
        variance: f64,
    },
    PredictableOutcome {
        accuracy: f64,
    },

    /// Adrenaline triggers
    CriticalError {
        recoverable: bool,
    },
    TimeConstraint {
        remaining_pct: f64,
    },
    HighStakesDecision {
        impact: f64,
    },

    /// Oxytocin triggers
    PartnershipReinforced {
        signal: f64,
    },
    MutualSuccess {
        shared_win: bool,
    },
    TransparentCommunication {
        clarity: f64,
    },

    /// Melatonin triggers
    SessionDuration {
        minutes: u64,
    },
    ContextUtilization {
        pct: f64,
    },
    CompletionSignal {
        tasks_done: u32,
    },

    /// Planetary triggers (RECURSION ρ)
    PlanetaryAlignment {
        /// Distance in AU (0.37 to 2.67)
        distance_au: f64,
        /// Days since last opposition (0 to 780)
        days_since_opposition: u32,
    },
}

impl Stimulus {
    /// Apply this stimulus to the endocrine state
    pub fn apply(&self, state: &mut EndocrineState) {
        match self {
            Stimulus::ErrorEncountered { severity } => {
                state.cortisol = state.cortisol.increase(*severity * 0.2);
            }
            Stimulus::DeadlinePressure { urgency } => {
                state.cortisol = state.cortisol.increase(*urgency * 0.15);
                state.adrenaline = state.adrenaline.increase(*urgency * 0.1);
            }
            Stimulus::UncertaintyDetected { confidence_gap } => {
                state.cortisol = state.cortisol.increase(*confidence_gap * 0.1);
            }
            Stimulus::TaskCompleted { complexity } => {
                state.dopamine = state.dopamine.increase(*complexity * 0.15);
                state.cortisol = state.cortisol.decrease(*complexity * 0.05);
            }
            Stimulus::PositiveFeedback { intensity } => {
                state.dopamine = state.dopamine.increase(*intensity * 0.2);
                state.oxytocin = state.oxytocin.increase(*intensity * 0.1);
            }
            Stimulus::PatternSuccess { reuse_count } => {
                let boost = (*reuse_count as f64 * 0.05).min(0.2);
                state.dopamine = state.dopamine.increase(boost);
            }
            Stimulus::ConsistentSession { variance } => {
                let stability = 1.0 - variance;
                state.serotonin = state.serotonin.increase(stability * 0.1);
            }
            Stimulus::PredictableOutcome { accuracy } => {
                state.serotonin = state.serotonin.increase(*accuracy * 0.1);
            }
            Stimulus::CriticalError { recoverable } => {
                state.adrenaline = state.adrenaline.increase(0.4);
                if !recoverable {
                    state.cortisol = state.cortisol.increase(0.3);
                }
            }
            Stimulus::TimeConstraint { remaining_pct } => {
                if *remaining_pct < 0.2 {
                    state.adrenaline = state.adrenaline.increase(0.3);
                    state.melatonin = state.melatonin.increase(0.2);
                }
            }
            Stimulus::HighStakesDecision { impact } => {
                state.adrenaline = state.adrenaline.increase(*impact * 0.2);
            }
            Stimulus::PartnershipReinforced { signal } => {
                state.oxytocin = state.oxytocin.increase(*signal * 0.15);
            }
            Stimulus::MutualSuccess { shared_win } => {
                if *shared_win {
                    state.oxytocin = state.oxytocin.increase(0.1);
                    state.dopamine = state.dopamine.increase(0.1);
                }
            }
            Stimulus::TransparentCommunication { clarity } => {
                state.oxytocin = state.oxytocin.increase(*clarity * 0.05);
            }
            Stimulus::SessionDuration { minutes } => {
                if *minutes > 60 {
                    let fatigue = ((*minutes - 60) as f64 / 120.0).min(0.5);
                    state.melatonin = state.melatonin.increase(fatigue);
                }
            }
            Stimulus::ContextUtilization { pct } => {
                if *pct > 0.7 {
                    state.melatonin = state.melatonin.increase((*pct - 0.7) * 0.5);
                }
            }
            Stimulus::CompletionSignal { tasks_done } => {
                if *tasks_done > 0 {
                    state.melatonin = state.melatonin.increase(0.1);
                }
            }
            Stimulus::PlanetaryAlignment {
                distance_au,
                days_since_opposition,
            } => {
                // Proximity boost (Distance component)
                // Normalize AU: 0.37 (close) to 2.67 (far)
                let proximity = (2.67 - distance_au.clamp(0.37, 2.67)) / (2.67 - 0.37);

                // Recursion phase (Cycle component)
                // 780 day period. Peak at 0 and 780.
                let cycle_phase =
                    ((*days_since_opposition % 780) as f64 / 780.0 * 2.0 * core::f64::consts::PI)
                        .cos();
                let recursive_boost = (cycle_phase + 1.0) / 2.0;

                // Forge effects
                state.dopamine = state.dopamine.increase(proximity * 0.1);
                state.serotonin = state.serotonin.increase(recursive_boost * 0.1);
                state.oxytocin = state.oxytocin.increase(proximity * recursive_boost * 0.05);

                if proximity > 0.8 {
                    state.adrenaline = state.adrenaline.increase(0.1); // High stakes proximity
                }
            }
        }
        state.last_updated = chrono::Utc::now();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- HormoneLevel tests ---

    #[test]
    fn hormone_level_clamps_above_one() {
        let level = HormoneLevel::new(1.5);
        assert!((level.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hormone_level_clamps_below_zero() {
        let level = HormoneLevel::new(-0.3);
        assert!((level.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hormone_level_baseline_is_half() {
        let level = HormoneLevel::baseline();
        assert!((level.value() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn hormone_level_increase_clamps() {
        let level = HormoneLevel::new(0.9);
        let increased = level.increase(0.5);
        assert!((increased.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hormone_level_decrease_clamps() {
        let level = HormoneLevel::new(0.1);
        let decreased = level.decrease(0.5);
        assert!((decreased.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hormone_level_decay_toward_baseline() {
        // Above baseline: should decrease toward 0.5
        let high = HormoneLevel::new(0.8);
        let decayed = high.decay_toward_baseline(0.5);
        assert!(decayed.value() < 0.8);
        assert!(decayed.value() > 0.5);

        // Below baseline: should increase toward 0.5
        let low = HormoneLevel::new(0.2);
        let decayed = low.decay_toward_baseline(0.5);
        assert!(decayed.value() > 0.2);
        assert!(decayed.value() < 0.5);

        // At baseline: should stay at 0.5
        let base = HormoneLevel::baseline();
        let decayed = base.decay_toward_baseline(0.5);
        assert!((decayed.value() - 0.5).abs() < f64::EPSILON);
    }

    // --- HormoneType tests ---

    #[test]
    fn hormone_type_all_contains_six() {
        assert_eq!(HormoneType::ALL.len(), 6);
    }

    #[test]
    fn hormone_type_decay_rates_positive() {
        for hormone in HormoneType::ALL {
            assert!(
                hormone.decay_rate() > 0.0,
                "{:?} decay rate should be positive",
                hormone
            );
            assert!(
                hormone.decay_rate() <= 1.0,
                "{:?} decay rate should be <= 1.0",
                hormone
            );
        }
    }

    #[test]
    fn adrenaline_decays_fastest() {
        let adrenaline_rate = HormoneType::Adrenaline.decay_rate();
        for hormone in HormoneType::ALL {
            if hormone != HormoneType::Adrenaline {
                assert!(
                    adrenaline_rate >= hormone.decay_rate(),
                    "Adrenaline should decay at least as fast as {:?}",
                    hormone
                );
            }
        }
    }

    #[test]
    fn hormone_type_names_non_empty() {
        for hormone in HormoneType::ALL {
            assert!(!hormone.name().is_empty());
        }
    }

    // --- EndocrineState tests ---

    #[test]
    fn endocrine_state_default_baselines() {
        let state = EndocrineState::default();
        assert!((state.cortisol.value() - 0.5).abs() < f64::EPSILON);
        assert!((state.dopamine.value() - 0.5).abs() < f64::EPSILON);
        assert!((state.serotonin.value() - 0.5).abs() < f64::EPSILON);
        assert!((state.adrenaline.value() - 0.0).abs() < f64::EPSILON);
        assert!((state.oxytocin.value() - 0.5).abs() < f64::EPSILON);
        assert!((state.melatonin.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn endocrine_state_get_set_roundtrip() {
        let mut state = EndocrineState::default();
        let new_level = HormoneLevel::new(0.75);
        state.set(HormoneType::Dopamine, new_level);
        assert!((state.get(HormoneType::Dopamine).value() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn endocrine_state_mood_score_bounded() {
        let state = EndocrineState::default();
        let mood = state.mood_score();
        assert!(
            mood >= 0.0 && mood <= 1.0,
            "mood_score={mood} should be in [0,1]"
        );
    }

    #[test]
    fn endocrine_state_risk_tolerance_bounded() {
        let state = EndocrineState::default();
        let risk = state.risk_tolerance();
        assert!(
            risk >= 0.0 && risk <= 1.0,
            "risk_tolerance={risk} should be in [0,1]"
        );
    }

    #[test]
    fn endocrine_state_crisis_mode_detection() {
        let mut state = EndocrineState::default();
        assert!(!state.is_crisis_mode());

        state.adrenaline = HormoneLevel::new(0.8);
        assert!(state.is_crisis_mode());
    }

    #[test]
    fn endocrine_state_trusted_partnership() {
        let mut state = EndocrineState::default();
        assert!(!state.is_trusted_partnership());

        state.oxytocin = HormoneLevel::new(0.7);
        assert!(state.is_trusted_partnership());
    }

    #[test]
    fn endocrine_state_should_rest() {
        let mut state = EndocrineState::default();
        assert!(!state.should_rest());

        state.melatonin = HormoneLevel::new(0.8);
        assert!(state.should_rest());
    }

    #[test]
    fn apply_decay_increments_session_count() {
        let mut state = EndocrineState::default();
        assert_eq!(state.session_count, 0);
        state.apply_decay();
        assert_eq!(state.session_count, 1);
        state.apply_decay();
        assert_eq!(state.session_count, 2);
    }

    // --- Stimulus tests ---

    #[test]
    fn stimulus_error_increases_cortisol() {
        let mut state = EndocrineState::default();
        let initial_cortisol = state.cortisol.value();

        let stimulus = Stimulus::ErrorEncountered { severity: 0.8 };
        stimulus.apply(&mut state);

        assert!(
            state.cortisol.value() > initial_cortisol,
            "Cortisol should increase after error"
        );
    }

    #[test]
    fn stimulus_task_completed_boosts_dopamine() {
        let mut state = EndocrineState::default();
        let initial_dopamine = state.dopamine.value();

        let stimulus = Stimulus::TaskCompleted { complexity: 0.9 };
        stimulus.apply(&mut state);

        assert!(
            state.dopamine.value() > initial_dopamine,
            "Dopamine should increase after task completion"
        );
    }

    #[test]
    fn stimulus_critical_error_triggers_adrenaline() {
        let mut state = EndocrineState::default();
        let initial_adrenaline = state.adrenaline.value();

        let stimulus = Stimulus::CriticalError { recoverable: false };
        stimulus.apply(&mut state);

        assert!(state.adrenaline.value() > initial_adrenaline);
        assert!(
            state.cortisol.value() > 0.5,
            "Non-recoverable critical error should spike cortisol above baseline"
        );
    }

    #[test]
    fn stimulus_partnership_increases_oxytocin() {
        let mut state = EndocrineState::default();
        let initial_oxytocin = state.oxytocin.value();

        let stimulus = Stimulus::PartnershipReinforced { signal: 0.8 };
        stimulus.apply(&mut state);

        assert!(state.oxytocin.value() > initial_oxytocin);
    }

    #[test]
    fn stimulus_long_session_increases_melatonin() {
        let mut state = EndocrineState::default();
        let initial_melatonin = state.melatonin.value();

        let stimulus = Stimulus::SessionDuration { minutes: 90 };
        stimulus.apply(&mut state);

        assert!(
            state.melatonin.value() > initial_melatonin,
            "Long session should increase melatonin/fatigue"
        );
    }

    // --- BehavioralModifiers tests ---

    #[test]
    fn behavioral_modifiers_from_default_state() {
        let state = EndocrineState::default();
        let modifiers = BehavioralModifiers::from(&state);

        assert!(modifiers.risk_tolerance >= 0.0 && modifiers.risk_tolerance <= 1.0);
        assert!(modifiers.validation_depth >= 0.0 && modifiers.validation_depth <= 1.0);
        assert!(
            !modifiers.crisis_mode,
            "Default state should not be in crisis"
        );
        assert!(
            !modifiers.rest_recommended,
            "Default state should not need rest"
        );
    }

    #[test]
    fn behavioral_modifiers_crisis_from_high_adrenaline() {
        let mut state = EndocrineState::default();
        state.adrenaline = HormoneLevel::new(0.9);

        let modifiers = BehavioralModifiers::from(&state);
        assert!(modifiers.crisis_mode);
    }

    // --- Serialization tests ---

    #[test]
    fn endocrine_state_serialization_roundtrip() {
        let state = EndocrineState::default();
        let json = serde_json::to_string(&state).unwrap_or_default();
        assert!(!json.is_empty());

        let deserialized: std::result::Result<EndocrineState, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok());

        let restored = deserialized.unwrap_or_default();
        assert!((restored.cortisol.value() - state.cortisol.value()).abs() < f64::EPSILON);
        assert!((restored.dopamine.value() - state.dopamine.value()).abs() < f64::EPSILON);
    }

    // --- File I/O tests ---

    #[test]
    fn save_and_load_from_temp_path() {
        let dir = std::env::temp_dir().join("nexcore_hormones_test");
        let path = dir.join("state.json");

        let mut state = EndocrineState::default();
        state.dopamine = HormoneLevel::new(0.75);

        let save_result = state.save_to_path(&path);
        assert!(
            save_result.is_ok(),
            "save_to_path should succeed: {:?}",
            save_result.err()
        );

        let loaded = EndocrineState::load_from_path(&path);
        assert!(loaded.is_ok());
        let loaded = loaded.unwrap_or_default();
        assert!((loaded.dopamine.value() - 0.75).abs() < f64::EPSILON);

        // Clean up test artifacts
        if dir.exists() {
            std::fs::remove_dir_all(&dir).ok();
        }
    }
}

/// Behavioral modifiers derived from hormone state
/// Tier: T3 (Grounded in T1 Mapping μ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralModifiers {
    /// Risk tolerance (0=cautious, 1=bold)
    pub risk_tolerance: f64,
    /// Validation depth (0=minimal, 1=exhaustive)
    pub validation_depth: f64,
    /// Exploration rate (0=conservative, 1=experimental)
    pub exploration_rate: f64,
    /// Verbosity (0=terse, 1=verbose)
    pub verbosity: f64,
    /// Crisis mode active
    pub crisis_mode: bool,
    /// Partnership mode active
    pub partnership_mode: bool,
    /// Rest recommended
    pub rest_recommended: bool,
}

impl From<&EndocrineState> for BehavioralModifiers {
    fn from(state: &EndocrineState) -> Self {
        Self {
            risk_tolerance: state.risk_tolerance(),
            validation_depth: 0.5 + (state.cortisol.value() - 0.5) * 0.5,
            exploration_rate: state.dopamine.value(),
            verbosity: 0.5 + (state.oxytocin.value() - 0.5) * 0.3,
            crisis_mode: state.is_crisis_mode(),
            partnership_mode: state.is_trusted_partnership(),
            rest_recommended: state.should_rest(),
        }
    }
}
