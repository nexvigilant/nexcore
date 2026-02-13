//! Edge Confidence Score (ECS) - EBGM adapted for sports betting.
//!
//! ECS combines three components following Campion Signal Theory:
//!
//! ```text
//! ECS = U * R * T
//! ```
//!
//! Where:
//! - **U (Unexpectedness)**: How rare/unexpected is this signal configuration?
//!   Adapted from Information Component (IC) in BCPNN method.
//! - **R (Reliability)**: How reliable is the data source?
//!   Adapted from data quality factors in EBGM calculations.
//! - **T (Temporal)**: How much has the signal decayed?
//!   Implements lambda decay function.
//!
//! The Bayesian credibility interval (`lower_credibility`, `upper_credibility`)
//! is adapted from EB05/EB95 in FDA's EBGM approach.
//!
//! # References
//! - DuMouchel W (1999) Am Stat 53:177-90 (EBGM)
//! - Bate A et al. (1998) Eur J Clin Pharmacol 54:315-21 (BCPNN/IC)
//!
//! # Codex Compliance
//! - **Tier**: T2-C / T3 (Domain-Specific)
//! - **Grounding**: All newtypes wrap T1 primitives
//! - **Commandment IV**: No naked primitives for domain values

use serde::{Deserialize, Serialize};
use std::fmt;

use super::temporal::{HoursToGame, SportType, TemporalZone, temporal_decay};
use super::thresholds::{ECS_MODERATE, SignalStrength};

// =============================================================================
// CONSTANTS - T1 Primitives
// =============================================================================

/// Default ECS threshold for actionable signals.
const ECS_ACTIONABLE: f64 = ECS_MODERATE;

/// Default lower credibility threshold (EB05 analog).
const LOWER_CREDIBILITY_THRESHOLD: f64 = 0.0;

/// Component weights in ECS calculation (additive formula variant).
const U_WEIGHT: f64 = 0.40;
const R_WEIGHT: f64 = 0.35;
const T_WEIGHT: f64 = 0.25;

/// Prior strength for Bayesian credibility interval.
const DEFAULT_PRIOR_STRENGTH: f64 = 0.5;

/// Default observation count for simplified calculations.
const DEFAULT_N_OBSERVATIONS: u32 = 10;

/// Scale factor for component normalization (0-10 range).
const COMPONENT_SCALE: f64 = 10.0;

/// Global ECS denominator for multiplicative formula.
const ECS_DENOMINATOR: f64 = 100.0;

/// Steam bonus for unexpectedness calculation.
const STEAM_BONUS: f64 = 0.5;

/// Log2 scale factor for unexpectedness transformation.
const LOG2_SCALE: f64 = 4.0;

/// Base surprise factor for Reverse Line Movement.
const RLM_BASE_SURPRISE: f64 = 0.5;

/// Scale factor for expected movement surprise (low surprise).
const EXPECTED_MOVEMENT_SCALE: f64 = 0.2;

/// Reliability weight for data completeness.
const R_WEIGHT_COMPLETENESS: f64 = 0.25;
/// Reliability weight for historical accuracy.
const R_WEIGHT_ACCURACY: f64 = 0.35;
/// Reliability weight for market liquidity.
const R_WEIGHT_LIQUIDITY: f64 = 0.25;
/// Reliability weight for source quality.
const R_WEIGHT_SOURCE: f64 = 0.15;

// =============================================================================
// NEWTYPES - T2-P (Primitive Wrappers)
// =============================================================================

/// ECS score value.
///
/// # Tier: T2-P
/// # Invariant: Non-negative
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct EcsScore(f64);

impl EcsScore {
    /// Create a new ECS score, clamping to non-negative.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.max(0.0))
    }

    /// Get the raw value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl From<f64> for EcsScore {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for EcsScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

/// Unexpectedness component (U in Campion Signal Theory).
///
/// Measures how surprising the signal configuration is.
/// Higher values indicate line moving against public expectation.
///
/// # Tier: T2-P
/// # Range: [0.0, 10.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Unexpectedness(f64);

impl Unexpectedness {
    /// Create unexpectedness, clamping to [0, 10].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, COMPONENT_SCALE))
    }

    /// Get the raw value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for Unexpectedness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

/// Reliability component (R in Campion Signal Theory).
///
/// Composite score from data quality factors.
///
/// # Tier: T2-P
/// # Range: [0.0, 10.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Reliability(f64);

impl Reliability {
    /// Create reliability, clamping to [0, 10].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, COMPONENT_SCALE))
    }

    /// Get the raw value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for Reliability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

/// Temporal decay component (T in Campion Signal Theory).
///
/// Higher values when closer to game time (more reliable signal).
///
/// # Tier: T2-P
/// # Range: [0.0, 10.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TemporalComponent(f64);

impl TemporalComponent {
    /// Create temporal component, clamping to [0, 10].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, COMPONENT_SCALE))
    }

    /// Get the raw value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for TemporalComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

/// Lower credibility bound (EB05 analog).
///
/// 5th percentile of the Bayesian posterior distribution.
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CredibilityLower(f64);

impl CredibilityLower {
    /// Create lower bound, ensuring non-negative.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.max(0.0))
    }

    /// Get the raw value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if lower bound excludes null hypothesis.
    #[must_use]
    pub fn excludes_null(&self) -> bool {
        self.0 > LOWER_CREDIBILITY_THRESHOLD
    }
}

/// Upper credibility bound (EB95 analog).
///
/// 95th percentile of the Bayesian posterior distribution.
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CredibilityUpper(f64);

impl CredibilityUpper {
    /// Create upper bound.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// Get the raw value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }
}

// =============================================================================
// INPUT TYPES - T2-C
// =============================================================================

/// Input factors for reliability component calculation.
///
/// All factors should be in range [0.0, 1.0].
///
/// # Tier: T2-C
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReliabilityInput {
    /// Data completeness factor (0-1).
    /// Higher when more data fields are available.
    pub completeness: f64,
    /// Historical accuracy factor (0-1).
    /// Track record of similar signals producing winning results.
    pub accuracy: f64,
    /// Market liquidity factor (0-1).
    /// Higher for major markets with deep liquidity.
    pub liquidity: f64,
    /// Odds source quality factor (0-1).
    /// Quality/reliability of the odds data provider.
    pub source_quality: f64,
}

impl Default for ReliabilityInput {
    fn default() -> Self {
        Self {
            completeness: 0.7,
            accuracy: 0.6,
            liquidity: 0.8,
            source_quality: 0.9,
        }
    }
}

impl ReliabilityInput {
    /// Create new reliability input with all factors.
    #[must_use]
    pub fn new(completeness: f64, accuracy: f64, liquidity: f64, source_quality: f64) -> Self {
        Self {
            completeness: completeness.clamp(0.0, 1.0),
            accuracy: accuracy.clamp(0.0, 1.0),
            liquidity: liquidity.clamp(0.0, 1.0),
            source_quality: source_quality.clamp(0.0, 1.0),
        }
    }

    /// Create with high reliability (for testing/elite signals).
    #[must_use]
    pub fn high() -> Self {
        Self {
            completeness: 0.95,
            accuracy: 0.85,
            liquidity: 0.95,
            source_quality: 0.95,
        }
    }

    /// Create with low reliability (for testing/marginal signals).
    #[must_use]
    pub fn low() -> Self {
        Self {
            completeness: 0.4,
            accuracy: 0.4,
            liquidity: 0.4,
            source_quality: 0.5,
        }
    }

    /// Compute weighted composite reliability.
    ///
    /// Weights:
    /// - Completeness: 25%
    /// - Accuracy: 35%
    /// - Liquidity: 25%
    /// - Source quality: 15%
    #[must_use]
    pub fn composite(&self) -> f64 {
        let c = self.completeness.clamp(0.0, 1.0);
        let a = self.accuracy.clamp(0.0, 1.0);
        let l = self.liquidity.clamp(0.0, 1.0);
        let s = self.source_quality.clamp(0.0, 1.0);

        R_WEIGHT_COMPLETENESS * c
            + R_WEIGHT_ACCURACY * a
            + R_WEIGHT_LIQUIDITY * l
            + R_WEIGHT_SOURCE * s
    }
}

// =============================================================================
// OUTPUT TYPES - T2-C / T3
// =============================================================================

/// ECS component breakdown.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EcsComponents {
    /// Unexpectedness score (0-10) - Higher when line moves against public.
    pub unexpectedness: f64,
    /// Reliability score (0-10) - Based on data completeness and accuracy.
    pub reliability: f64,
    /// Temporal decay factor (0-10) - Higher closer to game time.
    pub temporal: f64,
}

impl EcsComponents {
    /// Get typed unexpectedness component.
    #[must_use]
    pub fn u(&self) -> Unexpectedness {
        Unexpectedness::new(self.unexpectedness)
    }

    /// Get typed reliability component.
    #[must_use]
    pub fn r(&self) -> Reliability {
        Reliability::new(self.reliability)
    }

    /// Get typed temporal component.
    #[must_use]
    pub fn t(&self) -> TemporalComponent {
        TemporalComponent::new(self.temporal)
    }
}

/// Complete ECS calculation result.
///
/// # Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsResult {
    /// Final ECS score (U x R x T / 100).
    pub ecs: f64,
    /// Component breakdown.
    pub components: EcsComponents,
    /// Lower credibility bound (5th percentile, EB05 analog).
    pub lower_credibility: f64,
    /// Upper credibility bound (95th percentile, EB95 analog).
    pub upper_credibility: f64,
    /// Signal strength classification.
    pub signal_strength: SignalStrength,
    /// Whether the signal is actionable (ECS >= threshold AND lower > 0).
    pub is_actionable: bool,
}

impl EcsResult {
    /// Get typed ECS score.
    #[must_use]
    pub fn score(&self) -> EcsScore {
        EcsScore::new(self.ecs)
    }

    /// Get typed lower credibility bound.
    #[must_use]
    pub fn lower(&self) -> CredibilityLower {
        CredibilityLower::new(self.lower_credibility)
    }

    /// Get typed upper credibility bound.
    #[must_use]
    pub fn upper(&self) -> CredibilityUpper {
        CredibilityUpper::new(self.upper_credibility)
    }

    /// Generate human-readable interpretation.
    #[must_use]
    pub fn interpretation(&self) -> String {
        generate_interpretation(
            self.ecs,
            self.lower_credibility,
            self.is_actionable,
            self.signal_strength,
        )
    }

    /// Check if the credibility interval is narrow (high confidence).
    #[must_use]
    pub fn is_high_confidence(&self) -> bool {
        let width = self.upper_credibility - self.lower_credibility;
        width < self.ecs * 0.5 // Width less than 50% of point estimate
    }
}

/// Extended ECS result with temporal zone and interpretation.
///
/// # Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsResultExtended {
    /// Base ECS result.
    pub result: EcsResult,
    /// Temporal zone classification.
    pub temporal_zone: TemporalZone,
    /// Human-readable interpretation.
    pub interpretation: String,
}

// =============================================================================
// CORE CALCULATIONS - T1 Operations
// =============================================================================

/// Calculate the Unexpectedness (U) component.
///
/// Adapted from Information Component (IC) in BCPNN:
/// IC = log2(observed / expected)
///
/// In betting context:
/// - "Expected" = line moves WITH public money
/// - "Observed" = actual line movement
/// - High U when line moves AGAINST public (Reverse Line Movement)
///
/// # Arguments
/// - `public_normalized`: Public betting percentage (0.0 - 1.0)
/// - `line_direction`: -1 = toward underdog, 0 = no move, 1 = toward favorite
/// - `steam_detected`: Whether steam move was detected
///
/// # Returns
/// Unexpectedness score in [0, 10] range.
fn calculate_unexpectedness(
    public_normalized: f64,
    line_direction: i8,
    steam_detected: bool,
) -> f64 {
    // Expected direction: toward the side with more public money
    let expected_direction: i8 = if public_normalized > 0.5 { 1 } else { -1 };

    // Base unexpectedness from direction mismatch
    let public_imbalance = (public_normalized - 0.5).abs();

    let direction_surprise = match line_direction {
        0 => {
            // No movement - slightly unexpected if public heavily one-sided
            public_imbalance
        }
        dir if dir != expected_direction => {
            // Reverse line movement - very unexpected
            public_imbalance + RLM_BASE_SURPRISE
        }
        _ => {
            // Movement with public - expected, low surprise
            EXPECTED_MOVEMENT_SCALE * (1.0 - public_imbalance)
        }
    };

    // Steam moves add unexpectedness (rapid cross-book movement is rare)
    let steam_bonus = if steam_detected { STEAM_BONUS } else { 0.0 };

    // Apply log2 transformation similar to IC
    let raw_u = direction_surprise + steam_bonus;
    let u = (1.0 + raw_u * LOG2_SCALE).log2();

    u.clamp(0.0, COMPONENT_SCALE)
}

/// Calculate the Reliability (R) component.
///
/// Adapted from data quality adjustments in EBGM.
fn calculate_reliability(input: &ReliabilityInput) -> f64 {
    let composite = input.composite();
    (composite * COMPONENT_SCALE).clamp(0.0, COMPONENT_SCALE)
}

/// Calculate the Temporal (T) component.
///
/// Uses sport-specific lambda decay function.
fn calculate_temporal_component(hours_to_game: f64, sport: SportType) -> f64 {
    let decay_factor = temporal_decay(hours_to_game, sport);
    (decay_factor * COMPONENT_SCALE).clamp(0.0, COMPONENT_SCALE)
}

/// Calculate Bayesian credibility interval for ECS.
///
/// Adapted from EB05/EB95 in EBGM methodology.
///
/// Uses a gamma prior to shrink extreme estimates toward
/// the prior mean, especially when sample size is small.
///
/// # Arguments
/// - `ecs`: Point estimate of ECS
/// - `n_observations`: Number of supporting observations
/// - `prior_strength`: Strength of prior (higher = more shrinkage)
///
/// # Returns
/// Tuple of (lower, upper) credibility bounds (5th and 95th percentiles).
fn calculate_bayesian_credibility(
    ecs: f64,
    n_observations: u32,
    prior_strength: f64,
) -> (f64, f64) {
    if n_observations == 0 {
        // No data - return prior interval
        return (0.0, prior_strength * 2.0);
    }

    let n = f64::from(n_observations);

    // Calculate posterior parameters (gamma distribution)
    // Shape and rate parameters for gamma posterior
    let shape = n * ecs / 2.0 + prior_strength;
    let rate = n / 2.0 + prior_strength;

    // Guard against invalid parameters
    if shape <= 0.0 || rate <= 0.0 {
        return (0.0, ecs * 2.0);
    }

    // Calculate 5th and 95th percentiles using gamma quantile approximation
    let scale = 1.0 / rate;

    // For 5th percentile (p = 0.05, z ≈ -1.645)
    let lower = gamma_quantile(shape, scale, 0.05);

    // For 95th percentile (p = 0.95, z ≈ 1.645)
    let upper = gamma_quantile(shape, scale, 0.95);

    (lower.max(0.0), upper)
}

/// Approximate gamma distribution quantile using Wilson-Hilferty transformation.
///
/// This approximation is accurate for shape > 1 and reasonable for smaller values.
fn gamma_quantile(shape: f64, scale: f64, p: f64) -> f64 {
    // Convert probability to z-score
    let z = normal_quantile(p);

    // Wilson-Hilferty transformation
    // X ~ Gamma(shape, scale) => (X/scale)^(1/3) ≈ N(1 - 1/(9*shape), 1/(9*shape))
    let h = 1.0 / (9.0 * shape);
    let transformed = (1.0 - h + z * h.sqrt()).powi(3);

    // Scale back to original distribution
    (shape * scale * transformed).max(0.0)
}

/// Approximate normal distribution quantile (inverse CDF).
///
/// Uses Abramowitz and Stegun approximation.
fn normal_quantile(p: f64) -> f64 {
    // Handle edge cases
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    // Use rational approximation for central region
    let t = if p < 0.5 {
        (-2.0 * p.ln()).sqrt()
    } else {
        (-2.0 * (1.0 - p).ln()).sqrt()
    };

    // Coefficients for approximation
    const C0: f64 = 2.515517;
    const C1: f64 = 0.802853;
    const C2: f64 = 0.010328;
    const D1: f64 = 1.432788;
    const D2: f64 = 0.189269;
    const D3: f64 = 0.001308;

    let num = C0 + C1 * t + C2 * t * t;
    let den = 1.0 + D1 * t + D2 * t * t + D3 * t * t * t;
    let result = t - num / den;

    if p < 0.5 { -result } else { result }
}

/// Calculate credibility interval using simple approximation.
///
/// This is a simplified version that uses reliability as a proxy
/// for sample size / precision.
fn calculate_credibility_interval_simple(ecs: f64, reliability: f64) -> (f64, f64) {
    // Shrinkage toward prior based on reliability
    // Higher reliability = less shrinkage
    let shrinkage = 1.0 - (reliability * 0.8);
    let prior_mean = 1.0;

    let posterior_mean = ecs * (1.0 - shrinkage) + prior_mean * shrinkage;

    // Variance inversely proportional to reliability
    let variance = (1.0 / reliability.max(0.1)) * 0.5;
    let std_dev = variance.sqrt();

    // 5th and 95th percentiles (approximate)
    let lower = (posterior_mean - 1.645 * std_dev).max(0.0);
    let upper = posterior_mean + 1.645 * std_dev;

    (lower, upper)
}

/// Generate human-readable interpretation of ECS result.
fn generate_interpretation(
    ecs: f64,
    lower_credibility: f64,
    is_actionable: bool,
    signal_strength: SignalStrength,
) -> String {
    let strength_desc = match signal_strength {
        SignalStrength::Elite => "Elite",
        SignalStrength::Strong => "Strong",
        SignalStrength::Moderate => "Moderate",
        SignalStrength::Weak => "Weak",
        SignalStrength::Avoid => "Weak",
    };

    let action_desc = match signal_strength {
        SignalStrength::Elite => "Strong buy signal with high confidence",
        SignalStrength::Strong => "Actionable signal worth consideration",
        SignalStrength::Moderate => "Signal detected but proceed with caution",
        SignalStrength::Weak | SignalStrength::Avoid => "Insufficient edge - no action recommended",
    };

    let credibility_desc = if lower_credibility > LOWER_CREDIBILITY_THRESHOLD {
        "Lower bound excludes null (Bayesian-validated)"
    } else {
        "Lower bound does not exclude null - use caution"
    };

    if is_actionable {
        format!("{strength_desc} edge (ECS={ecs:.2}). {action_desc}. {credibility_desc}")
    } else {
        format!("{strength_desc} edge (ECS={ecs:.2}). {action_desc}. {credibility_desc}")
    }
}

// =============================================================================
// PUBLIC API - T3
// =============================================================================

/// Calculate Edge Confidence Score.
///
/// This is the primary Bayesian signal detection method,
/// combining Campion Signal Theory (U x R x T) with EBGM-style
/// credibility intervals.
///
/// Uses the multiplicative formula: ECS = (U x R x T) / 100
///
/// # Arguments
/// - `public_pct`: Public betting percentage normalized to [0, 1]
/// - `line_move_dir`: Direction of line movement (-1, 0, or 1)
/// - `steam_detected`: Whether a steam move was detected
/// - `reliability`: Reliability factor inputs
/// - `hours_to_game`: Hours until game starts
/// - `sport`: Sport type for temporal decay
///
/// # Returns
/// Complete ECS result with score, credibility interval, and components.
///
/// # Example
/// ```
/// use nexcore_vigilance::betting::ecs::{calculate_ecs, ReliabilityInput};
/// use nexcore_vigilance::betting::temporal::SportType;
///
/// let result = calculate_ecs(
///     0.70,  // 70% public on one side
///     -1,    // Line moving against public (RLM)
///     true,  // Steam move detected
///     &ReliabilityInput::default(),
///     12.0,  // 12 hours to game
///     SportType::Nfl,
/// );
///
/// println!("ECS: {:.2}, Actionable: {}", result.ecs, result.is_actionable);
/// ```
#[must_use]
pub fn calculate_ecs(
    public_pct: f64,
    line_move_dir: i8,
    steam_detected: bool,
    reliability: &ReliabilityInput,
    hours_to_game: f64,
    sport: SportType,
) -> EcsResult {
    // Calculate individual components
    let unexpectedness = calculate_unexpectedness(public_pct, line_move_dir, steam_detected);
    let reliability_score = calculate_reliability(reliability);
    let temporal = calculate_temporal_component(hours_to_game, sport);

    // Combine components: ECS = (U x R x T) / 100
    let ecs = (unexpectedness * reliability_score * temporal) / ECS_DENOMINATOR;

    // Calculate Bayesian credibility interval using simplified method
    let (lower_credibility, upper_credibility) =
        calculate_credibility_interval_simple(ecs, reliability.composite());

    // Classify signal
    let signal_strength = SignalStrength::from_ecs(ecs);

    // Determine if actionable
    let is_actionable = ecs >= ECS_ACTIONABLE && lower_credibility > LOWER_CREDIBILITY_THRESHOLD;

    EcsResult {
        ecs,
        components: EcsComponents {
            unexpectedness,
            reliability: reliability_score,
            temporal,
        },
        lower_credibility,
        upper_credibility,
        signal_strength,
        is_actionable,
    }
}

/// Calculate ECS with full gamma posterior credibility intervals.
///
/// Use this variant when you have specific historical context
/// with observation count.
///
/// # Arguments
/// - `public_pct`: Public betting percentage normalized to [0, 1]
/// - `line_move_dir`: Direction of line movement (-1, 0, or 1)
/// - `steam_detected`: Whether a steam move was detected
/// - `reliability`: Reliability factor inputs
/// - `hours_to_game`: Hours until game starts
/// - `sport`: Sport type for temporal decay
/// - `n_observations`: Number of supporting historical observations
/// - `prior_strength`: Strength of prior (higher = more shrinkage toward prior)
///
/// # Returns
/// Extended ECS result with temporal zone and interpretation.
#[must_use]
pub fn calculate_ecs_full(
    public_pct: f64,
    line_move_dir: i8,
    steam_detected: bool,
    reliability: &ReliabilityInput,
    hours_to_game: f64,
    sport: SportType,
    n_observations: u32,
    prior_strength: f64,
) -> EcsResultExtended {
    // Calculate individual components
    let unexpectedness = calculate_unexpectedness(public_pct, line_move_dir, steam_detected);
    let reliability_score = calculate_reliability(reliability);
    let temporal = calculate_temporal_component(hours_to_game, sport);

    // Combine components: ECS = (U x R x T) / 100
    let ecs = (unexpectedness * reliability_score * temporal) / ECS_DENOMINATOR;

    // Calculate full Bayesian credibility interval with gamma posterior
    let (lower_credibility, upper_credibility) =
        calculate_bayesian_credibility(ecs, n_observations, prior_strength);

    // Classify signal
    let signal_strength = SignalStrength::from_ecs(ecs);
    let temporal_zone = TemporalZone::from_hours(HoursToGame::new_clamped(hours_to_game));

    // Determine if actionable
    let is_actionable = ecs >= ECS_ACTIONABLE && lower_credibility > LOWER_CREDIBILITY_THRESHOLD;

    // Generate interpretation
    let interpretation =
        generate_interpretation(ecs, lower_credibility, is_actionable, signal_strength);

    EcsResultExtended {
        result: EcsResult {
            ecs,
            components: EcsComponents {
                unexpectedness,
                reliability: reliability_score,
                temporal,
            },
            lower_credibility,
            upper_credibility,
            signal_strength,
            is_actionable,
        },
        temporal_zone,
        interpretation,
    }
}

/// Simplified ECS calculation for quick analysis.
///
/// Uses default reliability factors and observation count.
/// Normalizes public percentage from 0-100 to 0-1.
///
/// # Arguments
/// - `public_pct`: Public betting percentage (0-100)
/// - `line_movement`: Points of line movement (positive = toward away)
/// - `hours_to_game`: Hours until game starts
/// - `sport`: Sport type for temporal decay
/// - `steam_detected`: Whether steam move was detected
#[must_use]
pub fn calculate_ecs_simple(
    public_pct: f64,
    line_movement: f64,
    hours_to_game: f64,
    sport: SportType,
    steam_detected: bool,
) -> EcsResult {
    // Normalize public percentage to [0, 1]
    let public_normalized = (public_pct / 100.0).clamp(0.0, 1.0);

    // Determine line direction from movement
    let line_direction: i8 = if line_movement.abs() < 0.5 {
        0 // No significant movement
    } else if line_movement > 0.0 {
        1 // Toward away team
    } else {
        -1 // Toward home team
    };

    calculate_ecs(
        public_normalized,
        line_direction,
        steam_detected,
        &ReliabilityInput::default(),
        hours_to_game,
        sport,
    )
}

/// Calculate ECS using additive weighting formula.
///
/// Alternative formula: ECS = 0.40*U + 0.35*R + 0.25*T
///
/// This variant gives more stable results across different
/// signal configurations.
#[must_use]
pub fn calculate_ecs_additive(
    public_pct: f64,
    line_move_dir: i8,
    steam_detected: bool,
    reliability: &ReliabilityInput,
    hours_to_game: f64,
    sport: SportType,
) -> EcsResult {
    // Calculate individual components
    let unexpectedness = calculate_unexpectedness(public_pct, line_move_dir, steam_detected);
    let reliability_score = calculate_reliability(reliability);
    let temporal = calculate_temporal_component(hours_to_game, sport);

    // Combine with additive weights
    let ecs = U_WEIGHT * unexpectedness + R_WEIGHT * reliability_score + T_WEIGHT * temporal;

    // Calculate Bayesian credibility interval
    let (lower_credibility, upper_credibility) =
        calculate_bayesian_credibility(ecs, DEFAULT_N_OBSERVATIONS, DEFAULT_PRIOR_STRENGTH);

    // Classify signal
    let signal_strength = SignalStrength::from_ecs(ecs);

    // Determine if actionable
    let is_actionable = ecs >= ECS_ACTIONABLE && lower_credibility > LOWER_CREDIBILITY_THRESHOLD;

    EcsResult {
        ecs,
        components: EcsComponents {
            unexpectedness,
            reliability: reliability_score,
            temporal,
        },
        lower_credibility,
        upper_credibility,
        signal_strength,
        is_actionable,
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Newtype tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_ecs_score_clamps_negative() {
        let score = EcsScore::new(-5.0);
        assert!((score.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ecs_score_from_f64() {
        let score: EcsScore = 3.5.into();
        assert!((score.value() - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ecs_score_display() {
        let score = EcsScore::new(3.456);
        assert_eq!(format!("{score}"), "3.46");
    }

    #[test]
    fn test_unexpectedness_clamps_range() {
        let low = Unexpectedness::new(-1.0);
        let high = Unexpectedness::new(15.0);

        assert!((low.value() - 0.0).abs() < f64::EPSILON);
        assert!((high.value() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reliability_clamps_range() {
        let low = Reliability::new(-1.0);
        let high = Reliability::new(15.0);

        assert!((low.value() - 0.0).abs() < f64::EPSILON);
        assert!((high.value() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_temporal_clamps_range() {
        let low = TemporalComponent::new(-1.0);
        let high = TemporalComponent::new(15.0);

        assert!((low.value() - 0.0).abs() < f64::EPSILON);
        assert!((high.value() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_credibility_lower_excludes_null() {
        let positive = CredibilityLower::new(0.5);
        let zero = CredibilityLower::new(0.0);
        let negative = CredibilityLower::new(-0.5);

        assert!(positive.excludes_null());
        assert!(!zero.excludes_null());
        // Negative clamped to 0
        assert!(!negative.excludes_null());
    }

    // -------------------------------------------------------------------------
    // Reliability input tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_reliability_composite() {
        let input = ReliabilityInput {
            completeness: 1.0,
            accuracy: 1.0,
            liquidity: 1.0,
            source_quality: 1.0,
        };
        assert!((input.composite() - 1.0).abs() < f64::EPSILON);

        let input_half = ReliabilityInput {
            completeness: 0.5,
            accuracy: 0.5,
            liquidity: 0.5,
            source_quality: 0.5,
        };
        assert!((input_half.composite() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reliability_composite_weights() {
        // Accuracy has highest weight (0.35)
        let high_accuracy = ReliabilityInput {
            completeness: 0.0,
            accuracy: 1.0,
            liquidity: 0.0,
            source_quality: 0.0,
        };
        assert!((high_accuracy.composite() - R_WEIGHT_ACCURACY).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reliability_presets() {
        let high = ReliabilityInput::high();
        let low = ReliabilityInput::low();

        assert!(high.composite() > 0.8);
        assert!(low.composite() < 0.5);
    }

    #[test]
    fn test_reliability_new_clamps() {
        let input = ReliabilityInput::new(1.5, -0.5, 2.0, 0.5);
        assert!((input.completeness - 1.0).abs() < f64::EPSILON);
        assert!((input.accuracy - 0.0).abs() < f64::EPSILON);
        assert!((input.liquidity - 1.0).abs() < f64::EPSILON);
        assert!((input.source_quality - 0.5).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Unexpectedness calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_unexpectedness_reverse_line_movement() {
        // 70% public on home (expected direction = 1), line moves -1 (against)
        let u = calculate_unexpectedness(0.70, -1, false);

        // Should have elevated unexpectedness
        assert!(u > 1.0, "RLM should produce elevated unexpectedness: {u}");
    }

    #[test]
    fn test_unexpectedness_with_steam() {
        let without_steam = calculate_unexpectedness(0.60, -1, false);
        let with_steam = calculate_unexpectedness(0.60, -1, true);

        assert!(
            with_steam > without_steam,
            "Steam should increase unexpectedness"
        );
    }

    #[test]
    fn test_unexpectedness_expected_movement() {
        // 70% public, direction 1 = with expected direction = low surprise
        let expected = calculate_unexpectedness(0.70, 1, false);
        // 70% public, direction -1 = against expected = high surprise
        let reverse = calculate_unexpectedness(0.70, -1, false);

        assert!(
            reverse > expected,
            "Reverse should be more unexpected than expected movement"
        );
    }

    #[test]
    fn test_unexpectedness_no_movement() {
        // No movement when public is split
        let balanced = calculate_unexpectedness(0.50, 0, false);
        // No movement when public is heavily one-sided
        let lopsided = calculate_unexpectedness(0.80, 0, false);

        assert!(
            lopsided > balanced,
            "Lopsided public with no movement should be more unexpected"
        );
    }

    // -------------------------------------------------------------------------
    // Bayesian credibility tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_bayesian_credibility_zero_observations() {
        let (lower, upper) = calculate_bayesian_credibility(5.0, 0, 0.5);
        assert!((lower - 0.0).abs() < f64::EPSILON);
        assert!((upper - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bayesian_credibility_ordering() {
        let (lower, upper) = calculate_bayesian_credibility(5.0, 10, 0.5);
        assert!(lower < upper, "Lower bound should be less than upper");
    }

    #[test]
    fn test_bayesian_credibility_more_observations() {
        let (lower_few, upper_few) = calculate_bayesian_credibility(5.0, 5, 0.5);
        let (lower_many, upper_many) = calculate_bayesian_credibility(5.0, 50, 0.5);

        // More observations should narrow the interval
        let width_few = upper_few - lower_few;
        let width_many = upper_many - lower_many;

        assert!(
            width_many < width_few,
            "More observations should narrow credibility interval"
        );
    }

    // -------------------------------------------------------------------------
    // Normal quantile approximation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_normal_quantile_median() {
        let median = normal_quantile(0.5);
        assert!(median.abs() < 0.001, "Median should be approximately 0");
    }

    #[test]
    fn test_normal_quantile_extremes() {
        let low = normal_quantile(0.05);
        let high = normal_quantile(0.95);

        assert!(low < -1.5, "5th percentile should be around -1.645");
        assert!(high > 1.5, "95th percentile should be around 1.645");
        assert!((low + high).abs() < 0.1, "Should be symmetric");
    }

    #[test]
    fn test_normal_quantile_edge_cases() {
        let zero = normal_quantile(0.0);
        let one = normal_quantile(1.0);

        assert!(zero.is_infinite() && zero.is_sign_negative());
        assert!(one.is_infinite() && one.is_sign_positive());
    }

    // -------------------------------------------------------------------------
    // Gamma quantile approximation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_gamma_quantile_ordering() {
        let p05 = gamma_quantile(5.0, 1.0, 0.05);
        let p50 = gamma_quantile(5.0, 1.0, 0.50);
        let p95 = gamma_quantile(5.0, 1.0, 0.95);

        assert!(p05 < p50, "5th percentile should be less than median");
        assert!(p50 < p95, "Median should be less than 95th percentile");
    }

    #[test]
    fn test_gamma_quantile_scale_effect() {
        let scale1 = gamma_quantile(5.0, 1.0, 0.5);
        let scale2 = gamma_quantile(5.0, 2.0, 0.5);

        // Doubling scale should approximately double quantile
        assert!(
            (scale2 - scale1 * 2.0).abs() < scale1 * 0.2,
            "Doubling scale should approximately double quantile"
        );
    }

    // -------------------------------------------------------------------------
    // Full ECS calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_calculate_ecs_basic() {
        let result = calculate_ecs(
            0.70,
            -1, // RLM
            true,
            &ReliabilityInput::default(),
            12.0,
            SportType::Nfl,
        );

        assert!(result.ecs > 0.0, "ECS should be positive");
        assert!(
            result.lower_credibility <= result.ecs,
            "Lower bound should be at most ECS"
        );
        assert!(result.components.temporal > 0.0);
    }

    #[test]
    fn test_calculate_ecs_steam_move() {
        let without_steam = calculate_ecs(
            0.70,
            -1,
            false,
            &ReliabilityInput::default(),
            6.0,
            SportType::Nba,
        );
        let with_steam = calculate_ecs(
            0.70,
            -1,
            true,
            &ReliabilityInput::default(),
            6.0,
            SportType::Nba,
        );

        assert!(
            with_steam.ecs > without_steam.ecs,
            "Steam move should increase ECS"
        );
    }

    #[test]
    fn test_calculate_ecs_temporal_decay() {
        let close_game = calculate_ecs(
            0.60,
            -1,
            false,
            &ReliabilityInput::default(),
            1.0,
            SportType::Nfl,
        );
        let far_game = calculate_ecs(
            0.60,
            -1,
            false,
            &ReliabilityInput::default(),
            72.0,
            SportType::Nfl,
        );

        // Closer games should have higher temporal component
        assert!(
            close_game.components.temporal > far_game.components.temporal,
            "Closer game should have higher temporal component"
        );
    }

    #[test]
    fn test_calculate_ecs_reliability_impact() {
        let low_reliability = ReliabilityInput::low();
        let high_reliability = ReliabilityInput::high();

        let low_result = calculate_ecs(0.60, -1, false, &low_reliability, 12.0, SportType::Nfl);
        let high_result = calculate_ecs(0.60, -1, false, &high_reliability, 12.0, SportType::Nfl);

        assert!(
            high_result.ecs > low_result.ecs,
            "Higher reliability should increase ECS"
        );
    }

    #[test]
    fn test_calculate_ecs_simple() {
        let result = calculate_ecs_simple(70.0, -1.5, 24.0, SportType::Nfl, false);

        assert!(result.ecs > 0.0);
        assert!(!result.interpretation().is_empty());
    }

    #[test]
    fn test_calculate_ecs_full() {
        let result = calculate_ecs_full(
            0.65,
            -1,
            true,
            &ReliabilityInput::default(),
            8.0,
            SportType::Nba,
            50,
            0.3,
        );

        assert!(!result.interpretation.is_empty());
        assert!(matches!(
            result.temporal_zone,
            TemporalZone::GameDay | TemporalZone::Approaching
        ));
    }

    #[test]
    fn test_calculate_ecs_additive() {
        let multiplicative = calculate_ecs(
            0.70,
            -1,
            true,
            &ReliabilityInput::default(),
            12.0,
            SportType::Nfl,
        );

        let additive = calculate_ecs_additive(
            0.70,
            -1,
            true,
            &ReliabilityInput::default(),
            12.0,
            SportType::Nfl,
        );

        // Both should produce positive scores
        assert!(multiplicative.ecs > 0.0);
        assert!(additive.ecs > 0.0);

        // Additive formula typically produces more moderate values
        // (no extreme multiplication effects)
    }

    // -------------------------------------------------------------------------
    // Signal classification tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_signal_strength_classification() {
        // Strong signal conditions: steam + RLM + high reliability + close to game
        let result = calculate_ecs(
            0.80,
            -1,
            true,
            &ReliabilityInput::high(),
            1.0,
            SportType::Nfl,
        );

        // This should produce a strong or elite signal
        assert!(
            result.ecs > 2.0,
            "Elite conditions should produce high ECS: {}",
            result.ecs
        );
    }

    #[test]
    fn test_actionability() {
        // Marginal case: weak signal
        let weak = calculate_ecs(
            0.55, // slight public lean
            0,    // no movement
            false,
            &ReliabilityInput::low(),
            48.0,
            SportType::Nfl,
        );

        // Strong case
        let strong = calculate_ecs(
            0.80,
            -1, // RLM
            true,
            &ReliabilityInput::high(),
            2.0,
            SportType::Nfl,
        );

        // Weak signals should generally not be actionable
        // Strong signals should be actionable
        assert!(
            strong.ecs > weak.ecs,
            "Strong signal should have higher ECS"
        );
    }

    // -------------------------------------------------------------------------
    // Interpretation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_interpretation_contains_ecs() {
        let result = calculate_ecs(
            0.70,
            -1,
            false,
            &ReliabilityInput::default(),
            12.0,
            SportType::Nfl,
        );

        assert!(
            result.interpretation().contains("ECS="),
            "Interpretation should contain ECS value"
        );
    }

    #[test]
    fn test_interpretation_contains_action() {
        let result = calculate_ecs_simple(80.0, -2.0, 1.0, SportType::Nba, true);

        // Should contain some action recommendation
        assert!(
            result.interpretation().contains("signal") || result.interpretation().contains("edge"),
            "Interpretation should contain signal or edge"
        );
    }

    #[test]
    fn test_ecs_result_methods() {
        let result = calculate_ecs(
            0.70,
            -1,
            true,
            &ReliabilityInput::default(),
            12.0,
            SportType::Nfl,
        );

        // Test typed accessors
        let score = result.score();
        assert!(score.value() > 0.0);

        let lower = result.lower();
        assert!(lower.value() >= 0.0);

        // Components
        let u = result.components.u();
        let r = result.components.r();
        let t = result.components.t();

        assert!(u.value() >= 0.0 && u.value() <= 10.0);
        assert!(r.value() >= 0.0 && r.value() <= 10.0);
        assert!(t.value() >= 0.0 && t.value() <= 10.0);
    }

    // -------------------------------------------------------------------------
    // Edge case tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_extreme_public_percentage() {
        // 100% public on one side (edge case)
        let result = calculate_ecs_simple(100.0, -2.0, 12.0, SportType::Nfl, false);
        assert!(
            result.ecs.is_finite(),
            "ECS should be finite for 100% public"
        );

        // 0% on one side
        let result2 = calculate_ecs_simple(0.0, 2.0, 12.0, SportType::Nfl, false);
        assert!(
            result2.ecs.is_finite(),
            "ECS should be finite for 0% public"
        );
    }

    #[test]
    fn test_zero_hours_to_game() {
        let result = calculate_ecs_simple(60.0, 1.0, 0.0, SportType::Nfl, false);
        assert!(result.ecs.is_finite());
        assert!((result.components.temporal - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_very_large_hours_to_game() {
        let result = calculate_ecs_simple(60.0, 1.0, 1000.0, SportType::Nfl, false);
        assert!(result.ecs.is_finite());
        assert!(
            result.components.temporal < 0.1,
            "Very far games should have near-zero temporal component"
        );
    }

    // -------------------------------------------------------------------------
    // Sport-specific tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sport_specific_decay() {
        let nfl = calculate_ecs_simple(60.0, -1.0, 24.0, SportType::Nfl, false);
        let nba = calculate_ecs_simple(60.0, -1.0, 24.0, SportType::Nba, false);

        // NBA has faster decay (higher lambda)
        assert!(
            nfl.components.temporal > nba.components.temporal,
            "NFL should have slower decay than NBA"
        );
    }

    #[test]
    fn test_all_sport_types() {
        for sport in [
            SportType::Nfl,
            SportType::Nba,
            SportType::Mlb,
            SportType::Nhl,
            SportType::Soccer,
        ] {
            let result = calculate_ecs_simple(60.0, -1.0, 24.0, sport, false);
            assert!(result.ecs.is_finite(), "ECS should be finite for {sport:?}");
        }
    }

    // -------------------------------------------------------------------------
    // Campion Signal Theory validation
    // -------------------------------------------------------------------------

    #[test]
    fn test_campion_signal_theory_components() {
        // U: High unexpectedness (RLM with lopsided public)
        // R: High reliability
        // T: Close to game
        let result = calculate_ecs(
            0.85, // 85% public
            -1,   // Line moving against
            true, // Steam
            &ReliabilityInput::high(),
            2.0, // Close to game
            SportType::Nfl,
        );

        // All components should be elevated
        assert!(
            result.components.unexpectedness > 2.0,
            "U should be elevated"
        );
        assert!(result.components.reliability > 8.0, "R should be high");
        assert!(
            result.components.temporal > 8.0,
            "T should be high (close game)"
        );

        // Combined ECS should be high
        assert!(
            result.ecs > 1.5,
            "Combined ECS should indicate strong signal"
        );
    }

    #[test]
    fn test_low_signal_all_components() {
        // U: Low unexpectedness (expected movement)
        // R: Low reliability
        // T: Far from game
        let result = calculate_ecs(
            0.50, // Split public
            0,    // No movement
            false,
            &ReliabilityInput::low(),
            72.0, // Far from game
            SportType::Nfl,
        );

        // All components should be low
        assert!(result.components.unexpectedness < 2.0, "U should be low");
        assert!(result.components.reliability < 5.0, "R should be low");
        assert!(
            result.components.temporal < 1.0,
            "T should be low (far game)"
        );
    }
}
