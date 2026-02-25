//! # Cross-Domain Transfer
//!
//! Compute transfer confidence when applying primitives across domains.
//!
//! ## Formula
//!
//! ```text
//! confidence = (structural × 0.4) + (functional × 0.4) + (contextual × 0.2)
//! final = confidence × tier_multiplier
//! ```
//!
//! ## Tier: T2-P (Quantity + Comparison + Boundary)

use crate::primitiva::{LexPrimitiva, PrimitiveComposition};
use crate::tier::Tier;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Weight for structural similarity in confidence calculation.
pub const STRUCTURAL_WEIGHT: f64 = 0.4;
/// Weight for functional similarity in confidence calculation.
pub const FUNCTIONAL_WEIGHT: f64 = 0.4;
/// Weight for contextual similarity in confidence calculation.
pub const CONTEXTUAL_WEIGHT: f64 = 0.2;

/// A domain for cross-domain transfer analysis.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Domain {
    /// Domain name.
    pub name: String,
    /// Domain description.
    pub description: String,
    /// Core primitives in this domain.
    pub core_primitives: Vec<LexPrimitiva>,
}

impl Domain {
    /// Create a new domain.
    #[must_use]
    pub fn new(name: &str, description: &str, primitives: Vec<LexPrimitiva>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            core_primitives: primitives,
        }
    }

    /// Get primitives as a set for fast lookup.
    #[must_use]
    pub fn primitive_set(&self) -> BTreeSet<LexPrimitiva> {
        self.core_primitives.iter().copied().collect()
    }

    /// Predefined: Computation domain.
    #[must_use]
    pub fn computation() -> Self {
        Self::new(
            "Computation",
            "Software, algorithms, data structures",
            vec![
                LexPrimitiva::Sequence,
                LexPrimitiva::Mapping,
                LexPrimitiva::State,
                LexPrimitiva::Recursion,
                LexPrimitiva::Causality,
                LexPrimitiva::Boundary,
            ],
        )
    }

    /// Predefined: Mathematics domain.
    #[must_use]
    pub fn mathematics() -> Self {
        Self::new(
            "Mathematics",
            "Pure mathematics, proofs, structures",
            vec![
                LexPrimitiva::Quantity,
                LexPrimitiva::Mapping,
                LexPrimitiva::Comparison,
                LexPrimitiva::Recursion,
                LexPrimitiva::Existence,
                LexPrimitiva::Void,
            ],
        )
    }

    /// Predefined: Physics domain.
    #[must_use]
    pub fn physics() -> Self {
        Self::new(
            "Physics",
            "Physical laws, forces, conservation",
            vec![
                LexPrimitiva::Quantity,
                LexPrimitiva::Causality,
                LexPrimitiva::Frequency,
                LexPrimitiva::Boundary,
                LexPrimitiva::Irreversibility,
            ],
        )
    }

    /// Predefined: Chemistry domain.
    #[must_use]
    pub fn chemistry() -> Self {
        Self::new(
            "Chemistry",
            "Molecular interactions, reactions",
            vec![
                LexPrimitiva::Quantity,
                LexPrimitiva::Mapping,
                LexPrimitiva::State,
                LexPrimitiva::Boundary,
                LexPrimitiva::Irreversibility,
            ],
        )
    }

    /// Predefined: Biology domain.
    #[must_use]
    pub fn biology() -> Self {
        Self::new(
            "Biology",
            "Living systems, evolution, genetics",
            vec![
                LexPrimitiva::Sequence,
                LexPrimitiva::State,
                LexPrimitiva::Recursion,
                LexPrimitiva::Existence,
                LexPrimitiva::Persistence,
            ],
        )
    }

    /// Predefined: Economics domain.
    #[must_use]
    pub fn economics() -> Self {
        Self::new(
            "Economics",
            "Markets, value, exchange",
            vec![
                LexPrimitiva::Quantity,
                LexPrimitiva::Mapping,
                LexPrimitiva::Comparison,
                LexPrimitiva::Causality,
                LexPrimitiva::State,
            ],
        )
    }

    /// Predefined: Law domain.
    #[must_use]
    pub fn law() -> Self {
        Self::new(
            "Law",
            "Legal systems, precedent, rules",
            vec![
                LexPrimitiva::Causality,
                LexPrimitiva::Boundary,
                LexPrimitiva::Comparison,
                LexPrimitiva::Sequence,
                LexPrimitiva::Persistence,
            ],
        )
    }
}

/// Result of a transfer confidence calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TransferResult {
    /// Source domain.
    pub source: String,
    /// Target domain.
    pub target: String,
    /// Structural similarity score.
    pub structural: f64,
    /// Functional similarity score.
    pub functional: f64,
    /// Contextual similarity score.
    pub contextual: f64,
    /// Aggregate confidence (before tier adjustment).
    pub aggregate: f64,
    /// Tier of the transferred concept.
    pub tier: Tier,
    /// Final confidence (after tier adjustment).
    pub final_confidence: f64,
    /// Shared primitives between domains.
    pub shared_primitives: Vec<LexPrimitiva>,
    /// Limiting factors for transfer.
    pub limiting_factors: Vec<String>,
    // ── Enhanced confidence calibration (C4) ──────────────────────────────
    /// Compositional isomorphism: 1.0 − normalized tree edit distance
    /// between composition trees. Uses structural (Jaccard) as proxy.
    #[serde(default)]
    pub compositional_isomorphism: f64,
    /// Fraction of relationships preserved under translation (functor
    /// faithfulness). Uses functional score as proxy.
    #[serde(default)]
    pub relational_preservation: f64,
    /// All three enhanced metrics > 0.7 — suitable for clinical decision
    /// support (high-confidence cross-domain transfer).
    #[serde(default)]
    pub is_clinical_grade: bool,
}

/// Calculator for cross-domain transfer confidence.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct TransferCalculator;

impl TransferCalculator {
    /// Create a new calculator.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Calculate transfer confidence between domains.
    #[must_use]
    pub fn calculate(
        &self,
        source: &Domain,
        target: &Domain,
        composition: &PrimitiveComposition,
    ) -> TransferResult {
        let source_set = source.primitive_set();
        let target_set = target.primitive_set();
        let shared = Self::shared_primitives(&source_set, &target_set);
        let structural = Self::structural_score(&source_set, &target_set, &shared);
        let functional = Self::functional_score(composition, &target_set);
        let contextual = Self::contextual_score(&source.name, &target.name);
        let aggregate = Self::aggregate_score(structural, functional, contextual);
        let tier = Tier::classify(composition);
        let final_confidence = aggregate * tier.transfer_multiplier();
        let limiting_factors = Self::find_limits(
            &source.name,
            &target.name,
            structural,
            functional,
            contextual,
        );

        // C4: enhanced confidence calibration
        let compositional_isomorphism = structural; // Jaccard proxy for tree edit distance
        let relational_preservation = functional; // functional score as functor proxy
        let is_clinical_grade = structural > 0.7 && functional > 0.7 && contextual > 0.7;

        TransferResult {
            source: source.name.clone(),
            target: target.name.clone(),
            structural,
            functional,
            contextual,
            aggregate,
            tier,
            final_confidence,
            shared_primitives: shared.into_iter().collect(),
            limiting_factors,
            compositional_isomorphism,
            relational_preservation,
            is_clinical_grade,
        }
    }

    /// Find primitives in both sets.
    fn shared_primitives(
        source: &BTreeSet<LexPrimitiva>,
        target: &BTreeSet<LexPrimitiva>,
    ) -> BTreeSet<LexPrimitiva> {
        source.intersection(target).copied().collect()
    }

    /// Calculate structural similarity (Jaccard index).
    fn structural_score(
        source: &BTreeSet<LexPrimitiva>,
        target: &BTreeSet<LexPrimitiva>,
        shared: &BTreeSet<LexPrimitiva>,
    ) -> f64 {
        let union_size = source.union(target).count();
        if union_size == 0 {
            return 0.0;
        }
        #[allow(
            clippy::as_conversions,
            reason = "shared.len() and union_size bounded by 16; safe cast to f64"
        )]
        let result = shared.len() as f64 / union_size as f64;
        result
    }

    /// Calculate functional similarity.
    fn functional_score(
        composition: &PrimitiveComposition,
        target: &BTreeSet<LexPrimitiva>,
    ) -> f64 {
        let unique = composition.unique();
        if unique.is_empty() {
            return 1.0;
        }
        let matching = unique.iter().filter(|p| target.contains(p)).count();
        #[allow(
            clippy::as_conversions,
            reason = "matching and unique.len() bounded by 16; safe cast to f64"
        )]
        let result = matching as f64 / unique.len() as f64;
        result
    }

    /// Calculate contextual similarity from domain affinity.
    fn contextual_score(source: &str, target: &str) -> f64 {
        if source == target {
            return 1.0;
        }
        Self::domain_affinity(source, target)
    }

    /// Domain affinity lookup.
    fn domain_affinity(a: &str, b: &str) -> f64 {
        let pairs = [
            (("Computation", "Mathematics"), 0.9),
            (("Mathematics", "Computation"), 0.9),
            (("Physics", "Mathematics"), 0.85),
            (("Mathematics", "Physics"), 0.85),
            (("Physics", "Chemistry"), 0.8),
            (("Chemistry", "Physics"), 0.8),
            (("Chemistry", "Biology"), 0.75),
            (("Biology", "Chemistry"), 0.75),
            (("Economics", "Mathematics"), 0.7),
            (("Mathematics", "Economics"), 0.7),
            (("Law", "Computation"), 0.5),
            (("Computation", "Law"), 0.5),
        ];
        for ((x, y), score) in pairs {
            if a == x && b == y {
                return score;
            }
        }
        0.6 // Default
    }

    /// Compute weighted aggregate.
    fn aggregate_score(structural: f64, functional: f64, contextual: f64) -> f64 {
        structural * STRUCTURAL_WEIGHT
            + functional * FUNCTIONAL_WEIGHT
            + contextual * CONTEXTUAL_WEIGHT
    }

    /// Identify limiting factors.
    fn find_limits(
        source: &str,
        target: &str,
        structural: f64,
        functional: f64,
        contextual: f64,
    ) -> Vec<String> {
        let mut factors = Vec::new();
        if structural < 0.5 {
            factors.push(format!("Low structural overlap: {} ↔ {}", source, target));
        }
        if functional < 0.5 {
            factors.push("Composition primitives not core to target".to_string());
        }
        if contextual < 0.6 {
            factors.push(format!("Domain distance: {} → {}", source, target));
        }
        factors
    }

    /// Quick transfer check.
    #[must_use]
    pub fn quick_check(&self, composition: &PrimitiveComposition, target: &Domain) -> bool {
        let tier = Tier::classify(composition);
        let functional = Self::functional_score(composition, &target.primitive_set());
        functional >= 0.5 && tier.transfer_multiplier() >= 0.7
    }
}

/// Shortcut: calculate transfer from computation to another domain.
#[must_use]
pub fn from_computation(target: &Domain, composition: &PrimitiveComposition) -> TransferResult {
    TransferCalculator::new().calculate(&Domain::computation(), target, composition)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_domain_transfer() {
        let calc = TransferCalculator::new();
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Sequence]);
        let result = calc.calculate(&Domain::computation(), &Domain::computation(), &comp);
        assert!((result.contextual - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_computation_to_math() {
        let calc = TransferCalculator::new();
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Recursion]);
        let result = calc.calculate(&Domain::computation(), &Domain::mathematics(), &comp);
        assert!(result.final_confidence > 0.5);
    }

    #[test]
    fn test_tier_affects_confidence() {
        let calc = TransferCalculator::new();
        let t1_comp = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        let t3_comp = PrimitiveComposition::new(LexPrimitiva::all().to_vec());
        let t1_result = calc.calculate(&Domain::computation(), &Domain::mathematics(), &t1_comp);
        let t3_result = calc.calculate(&Domain::computation(), &Domain::mathematics(), &t3_comp);
        assert!(t1_result.tier.transfer_multiplier() > t3_result.tier.transfer_multiplier());
    }

    #[test]
    fn test_shared_primitives() {
        let calc = TransferCalculator::new();
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        let result = calc.calculate(&Domain::physics(), &Domain::chemistry(), &comp);
        assert!(result.shared_primitives.contains(&LexPrimitiva::Quantity));
    }

    #[test]
    fn test_quick_check() {
        let calc = TransferCalculator::new();
        let good_comp = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        assert!(calc.quick_check(&good_comp, &Domain::mathematics()));
    }

    #[test]
    fn test_from_computation_shortcut() {
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Causality]);
        let result = from_computation(&Domain::law(), &comp);
        assert_eq!(result.source, "Computation");
    }

    #[test]
    fn test_predefined_domains() {
        let domains = [
            Domain::computation(),
            Domain::mathematics(),
            Domain::physics(),
            Domain::chemistry(),
            Domain::biology(),
            Domain::economics(),
            Domain::law(),
        ];
        for domain in &domains {
            assert!(!domain.name.is_empty());
            assert!(!domain.core_primitives.is_empty());
        }
    }
}
