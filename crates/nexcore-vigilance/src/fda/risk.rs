//! Model Risk Assessment (Step 3)
//!
//! ## T1 Grounding
//!
//! - **ModelRisk**: κ (Comparison) + N (Quantity)
//!   - Risk = f(ModelInfluence, DecisionConsequence)
//!   - Matrix-based comparison
//!
//! - **ModelInfluence**: ∝ (Proportionality)
//!   - Contribution of AI relative to other evidence
//!
//! - **DecisionConsequence**: N (Quantity) + κ (Comparison)
//!   - Severity × Probability × Detectability

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// Step 3: Model influence on decision-making
///
/// T1 Grounding: ∝ (Proportionality) — AI contribution relative to total evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ModelInfluence {
    /// AI is one of many evidence sources
    Low,
    /// AI is significant but not dominant
    Medium,
    /// AI is sole or primary determinant
    High,
}

impl ModelInfluence {
    /// Converts to numeric score for calculations
    ///
    /// T1 Grounding: N (Quantity)
    pub fn score(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
        }
    }

    /// Infers from evidence count (more sources = lower influence)
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn from_evidence_count(count: usize) -> Self {
        match count {
            0 => Self::High,       // Sole source
            1..=2 => Self::Medium, // Primary with confirmation
            _ => Self::Low,        // One of many
        }
    }
}

impl fmt::Display for ModelInfluence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
        }
    }
}

/// Magnitude of potential adverse outcome
///
/// T1 Grounding: N (Quantity) — Severity, Probability, Detectability
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DecisionConsequence {
    /// Easily reversible, low impact
    Low,
    /// Moderate impact, partially reversible
    Medium,
    /// Irreversible harm, high impact
    High,
}

impl DecisionConsequence {
    /// Converts to numeric score
    ///
    /// T1 Grounding: N (Quantity)
    pub fn score(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
        }
    }

    /// Derives from severity, probability, and detectability
    ///
    /// T1 Grounding: μ (Mapping) — Three factors → Consequence
    pub fn from_factors(severity: u8, probability: u8, detectability: u8) -> Self {
        let product = severity * probability * detectability;
        match product {
            0..=8 => Self::Low,
            9..=18 => Self::Medium,
            _ => Self::High,
        }
    }
}

impl fmt::Display for DecisionConsequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
        }
    }
}

/// Overall model risk level (FDA Figure 1 matrix)
///
/// T1 Grounding: κ (Comparison) — Risk matrix classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl RiskLevel {
    /// Computes from influence and consequence per FDA matrix
    ///
    /// ## Matrix
    ///
    /// ```text
    ///               High Risk
    ///                 ↗
    /// Consequence   3│●●●
    ///               2│●●
    ///               1│●
    ///                └─────────►
    ///                1 2 3  Influence
    /// ```
    ///
    /// T1 Grounding: μ (Mapping) — (Influence, Consequence) → Risk
    pub fn from_matrix(influence: ModelInfluence, consequence: DecisionConsequence) -> Self {
        let i = influence.score();
        let c = consequence.score();

        match (i, c) {
            (3, 3) => Self::High,                     // High influence, high consequence
            (3, 2) | (2, 3) => Self::Medium,          // One high factor
            (3, 1) | (2, 2) | (1, 3) => Self::Medium, // Diagonal
            (2, 1) | (1, 2) => Self::Medium,          // Mixed low
            (1, 1) => Self::Low,                      // Both low
            _ => Self::Low,                           // Fallback
        }
    }

    /// Returns minimum credibility evidence required
    ///
    /// T1 Grounding: N (Quantity)
    pub fn min_evidence_count(&self) -> usize {
        match self {
            Self::Low => 2,
            Self::Medium => 4,
            Self::High => 8,
        }
    }
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
        }
    }
}

/// Step 3: Model Risk = f(Model Influence, Decision Consequence)
///
/// T1 Grounding: κ (Comparison) + N (Quantity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRisk {
    influence: ModelInfluence,
    consequence: DecisionConsequence,
    level: RiskLevel,
}

impl ModelRisk {
    /// Creates a new model risk assessment
    ///
    /// T1 Grounding: μ (Mapping) — Two factors → Risk
    pub fn new(influence: ModelInfluence, consequence: DecisionConsequence) -> Self {
        let level = RiskLevel::from_matrix(influence, consequence);
        Self {
            influence,
            consequence,
            level,
        }
    }

    pub fn influence(&self) -> ModelInfluence {
        self.influence
    }

    pub fn consequence(&self) -> DecisionConsequence {
        self.consequence
    }

    pub fn level(&self) -> RiskLevel {
        self.level
    }

    /// Returns true if risk level is acceptable for given evidence
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn is_acceptable_with_evidence(&self, evidence_count: usize) -> bool {
        evidence_count >= self.level.min_evidence_count()
    }
}

impl PartialOrd for ModelRisk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ModelRisk {
    fn cmp(&self, other: &Self) -> Ordering {
        self.level.cmp(&other.level)
    }
}

impl fmt::Display for ModelRisk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} Risk (Influence: {}, Consequence: {})",
            self.level, self.influence, self.consequence
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_influence_ordering() {
        assert!(ModelInfluence::Low < ModelInfluence::Medium);
        assert!(ModelInfluence::Medium < ModelInfluence::High);
    }

    #[test]
    fn test_model_influence_from_evidence_count() {
        assert_eq!(ModelInfluence::from_evidence_count(0), ModelInfluence::High);
        assert_eq!(
            ModelInfluence::from_evidence_count(1),
            ModelInfluence::Medium
        );
        assert_eq!(ModelInfluence::from_evidence_count(5), ModelInfluence::Low);
    }

    #[test]
    fn test_decision_consequence_from_factors() {
        // Low: 1*1*1 = 1
        assert_eq!(
            DecisionConsequence::from_factors(1, 1, 1),
            DecisionConsequence::Low
        );
        // Medium: 2*2*2 = 8
        assert_eq!(
            DecisionConsequence::from_factors(2, 2, 2),
            DecisionConsequence::Low
        );
        // Medium: 3*2*2 = 12
        assert_eq!(
            DecisionConsequence::from_factors(3, 2, 2),
            DecisionConsequence::Medium
        );
        // High: 3*3*3 = 27
        assert_eq!(
            DecisionConsequence::from_factors(3, 3, 3),
            DecisionConsequence::High
        );
    }

    #[test]
    fn test_risk_level_matrix_high() {
        let risk = RiskLevel::from_matrix(ModelInfluence::High, DecisionConsequence::High);
        assert_eq!(risk, RiskLevel::High);
    }

    #[test]
    fn test_risk_level_matrix_medium() {
        let risk = RiskLevel::from_matrix(ModelInfluence::High, DecisionConsequence::Medium);
        assert_eq!(risk, RiskLevel::Medium);

        let risk2 = RiskLevel::from_matrix(ModelInfluence::Medium, DecisionConsequence::High);
        assert_eq!(risk2, RiskLevel::Medium);
    }

    #[test]
    fn test_risk_level_matrix_low() {
        let risk = RiskLevel::from_matrix(ModelInfluence::Low, DecisionConsequence::Low);
        assert_eq!(risk, RiskLevel::Low);
    }

    #[test]
    fn test_model_risk_creation() {
        let risk = ModelRisk::new(ModelInfluence::High, DecisionConsequence::High);
        assert_eq!(risk.level(), RiskLevel::High);
        assert_eq!(risk.influence(), ModelInfluence::High);
        assert_eq!(risk.consequence(), DecisionConsequence::High);
    }

    #[test]
    fn test_model_risk_acceptable_evidence() {
        let low_risk = ModelRisk::new(ModelInfluence::Low, DecisionConsequence::Low);
        assert!(low_risk.is_acceptable_with_evidence(2));
        assert!(!low_risk.is_acceptable_with_evidence(1));

        let high_risk = ModelRisk::new(ModelInfluence::High, DecisionConsequence::High);
        assert!(high_risk.is_acceptable_with_evidence(8));
        assert!(!high_risk.is_acceptable_with_evidence(7));
    }

    #[test]
    fn test_model_risk_ordering() {
        let low = ModelRisk::new(ModelInfluence::Low, DecisionConsequence::Low);
        let high = ModelRisk::new(ModelInfluence::High, DecisionConsequence::High);
        assert!(low < high);
    }

    #[test]
    fn test_model_risk_display() {
        let risk = ModelRisk::new(ModelInfluence::Medium, DecisionConsequence::High);
        let s = risk.to_string();
        assert!(s.contains("Medium Risk"));
        assert!(s.contains("Influence: Medium"));
        assert!(s.contains("Consequence: High"));
    }
}
