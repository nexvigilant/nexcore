//! Temporal decay functions for betting signal value.
//!
//! # Campion Signal Theory - Temporal Component
//!
//! In pharmacovigilance, signal strength often changes over time as more
//! data accumulates. In sports betting, edge value decays as game time
//! approaches due to market efficiency.
//!
//! This module implements the lambda (lambda) decay function:
//!
//! ```text
//! decay_factor = exp(-lambda * hours_to_game)
//! ```
//!
//! # Lambda Values
//!
//! | Profile | Lambda | Description |
//! |---------|--------|-------------|
//! | Slow    | 0.05   | Signals maintain value longer |
//! | Standard| 0.10   | Default decay rate |
//! | Fast    | 0.20   | Signals degrade quickly |
//!
//! # Sport-Specific Lambda
//!
//! | Sport | Lambda | Rationale |
//! |-------|--------|-----------|
//! | NFL   | 0.08   | Weekly games, slow information accumulation |
//! | NBA   | 0.12   | Frequent games, rapid market adjustment |
//! | MLB   | 0.10   | Daily games, standard decay |
//! | NHL   | 0.10   | Similar frequency to MLB |
//!
//! # Codex Compliance
//!
//! - **T1**: `LAMBDA_*` constants
//! - **T2-P**: `HoursToGame`, `DecayFactor`, `LambdaCoefficient`
//! - **T2-C**: `TemporalDecay`, `DecayProfile`, `ActionWindow`
//! - **T3**: `SportType`, full calculation with interpretation

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

// =============================================================================
// T1 PRIMITIVES - Constants
// =============================================================================

/// Lambda for slow decay profile.
pub const LAMBDA_SLOW: f64 = 0.05;

/// Lambda for standard decay profile.
pub const LAMBDA_STANDARD: f64 = 0.10;

/// Lambda for fast decay profile.
pub const LAMBDA_FAST: f64 = 0.20;

/// Lambda for NFL (weekly games, slow information accumulation).
pub const LAMBDA_NFL: f64 = 0.08;

/// Lambda for NBA (frequent games, rapid market adjustment).
pub const LAMBDA_NBA: f64 = 0.12;

/// Lambda for MLB (daily games, standard decay).
pub const LAMBDA_MLB: f64 = 0.10;

/// Lambda for NHL (similar to MLB).
pub const LAMBDA_NHL: f64 = 0.10;

/// Lambda for Soccer (variable schedule, standard decay).
pub const LAMBDA_SOCCER: f64 = 0.10;

/// Temporal zone boundary: Very early (hours).
pub const ZONE_VERY_EARLY_HOURS: f64 = 72.0;

/// Temporal zone boundary: Early (hours).
pub const ZONE_EARLY_HOURS: f64 = 48.0;

/// Temporal zone boundary: Day before (hours).
pub const ZONE_DAY_BEFORE_HOURS: f64 = 24.0;

/// Temporal zone boundary: Game day (hours).
pub const ZONE_GAME_DAY_HOURS: f64 = 6.0;

/// Temporal zone boundary: Approaching (hours).
pub const ZONE_APPROACHING_HOURS: f64 = 1.0;

/// Urgency normalization factor (6 hours = moderate urgency).
pub const URGENCY_NORMALIZATION_HOURS: f64 = 6.0;

// =============================================================================
// T2-P NEWTYPES - Cross-domain primitives
// =============================================================================

/// Error for invalid temporal values.
#[derive(Debug, Clone, PartialEq)]
pub enum TemporalError {
    /// Value is negative (invalid for time-based metrics).
    Negative(f64),
    /// Value is NaN.
    NaN,
    /// Value is infinite.
    Infinite,
    /// Lambda coefficient is zero or negative.
    InvalidLambda(f64),
}

impl fmt::Display for TemporalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negative(v) => write!(f, "Temporal value cannot be negative: {v}"),
            Self::NaN => write!(f, "Temporal value cannot be NaN"),
            Self::Infinite => write!(f, "Temporal value cannot be infinite"),
            Self::InvalidLambda(v) => write!(f, "Lambda must be positive: {v}"),
        }
    }
}

impl std::error::Error for TemporalError {}

/// Hours until game start (T2-P newtype).
///
/// Non-negative value representing time remaining until game start.
/// Zero means the game is starting now.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HoursToGame(f64);

impl HoursToGame {
    /// Create a new `HoursToGame` value.
    ///
    /// # Errors
    ///
    /// Returns `TemporalError` if value is negative, NaN, or infinite.
    #[inline]
    pub fn new(value: f64) -> Result<Self, TemporalError> {
        if value.is_nan() {
            Err(TemporalError::NaN)
        } else if value.is_infinite() {
            Err(TemporalError::Infinite)
        } else if value < 0.0 {
            Err(TemporalError::Negative(value))
        } else {
            Ok(Self(value))
        }
    }

    /// Create a new `HoursToGame` value, clamping negatives to zero.
    #[inline]
    #[must_use]
    pub fn new_clamped(value: f64) -> Self {
        if value.is_nan() || value < 0.0 {
            Self(0.0)
        } else if value.is_infinite() {
            Self(f64::MAX)
        } else {
            Self(value)
        }
    }

    /// Create from raw value without validation.
    #[inline]
    #[must_use]
    pub const fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    /// Get the raw f64 value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Convert to minutes.
    #[inline]
    #[must_use]
    pub fn to_minutes(self) -> f64 {
        self.0 * 60.0
    }

    /// Game is starting now.
    pub const ZERO: Self = Self(0.0);

    /// One day before game.
    pub const ONE_DAY: Self = Self(24.0);

    /// One week before game.
    pub const ONE_WEEK: Self = Self(168.0);
}

impl Default for HoursToGame {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for HoursToGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}h", self.0)
    }
}

impl PartialEq for HoursToGame {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON
    }
}

impl PartialOrd for HoursToGame {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// Decay factor (T2-P newtype).
///
/// Value in range (0, 1] representing the proportion of signal value retained.
/// - 1.0 = full value retained (game is now)
/// - 0.0 = no value retained (far future)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DecayFactor(f64);

impl DecayFactor {
    /// Create a new `DecayFactor` value.
    ///
    /// # Errors
    ///
    /// Returns `TemporalError` if value is outside (0, 1], NaN, or infinite.
    #[inline]
    pub fn new(value: f64) -> Result<Self, TemporalError> {
        if value.is_nan() {
            Err(TemporalError::NaN)
        } else if value.is_infinite() {
            Err(TemporalError::Infinite)
        } else if value < 0.0 {
            Err(TemporalError::Negative(value))
        } else {
            Ok(Self(value.clamp(0.0, 1.0)))
        }
    }

    /// Create a new `DecayFactor` value, clamping to valid range.
    #[inline]
    #[must_use]
    pub fn new_clamped(value: f64) -> Self {
        if value.is_nan() || value < 0.0 {
            Self(0.0)
        } else {
            Self(value.min(1.0))
        }
    }

    /// Create from raw value without validation.
    #[inline]
    #[must_use]
    pub const fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    /// Get the raw f64 value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Convert to percentage retained.
    #[inline]
    #[must_use]
    pub fn as_percentage(self) -> f64 {
        self.0 * 100.0
    }

    /// Full value retained (game is now).
    pub const FULL: Self = Self(1.0);

    /// No value retained.
    pub const ZERO: Self = Self(0.0);

    /// Half value retained.
    pub const HALF: Self = Self(0.5);
}

impl Default for DecayFactor {
    fn default() -> Self {
        Self::FULL
    }
}

impl fmt::Display for DecayFactor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}%", self.0 * 100.0)
    }
}

impl PartialEq for DecayFactor {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON
    }
}

impl PartialOrd for DecayFactor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// Lambda coefficient for decay calculation (T2-P newtype).
///
/// Positive value controlling decay rate.
/// Higher lambda = faster decay.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LambdaCoefficient(f64);

impl LambdaCoefficient {
    /// Create a new `LambdaCoefficient` value.
    ///
    /// # Errors
    ///
    /// Returns `TemporalError` if value is non-positive, NaN, or infinite.
    #[inline]
    pub fn new(value: f64) -> Result<Self, TemporalError> {
        if value.is_nan() {
            Err(TemporalError::NaN)
        } else if value.is_infinite() {
            Err(TemporalError::Infinite)
        } else if value <= 0.0 {
            Err(TemporalError::InvalidLambda(value))
        } else {
            Ok(Self(value))
        }
    }

    /// Create a new `LambdaCoefficient` value, using default for invalid values.
    #[inline]
    #[must_use]
    pub fn new_clamped(value: f64) -> Self {
        if value.is_nan() || value.is_infinite() || value <= 0.0 {
            Self(LAMBDA_STANDARD)
        } else {
            Self(value)
        }
    }

    /// Create from raw value without validation.
    #[inline]
    #[must_use]
    pub const fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    /// Get the raw f64 value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Slow decay (signals maintain value longer).
    pub const SLOW: Self = Self(LAMBDA_SLOW);

    /// Standard decay rate.
    pub const STANDARD: Self = Self(LAMBDA_STANDARD);

    /// Fast decay (signals degrade quickly).
    pub const FAST: Self = Self(LAMBDA_FAST);
}

impl Default for LambdaCoefficient {
    fn default() -> Self {
        Self::STANDARD
    }
}

impl fmt::Display for LambdaCoefficient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lambda={:.2}", self.0)
    }
}

impl PartialEq for LambdaCoefficient {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON
    }
}

impl PartialOrd for LambdaCoefficient {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// =============================================================================
// T2-C COMPOSITES - Cross-domain structures
// =============================================================================

/// Decay profile preset (T2-C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DecayProfile {
    /// Slow decay - signals maintain value longer (lambda = 0.05).
    Slow,
    /// Standard decay - default rate (lambda = 0.10).
    #[default]
    Standard,
    /// Fast decay - signals degrade quickly (lambda = 0.20).
    Fast,
}

impl DecayProfile {
    /// Get the lambda coefficient for this profile.
    #[must_use]
    pub const fn lambda(&self) -> LambdaCoefficient {
        match self {
            Self::Slow => LambdaCoefficient::SLOW,
            Self::Standard => LambdaCoefficient::STANDARD,
            Self::Fast => LambdaCoefficient::FAST,
        }
    }
}

impl From<DecayProfile> for LambdaCoefficient {
    fn from(profile: DecayProfile) -> Self {
        profile.lambda()
    }
}

/// Sport type for temporal decay calculation (T3 domain type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SportType {
    /// NFL - Weekly games, slow info accumulation (lambda = 0.08).
    #[default]
    Nfl,
    /// NBA - Frequent games, rapid adjustment (lambda = 0.12).
    Nba,
    /// MLB - Daily games (lambda = 0.10).
    Mlb,
    /// NHL - Daily games (lambda = 0.10).
    Nhl,
    /// Soccer - Variable schedule (lambda = 0.10).
    Soccer,
    /// Custom lambda (stored as lambda * 1000 for precision).
    Custom(u32),
}

impl SportType {
    /// Get decay lambda for sport.
    #[must_use]
    pub fn lambda(&self) -> LambdaCoefficient {
        let raw = match self {
            Self::Nfl => LAMBDA_NFL,
            Self::Nba => LAMBDA_NBA,
            Self::Mlb | Self::Nhl | Self::Soccer => LAMBDA_MLB,
            Self::Custom(millis) => *millis as f64 / 1000.0,
        };
        LambdaCoefficient::new_unchecked(raw)
    }

    /// Create from custom lambda value.
    #[must_use]
    pub fn custom(lambda: f64) -> Self {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let millis = (lambda * 1000.0).round() as u32;
        Self::Custom(millis)
    }
}

impl From<SportType> for LambdaCoefficient {
    fn from(sport: SportType) -> Self {
        sport.lambda()
    }
}

/// Temporal zone classification (T2-C).
///
/// Classifies how far the game is based on hours remaining.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalZone {
    /// > 72 hours - Very early, signal may not persist.
    VeryEarly,
    /// 48-72 hours - Early market, high uncertainty.
    Early,
    /// 24-48 hours - Day before, moderate confidence.
    DayBefore,
    /// 6-24 hours - Game day, increasingly reliable.
    GameDay,
    /// 1-6 hours - Approaching kickoff, high confidence.
    Approaching,
    /// < 1 hour - Final hour, max reliability but limited action time.
    FinalHour,
    /// Game has started - signal no longer actionable.
    Started,
}

impl TemporalZone {
    /// Classify hours to game into temporal zone.
    #[must_use]
    pub fn from_hours(hours: HoursToGame) -> Self {
        let h = hours.value();
        if h <= 0.0 {
            Self::Started
        } else if h <= ZONE_APPROACHING_HOURS {
            Self::FinalHour
        } else if h <= ZONE_GAME_DAY_HOURS {
            Self::Approaching
        } else if h <= ZONE_DAY_BEFORE_HOURS {
            Self::GameDay
        } else if h <= ZONE_EARLY_HOURS {
            Self::DayBefore
        } else if h <= ZONE_VERY_EARLY_HOURS {
            Self::Early
        } else {
            Self::VeryEarly
        }
    }

    /// Get confidence multiplier for zone.
    #[must_use]
    pub const fn confidence_multiplier(&self) -> f64 {
        match self {
            Self::VeryEarly => 0.3,
            Self::Early => 0.5,
            Self::DayBefore => 0.7,
            Self::GameDay => 0.85,
            Self::Approaching => 0.95,
            Self::FinalHour => 1.0,
            Self::Started => 0.0,
        }
    }

    /// Check if signal is still actionable in this zone.
    #[must_use]
    pub const fn is_actionable(&self) -> bool {
        !matches!(self, Self::Started)
    }

    /// Get zone description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::VeryEarly => "Very early market - signal may not persist",
            Self::Early => "Early market - high uncertainty",
            Self::DayBefore => "Day before - moderate confidence",
            Self::GameDay => "Game day - increasingly reliable",
            Self::Approaching => "Approaching start - high confidence window",
            Self::FinalHour => "Final hour - maximum reliability, limited action time",
            Self::Started => "Game started - signal no longer actionable",
        }
    }
}

impl Default for TemporalZone {
    fn default() -> Self {
        Self::GameDay
    }
}

/// Temporal decay calculation result (T2-C / T3).
///
/// Contains the decay factor and contextual interpretation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalDecay {
    /// Hours until game starts.
    pub hours_to_game: HoursToGame,
    /// Lambda coefficient used for calculation.
    pub lambda_coefficient: LambdaCoefficient,
    /// Calculated decay factor (0-1).
    pub decay_factor: DecayFactor,
    /// Temporal zone classification.
    pub zone: TemporalZone,
    /// Human-readable interpretation.
    pub interpretation: String,
}

impl TemporalDecay {
    /// Check if signal is still actionable.
    #[must_use]
    pub fn is_actionable(&self) -> bool {
        self.zone.is_actionable()
    }

    /// Get the effective value multiplier (decay_factor * zone_confidence).
    #[must_use]
    pub fn effective_multiplier(&self) -> f64 {
        self.decay_factor.value() * self.zone.confidence_multiplier()
    }
}

/// Optimal action window result (T2-C / T3).
///
/// Calculates when a signal will drop below actionable threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionWindow {
    /// Hours remaining until signal drops below threshold.
    pub hours_remaining: f64,
    /// Urgency factor (0-1, higher = more urgent).
    pub urgency: f64,
    /// Whether the signal is currently actionable.
    pub is_actionable: bool,
}

impl ActionWindow {
    /// Signal is already below threshold (no action window).
    pub const EXPIRED: Self = Self {
        hours_remaining: 0.0,
        urgency: 1.0,
        is_actionable: false,
    };

    /// Signal has no decay (infinite window).
    pub const INFINITE: Self = Self {
        hours_remaining: f64::INFINITY,
        urgency: 0.0,
        is_actionable: true,
    };
}

// =============================================================================
// FUNCTIONS - T3 Domain Logic
// =============================================================================

/// Calculate temporal decay factor.
///
/// Formula: `decay = exp(-lambda * hours_to_game)`
///
/// Returns value in (0, 1] where:
/// - 1.0 = game is now (maximum reliability)
/// - ~0.5 = game is at half-life
/// - approaching 0 = game is far in future
///
/// # Arguments
///
/// * `hours_to_game` - Hours until game start
/// * `sport` - Sport type for lambda selection
///
/// # Example
///
/// ```
/// use nexcore_vigilance::betting::temporal::{temporal_decay, SportType};
///
/// let decay = temporal_decay(24.0, SportType::Nfl);
/// assert!(decay > 0.0 && decay < 1.0);
/// ```
#[must_use]
pub fn temporal_decay(hours_to_game: f64, sport: SportType) -> f64 {
    let hours = hours_to_game.max(0.0);
    let lambda = sport.lambda().value();
    (-lambda * hours).exp()
}

/// Calculate complete temporal decay with interpretation.
///
/// # Arguments
///
/// * `hours_to_game` - Hours until game start
/// * `lambda_coefficient` - Explicit lambda value (overrides sport/profile)
/// * `sport` - Use sport-specific lambda if provided
/// * `decay_profile` - Use preset profile (slow/standard/fast)
///
/// Priority: `lambda_coefficient` > `sport` > `decay_profile` > default (standard)
///
/// # Example
///
/// ```
/// use nexcore_vigilance::betting::temporal::{calculate_temporal_decay, SportType};
///
/// let decay = calculate_temporal_decay(24.0, None, Some(SportType::Nfl), None);
/// println!("Value retained: {}", decay.decay_factor);
/// ```
#[must_use]
pub fn calculate_temporal_decay(
    hours_to_game: f64,
    lambda_coefficient: Option<f64>,
    sport: Option<SportType>,
    decay_profile: Option<DecayProfile>,
) -> TemporalDecay {
    // Determine lambda coefficient (priority order)
    let lambda = if let Some(lam) = lambda_coefficient {
        LambdaCoefficient::new_clamped(lam)
    } else if let Some(s) = sport {
        s.lambda()
    } else if let Some(profile) = decay_profile {
        profile.lambda()
    } else {
        LambdaCoefficient::STANDARD
    };

    // Ensure hours is non-negative
    let hours = HoursToGame::new_clamped(hours_to_game);

    // Calculate decay factor: exp(-lambda * t)
    let decay_raw = (-lambda.value() * hours.value()).exp();
    let decay_factor = DecayFactor::new_clamped(decay_raw);

    // Classify temporal zone
    let zone = TemporalZone::from_hours(hours);

    // Generate interpretation
    let interpretation = generate_interpretation(hours, decay_factor, lambda, zone);

    TemporalDecay {
        hours_to_game: hours,
        lambda_coefficient: lambda,
        decay_factor,
        zone,
        interpretation,
    }
}

/// Generate human-readable interpretation of temporal decay.
fn generate_interpretation(
    hours: HoursToGame,
    decay: DecayFactor,
    lambda: LambdaCoefficient,
    zone: TemporalZone,
) -> String {
    let h = hours.value();
    let pct = decay.as_percentage();
    let lam = lambda.value();

    match zone {
        TemporalZone::VeryEarly => {
            format!(
                "Very early market ({h:.0}h out) - signal may not persist, {pct:.1}% value retained with lambda={lam:.2}"
            )
        }
        TemporalZone::Early => {
            format!("Early market ({h:.0}h out) - high uncertainty, {pct:.1}% value retained")
        }
        TemporalZone::DayBefore => {
            format!("Day before ({h:.0}h out) - moderate confidence, {pct:.1}% value retained")
        }
        TemporalZone::GameDay => {
            format!(
                "Game day ({h:.1}h out) - signal increasingly reliable, {pct:.1}% value retained"
            )
        }
        TemporalZone::Approaching => {
            format!(
                "Approaching start ({h:.1}h out) - high confidence window, {pct:.1}% value retained"
            )
        }
        TemporalZone::FinalHour => {
            let mins = hours.to_minutes();
            format!(
                "Final hour ({mins:.0}min out) - maximum reliability, {pct:.1}% value retained, limited action time"
            )
        }
        TemporalZone::Started => "Game started - signal no longer actionable".to_string(),
    }
}

/// Calculate the optimal action window for a signal.
///
/// Given an initial BDI value and decay rate, determine when the
/// signal will drop below actionable threshold.
///
/// # Arguments
///
/// * `initial_bdi` - Starting BDI value
/// * `min_actionable_bdi` - Minimum BDI to consider actionable (default: 2.0)
/// * `lambda_coefficient` - Decay rate (default: 0.10)
///
/// # Returns
///
/// `ActionWindow` containing:
/// - `hours_remaining`: Hours until signal drops below threshold
/// - `urgency`: Urgency factor (0-1, higher = more urgent)
/// - `is_actionable`: Whether signal is currently actionable
///
/// # Example
///
/// ```
/// use nexcore_vigilance::betting::temporal::calculate_optimal_action_window;
///
/// let window = calculate_optimal_action_window(4.0, 2.0, 0.10);
/// println!("Signal actionable for {:.1} more hours", window.hours_remaining);
/// ```
#[must_use]
pub fn calculate_optimal_action_window(
    initial_bdi: f64,
    min_actionable_bdi: f64,
    lambda_coefficient: f64,
) -> ActionWindow {
    // Validate inputs
    if initial_bdi.is_nan() || min_actionable_bdi.is_nan() || lambda_coefficient.is_nan() {
        return ActionWindow::EXPIRED;
    }

    // Already below threshold
    if initial_bdi <= min_actionable_bdi {
        return ActionWindow::EXPIRED;
    }

    // No decay means infinite window
    if lambda_coefficient <= 0.0 {
        return ActionWindow::INFINITE;
    }

    // Solve: initial_bdi * exp(-lambda * t) = min_actionable_bdi
    // t = -ln(min_actionable_bdi / initial_bdi) / lambda
    let ratio = min_actionable_bdi / initial_bdi;
    let hours_remaining = -ratio.ln() / lambda_coefficient;

    // Calculate urgency factor (inverse of time remaining, normalized)
    // 0 hours = 1.0 urgency, 24 hours = ~0.0 urgency
    let urgency = 1.0 / (1.0 + hours_remaining / URGENCY_NORMALIZATION_HOURS);

    ActionWindow {
        hours_remaining,
        urgency,
        is_actionable: true,
    }
}

/// Calculate half-life (hours until decay factor reaches 0.5).
///
/// Formula: `t_half = ln(2) / lambda`
///
/// # Arguments
///
/// * `lambda` - Lambda coefficient for decay
///
/// # Returns
///
/// Hours until decay factor reaches 0.5.
#[must_use]
pub fn calculate_half_life(lambda: LambdaCoefficient) -> f64 {
    core::f64::consts::LN_2 / lambda.value()
}

/// Calculate the lambda coefficient needed to achieve a target decay at a given time.
///
/// Formula: `lambda = -ln(target_decay) / hours`
///
/// # Arguments
///
/// * `target_decay` - Target decay factor (0-1)
/// * `hours` - Hours at which target should be reached
///
/// # Returns
///
/// Lambda coefficient, or `None` if inputs are invalid.
#[must_use]
pub fn calculate_lambda_for_target(target_decay: f64, hours: f64) -> Option<LambdaCoefficient> {
    if target_decay <= 0.0 || target_decay >= 1.0 || hours <= 0.0 {
        return None;
    }

    let lambda = -target_decay.ln() / hours;
    LambdaCoefficient::new(lambda).ok()
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Newtype Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_hours_to_game_validation() {
        assert!(HoursToGame::new(24.0).is_ok());
        assert!(HoursToGame::new(0.0).is_ok());
        assert!(HoursToGame::new(-1.0).is_err());
        assert!(HoursToGame::new(f64::NAN).is_err());
        assert!(HoursToGame::new(f64::INFINITY).is_err());
    }

    #[test]
    fn test_hours_to_game_clamping() {
        let clamped = HoursToGame::new_clamped(-5.0);
        assert!((clamped.value() - 0.0).abs() < f64::EPSILON);

        let nan_clamped = HoursToGame::new_clamped(f64::NAN);
        assert!((nan_clamped.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_decay_factor_bounds() {
        let df = DecayFactor::new(0.5);
        assert!(df.is_ok());
        assert!((df.map(|d| d.value()).unwrap_or(0.0) - 0.5).abs() < f64::EPSILON);

        // Clamps to 1.0
        let over = DecayFactor::new(1.5);
        assert!(over.is_ok());
        assert!((over.map(|d| d.value()).unwrap_or(0.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lambda_coefficient_validation() {
        assert!(LambdaCoefficient::new(0.1).is_ok());
        assert!(LambdaCoefficient::new(0.0).is_err());
        assert!(LambdaCoefficient::new(-0.1).is_err());
        assert!(LambdaCoefficient::new(f64::NAN).is_err());
    }

    // -------------------------------------------------------------------------
    // Temporal Decay Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_temporal_decay_at_game_time() {
        let decay = temporal_decay(0.0, SportType::Nfl);
        assert!(
            (decay - 1.0).abs() < 0.001,
            "At game time, decay should be 1.0"
        );
    }

    #[test]
    fn test_temporal_decay_decreases_with_time() {
        let at_game = temporal_decay(0.0, SportType::Nfl);
        let day_before = temporal_decay(24.0, SportType::Nfl);
        let week_before = temporal_decay(168.0, SportType::Nfl);

        assert!(day_before < at_game, "Decay should decrease with time");
        assert!(week_before < day_before, "Decay should continue decreasing");
    }

    #[test]
    fn test_sport_specific_lambda_nba_faster_than_nfl() {
        let nfl_decay = temporal_decay(24.0, SportType::Nfl);
        let nba_decay = temporal_decay(24.0, SportType::Nba);

        assert!(
            nba_decay < nfl_decay,
            "NBA (lambda=0.12) should decay faster than NFL (lambda=0.08)"
        );
    }

    #[test]
    fn test_sport_lambda_values() {
        assert!((SportType::Nfl.lambda().value() - LAMBDA_NFL).abs() < f64::EPSILON);
        assert!((SportType::Nba.lambda().value() - LAMBDA_NBA).abs() < f64::EPSILON);
        assert!((SportType::Mlb.lambda().value() - LAMBDA_MLB).abs() < f64::EPSILON);
    }

    #[test]
    fn test_custom_sport_lambda() {
        let custom = SportType::custom(0.15);
        assert!((custom.lambda().value() - 0.15).abs() < 0.001);
    }

    // -------------------------------------------------------------------------
    // Temporal Zone Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_temporal_zone_classification() {
        assert_eq!(
            TemporalZone::from_hours(HoursToGame::new_unchecked(100.0)),
            TemporalZone::VeryEarly
        );
        assert_eq!(
            TemporalZone::from_hours(HoursToGame::new_unchecked(50.0)),
            TemporalZone::Early
        );
        assert_eq!(
            TemporalZone::from_hours(HoursToGame::new_unchecked(30.0)),
            TemporalZone::DayBefore
        );
        assert_eq!(
            TemporalZone::from_hours(HoursToGame::new_unchecked(12.0)),
            TemporalZone::GameDay
        );
        assert_eq!(
            TemporalZone::from_hours(HoursToGame::new_unchecked(3.0)),
            TemporalZone::Approaching
        );
        assert_eq!(
            TemporalZone::from_hours(HoursToGame::new_unchecked(0.5)),
            TemporalZone::FinalHour
        );
        assert_eq!(
            TemporalZone::from_hours(HoursToGame::new_unchecked(0.0)),
            TemporalZone::Started
        );
    }

    #[test]
    fn test_temporal_zone_actionability() {
        assert!(TemporalZone::VeryEarly.is_actionable());
        assert!(TemporalZone::FinalHour.is_actionable());
        assert!(!TemporalZone::Started.is_actionable());
    }

    #[test]
    fn test_temporal_zone_confidence_ordering() {
        // Confidence should increase as we get closer to game time
        let zones = [
            TemporalZone::VeryEarly,
            TemporalZone::Early,
            TemporalZone::DayBefore,
            TemporalZone::GameDay,
            TemporalZone::Approaching,
            TemporalZone::FinalHour,
        ];

        for i in 0..zones.len() - 1 {
            assert!(
                zones[i].confidence_multiplier() < zones[i + 1].confidence_multiplier(),
                "Confidence should increase closer to game"
            );
        }
    }

    // -------------------------------------------------------------------------
    // Complete Calculation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_calculate_temporal_decay_with_sport() {
        let result = calculate_temporal_decay(24.0, None, Some(SportType::Nfl), None);

        assert!(result.hours_to_game.value() > 0.0);
        assert!(result.decay_factor.value() > 0.0);
        assert!(result.decay_factor.value() < 1.0);
        assert!((result.lambda_coefficient.value() - LAMBDA_NFL).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_temporal_decay_priority() {
        // Explicit lambda should override sport
        let result = calculate_temporal_decay(24.0, Some(0.05), Some(SportType::Nba), None);
        assert!((result.lambda_coefficient.value() - 0.05).abs() < f64::EPSILON);

        // Sport should override profile
        let result2 =
            calculate_temporal_decay(24.0, None, Some(SportType::Nfl), Some(DecayProfile::Fast));
        assert!((result2.lambda_coefficient.value() - LAMBDA_NFL).abs() < f64::EPSILON);

        // Profile should be used when no sport
        let result3 = calculate_temporal_decay(24.0, None, None, Some(DecayProfile::Slow));
        assert!((result3.lambda_coefficient.value() - LAMBDA_SLOW).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_temporal_decay_interpretation() {
        let very_early = calculate_temporal_decay(100.0, None, None, None);
        assert!(very_early.interpretation.contains("Very early"));

        let final_hour = calculate_temporal_decay(0.5, None, None, None);
        assert!(final_hour.interpretation.contains("Final hour"));

        let started = calculate_temporal_decay(0.0, None, None, None);
        assert!(started.interpretation.contains("started"));
    }

    // -------------------------------------------------------------------------
    // Action Window Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_action_window_calculation() {
        let window = calculate_optimal_action_window(4.0, 2.0, 0.10);

        assert!(window.is_actionable);
        assert!(window.hours_remaining > 0.0);
        assert!(window.urgency > 0.0 && window.urgency < 1.0);

        // Verify: at hours_remaining, value should be near threshold
        let decay_at_deadline = (-0.10 * window.hours_remaining).exp();
        let bdi_at_deadline = 4.0 * decay_at_deadline;
        assert!(
            (bdi_at_deadline - 2.0).abs() < 0.01,
            "BDI at deadline should be ~2.0"
        );
    }

    #[test]
    fn test_action_window_already_below_threshold() {
        let window = calculate_optimal_action_window(1.5, 2.0, 0.10);

        assert!(!window.is_actionable);
        assert!((window.hours_remaining - 0.0).abs() < f64::EPSILON);
        assert!((window.urgency - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_action_window_no_decay() {
        let window = calculate_optimal_action_window(4.0, 2.0, 0.0);

        assert!(window.is_actionable);
        assert!(window.hours_remaining.is_infinite());
        assert!((window.urgency - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_action_window_nan_handling() {
        let window = calculate_optimal_action_window(f64::NAN, 2.0, 0.10);
        assert!(!window.is_actionable);
    }

    // -------------------------------------------------------------------------
    // Utility Function Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_half_life_calculation() {
        let lambda = LambdaCoefficient::new_unchecked(0.10);
        let half_life = calculate_half_life(lambda);

        // At half-life, decay should be ~0.5
        let decay_at_half = (-0.10 * half_life).exp();
        assert!(
            (decay_at_half - 0.5).abs() < 0.01,
            "Decay at half-life should be ~0.5"
        );
    }

    #[test]
    fn test_lambda_for_target() {
        // Want 50% decay at 24 hours
        let lambda = calculate_lambda_for_target(0.5, 24.0);
        assert!(lambda.is_some());

        let lam = lambda.map(|l| l.value()).unwrap_or(0.0);
        let decay_at_24h = (-lam * 24.0).exp();
        assert!(
            (decay_at_24h - 0.5).abs() < 0.01,
            "Should achieve target decay"
        );
    }

    #[test]
    fn test_lambda_for_target_invalid_inputs() {
        assert!(calculate_lambda_for_target(0.0, 24.0).is_none());
        assert!(calculate_lambda_for_target(1.0, 24.0).is_none());
        assert!(calculate_lambda_for_target(0.5, 0.0).is_none());
        assert!(calculate_lambda_for_target(0.5, -1.0).is_none());
    }

    // -------------------------------------------------------------------------
    // Decay Profile Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_decay_profiles() {
        let slow = DecayProfile::Slow.lambda().value();
        let standard = DecayProfile::Standard.lambda().value();
        let fast = DecayProfile::Fast.lambda().value();

        assert!(slow < standard, "Slow should have smaller lambda");
        assert!(standard < fast, "Fast should have larger lambda");
    }

    #[test]
    fn test_decay_profile_effect() {
        let hours = 24.0;

        let slow_decay = calculate_temporal_decay(hours, None, None, Some(DecayProfile::Slow));
        let fast_decay = calculate_temporal_decay(hours, None, None, Some(DecayProfile::Fast));

        assert!(
            slow_decay.decay_factor.value() > fast_decay.decay_factor.value(),
            "Slow decay should retain more value"
        );
    }
}
