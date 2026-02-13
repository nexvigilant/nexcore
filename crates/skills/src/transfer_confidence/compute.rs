//! Transfer confidence computation

use serde::{Deserialize, Serialize};

/// Confidence tier based on score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceTier {
    /// High confidence (>0.75)
    High,
    /// Medium confidence (0.50-0.75)
    Medium,
    /// Low confidence (0.25-0.50)
    Low,
    /// Very low confidence (<0.25)
    VeryLow,
}

/// Transfer confidence result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferScore {
    /// Overall confidence (0.0-1.0)
    pub confidence: f64,
    /// Structural similarity score
    pub structural: f64,
    /// Functional similarity score
    pub functional: f64,
    /// Contextual similarity score
    pub contextual: f64,
    /// Confidence tier
    pub tier: ConfidenceTier,
}

/// Transfer confidence calculator
pub struct TransferConfidence {
    /// Weight for structural dimension
    pub w_structural: f64,
    /// Weight for functional dimension
    pub w_functional: f64,
    /// Weight for contextual dimension
    pub w_contextual: f64,
}

impl Default for TransferConfidence {
    fn default() -> Self {
        Self {
            w_structural: 0.4,
            w_functional: 0.4,
            w_contextual: 0.2,
        }
    }
}

impl TransferConfidence {
    /// Compute transfer confidence
    pub fn compute(&self, structural: f64, functional: f64, contextual: f64) -> TransferScore {
        let s = clamp(structural);
        let f = clamp(functional);
        let c = clamp(contextual);

        let confidence = weighted_sum(
            s,
            f,
            c,
            self.w_structural,
            self.w_functional,
            self.w_contextual,
        );
        let tier = classify_tier(confidence);

        TransferScore {
            confidence,
            structural: s,
            functional: f,
            contextual: c,
            tier,
        }
    }
}

/// Convenience function with default weights
pub fn compute_confidence(structural: f64, functional: f64, contextual: f64) -> TransferScore {
    TransferConfidence::default().compute(structural, functional, contextual)
}

fn clamp(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

fn weighted_sum(s: f64, f: f64, c: f64, ws: f64, wf: f64, wc: f64) -> f64 {
    (s * ws) + (f * wf) + (c * wc)
}

fn classify_tier(confidence: f64) -> ConfidenceTier {
    match confidence {
        c if c >= 0.75 => ConfidenceTier::High,
        c if c >= 0.50 => ConfidenceTier::Medium,
        c if c >= 0.25 => ConfidenceTier::Low,
        _ => ConfidenceTier::VeryLow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_confidence() {
        let result = compute_confidence(0.8, 0.8, 0.8);
        assert!((result.confidence - 0.8).abs() < 0.001);
        assert_eq!(result.tier, ConfidenceTier::High);
    }

    #[test]
    fn test_weights_sum_to_one() {
        let tc = TransferConfidence::default();
        let sum = tc.w_structural + tc.w_functional + tc.w_contextual;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_tier_classification() {
        assert_eq!(classify_tier(0.9), ConfidenceTier::High);
        assert_eq!(classify_tier(0.6), ConfidenceTier::Medium);
        assert_eq!(classify_tier(0.3), ConfidenceTier::Low);
        assert_eq!(classify_tier(0.1), ConfidenceTier::VeryLow);
    }
}
