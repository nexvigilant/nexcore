//! Cross-domain transfer confidence computation.
//!
//! Formula: `confidence = structural × 0.4 + functional × 0.4 + contextual × 0.2`

use serde::{Deserialize, Serialize};

/// Weights for the three transfer dimensions.
const W_STRUCTURAL: f64 = 0.4;
const W_FUNCTIONAL: f64 = 0.4;
const W_CONTEXTUAL: f64 = 0.2;

/// Three-dimensional transfer score between domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TransferScore {
    /// Structural similarity (topology, dependencies, shape) — 0.0 to 1.0
    pub structural: f64,
    /// Functional similarity (input/output, purpose) — 0.0 to 1.0
    pub functional: f64,
    /// Contextual similarity (time scales, stakeholders, environment) — 0.0 to 1.0
    pub contextual: f64,
}

impl TransferScore {
    /// Create a new transfer score, clamping each dimension to [0.0, 1.0].
    pub fn new(structural: f64, functional: f64, contextual: f64) -> Self {
        Self {
            structural: structural.clamp(0.0, 1.0),
            functional: functional.clamp(0.0, 1.0),
            contextual: contextual.clamp(0.0, 1.0),
        }
    }

    /// Weighted confidence: `S×0.4 + F×0.4 + C×0.2`
    pub fn confidence(&self) -> f64 {
        self.structural * W_STRUCTURAL
            + self.functional * W_FUNCTIONAL
            + self.contextual * W_CONTEXTUAL
    }

    /// Identify which dimension is the limiting factor.
    pub fn limiting_factor(&self) -> &'static str {
        if self.structural <= self.functional && self.structural <= self.contextual {
            "structural"
        } else if self.functional <= self.contextual {
            "functional"
        } else {
            "contextual"
        }
    }

    /// Compositional isomorphism: 1.0 − normalized distance between composition
    /// trees.  Uses Jaccard (structural) as proxy for tree edit distance.
    pub fn compositional_isomorphism(&self) -> f64 {
        self.structural
    }

    /// Fraction of relationships preserved under translation (functor
    /// faithfulness).  Uses functional score as proxy.
    pub fn relational_preservation(&self) -> f64 {
        self.functional
    }

    /// All three enhanced metrics > 0.7 — suitable for clinical decision
    /// support (high-confidence cross-domain transfer).
    pub fn is_clinical_grade(&self) -> bool {
        self.structural > 0.7 && self.functional > 0.7 && self.contextual > 0.7
    }
}

/// A computed transfer from one primitive to a target domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DomainTransfer {
    /// Name of the primitive being transferred.
    pub primitive_name: String,
    /// Target domain for the transfer.
    pub target_domain: String,
    /// Three-dimensional score.
    pub score: TransferScore,
    /// Human-readable description of the limiting factor.
    pub limiting_description: String,
}

impl DomainTransfer {
    pub fn confidence(&self) -> f64 {
        self.score.confidence()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_formula() {
        let score = TransferScore::new(0.95, 0.88, 0.75);
        let c = score.confidence();
        // 0.95*0.4 + 0.88*0.4 + 0.75*0.2 = 0.38 + 0.352 + 0.15 = 0.882
        assert!((c - 0.882).abs() < 1e-10);
    }

    #[test]
    fn perfect_transfer() {
        let score = TransferScore::new(1.0, 1.0, 1.0);
        assert!((score.confidence() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn zero_transfer() {
        let score = TransferScore::new(0.0, 0.0, 0.0);
        assert!((score.confidence()).abs() < 1e-10);
    }

    #[test]
    fn clamping() {
        let score = TransferScore::new(1.5, -0.3, 0.5);
        assert!((score.structural - 1.0).abs() < 1e-10);
        assert!((score.functional - 0.0).abs() < 1e-10);
    }

    #[test]
    fn limiting_factor_structural() {
        let score = TransferScore::new(0.5, 0.9, 0.8);
        assert_eq!(score.limiting_factor(), "structural");
    }

    #[test]
    fn limiting_factor_functional() {
        let score = TransferScore::new(0.9, 0.4, 0.8);
        assert_eq!(score.limiting_factor(), "functional");
    }

    #[test]
    fn limiting_factor_contextual() {
        let score = TransferScore::new(0.9, 0.9, 0.3);
        assert_eq!(score.limiting_factor(), "contextual");
    }

    #[test]
    fn clinical_grade_all_high() {
        let score = TransferScore::new(0.85, 0.80, 0.75);
        assert!(score.is_clinical_grade());
        assert!((score.compositional_isomorphism() - 0.85).abs() < 1e-10);
        assert!((score.relational_preservation() - 0.80).abs() < 1e-10);
    }

    #[test]
    fn clinical_grade_one_low() {
        let score = TransferScore::new(0.85, 0.60, 0.75);
        assert!(!score.is_clinical_grade());
    }
}
