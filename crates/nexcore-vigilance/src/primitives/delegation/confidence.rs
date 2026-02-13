//! Confidence Scoring - T2-Primitive
//!
//! Multi-dimensional scoring to scalar confidence.
//! Decomposes to: Mapping (dimension→score) + Sequence (weighted sum)

use serde::{Deserialize, Serialize};

/// A dimension contributing to confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDimension {
    pub name: String,
    pub value: f64,
    pub weight: f64,
}

impl ScoreDimension {
    pub fn new(name: impl Into<String>, value: f64, weight: f64) -> Self {
        Self {
            name: name.into(),
            value: value.clamp(0.0, 1.0),
            weight,
        }
    }

    /// Weighted contribution
    pub fn contribution(&self) -> f64 {
        self.value * self.weight
    }
}

/// Confidence score with breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    pub total: f64,
    pub dimensions: Vec<ScoreDimension>,
}

impl ConfidenceScore {
    /// Compute confidence from dimensions
    /// T1: Sequence (iterate) + Mapping (dimension→contribution)
    pub fn compute(dimensions: Vec<ScoreDimension>) -> Self {
        let total_weight: f64 = dimensions.iter().map(|d| d.weight).sum();
        let weighted_sum: f64 = dimensions.iter().map(|d| d.contribution()).sum();

        let total = if total_weight > 0.0 {
            (weighted_sum / total_weight).clamp(0.0, 1.0)
        } else {
            0.0
        };

        Self { total, dimensions }
    }

    /// Check if confidence exceeds threshold
    pub fn exceeds(&self, threshold: f64) -> bool {
        self.total >= threshold
    }
}

/// Delegation-specific confidence calculator
#[derive(Debug, Clone)]
pub struct DelegationConfidence {
    pub pattern_score: f64,
    pub item_count_score: f64,
    pub error_tolerance_score: f64,
}

impl DelegationConfidence {
    /// Standard weights for delegation decision
    const PATTERN_WEIGHT: f64 = 0.4;
    const ITEM_COUNT_WEIGHT: f64 = 0.4;
    const TOLERANCE_WEIGHT: f64 = 0.2;

    pub fn new(patterns_matched: usize, item_count: usize, error_tolerance: f64) -> Self {
        Self {
            pattern_score: (patterns_matched as f64 * 0.2).min(0.6),
            item_count_score: Self::item_score(item_count),
            error_tolerance_score: error_tolerance,
        }
    }

    fn item_score(count: usize) -> f64 {
        match count {
            0..=9 => 0.1,
            10..=49 => 0.2,
            50..=99 => 0.3,
            _ => 0.4,
        }
    }

    /// Compute final confidence score
    pub fn compute(&self) -> ConfidenceScore {
        ConfidenceScore::compute(vec![
            ScoreDimension::new("patterns", self.pattern_score, Self::PATTERN_WEIGHT),
            ScoreDimension::new("item_count", self.item_count_score, Self::ITEM_COUNT_WEIGHT),
            ScoreDimension::new(
                "error_tolerance",
                self.error_tolerance_score,
                Self::TOLERANCE_WEIGHT,
            ),
        ])
    }

    /// Should delegate based on confidence
    pub fn should_delegate(&self, threshold: f64) -> bool {
        self.compute().exceeds(threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_scoring() {
        let confidence = DelegationConfidence::new(2, 100, 0.8);
        let score = confidence.compute();
        assert!(score.total > 0.3);
        assert!(confidence.should_delegate(0.3));
    }
}
