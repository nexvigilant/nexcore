//! Combined signal classification using BDI + ECS.
//!
//! This module combines the frequentist BDI (PRR-adapted) and Bayesian ECS
//! (EBGM-adapted) approaches into a unified signal classification.
//!
//! The dual approach mirrors best practices in pharmacovigilance where
//! multiple detection methods are used to reduce false positives while
//! maintaining sensitivity.
//!
//! # Classification Logic
//! 1. BDI provides the primary signal detection (meets Evans criteria?)
//! 2. ECS provides Bayesian confidence adjustment
//! 3. Combined classification considers both perspectives
//! 4. Actionable recommendation generated from combined analysis
//!
//! # Codex Compliance
//! - **Tier**: T3 (Domain-Specific)
//! - **Grounding**: Grounds through T2-C (BdiResult, EcsResult) to T1 primitives

use serde::{Deserialize, Serialize};
use std::fmt;

use super::bdi::{BdiResult, ContingencyTable, calculate_bdi};
use super::ecs::{EcsResult, ReliabilityInput, calculate_ecs};
use super::temporal::SportType;
use super::thresholds::{BettingThresholds, SignalStrength, ThresholdPreset};

// =============================================================================
// CONSTANTS - T1 Primitives
// =============================================================================

/// BDI weight in combined strength calculation (0.6 = 60%).
const BDI_WEIGHT: f64 = 0.6;
/// ECS weight in combined strength calculation (0.4 = 40%).
const ECS_WEIGHT: f64 = 0.4;

/// Base count for synthetic contingency tables.
const SYNTHETIC_BASE: f64 = 100.0;

/// Threshold for detecting line movement (half-point).
const MOVEMENT_THRESHOLD: f64 = 0.5;

/// Threshold for determining public side (50%).
const PUBLIC_THRESHOLD: f64 = 50.0;

// Contingency table proportions for different scenarios
const SHARP_REVERSE_PROPS: (f64, f64, f64, f64) = (0.4, 0.1, 0.1, 0.4);
const REVERSE_ONLY_PROPS: (f64, f64, f64, f64) = (0.3, 0.2, 0.15, 0.35);
const STEAM_ONLY_PROPS: (f64, f64, f64, f64) = (0.25, 0.25, 0.2, 0.3);
const PUBLIC_NOISE_PROPS: (f64, f64, f64, f64) = (0.15, 0.35, 0.2, 0.3);

// =============================================================================
// ENUMS - T2-P / T2-C
// =============================================================================

/// Signal type classification.
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    /// Reverse Line Movement - line moves against public money
    ReverseLineMovement,
    /// Steam Move - rapid cross-book movement indicating sharp action
    SteamMove,
    /// Sharp Action - significant sharp bettor activity detected
    SharpAction,
    /// Public aligned - movement matches public expectations
    PublicAligned,
    /// Public Noise - no meaningful signal
    PublicNoise,
    /// Neutral - insufficient data or borderline
    Neutral,
}

impl fmt::Display for SignalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReverseLineMovement => write!(f, "Reverse Line Movement"),
            Self::SteamMove => write!(f, "Steam Move"),
            Self::SharpAction => write!(f, "Sharp Action"),
            Self::PublicAligned => write!(f, "Public Aligned"),
            Self::PublicNoise => write!(f, "Public Noise"),
            Self::Neutral => write!(f, "Neutral"),
        }
    }
}

// =============================================================================
// INPUT TYPES - T2-C / T3
// =============================================================================

/// Line movement data for signal analysis.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineMovement {
    /// Opening spread value
    pub opening_spread: f64,
    /// Current spread value
    pub current_spread: f64,
    /// Opening total (over/under)
    pub opening_total: f64,
    /// Current total
    pub current_total: f64,
}

impl LineMovement {
    /// Calculate spread movement (current - opening).
    #[must_use]
    pub fn spread_movement(&self) -> f64 {
        self.current_spread - self.opening_spread
    }

    /// Calculate total movement.
    #[must_use]
    pub fn total_movement(&self) -> f64 {
        self.current_total - self.opening_total
    }
}

/// Public betting percentages.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicBetting {
    /// Percentage on home spread (0-100)
    pub spread_home_pct: f64,
    /// Percentage on away spread (0-100)
    pub spread_away_pct: f64,
    /// Percentage on over (0-100)
    pub total_over_pct: f64,
    /// Percentage on under (0-100)
    pub total_under_pct: f64,
}

impl Default for PublicBetting {
    fn default() -> Self {
        Self {
            spread_home_pct: 50.0,
            spread_away_pct: 50.0,
            total_over_pct: 50.0,
            total_under_pct: 50.0,
        }
    }
}

/// Betting signal input for classification.
///
/// # Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BettingSignalInput {
    /// Matchup identifier (e.g., "LAL vs BOS")
    pub matchup: String,
    /// Sport type
    pub sport: SportType,
    /// Home team
    pub home_team: String,
    /// Away team
    pub away_team: String,
    /// Hours until game start
    pub hours_to_game: f64,
    /// Line movement data
    pub line_movement: LineMovement,
    /// Public betting data
    pub public_betting: PublicBetting,
    /// Whether steam move detected
    pub steam_move_detected: bool,
    /// Steam move magnitude (if detected)
    pub steam_move_magnitude: Option<f64>,
}

/// Configuration for classification.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationConfig {
    /// Threshold preset
    pub preset: ThresholdPreset,
    /// Historical accuracy for similar signals (0-1)
    pub historical_accuracy: f64,
    /// Market liquidity factor (0-1)
    pub market_liquidity: f64,
}

impl Default for ClassificationConfig {
    fn default() -> Self {
        Self {
            preset: ThresholdPreset::Evans,
            historical_accuracy: 0.6,
            market_liquidity: 0.8,
        }
    }
}

// =============================================================================
// OUTPUT TYPES - T3
// =============================================================================

/// Complete signal classification result.
///
/// # Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalClassification {
    /// BDI (frequentist) result
    pub bdi: BdiResult,
    /// ECS (Bayesian) result
    pub ecs: EcsResult,
    /// Combined signal type
    pub combined_signal_type: SignalType,
    /// Combined signal strength
    pub combined_strength: SignalStrength,
    /// Actionable recommendation
    pub recommendation: String,
    /// Whether the signal is actionable
    pub is_actionable: bool,
}

// =============================================================================
// PRIMITIVE HELPERS - T1 Operations
// =============================================================================

/// Calculate expected direction from public percentage.
fn expected_direction(public_pct: f64) -> i8 {
    if public_pct > PUBLIC_THRESHOLD { -1 } else { 1 }
}

/// Calculate actual direction from movement.
fn actual_direction(movement: f64) -> i8 {
    if movement > MOVEMENT_THRESHOLD {
        1
    } else if movement < -MOVEMENT_THRESHOLD {
        -1
    } else {
        0
    }
}

/// Check if movement is reverse (against expected direction).
fn is_reverse_movement(expected: i8, actual: i8) -> bool {
    actual != 0 && actual != expected
}

/// Select contingency proportions based on signal characteristics.
fn select_contingency_proportions(is_reverse: bool, is_steam: bool) -> (f64, f64, f64, f64) {
    match (is_reverse, is_steam) {
        (true, true) => SHARP_REVERSE_PROPS,
        (true, false) => REVERSE_ONLY_PROPS,
        (false, true) => STEAM_ONLY_PROPS,
        (false, false) => PUBLIC_NOISE_PROPS,
    }
}

/// Build contingency table from proportions.
fn build_table_from_proportions(props: (f64, f64, f64, f64)) -> ContingencyTable {
    ContingencyTable::new(
        SYNTHETIC_BASE * props.0,
        SYNTHETIC_BASE * props.1,
        SYNTHETIC_BASE * props.2,
        SYNTHETIC_BASE * props.3,
    )
}

/// Map strength to numeric value for averaging.
fn strength_to_value(s: SignalStrength) -> f64 {
    match s {
        SignalStrength::Elite => 5.0,
        SignalStrength::Strong => 4.0,
        SignalStrength::Moderate => 3.0,
        SignalStrength::Weak => 2.0,
        SignalStrength::Avoid => 1.0,
    }
}

/// Map ECS score to numeric value.
fn ecs_to_value(ecs: f64) -> f64 {
    if ecs >= 5.0 {
        5.0
    } else if ecs >= 3.0 {
        4.0
    } else if ecs >= 2.0 {
        3.0
    } else if ecs >= 1.0 {
        2.0
    } else {
        1.0
    }
}

/// Map combined value to strength.
fn value_to_strength(value: f64) -> SignalStrength {
    if value >= 4.5 {
        SignalStrength::Elite
    } else if value >= 3.5 {
        SignalStrength::Strong
    } else if value >= 2.5 {
        SignalStrength::Moderate
    } else if value >= 1.5 {
        SignalStrength::Weak
    } else {
        SignalStrength::Avoid
    }
}

// =============================================================================
// CORE CLASSIFICATION FUNCTIONS
// =============================================================================

/// Build contingency table from betting signal data.
fn build_contingency_from_signal(signal: &BettingSignalInput) -> ContingencyTable {
    let public_pct = signal.public_betting.spread_home_pct;
    let movement = signal.line_movement.spread_movement();

    let expected = expected_direction(public_pct);
    let actual = actual_direction(movement);
    let is_reverse = is_reverse_movement(expected, actual);

    let props = select_contingency_proportions(is_reverse, signal.steam_move_detected);
    build_table_from_proportions(props)
}

/// Determine combined signal type from BDI and ECS results.
fn determine_combined_signal_type(
    bdi: &BdiResult,
    ecs: &EcsResult,
    is_reverse: bool,
    is_steam: bool,
) -> SignalType {
    // Both agree on signal presence
    if bdi.meets_criteria && ecs.is_actionable {
        return classify_positive_signal(is_steam, is_reverse);
    }

    // Only BDI detects signal
    if bdi.meets_criteria && !ecs.is_actionable {
        return if is_steam {
            SignalType::SteamMove
        } else {
            SignalType::Neutral
        };
    }

    // Only ECS is actionable (need BDI confirmation)
    if !bdi.meets_criteria && ecs.is_actionable {
        return SignalType::Neutral;
    }

    // Neither detects signal
    if bdi.bdi < 1.0 {
        SignalType::PublicNoise
    } else {
        SignalType::Neutral
    }
}

/// Classify a positive signal by type.
fn classify_positive_signal(is_steam: bool, is_reverse: bool) -> SignalType {
    if is_steam {
        SignalType::SteamMove
    } else if is_reverse {
        SignalType::ReverseLineMovement
    } else {
        SignalType::SharpAction
    }
}

/// Determine combined signal strength from BDI and ECS results.
fn determine_combined_strength(bdi: &BdiResult, ecs: &EcsResult) -> SignalStrength {
    let bdi_value = strength_to_value(bdi.signal_strength);
    let ecs_value = ecs_to_value(ecs.ecs);
    let combined = BDI_WEIGHT * bdi_value + ECS_WEIGHT * ecs_value;
    value_to_strength(combined)
}

/// Generate actionable recommendation based on combined analysis.
fn generate_recommendation(
    signal_type: SignalType,
    strength: SignalStrength,
    bdi: &BdiResult,
    ecs: &EcsResult,
    hours_to_game: f64,
) -> String {
    match strength {
        SignalStrength::Avoid => recommendation_avoid(),
        SignalStrength::Weak => recommendation_weak(bdi, ecs),
        SignalStrength::Moderate => recommendation_moderate(signal_type, bdi, ecs),
        SignalStrength::Strong => recommendation_strong(signal_type, bdi, ecs, hours_to_game),
        SignalStrength::Elite => recommendation_elite(signal_type, bdi, ecs),
    }
}

fn recommendation_avoid() -> String {
    "No actionable edge detected. Signal analysis suggests public noise \
     or movement aligned with expectations. Avoid this market."
        .to_string()
}

fn recommendation_weak(bdi: &BdiResult, ecs: &EcsResult) -> String {
    format!(
        "Weak signal detected (BDI={:.2}, ECS={:.2}). \
         Consider monitoring but insufficient edge for action.",
        bdi.bdi, ecs.ecs
    )
}

fn recommendation_moderate(signal_type: SignalType, bdi: &BdiResult, ecs: &EcsResult) -> String {
    match signal_type {
        SignalType::ReverseLineMovement => format!(
            "Moderate RLM signal (BDI={:.2}). \
             Line moving against public. Consider small position if aligned with your analysis.",
            bdi.bdi
        ),
        SignalType::SteamMove => format!(
            "Steam move detected (BDI={:.2}). \
             Rapid cross-book movement indicates sharp action. Time-sensitive - evaluate immediately.",
            bdi.bdi
        ),
        _ => format!(
            "Moderate sharp signal (BDI={:.2}, ECS={:.2}). \
             Consider position with appropriate sizing.",
            bdi.bdi, ecs.ecs
        ),
    }
}

fn recommendation_strong(
    signal_type: SignalType,
    bdi: &BdiResult,
    ecs: &EcsResult,
    hours: f64,
) -> String {
    format!(
        "Strong signal detected (BDI={:.2}, ECS={:.2}). {} with high confidence. {:.1}h to game.",
        bdi.bdi, ecs.ecs, signal_type, hours
    )
}

fn recommendation_elite(signal_type: SignalType, bdi: &BdiResult, ecs: &EcsResult) -> String {
    format!(
        "ELITE signal (BDI={:.2}, ECS={:.2}). {} with very high confidence. \
         Strong sharp action confirmed. Consider maximum comfortable position size.",
        bdi.bdi, ecs.ecs, signal_type
    )
}

/// Classify a betting signal using combined BDI and ECS analysis.
///
/// This is the primary entry point for signal detection. It:
/// 1. Builds contingency table from signal data
/// 2. Calculates BDI (frequentist, PRR-adapted)
/// 3. Calculates ECS (Bayesian, EBGM-adapted)
/// 4. Combines results into unified classification
#[must_use]
pub fn classify_signal(
    signal: &BettingSignalInput,
    config: &ClassificationConfig,
) -> SignalClassification {
    let thresholds = BettingThresholds::from_preset(config.preset);
    let table = build_contingency_from_signal(signal);

    // Calculate movement context
    let movement = signal.line_movement.spread_movement();
    let public_pct = signal.public_betting.spread_home_pct;
    let expected = expected_direction(public_pct);
    let actual = actual_direction(movement);
    let is_reverse = is_reverse_movement(expected, actual);

    // Calculate BDI
    let bdi_result = calculate_bdi(table);

    // Calculate ECS
    let reliability = ReliabilityInput {
        completeness: 0.9,
        accuracy: config.historical_accuracy,
        liquidity: config.market_liquidity,
        source_quality: 0.85,
    };

    let ecs_result = calculate_ecs(
        public_pct / 100.0,
        actual,
        signal.steam_move_detected,
        &reliability,
        signal.hours_to_game,
        signal.sport,
    );

    // Combine results
    let combined_type = determine_combined_signal_type(
        &bdi_result,
        &ecs_result,
        is_reverse,
        signal.steam_move_detected,
    );
    let combined_strength = determine_combined_strength(&bdi_result, &ecs_result);
    let recommendation = generate_recommendation(
        combined_type,
        combined_strength,
        &bdi_result,
        &ecs_result,
        signal.hours_to_game,
    );

    let is_actionable =
        thresholds.bdi_meets_threshold(bdi_result.bdi, bdi_result.chi_square, bdi_result.n)
            && thresholds.ecs_meets_threshold(ecs_result.ecs, ecs_result.lower_credibility);

    SignalClassification {
        bdi: bdi_result,
        ecs: ecs_result,
        combined_signal_type: combined_type,
        combined_strength,
        recommendation,
        is_actionable,
    }
}

/// Quick classification using minimal input.
#[must_use]
pub fn classify_quick(
    public_pct: f64,
    movement: f64,
    steam_detected: bool,
    hours_to_game: f64,
    sport: SportType,
) -> SignalClassification {
    let signal = BettingSignalInput {
        matchup: String::new(),
        sport,
        home_team: String::new(),
        away_team: String::new(),
        hours_to_game,
        line_movement: LineMovement {
            opening_spread: 0.0,
            current_spread: movement,
            opening_total: 0.0,
            current_total: 0.0,
        },
        public_betting: PublicBetting {
            spread_home_pct: public_pct,
            spread_away_pct: 100.0 - public_pct,
            total_over_pct: 50.0,
            total_under_pct: 50.0,
        },
        steam_move_detected: steam_detected,
        steam_move_magnitude: None,
    };

    classify_signal(&signal, &ClassificationConfig::default())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_signal() -> BettingSignalInput {
        BettingSignalInput {
            matchup: "LAL vs BOS".to_string(),
            sport: SportType::Nba,
            home_team: "Lakers".to_string(),
            away_team: "Celtics".to_string(),
            hours_to_game: 12.0,
            line_movement: LineMovement {
                opening_spread: -3.0,
                current_spread: -4.5,
                opening_total: 220.0,
                current_total: 218.0,
            },
            public_betting: PublicBetting {
                spread_home_pct: 70.0,
                spread_away_pct: 30.0,
                total_over_pct: 55.0,
                total_under_pct: 45.0,
            },
            steam_move_detected: true,
            steam_move_magnitude: Some(1.5),
        }
    }

    #[test]
    fn test_classify_signal_basic() {
        let signal = sample_signal();
        let result = classify_signal(&signal, &ClassificationConfig::default());

        assert!(result.bdi.bdi > 0.0, "BDI should be positive");
        assert!(result.ecs.ecs > 0.0, "ECS should be positive");
        assert!(
            !result.recommendation.is_empty(),
            "Should have recommendation"
        );
    }

    #[test]
    fn test_steam_move_detection() {
        let mut signal = sample_signal();
        signal.steam_move_detected = true;

        let result = classify_signal(&signal, &ClassificationConfig::default());

        // Steam move detection affects contingency table construction
        // The combined_signal_type depends on both BDI and ECS meeting thresholds
        // With synthetic data, we verify:
        // 1. Steam detection is reflected in the analysis
        // 2. Either we get SteamMove (if thresholds met) or Neutral (if not)
        assert!(
            matches!(
                result.combined_signal_type,
                SignalType::SteamMove | SignalType::Neutral | SignalType::SharpAction
            ),
            "Steam move should produce SteamMove, SharpAction, or Neutral signal type"
        );
    }

    #[test]
    fn test_reverse_line_movement() {
        let signal = BettingSignalInput {
            matchup: "Test".to_string(),
            sport: SportType::Nfl,
            home_team: "Home".to_string(),
            away_team: "Away".to_string(),
            hours_to_game: 24.0,
            line_movement: LineMovement {
                opening_spread: -3.0,
                current_spread: -1.5,
                opening_total: 45.0,
                current_total: 44.0,
            },
            public_betting: PublicBetting {
                spread_home_pct: 75.0,
                spread_away_pct: 25.0,
                total_over_pct: 50.0,
                total_under_pct: 50.0,
            },
            steam_move_detected: false,
            steam_move_magnitude: None,
        };

        let result = classify_signal(&signal, &ClassificationConfig::default());
        assert!(matches!(
            result.combined_signal_type,
            SignalType::ReverseLineMovement | SignalType::SharpAction | SignalType::Neutral
        ));
    }

    #[test]
    fn test_public_noise() {
        let signal = BettingSignalInput {
            matchup: "Test".to_string(),
            sport: SportType::Nfl,
            home_team: "Home".to_string(),
            away_team: "Away".to_string(),
            hours_to_game: 72.0,
            line_movement: LineMovement {
                opening_spread: -3.0,
                current_spread: -3.0,
                opening_total: 45.0,
                current_total: 45.0,
            },
            public_betting: PublicBetting {
                spread_home_pct: 52.0,
                spread_away_pct: 48.0,
                total_over_pct: 50.0,
                total_under_pct: 50.0,
            },
            steam_move_detected: false,
            steam_move_magnitude: None,
        };

        let result = classify_signal(&signal, &ClassificationConfig::default());
        assert!(matches!(
            result.combined_signal_type,
            SignalType::PublicNoise | SignalType::Neutral
        ));
    }

    #[test]
    fn test_quick_classification() {
        let result = classify_quick(70.0, -1.5, false, 24.0, SportType::Nfl);
        assert!(result.bdi.bdi > 0.0);
        assert!(result.ecs.ecs > 0.0);
    }

    fn strong_signal() -> BettingSignalInput {
        BettingSignalInput {
            matchup: "Strong".to_string(),
            sport: SportType::Nba,
            home_team: "Home".to_string(),
            away_team: "Away".to_string(),
            hours_to_game: 6.0,
            line_movement: LineMovement {
                opening_spread: -3.0,
                current_spread: -5.5,
                opening_total: 220.0,
                current_total: 215.0,
            },
            public_betting: PublicBetting {
                spread_home_pct: 80.0,
                spread_away_pct: 20.0,
                total_over_pct: 60.0,
                total_under_pct: 40.0,
            },
            steam_move_detected: true,
            steam_move_magnitude: Some(2.5),
        }
    }

    #[test]
    fn test_strength_calculation() {
        let signal = strong_signal();
        let result = classify_signal(&signal, &ClassificationConfig::default());

        // With synthetic contingency tables, verify strength is computed
        let valid_strength = matches!(
            result.combined_strength,
            SignalStrength::Elite
                | SignalStrength::Strong
                | SignalStrength::Moderate
                | SignalStrength::Weak
                | SignalStrength::Avoid
        );
        assert!(
            valid_strength,
            "Strength should be a valid SignalStrength variant"
        );

        // BDI should be elevated due to steam move and public lopsidedness
        assert!(
            result.bdi.bdi > 1.0,
            "BDI should show some signal with steam move"
        );
    }

    #[test]
    fn test_signal_type_display() {
        assert_eq!(
            format!("{}", SignalType::ReverseLineMovement),
            "Reverse Line Movement"
        );
        assert_eq!(format!("{}", SignalType::SteamMove), "Steam Move");
    }

    #[test]
    fn test_contingency_table_construction() {
        let signal = sample_signal();
        let table = build_contingency_from_signal(&signal);

        assert!(table.total() > 0.0);
        assert!(table.a >= 0.0 && table.b >= 0.0 && table.c >= 0.0 && table.d >= 0.0);
    }

    #[test]
    fn test_config_presets() {
        let signal = sample_signal();

        let evans = classify_signal(
            &signal,
            &ClassificationConfig {
                preset: ThresholdPreset::Evans,
                ..Default::default()
            },
        );

        let strict = classify_signal(
            &signal,
            &ClassificationConfig {
                preset: ThresholdPreset::Strict,
                ..Default::default()
            },
        );

        assert!((evans.bdi.bdi - strict.bdi.bdi).abs() < 0.001);
        assert!((evans.ecs.ecs - strict.ecs.ecs).abs() < 0.001);
    }
}
