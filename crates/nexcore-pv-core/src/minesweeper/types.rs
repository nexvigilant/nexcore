//! Core types and constants for Minesweeper-PV.
//!
//! ## Safety Axioms
//!
//! The thresholds defined here implement the **Signal Confirmation Axiom**:
//! - FLAG_THRESHOLD (0.3): Minimum belief for flagging investigation
//! - CONFIRM_THRESHOLD (0.8): Minimum belief for signal confirmation
//! - UNFLAG_THRESHOLD (0.15): Maximum belief for deferring investigation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// CONSTANTS AND THRESHOLDS
// =============================================================================

/// Minimum belief threshold to flag a cell for investigation
pub const FLAG_THRESHOLD: f64 = 0.3;

/// Minimum belief threshold to confirm a signal
pub const CONFIRM_THRESHOLD: f64 = 0.8;

/// Maximum uncertainty (sigma) allowed for confirmation
pub const CONFIRM_UNCERTAINTY: f64 = 0.3;

/// Maximum belief threshold to defer (unflag) a cell
pub const UNFLAG_THRESHOLD: f64 = 0.15;

/// Decay factor for evidence propagation per hop
pub const PROPAGATION_DECAY: f64 = 0.8;

/// Minimum weight threshold for propagation to continue
pub const PROPAGATION_THRESHOLD: f64 = 0.1;

/// Minimum EIG value to recommend investigation
pub const MIN_EIG_FOR_ACTION: f64 = 0.5;

/// Prior probability for a drug-event pair being a true signal
pub const BASE_SIGNAL_RATE: f64 = 0.01;

/// Default adjacency weights by relationship type
#[allow(dead_code)] // Alternative to const array - keep for HashMap convenience
pub fn default_adjacency_weights() -> HashMap<AdjacencyType, f64> {
    let mut weights = HashMap::new();
    weights.insert(AdjacencyType::Mechanistic, 0.35);
    weights.insert(AdjacencyType::Phenotypic, 0.25);
    weights.insert(AdjacencyType::Temporal, 0.15);
    weights.insert(AdjacencyType::Demographic, 0.15);
    weights.insert(AdjacencyType::Concomitant, 0.10);
    weights
}

/// Constant re-export for HashMap-based default weights
pub const DEFAULT_ADJACENCY_WEIGHTS: &[(&str, f64)] = &[
    ("mechanistic", 0.35),
    ("phenotypic", 0.25),
    ("temporal", 0.15),
    ("demographic", 0.15),
    ("concomitant", 0.10),
];

// =============================================================================
// ENUMS
// =============================================================================

/// Status of a cell in the CSP grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellStatus {
    /// Not yet investigated
    #[default]
    Unknown,
    /// Has been investigated with evidence collected
    Investigated,
    /// Flagged as potential signal requiring attention
    Flagged,
    /// Confirmed as a validated safety signal
    Confirmed,
    /// Deferred due to low probability or insufficient evidence
    Deferred,
}

impl CellStatus {
    /// Convert to display string
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Investigated => "Investigated",
            Self::Flagged => "Flagged",
            Self::Confirmed => "Confirmed",
            Self::Deferred => "Deferred",
        }
    }
}

/// Type of adjacency relationship between cells
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjacencyType {
    /// Same drug class / mechanism of action
    Mechanistic,
    /// Similar clinical presentation / phenotype
    Phenotypic,
    /// Similar time-to-onset pattern
    Temporal,
    /// Similar patient demographics
    Demographic,
    /// Concomitant medication relationship
    Concomitant,
}

impl AdjacencyType {
    /// Get the default weight for this adjacency type
    #[must_use]
    pub const fn default_weight(&self) -> f64 {
        match self {
            Self::Mechanistic => 0.35,
            Self::Phenotypic => 0.25,
            Self::Temporal => 0.15,
            Self::Demographic => 0.15,
            Self::Concomitant => 0.10,
        }
    }
}

/// Temporal window for time-to-onset categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalWindow {
    /// 0-24 hours (immediate reactions)
    Acute,
    /// 1-7 days (early onset)
    Early,
    /// 1-4 weeks (intermediate onset)
    #[default]
    Intermediate,
    /// 1-3 months (late onset)
    Late,
    /// 3+ months (delayed reactions)
    Delayed,
}

impl TemporalWindow {
    /// Get the numeric index for ordering
    #[must_use]
    pub const fn index(&self) -> u8 {
        match self {
            Self::Acute => 0,
            Self::Early => 1,
            Self::Intermediate => 2,
            Self::Late => 3,
            Self::Delayed => 4,
        }
    }

    /// Get descriptive duration string
    #[must_use]
    pub const fn duration_description(&self) -> &'static str {
        match self {
            Self::Acute => "0-24 hours",
            Self::Early => "1-7 days",
            Self::Intermediate => "1-4 weeks",
            Self::Late => "1-3 months",
            Self::Delayed => "3+ months",
        }
    }
}

/// Temporal pattern strength for evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalPatternStrength {
    /// Strong temporal association (consistent, biologically plausible)
    Strong,
    /// Moderate temporal association
    Moderate,
    /// Weak temporal association
    Weak,
    /// Inconsistent temporal pattern (may suggest no causal relationship)
    Inconsistent,
    /// Unknown temporal pattern
    #[default]
    Unknown,
}

impl TemporalPatternStrength {
    /// Get likelihood ratio contribution for this strength
    #[must_use]
    pub const fn likelihood_ratio(&self) -> f64 {
        match self {
            Self::Strong => 3.0,
            Self::Moderate => 1.5,
            Self::Weak => 1.0,
            Self::Inconsistent => 0.5,
            Self::Unknown => 1.0,
        }
    }
}

/// Mechanistic plausibility for evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MechanisticPlausibility {
    /// Established mechanism with strong scientific support
    Established,
    /// Plausible mechanism based on known pharmacology
    Plausible,
    /// Speculative mechanism with limited evidence
    Speculative,
    /// Implausible mechanism (contradicts known pharmacology)
    Implausible,
    /// Unknown mechanism
    #[default]
    Unknown,
}

impl MechanisticPlausibility {
    /// Get likelihood ratio contribution for this plausibility
    #[must_use]
    pub const fn likelihood_ratio(&self) -> f64 {
        match self {
            Self::Established => 4.0,
            Self::Plausible => 2.0,
            Self::Speculative => 1.2,
            Self::Implausible => 0.3,
            Self::Unknown => 1.0,
        }
    }
}

/// Source of evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    /// Direct observation/investigation
    #[default]
    Direct,
    /// Propagated from adjacent cell
    Propagated,
    /// From literature review
    Literature,
    /// From regulatory database (FAERS, etc.)
    Regulatory,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_status_default() {
        assert_eq!(CellStatus::default(), CellStatus::Unknown);
    }

    #[test]
    fn test_adjacency_type_weights() {
        assert!((AdjacencyType::Mechanistic.default_weight() - 0.35).abs() < f64::EPSILON);
        assert!((AdjacencyType::Concomitant.default_weight() - 0.10).abs() < f64::EPSILON);
    }

    #[test]
    fn test_temporal_window_ordering() {
        assert!(TemporalWindow::Acute.index() < TemporalWindow::Early.index());
        assert!(TemporalWindow::Late.index() < TemporalWindow::Delayed.index());
    }

    #[test]
    fn test_temporal_pattern_lr() {
        assert!(TemporalPatternStrength::Strong.likelihood_ratio() > 1.0);
        assert!(TemporalPatternStrength::Inconsistent.likelihood_ratio() < 1.0);
        assert!((TemporalPatternStrength::Unknown.likelihood_ratio() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mechanistic_plausibility_lr() {
        assert!(
            MechanisticPlausibility::Established.likelihood_ratio()
                > MechanisticPlausibility::Plausible.likelihood_ratio()
        );
        assert!(MechanisticPlausibility::Implausible.likelihood_ratio() < 1.0);
    }
}
