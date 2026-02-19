//! Composite quality scoring for care episodes.
//!
//! # T1 Grounding
//! - κ (comparison): Weighted scoring compares factors
//! - ∝ (proportionality): Normalization functions map to [0, 1]
//! - ∂ (boundary): Quality rating thresholds define categories
//!
//! # Scoring Model
//! | Factor | Weight | Normalization |
//! |--------|--------|---------------|
//! | Bioavailability | 0.30 | sigmoid(bioavail, midpoint=0.5) |
//! | Stability | 0.25 | 1 - CV(levels) |
//! | Safety margin | 0.25 | therapeutic_index |
//! | Persistence | 0.20 | bell_curve(half_life, optimal=24h) |

use serde::{Deserialize, Serialize};

use crate::episode::Episode;
use crate::error::CcpError;
use crate::kinetics;
use crate::types::{PlasmaLevel, TherapeuticWindow};

/// Weights for each quality factor.
const W_BIOAVAILABILITY: f64 = 0.30;
const W_STABILITY: f64 = 0.25;
const W_SAFETY: f64 = 0.25;
const W_PERSISTENCE: f64 = 0.20;

/// Optimal half-life for bell curve scoring (24 hours).
const OPTIMAL_HALF_LIFE: f64 = 24.0;

/// Individual quality factor scores.
///
/// Tier: T2-C (composed normalized metrics)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityComponents {
    /// Bioavailability score [0, 1].
    pub bioavailability: f64,
    /// Stability score [0, 1] (inverse of coefficient of variation).
    pub stability: f64,
    /// Safety margin score [0, 1] (therapeutic index).
    pub safety_margin: f64,
    /// Persistence score [0, 1] (bell curve around optimal half-life).
    pub persistence: f64,
}

/// Quality rating categories mapped to score ranges.
///
/// Tier: T2-P (enum over κ comparison thresholds)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QualityRating {
    /// Score [0, 2) — insufficient support.
    Subtherapeutic,
    /// Score [2, 4) — below optimal.
    Marginal,
    /// Score [4, 6) — adequate support.
    Therapeutic,
    /// Score [6, 8) — good support.
    Optimal,
    /// Score [8, 10] — excellent support.
    Exemplary,
}

impl QualityRating {
    /// Classify a raw score [0, 10] into a rating.
    #[must_use]
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 2.0 => Self::Subtherapeutic,
            s if s < 4.0 => Self::Marginal,
            s if s < 6.0 => Self::Therapeutic,
            s if s < 8.0 => Self::Optimal,
            _ => Self::Exemplary,
        }
    }
}

impl std::fmt::Display for QualityRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Subtherapeutic => write!(f, "Subtherapeutic"),
            Self::Marginal => write!(f, "Marginal"),
            Self::Therapeutic => write!(f, "Therapeutic"),
            Self::Optimal => write!(f, "Optimal"),
            Self::Exemplary => write!(f, "Exemplary"),
        }
    }
}

/// Composite quality score result.
///
/// Tier: T2-C (composes score + components + rating)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityScore {
    /// Total quality score [0, 10].
    pub total: f64,
    /// Individual factor scores.
    pub components: QualityComponents,
    /// Categorical rating.
    pub rating: QualityRating,
}

/// Sigmoid normalization: 1 / (1 + e^(-k*(x - midpoint)))
#[must_use]
fn sigmoid(x: f64, midpoint: f64, steepness: f64) -> f64 {
    1.0 / (1.0 + (-steepness * (x - midpoint)).exp())
}

/// Bell curve normalization: e^(-(x - center)² / (2σ²))
#[must_use]
fn bell_curve(x: f64, center: f64, sigma: f64) -> f64 {
    if sigma <= 0.0 {
        return 0.0;
    }
    (-(x - center).powi(2) / (2.0 * sigma.powi(2))).exp()
}

/// Coefficient of variation: std_dev / mean.
///
/// Returns 0.0 if mean is zero or data is empty.
#[must_use]
fn coefficient_of_variation(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    if mean.abs() < f64::EPSILON {
        return 0.0;
    }
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    variance.sqrt() / mean
}

/// Score a care episode's quality on a [0, 10] scale.
///
/// # Errors
/// Returns `CcpError::NoInterventions` if the episode has no interventions.
pub fn score_episode(episode: &Episode) -> Result<QualityScore, CcpError> {
    score_episode_with_window(episode, TherapeuticWindow::default_window())
}

/// Score with a custom therapeutic window.
///
/// # Errors
/// Returns `CcpError::NoInterventions` if the episode has no interventions.
pub fn score_episode_with_window(
    episode: &Episode,
    window: TherapeuticWindow,
) -> Result<QualityScore, CcpError> {
    if episode.interventions.is_empty() {
        return Err(CcpError::NoInterventions);
    }

    // 1. Bioavailability: sigmoid of average bioavailability
    let avg_bio: f64 = episode
        .interventions
        .iter()
        .map(|i| i.bioavailability.value())
        .sum::<f64>()
        / episode.interventions.len() as f64;
    let bioavailability_score = sigmoid(avg_bio, 0.5, 10.0);

    // 2. Stability: 1 - CV of level history
    let level_history = episode.level_history();
    let level_values: Vec<f64> = level_history.iter().map(|l| l.value()).collect();
    let cv = coefficient_of_variation(&level_values);
    let stability_score = (1.0 - cv).clamp(0.0, 1.0);

    // 3. Safety margin: therapeutic index of current level
    let safety_score = kinetics::therapeutic_index(window, episode.plasma_level);

    // 4. Persistence: bell curve of average half-life around optimal
    let avg_hl: f64 = episode
        .interventions
        .iter()
        .map(|i| i.half_life.value())
        .sum::<f64>()
        / episode.interventions.len() as f64;
    let persistence_score = bell_curve(avg_hl, OPTIMAL_HALF_LIFE, 12.0);

    // Weighted composite → [0, 1] → scale to [0, 10]
    let composite = bioavailability_score * W_BIOAVAILABILITY
        + stability_score * W_STABILITY
        + safety_score * W_SAFETY
        + persistence_score * W_PERSISTENCE;
    let total = (composite * 10.0).clamp(0.0, 10.0);

    let components = QualityComponents {
        bioavailability: bioavailability_score,
        stability: stability_score,
        safety_margin: safety_score,
        persistence: persistence_score,
    };

    Ok(QualityScore {
        total,
        components,
        rating: QualityRating::from_score(total),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::Intervention;
    use crate::types::{BioAvailability, Dose, DosingStrategy, HalfLife};

    fn make_intervention(dose: f64, bio: f64, hl: f64) -> Intervention {
        Intervention {
            dose: Dose::new(dose).unwrap_or(Dose::ZERO),
            bioavailability: BioAvailability::new(bio).unwrap_or_else(|_| {
                BioAvailability::new(1.0).unwrap_or_else(|_| panic!("unreachable"))
            }),
            half_life: HalfLife::new(hl)
                .unwrap_or_else(|_| HalfLife::new(1.0).unwrap_or_else(|_| panic!("unreachable"))),
            strategy: DosingStrategy::Therapeutic,
            administered_at: 0.0,
        }
    }

    fn make_episode_with_interventions(interventions: Vec<Intervention>, plasma: f64) -> Episode {
        let mut ep = Episode::new("test", 0.0);
        for i in interventions {
            ep.interventions.push(i);
        }
        ep.plasma_level = PlasmaLevel(plasma);
        ep
    }

    // ── Normalization functions ─────────────────────────────────────────

    #[test]
    fn sigmoid_at_midpoint() {
        let s = sigmoid(0.5, 0.5, 10.0);
        assert!((s - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn sigmoid_above_midpoint() {
        let s = sigmoid(0.9, 0.5, 10.0);
        assert!(s > 0.9);
    }

    #[test]
    fn bell_curve_at_center() {
        let b = bell_curve(24.0, 24.0, 12.0);
        assert!((b - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bell_curve_away_from_center() {
        let b = bell_curve(48.0, 24.0, 12.0);
        assert!(b < 0.5);
    }

    #[test]
    fn cv_constant_values() {
        let cv = coefficient_of_variation(&[0.5, 0.5, 0.5]);
        assert!((cv).abs() < f64::EPSILON);
    }

    #[test]
    fn cv_variable_values() {
        let cv = coefficient_of_variation(&[0.1, 0.5, 0.9]);
        assert!(cv > 0.0);
    }

    // ── Rating classification ───────────────────────────────────────────

    #[test]
    fn rating_boundaries() {
        assert_eq!(
            QualityRating::from_score(0.0),
            QualityRating::Subtherapeutic
        );
        assert_eq!(
            QualityRating::from_score(1.9),
            QualityRating::Subtherapeutic
        );
        assert_eq!(QualityRating::from_score(2.0), QualityRating::Marginal);
        assert_eq!(QualityRating::from_score(4.0), QualityRating::Therapeutic);
        assert_eq!(QualityRating::from_score(6.0), QualityRating::Optimal);
        assert_eq!(QualityRating::from_score(8.0), QualityRating::Exemplary);
        assert_eq!(QualityRating::from_score(10.0), QualityRating::Exemplary);
    }

    // ── Scoring ────────────────────────────────────────────────────────

    #[test]
    fn score_no_interventions_errors() {
        let ep = Episode::new("test", 0.0);
        assert!(score_episode(&ep).is_err());
    }

    #[test]
    fn score_high_quality_episode() {
        let ep = make_episode_with_interventions(
            vec![make_intervention(0.5, 0.9, 24.0)],
            0.55, // within therapeutic window
        );
        let result = score_episode(&ep);
        assert!(result.is_ok());
        let score = result.unwrap_or_else(|_| QualityScore {
            total: 0.0,
            components: QualityComponents {
                bioavailability: 0.0,
                stability: 0.0,
                safety_margin: 0.0,
                persistence: 0.0,
            },
            rating: QualityRating::Subtherapeutic,
        });
        assert!(score.total > 5.0);
    }

    #[test]
    fn score_low_quality_episode() {
        let ep = make_episode_with_interventions(
            vec![make_intervention(0.1, 0.2, 2.0)],
            0.02, // well below therapeutic window
        );
        let result = score_episode(&ep);
        assert!(result.is_ok());
        let score = result.unwrap_or_else(|_| QualityScore {
            total: 10.0,
            components: QualityComponents {
                bioavailability: 1.0,
                stability: 1.0,
                safety_margin: 1.0,
                persistence: 1.0,
            },
            rating: QualityRating::Exemplary,
        });
        assert!(score.total < 5.0);
    }

    #[test]
    fn score_is_bounded_0_10() {
        let ep = make_episode_with_interventions(vec![make_intervention(1.0, 1.0, 24.0)], 0.55);
        let result = score_episode(&ep);
        assert!(result.is_ok());
        let score = result.unwrap_or_else(|_| QualityScore {
            total: 5.0,
            components: QualityComponents {
                bioavailability: 0.5,
                stability: 0.5,
                safety_margin: 0.5,
                persistence: 0.5,
            },
            rating: QualityRating::Therapeutic,
        });
        assert!(score.total >= 0.0 && score.total <= 10.0);
    }

    #[test]
    fn components_are_bounded_0_1() {
        let ep = make_episode_with_interventions(vec![make_intervention(0.5, 0.7, 24.0)], 0.35);
        let result = score_episode(&ep);
        assert!(result.is_ok());
        let score = result.unwrap_or_else(|_| QualityScore {
            total: 5.0,
            components: QualityComponents {
                bioavailability: 0.5,
                stability: 0.5,
                safety_margin: 0.5,
                persistence: 0.5,
            },
            rating: QualityRating::Therapeutic,
        });
        let c = &score.components;
        assert!(c.bioavailability >= 0.0 && c.bioavailability <= 1.0);
        assert!(c.stability >= 0.0 && c.stability <= 1.0);
        assert!(c.safety_margin >= 0.0 && c.safety_margin <= 1.0);
        assert!(c.persistence >= 0.0 && c.persistence <= 1.0);
    }

    #[test]
    fn custom_window_scoring() {
        let ep = make_episode_with_interventions(vec![make_intervention(0.5, 0.8, 24.0)], 0.5);
        let wide_window =
            TherapeuticWindow::new(0.1, 0.9).unwrap_or(TherapeuticWindow::default_window());
        let result = score_episode_with_window(&ep, wide_window);
        assert!(result.is_ok());
    }
}
