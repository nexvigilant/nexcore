//! Campion Signal Theory implementation.
//!
//! ## Safety Axioms
//!
//! The Campion Signal Theory (S = U × R × T) is a fundamental axiom of the
//! Tree of Vigilance for quantifying pharmacovigilance signal strength:
//!
//! - **U (Rarity)**: Information-theoretic measure of signal unexpectedness
//! - **R (Recognition)**: Confidence in signal identification
//! - **T (Temporal)**: Recency and relevance of the observation
//!
//! The product of these three factors provides a unified signal score.

// ============================================================================
// Constants - Campion Signal Theory
// ============================================================================

/// Recognition confidence values by cell status
/// Range: 0.0-1.0 where 1.0 = fully confirmed signal
pub mod recognition_scores {
    /// Unknown status: low confidence (30%)
    /// Rationale: Unexamined signals have high uncertainty
    pub const UNKNOWN: f64 = 0.3;
    /// Deferred status: slightly higher than unknown (40%)
    /// Rationale: Examined but postponed implies some knowledge
    pub const DEFERRED: f64 = 0.4;
    /// Investigated status: high confidence (80%)
    /// Rationale: Active investigation implies substantial evidence
    pub const INVESTIGATED: f64 = 0.8;
    /// Flagged status: very high confidence (90%)
    /// Rationale: Flagged for review indicates strong evidence
    pub const FLAGGED: f64 = 0.9;
    /// Confirmed status: full confidence (100%)
    /// Rationale: Confirmed signals have definitive evidence
    pub const CONFIRMED: f64 = 1.0;
}

/// Signal interpretation thresholds (S = U × R × T)
/// Based on clinical pharmacovigilance practice
pub mod signal_thresholds {
    /// Below this: negligible signal (no action needed)
    /// Validated: 0.1 threshold filters noise effectively
    pub const NEGLIGIBLE_UPPER: f64 = 0.1;
    /// Below this: weak signal (monitor passively)
    pub const WEAK_UPPER: f64 = 1.0;
    /// Below this: moderate signal (investigate actively)
    pub const MODERATE_UPPER: f64 = 5.0;
    /// Below this: strong signal (prioritize investigation)
    /// Above this: critical signal (immediate action)
    pub const STRONG_UPPER: f64 = 15.0;
}

/// Rarity (U) calculation constants
pub mod rarity_config {
    /// Maximum rarity value when probability approaches zero
    /// Represents ~1 in 1 million event (log₂(10⁶) ≈ 20)
    pub const MAX_RARITY: f64 = 20.0;
    /// Normalization factor for limiting factor comparison
    pub const NORMALIZATION_FACTOR: f64 = 20.0;
}

/// Temporal decay constants
pub mod temporal_config {
    /// Decay rate per day (λ = 0.02)
    /// Half-life = ln(2)/0.02 ≈ 35 days
    /// Rationale: PV signals remain relevant for ~2-3 months
    pub const DECAY_RATE_PER_DAY: f64 = 0.02;
}

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

use super::belief::Cell;
use super::grid::CSPGrid;
use crate::minesweeper::types::CellStatus;

/// Result of Campion Signal Theory calculation.
///
/// S = U × R × T where:
/// - U (Rarity): -log₂(P) information content
/// - R (Recognition): Status-based recognition confidence
/// - T (Temporal): Time decay factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampionSignalResult {
    /// Combined signal value (S = U × R × T)
    pub signal_value: f64,
    /// Log₂ of signal value for comparison
    pub log_signal_value: f64,
    /// Rarity component (U): information content in bits
    pub rarity_u: f64,
    /// Recognition component (R): identification confidence
    pub recognition_r: f64,
    /// Temporal component (T): recency factor
    pub temporal_t: f64,
    /// Human-readable interpretation
    pub interpretation: SignalInterpretation,
    /// Which component is the limiting factor
    pub limiting_factor: LimitingFactor,
}

/// Interpretation category for signal strength
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalInterpretation {
    /// S < 0.1: No action needed
    Negligible,
    /// 0.1 ≤ S < 1: Monitor passively
    Weak,
    /// 1 ≤ S < 5: Active investigation
    Moderate,
    /// 5 ≤ S < 15: High priority
    Strong,
    /// S ≥ 15: Immediate action required
    Critical,
}

impl SignalInterpretation {
    /// Get recommended action for this interpretation
    #[must_use]
    pub const fn recommended_action(&self) -> &'static str {
        match self {
            Self::Negligible => "No action needed",
            Self::Weak => "Monitor passively",
            Self::Moderate => "Investigate actively",
            Self::Strong => "Prioritize investigation",
            Self::Critical => "Immediate action required",
        }
    }
}

/// Which component is limiting the signal score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LimitingFactor {
    /// Rarity is the smallest normalized component
    Rarity,
    /// Recognition is the smallest normalized component
    Recognition,
    /// Temporal is the smallest normalized component
    Temporal,
}

/// Calculate Campion Signal Score for a cell.
///
/// The score combines three factors:
/// - **U (Rarity)**: Information content based on probability
/// - **R (Recognition)**: Confidence based on investigation status
/// - **T (Temporal)**: Recency factor based on last update
///
/// # Arguments
///
/// * `cell` - The cell to evaluate
/// * `grid` - The CSP grid (for neighbor probability calculation)
///
/// # Returns
///
/// A `CampionSignalResult` with the signal score and component breakdown.
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::minesweeper::{CSPGrid, cell_to_signal_score};
///
/// let mut grid = CSPGrid::new();
/// grid.add_cell("aspirin", "gi_bleeding", "all");
///
/// if let Some(cell) = grid.get_cell("aspirin", "gi_bleeding", "all") {
///     let result = cell_to_signal_score(cell, &grid);
///     println!("Signal: {} ({})", result.signal_value, result.interpretation.recommended_action());
/// }
/// ```
#[must_use]
pub fn cell_to_signal_score(cell: &Cell, grid: &CSPGrid) -> CampionSignalResult {
    // U (Rarity): Information content
    // Consider neighbor probabilities for configuration rarity
    let neighbors = grid.get_neighbors(&cell.id);
    let configuration_prob = if neighbors.is_empty() {
        cell.theta()
    } else {
        let neighbor_prob_product: f64 = neighbors
            .keys()
            .filter_map(|n_id| grid.get_cell_by_id(n_id))
            .map(|n| n.theta())
            .product();
        cell.theta() * neighbor_prob_product.max(f64::MIN_POSITIVE)
    };

    let u = if configuration_prob > 0.0 {
        -configuration_prob.log2()
    } else {
        rarity_config::MAX_RARITY
    };

    // R (Recognition): Status-based confidence
    let r = match cell.status {
        CellStatus::Unknown => recognition_scores::UNKNOWN,
        CellStatus::Investigated => recognition_scores::INVESTIGATED,
        CellStatus::Flagged => recognition_scores::FLAGGED,
        CellStatus::Confirmed => recognition_scores::CONFIRMED,
        CellStatus::Deferred => recognition_scores::DEFERRED,
    };

    // T (Temporal): Time decay
    let t = calculate_temporal_factor(cell.last_updated);

    // Combined signal score
    let s = u * r * t;
    let log_s = if s > 0.0 { s.log2() } else { 0.0 };

    // Interpretation
    let interpretation = if s < signal_thresholds::NEGLIGIBLE_UPPER {
        SignalInterpretation::Negligible
    } else if s < signal_thresholds::WEAK_UPPER {
        SignalInterpretation::Weak
    } else if s < signal_thresholds::MODERATE_UPPER {
        SignalInterpretation::Moderate
    } else if s < signal_thresholds::STRONG_UPPER {
        SignalInterpretation::Strong
    } else {
        SignalInterpretation::Critical
    };

    // Determine limiting factor (normalized comparison)
    let u_normalized = u / rarity_config::NORMALIZATION_FACTOR;
    let r_normalized = r;
    let t_normalized = t;

    let limiting_factor = if u_normalized <= r_normalized && u_normalized <= t_normalized {
        LimitingFactor::Rarity
    } else if r_normalized <= t_normalized {
        LimitingFactor::Recognition
    } else {
        LimitingFactor::Temporal
    };

    CampionSignalResult {
        signal_value: s,
        log_signal_value: log_s,
        rarity_u: u,
        recognition_r: r,
        temporal_t: t,
        interpretation,
        limiting_factor,
    }
}

/// Calculate temporal decay factor based on last update time.
///
/// Uses exponential decay: T = e^(-0.02 × days_since_update)
fn calculate_temporal_factor(last_updated: DateTime) -> f64 {
    let now = DateTime::now();
    let duration = now.signed_duration_since(last_updated);
    let days = duration.num_days().max(0) as f64;

    // Exponential decay with half-life of ~35 days
    (-temporal_config::DECAY_RATE_PER_DAY * days).exp()
}

/// Calculate batch signal scores for multiple cells
#[must_use]
#[allow(dead_code)] // API for batch processing - expose when needed
pub fn batch_signal_scores(grid: &CSPGrid) -> Vec<(String, CampionSignalResult)> {
    grid.cells()
        .iter()
        .map(|(cell_id, cell)| {
            let result = cell_to_signal_score(cell, grid);
            (cell_id.clone(), result)
        })
        .collect()
}

/// Get cells sorted by signal score (descending)
#[must_use]
#[allow(dead_code)] // API for ranked retrieval - expose when needed
pub fn ranked_by_signal(grid: &CSPGrid, top_n: usize) -> Vec<(String, CampionSignalResult)> {
    let mut scores = batch_signal_scores(grid);
    scores.sort_by(|a, b| {
        b.1.signal_value
            .partial_cmp(&a.1.signal_value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scores.truncate(top_n);
    scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minesweeper::Evidence;

    fn create_test_grid() -> CSPGrid {
        let mut grid = CSPGrid::new();
        grid.add_cell("aspirin", "gi_bleeding", "all");
        grid.add_cell("aspirin", "tinnitus", "all");
        grid
    }

    #[test]
    fn test_signal_score_calculation() {
        let grid = create_test_grid();
        let cell = grid.get_cell("aspirin", "gi_bleeding", "all").unwrap();

        let result = cell_to_signal_score(cell, &grid);

        // Unknown cell should have weak recognition
        assert!((result.recognition_r - recognition_scores::UNKNOWN).abs() < f64::EPSILON);
        // Temporal should be close to 1.0 for new cell
        assert!(result.temporal_t > 0.99);
    }

    #[test]
    fn test_signal_interpretation() {
        let grid = create_test_grid();
        let cell = grid.get_cell("aspirin", "gi_bleeding", "all").unwrap();

        let result = cell_to_signal_score(cell, &grid);

        // New unknown cell: U is high (rare) but R is low (unknown)
        assert!(matches!(
            result.interpretation,
            SignalInterpretation::Negligible
                | SignalInterpretation::Weak
                | SignalInterpretation::Moderate
        ));
    }

    #[test]
    fn test_signal_increases_with_evidence() {
        let mut grid = create_test_grid();

        let initial_cell = grid.get_cell("aspirin", "gi_bleeding", "all").unwrap();
        let initial_result = cell_to_signal_score(initial_cell, &grid);

        // Add strong evidence
        let evidence = Evidence::builder()
            .prr(5.0)
            .chi2(15.0)
            .count(20)
            .temporal_pattern("strong")
            .mechanism("established")
            .build();

        grid.propagate_evidence("aspirin|gi_bleeding|all", &evidence);

        let updated_cell = grid.get_cell("aspirin", "gi_bleeding", "all").unwrap();
        let updated_result = cell_to_signal_score(updated_cell, &grid);

        // Signal should increase
        assert!(
            updated_result.signal_value > initial_result.signal_value,
            "Expected signal to increase: {} -> {}",
            initial_result.signal_value,
            updated_result.signal_value
        );
    }

    #[test]
    fn test_limiting_factor() {
        let mut grid = CSPGrid::new();
        grid.add_cell("drug_x", "event_y", "all");

        let cell = grid.get_cell("drug_x", "event_y", "all").unwrap();
        let result = cell_to_signal_score(cell, &grid);

        // For unknown cell, recognition (0.3) should be limiting
        assert_eq!(result.limiting_factor, LimitingFactor::Recognition);
    }

    #[test]
    fn test_batch_signal_scores() {
        let grid = create_test_grid();
        let scores = batch_signal_scores(&grid);

        assert_eq!(scores.len(), 2);
    }

    #[test]
    fn test_ranked_by_signal() {
        let mut grid = create_test_grid();

        // Add evidence to one cell
        let evidence = Evidence::builder().prr(3.0).chi2(8.0).count(5).build();
        grid.propagate_evidence("aspirin|gi_bleeding|all", &evidence);

        let ranked = ranked_by_signal(&grid, 10);

        // Investigated cell should rank higher
        assert_eq!(ranked[0].0, "aspirin|gi_bleeding|all");
    }

    #[test]
    fn test_interpretation_thresholds() {
        assert_eq!(
            SignalInterpretation::Negligible.recommended_action(),
            "No action needed"
        );
        assert_eq!(
            SignalInterpretation::Critical.recommended_action(),
            "Immediate action required"
        );
    }
}
