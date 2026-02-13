// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Cross-domain transfer confidence computation.
//!
//! ## Primitive Grounding: κ (Comparison) + N (Numeric) + μ (Mapping)
//!
//! Transfer confidence = structural × 0.4 + functional × 0.4 + contextual × 0.2

use crate::{PrimaTier, Subject};
use serde::{Deserialize, Serialize};

/// Transfer calculation between two domains.
///
/// ## Tier: T2-P (κ + N + μ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResult {
    /// Source domain.
    pub from: String,
    /// Target domain.
    pub to: String,
    /// Structural similarity (0.0 - 1.0).
    pub structural: f64,
    /// Functional similarity (0.0 - 1.0).
    pub functional: f64,
    /// Contextual relevance (0.0 - 1.0).
    pub contextual: f64,
    /// Overall transfer confidence.
    pub confidence: f64,
    /// Transfer tier classification.
    pub tier: PrimaTier,
}

impl TransferResult {
    /// Compute transfer confidence from three dimensions.
    ///
    /// Formula: `structural × 0.4 + functional × 0.4 + contextual × 0.2`
    #[must_use]
    pub fn compute(
        from: &str,
        to: &str,
        structural: f64,
        functional: f64,
        contextual: f64,
    ) -> Self {
        let s = structural.clamp(0.0, 1.0);
        let f = functional.clamp(0.0, 1.0);
        let c = contextual.clamp(0.0, 1.0);

        let confidence = s * 0.4 + f * 0.4 + c * 0.2;

        let tier = if confidence >= 0.9 {
            PrimaTier::T1
        } else if confidence >= 0.7 {
            PrimaTier::T2P
        } else if confidence >= 0.5 {
            PrimaTier::T2C
        } else {
            PrimaTier::T3
        };

        Self {
            from: from.to_string(),
            to: to.to_string(),
            structural: s,
            functional: f,
            contextual: c,
            confidence,
            tier,
        }
    }

    /// Quick transfer from tier alone.
    #[must_use]
    pub fn from_tier(from: &str, to: &str, tier: PrimaTier) -> Self {
        let confidence = tier.transfer_confidence();
        Self {
            from: from.to_string(),
            to: to.to_string(),
            structural: confidence,
            functional: confidence,
            contextual: confidence,
            confidence,
            tier,
        }
    }
}

/// Domain affinity matrix for transfer calculations.
///
/// ## Tier: T2-C (μ + σ + κ)
#[derive(Debug, Clone, Default)]
pub struct AffinityMatrix {
    entries: Vec<(Subject, Subject, f64)>,
}

impl AffinityMatrix {
    /// Create a new affinity matrix with default STEM/Healthcare affinities.
    #[must_use]
    pub fn new() -> Self {
        let mut matrix = Self::default();

        // STEM internal affinities (high structural transfer)
        matrix.add(Subject::Mathematics, Subject::Physics, 0.9);
        matrix.add(Subject::Mathematics, Subject::ComputerScience, 0.85);
        matrix.add(Subject::Mathematics, Subject::Statistics, 0.95);
        matrix.add(Subject::Physics, Subject::Chemistry, 0.8);
        matrix.add(Subject::Physics, Subject::Engineering, 0.85);
        matrix.add(Subject::Chemistry, Subject::Biology, 0.75);
        matrix.add(Subject::Biology, Subject::Medicine, 0.8);
        matrix.add(Subject::ComputerScience, Subject::Statistics, 0.8);

        // Healthcare internal affinities
        matrix.add(Subject::Medicine, Subject::Pharmacology, 0.9);
        matrix.add(Subject::Medicine, Subject::Nursing, 0.75);
        matrix.add(Subject::Pharmacology, Subject::Chemistry, 0.7);
        matrix.add(Subject::PublicHealth, Subject::Medicine, 0.65);
        matrix.add(Subject::PublicHealth, Subject::Statistics, 0.7);

        // Cross-domain bridges
        matrix.add(Subject::Statistics, Subject::PublicHealth, 0.7);
        matrix.add(Subject::ComputerScience, Subject::Biology, 0.5); // Bioinformatics
        matrix.add(Subject::Mathematics, Subject::Economics, 0.75);
        matrix.add(Subject::Psychology, Subject::Medicine, 0.5);

        matrix
    }

    /// Add or update affinity between two subjects.
    pub fn add(&mut self, a: Subject, b: Subject, affinity: f64) {
        let affinity = affinity.clamp(0.0, 1.0);

        // Check if already exists
        for entry in &mut self.entries {
            if (entry.0 == a && entry.1 == b) || (entry.0 == b && entry.1 == a) {
                entry.2 = affinity;
                return;
            }
        }

        self.entries.push((a, b, affinity));
    }

    /// Get affinity between two subjects.
    #[must_use]
    pub fn affinity(&self, a: Subject, b: Subject) -> f64 {
        // Same subject = perfect affinity
        if a == b {
            return 1.0;
        }

        // Check matrix
        for &(s1, s2, aff) in &self.entries {
            if (s1 == a && s2 == b) || (s1 == b && s2 == a) {
                return aff;
            }
        }

        // Check if same category (STEM/Healthcare)
        if a.is_stem() && b.is_stem() {
            return 0.4; // Base STEM affinity
        }
        if a.is_healthcare() && b.is_healthcare() {
            return 0.5; // Base healthcare affinity
        }

        // Default minimal affinity
        0.2
    }

    /// Compute transfer between two subjects using the matrix.
    #[must_use]
    pub fn transfer(&self, from: Subject, to: Subject) -> TransferResult {
        let structural = self.affinity(from, to);

        // Functional similarity based on category overlap
        let functional =
            if from.is_stem() == to.is_stem() && from.is_healthcare() == to.is_healthcare() {
                0.7
            } else if from.is_stem() && to.is_healthcare() || from.is_healthcare() && to.is_stem() {
                0.4 // Some overlap (e.g., bioinformatics, medical physics)
            } else {
                0.3
            };

        // Contextual is always moderate unless same subject
        let contextual = if from == to { 1.0 } else { 0.5 };

        TransferResult::compute(from.code(), to.code(), structural, functional, contextual)
    }
}

/// Capability multiplier: tracks capability growth.
///
/// ## Tier: T2-C (σ + N + →)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityMultiplier {
    /// Base capabilities count.
    pub base: u32,
    /// Current capabilities count.
    pub current: u32,
    /// Multiplier ratio.
    pub ratio: f64,
    /// Growth history.
    pub history: Vec<(String, u32)>,
}

impl CapabilityMultiplier {
    /// Create new multiplier starting at base count.
    #[must_use]
    pub fn new(base: u32) -> Self {
        Self {
            base,
            current: base,
            ratio: 1.0,
            history: vec![("initial".to_string(), base)],
        }
    }

    /// Add capabilities.
    pub fn add(&mut self, label: &str, count: u32) {
        self.current += count;
        self.ratio = self.current as f64 / self.base.max(1) as f64;
        self.history.push((label.to_string(), self.current));
    }

    /// Get current multiplier ratio.
    #[must_use]
    pub fn multiplier(&self) -> f64 {
        self.ratio
    }

    /// Check if 2x multiplication achieved.
    #[must_use]
    pub fn doubled(&self) -> bool {
        self.ratio >= 2.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_compute() {
        let result = TransferResult::compute("A", "B", 0.8, 0.9, 0.7);
        // 0.8*0.4 + 0.9*0.4 + 0.7*0.2 = 0.32 + 0.36 + 0.14 = 0.82
        assert!((result.confidence - 0.82).abs() < 0.01);
        assert_eq!(result.tier, PrimaTier::T2P);
    }

    #[test]
    fn test_transfer_from_tier() {
        let result = TransferResult::from_tier("MTH", "PHY", PrimaTier::T1);
        assert!((result.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_affinity_same_subject() {
        let matrix = AffinityMatrix::new();
        assert!(
            (matrix.affinity(Subject::Mathematics, Subject::Mathematics) - 1.0).abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn test_affinity_known_pair() {
        let matrix = AffinityMatrix::new();
        let aff = matrix.affinity(Subject::Mathematics, Subject::Physics);
        assert!(aff > 0.8); // High affinity
    }

    #[test]
    fn test_affinity_stem_default() {
        let matrix = AffinityMatrix::new();
        let aff = matrix.affinity(Subject::Engineering, Subject::Biology);
        assert!(aff >= 0.4); // At least base STEM affinity
    }

    #[test]
    fn test_affinity_cross_category() {
        let matrix = AffinityMatrix::new();
        let aff = matrix.affinity(Subject::Philosophy, Subject::Engineering);
        assert!(aff < 0.4); // Low cross-category affinity
    }

    #[test]
    fn test_matrix_transfer() {
        let matrix = AffinityMatrix::new();
        let result = matrix.transfer(Subject::Mathematics, Subject::Physics);
        assert!(result.confidence > 0.6); // Good transfer
        assert!(result.tier == PrimaTier::T2P || result.tier == PrimaTier::T1);
    }

    #[test]
    fn test_capability_multiplier() {
        let mut mult = CapabilityMultiplier::new(100);
        assert!(!mult.doubled());

        mult.add("first", 50);
        assert!((mult.multiplier() - 1.5).abs() < 0.01);

        mult.add("second", 50);
        assert!(mult.doubled());
    }

    #[test]
    fn test_transfer_tier_boundaries() {
        // T1: >= 0.9
        let t1 = TransferResult::compute("A", "B", 1.0, 1.0, 1.0);
        assert_eq!(t1.tier, PrimaTier::T1);

        // T2-P: >= 0.7
        let t2p = TransferResult::compute("A", "B", 0.8, 0.8, 0.5);
        assert_eq!(t2p.tier, PrimaTier::T2P);

        // T2-C: >= 0.5
        let t2c = TransferResult::compute("A", "B", 0.5, 0.5, 0.5);
        assert_eq!(t2c.tier, PrimaTier::T2C);

        // T3: < 0.5
        let t3 = TransferResult::compute("A", "B", 0.3, 0.3, 0.3);
        assert_eq!(t3.tier, PrimaTier::T3);
    }
}
