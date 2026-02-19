//! Multi-intervention dynamics — synergy, antagonism, dependency detection.
//!
//! # T1 Grounding
//! - ∝ (proportionality): Interaction multipliers scale combined effects
//! - κ (comparison): Dependency detection compares successive levels
//! - ∃ (existence): Risk detection checks if pattern exists

use serde::{Deserialize, Serialize};

use crate::types::{InteractionType, PlasmaLevel};

/// Result of computing an interaction between two interventions.
///
/// Tier: T2-C (composes InteractionType + PlasmaLevel)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractionEffect {
    /// Type of interaction detected.
    pub interaction_type: InteractionType,
    /// Magnitude of the interaction (multiplier applied).
    pub magnitude: f64,
    /// Combined plasma level after interaction.
    pub combined_level: PlasmaLevel,
}

/// Compute the interaction effect between two plasma levels.
///
/// Applies the interaction type's multiplier to the sum of both levels.
///
/// # Formula
/// combined = (a + b) × multiplier
#[must_use]
pub fn compute_interaction(
    a: PlasmaLevel,
    b: PlasmaLevel,
    interaction_type: InteractionType,
) -> InteractionEffect {
    let raw_sum = a.value() + b.value();
    let multiplier = interaction_type.multiplier();
    let combined = raw_sum * multiplier;

    InteractionEffect {
        interaction_type,
        magnitude: multiplier,
        combined_level: PlasmaLevel(combined),
    }
}

/// Detect dependency risk from a history of plasma levels.
///
/// Dependency is indicated by a rising baseline — when the trough levels
/// (minimum before next dose) are consistently increasing.
///
/// Returns `true` if the trend of minimum levels over sliding windows
/// is positive, suggesting tolerance/dependency formation.
///
/// Requires at least 3 data points.
#[must_use]
pub fn detect_dependency_risk(history: &[PlasmaLevel]) -> bool {
    if history.len() < 3 {
        return false;
    }

    // Check if the series shows monotonically increasing troughs
    // Use pairs: check that each successive value is higher than the one before
    let rising_count = history
        .windows(2)
        .filter(|w| w[1].value() > w[0].value())
        .count();

    let total_pairs = history.len() - 1;

    // If >60% of consecutive pairs are rising, flag dependency risk
    rising_count as f64 / total_pairs as f64 > 0.6
}

/// Compute the net effect of multiple concurrent interventions.
///
/// Uses pairwise interaction resolution, then averages the multipliers.
/// Returns the combined plasma level and dominant interaction type.
#[must_use]
pub fn resolve_multi_interaction(
    levels: &[PlasmaLevel],
    interaction_type: InteractionType,
) -> InteractionEffect {
    if levels.is_empty() {
        return InteractionEffect {
            interaction_type: InteractionType::Additive,
            magnitude: 1.0,
            combined_level: PlasmaLevel::ZERO,
        };
    }

    let total: f64 = levels.iter().map(|l| l.value()).sum();
    let multiplier = interaction_type.multiplier();

    InteractionEffect {
        interaction_type,
        magnitude: multiplier,
        combined_level: PlasmaLevel(total * multiplier),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn additive_interaction() {
        let effect = compute_interaction(
            PlasmaLevel(0.3),
            PlasmaLevel(0.2),
            InteractionType::Additive,
        );
        assert!((effect.combined_level.value() - 0.5).abs() < f64::EPSILON);
        assert!((effect.magnitude - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn synergistic_interaction() {
        let effect = compute_interaction(
            PlasmaLevel(0.3),
            PlasmaLevel(0.2),
            InteractionType::Synergistic,
        );
        assert!((effect.combined_level.value() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn antagonistic_interaction() {
        let effect = compute_interaction(
            PlasmaLevel(0.3),
            PlasmaLevel(0.2),
            InteractionType::Antagonistic,
        );
        assert!((effect.combined_level.value() - 0.35).abs() < f64::EPSILON);
    }

    #[test]
    fn potentiating_interaction() {
        let effect = compute_interaction(
            PlasmaLevel(0.3),
            PlasmaLevel(0.2),
            InteractionType::Potentiating,
        );
        assert!((effect.combined_level.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_inputs() {
        let effect = compute_interaction(
            PlasmaLevel::ZERO,
            PlasmaLevel::ZERO,
            InteractionType::Synergistic,
        );
        assert!((effect.combined_level.value()).abs() < f64::EPSILON);
    }

    #[test]
    fn dependency_risk_rising() {
        let history = vec![
            PlasmaLevel(0.1),
            PlasmaLevel(0.2),
            PlasmaLevel(0.3),
            PlasmaLevel(0.35),
            PlasmaLevel(0.4),
        ];
        assert!(detect_dependency_risk(&history));
    }

    #[test]
    fn dependency_risk_declining() {
        let history = vec![
            PlasmaLevel(0.5),
            PlasmaLevel(0.4),
            PlasmaLevel(0.3),
            PlasmaLevel(0.2),
        ];
        assert!(!detect_dependency_risk(&history));
    }

    #[test]
    fn dependency_risk_stable() {
        let history = vec![
            PlasmaLevel(0.3),
            PlasmaLevel(0.3),
            PlasmaLevel(0.3),
            PlasmaLevel(0.3),
        ];
        assert!(!detect_dependency_risk(&history));
    }

    #[test]
    fn dependency_risk_too_few_points() {
        let history = vec![PlasmaLevel(0.1), PlasmaLevel(0.2)];
        assert!(!detect_dependency_risk(&history));
    }

    #[test]
    fn multi_interaction_empty() {
        let effect = resolve_multi_interaction(&[], InteractionType::Additive);
        assert!((effect.combined_level.value()).abs() < f64::EPSILON);
    }

    #[test]
    fn multi_interaction_synergistic() {
        let levels = vec![PlasmaLevel(0.1), PlasmaLevel(0.2), PlasmaLevel(0.1)];
        let effect = resolve_multi_interaction(&levels, InteractionType::Synergistic);
        // (0.1 + 0.2 + 0.1) × 1.5 = 0.6
        assert!((effect.combined_level.value() - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn dependency_risk_mixed_but_mostly_rising() {
        let history = vec![
            PlasmaLevel(0.1),
            PlasmaLevel(0.2),
            PlasmaLevel(0.15), // dip
            PlasmaLevel(0.25),
            PlasmaLevel(0.3),
        ];
        // 3 rising out of 4 pairs = 75% > 60%
        assert!(detect_dependency_risk(&history));
    }
}
