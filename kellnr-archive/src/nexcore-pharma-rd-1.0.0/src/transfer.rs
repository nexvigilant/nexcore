//! Cross-domain transfer confidence computation.
//!
//! Three-dimensional formula: `TC = structural × 0.4 + functional × 0.4 + contextual × 0.2`
//! with optional source confidence modifier.

use crate::taxonomy::PharmaPrimitive;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Target domains for transfer confidence computation.
///
/// Tier: T2-P | Dominant: Σ (Sum) — enum alternation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransferDomain {
    Biotechnology,
    MedicalDevices,
    Agrochemical,
}

impl TransferDomain {
    /// All target domains.
    pub const ALL: &'static [Self] = &[
        Self::Biotechnology,
        Self::MedicalDevices,
        Self::Agrochemical,
    ];
}

impl fmt::Display for TransferDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Biotechnology => write!(f, "Biotech"),
            Self::MedicalDevices => write!(f, "MedTech"),
            Self::Agrochemical => write!(f, "Agrochemical"),
        }
    }
}

/// Three-dimensional transfer confidence score.
///
/// Tier: T2-C | Dominant: N (Quantity) + κ (Comparison) + μ (Mapping).
///
/// Formula: `TC = structural × 0.4 + functional × 0.4 + contextual × 0.2`
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransferConfidence {
    /// Structural similarity (0.0-1.0): shared mathematical/physical foundations.
    pub structural: f64,
    /// Functional similarity (0.0-1.0): same operational role across domains.
    pub functional: f64,
    /// Contextual similarity (0.0-1.0): shared regulatory/cultural framework.
    pub contextual: f64,
    /// Source confidence modifier (default 1.0, expert-generated = 0.6).
    pub source_modifier: f64,
}

impl TransferConfidence {
    /// Create with explicit three dimensions and source modifier.
    #[must_use]
    pub fn new(structural: f64, functional: f64, contextual: f64, source_modifier: f64) -> Self {
        Self {
            structural: structural.clamp(0.0, 1.0),
            functional: functional.clamp(0.0, 1.0),
            contextual: contextual.clamp(0.0, 1.0),
            source_modifier: source_modifier.clamp(0.0, 1.0),
        }
    }

    /// Create with default source modifier (1.0 = corpus-validated).
    #[must_use]
    pub fn corpus_validated(structural: f64, functional: f64, contextual: f64) -> Self {
        Self::new(structural, functional, contextual, 1.0)
    }

    /// Create with expert-generation penalty (0.6 modifier).
    #[must_use]
    pub fn expert_generated(structural: f64, functional: f64, contextual: f64) -> Self {
        Self::new(structural, functional, contextual, 0.6)
    }

    /// Raw score before source modifier.
    #[must_use]
    pub fn raw_score(&self) -> f64 {
        self.structural * 0.4 + self.functional * 0.4 + self.contextual * 0.2
    }

    /// Final score with source modifier applied.
    #[must_use]
    pub fn final_score(&self) -> f64 {
        self.raw_score() * self.source_modifier
    }

    /// Qualitative confidence label.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self.final_score() {
            s if s >= 0.8 => "Very High",
            s if s >= 0.6 => "High",
            s if s >= 0.4 => "Moderate",
            s if s >= 0.2 => "Low",
            _ => "Very Low",
        }
    }

    /// Limiting factor: which dimension is weakest.
    #[must_use]
    pub fn limiting_factor(&self) -> &'static str {
        if self.structural <= self.functional && self.structural <= self.contextual {
            "Structural mismatch"
        } else if self.functional <= self.contextual {
            "Functional mismatch"
        } else {
            "Contextual mismatch"
        }
    }
}

impl fmt::Display for TransferConfidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.3} ({}) [S:{:.2} F:{:.2} C:{:.2} ×{:.1}]",
            self.final_score(),
            self.label(),
            self.structural,
            self.functional,
            self.contextual,
            self.source_modifier,
        )
    }
}

/// Look up pre-computed transfer confidence for a primitive to a domain.
///
/// These are expert-generated (×0.6 modifier) from domain analysis.
///
/// Grounding: μ (Mapping) — from (primitive, domain) → confidence score.
#[must_use]
pub fn lookup_transfer(primitive: PharmaPrimitive, domain: TransferDomain) -> TransferConfidence {
    use PharmaPrimitive::*;
    use TransferDomain::*;

    match (primitive, domain) {
        // ── Binding Affinity ──
        (BindingAffinity, Biotechnology) => TransferConfidence::expert_generated(0.95, 0.90, 0.85),
        (BindingAffinity, MedicalDevices) => TransferConfidence::expert_generated(0.15, 0.10, 0.05),
        (BindingAffinity, Agrochemical) => TransferConfidence::expert_generated(0.90, 0.90, 0.80),
        // ── Absorption ──
        (Absorption, Biotechnology) => TransferConfidence::expert_generated(0.50, 0.40, 0.60),
        (Absorption, MedicalDevices) => TransferConfidence::expert_generated(0.30, 0.20, 0.20),
        (Absorption, Agrochemical) => TransferConfidence::expert_generated(0.85, 0.80, 0.70),
        // ── Metabolism ──
        (Metabolism, Biotechnology) => TransferConfidence::expert_generated(0.45, 0.40, 0.55),
        (Metabolism, MedicalDevices) => TransferConfidence::expert_generated(0.15, 0.10, 0.15),
        (Metabolism, Agrochemical) => TransferConfidence::expert_generated(0.80, 0.80, 0.70),
        // ── Distribution ──
        (Distribution, Biotechnology) => TransferConfidence::expert_generated(0.50, 0.45, 0.55),
        (Distribution, MedicalDevices) => TransferConfidence::expert_generated(0.25, 0.20, 0.20),
        (Distribution, Agrochemical) => TransferConfidence::expert_generated(0.80, 0.75, 0.65),
        // ── Elimination ──
        (Elimination, Biotechnology) => TransferConfidence::expert_generated(0.55, 0.50, 0.50),
        (Elimination, MedicalDevices) => TransferConfidence::expert_generated(0.25, 0.20, 0.20),
        (Elimination, Agrochemical) => TransferConfidence::expert_generated(0.85, 0.80, 0.65),
        // ── Toxicity ──
        (Toxicity, Biotechnology) => TransferConfidence::expert_generated(0.80, 0.75, 0.70),
        (Toxicity, MedicalDevices) => TransferConfidence::expert_generated(0.70, 0.65, 0.60),
        (Toxicity, Agrochemical) => TransferConfidence::expert_generated(0.90, 0.85, 0.75),
        // ── Efficacy ──
        (Efficacy, Biotechnology) => TransferConfidence::expert_generated(0.90, 0.85, 0.80),
        (Efficacy, MedicalDevices) => TransferConfidence::expert_generated(0.80, 0.75, 0.65),
        (Efficacy, Agrochemical) => TransferConfidence::expert_generated(0.85, 0.80, 0.65),
        // ── Potency ──
        (Potency, Biotechnology) => TransferConfidence::expert_generated(0.85, 0.85, 0.75),
        (Potency, MedicalDevices) => TransferConfidence::expert_generated(0.15, 0.10, 0.15),
        (Potency, Agrochemical) => TransferConfidence::expert_generated(0.90, 0.90, 0.75),
        // ── Exposure ──
        (Exposure, Biotechnology) => TransferConfidence::expert_generated(0.80, 0.75, 0.70),
        (Exposure, MedicalDevices) => TransferConfidence::expert_generated(0.55, 0.50, 0.45),
        (Exposure, Agrochemical) => TransferConfidence::expert_generated(0.80, 0.75, 0.60),
        // ── Target ──
        (Target, Biotechnology) => TransferConfidence::expert_generated(0.90, 0.85, 0.75),
        (Target, MedicalDevices) => TransferConfidence::expert_generated(0.15, 0.10, 0.15),
        (Target, Agrochemical) => TransferConfidence::expert_generated(0.90, 0.85, 0.65),
        // ── DoseResponse ──
        (DoseResponse, Biotechnology) => TransferConfidence::expert_generated(0.85, 0.85, 0.80),
        (DoseResponse, MedicalDevices) => TransferConfidence::expert_generated(0.35, 0.30, 0.25),
        (DoseResponse, Agrochemical) => TransferConfidence::expert_generated(0.90, 0.90, 0.75),
        // ── Signal ──
        (Signal, Biotechnology) => TransferConfidence::expert_generated(0.90, 0.85, 0.90),
        (Signal, MedicalDevices) => TransferConfidence::expert_generated(0.70, 0.65, 0.55),
        (Signal, Agrochemical) => TransferConfidence::expert_generated(0.50, 0.40, 0.30),
        // ── MolecularStructure ──
        (MolecularStructure, Biotechnology) => {
            TransferConfidence::expert_generated(0.80, 0.75, 0.70)
        }
        (MolecularStructure, MedicalDevices) => {
            TransferConfidence::expert_generated(0.25, 0.20, 0.20)
        }
        (MolecularStructure, Agrochemical) => {
            TransferConfidence::expert_generated(0.95, 0.90, 0.80)
        }
        // ── Randomization ──
        (Randomization, Biotechnology) => TransferConfidence::expert_generated(0.95, 0.95, 0.90),
        (Randomization, MedicalDevices) => TransferConfidence::expert_generated(0.80, 0.75, 0.60),
        (Randomization, Agrochemical) => TransferConfidence::expert_generated(0.30, 0.25, 0.15),
        // ── Blinding ──
        (Blinding, Biotechnology) => TransferConfidence::expert_generated(0.95, 0.95, 0.90),
        (Blinding, MedicalDevices) => TransferConfidence::expert_generated(0.60, 0.50, 0.40),
        (Blinding, Agrochemical) => TransferConfidence::expert_generated(0.15, 0.10, 0.05),
        // ── Endpoint ──
        (Endpoint, Biotechnology) => TransferConfidence::expert_generated(0.95, 0.95, 0.90),
        (Endpoint, MedicalDevices) => TransferConfidence::expert_generated(0.85, 0.75, 0.60),
        (Endpoint, Agrochemical) => TransferConfidence::expert_generated(0.40, 0.35, 0.20),
        // ── HalfLife ──
        (HalfLife, Biotechnology) => TransferConfidence::expert_generated(0.80, 0.75, 0.70),
        (HalfLife, MedicalDevices) => TransferConfidence::expert_generated(0.25, 0.20, 0.20),
        (HalfLife, Agrochemical) => TransferConfidence::expert_generated(0.90, 0.85, 0.70),
        // ── Confounder ──
        (Confounder, Biotechnology) => TransferConfidence::expert_generated(0.90, 0.85, 0.80),
        (Confounder, MedicalDevices) => TransferConfidence::expert_generated(0.80, 0.70, 0.55),
        (Confounder, Agrochemical) => TransferConfidence::expert_generated(0.40, 0.35, 0.20),
        // ── Permeability ──
        (Permeability, Biotechnology) => TransferConfidence::expert_generated(0.50, 0.40, 0.50),
        (Permeability, MedicalDevices) => TransferConfidence::expert_generated(0.40, 0.30, 0.25),
        (Permeability, Agrochemical) => TransferConfidence::expert_generated(0.85, 0.80, 0.65),
        // ── Solubility ──
        (Solubility, Biotechnology) => TransferConfidence::expert_generated(0.60, 0.50, 0.55),
        (Solubility, MedicalDevices) => TransferConfidence::expert_generated(0.30, 0.20, 0.20),
        (Solubility, Agrochemical) => TransferConfidence::expert_generated(0.90, 0.85, 0.70),
        // ── MechanismOfAction ──
        (MechanismOfAction, Biotechnology) => {
            TransferConfidence::expert_generated(0.85, 0.85, 0.75)
        }
        (MechanismOfAction, MedicalDevices) => {
            TransferConfidence::expert_generated(0.25, 0.20, 0.20)
        }
        (MechanismOfAction, Agrochemical) => TransferConfidence::expert_generated(0.90, 0.90, 0.70),
        // ── Biomarker ──
        (Biomarker, Biotechnology) => TransferConfidence::expert_generated(0.90, 0.90, 0.80),
        (Biomarker, MedicalDevices) => TransferConfidence::expert_generated(0.55, 0.50, 0.40),
        (Biomarker, Agrochemical) => TransferConfidence::expert_generated(0.45, 0.40, 0.30),
        // ── PlaceboEffect ──
        (PlaceboEffect, Biotechnology) => TransferConfidence::expert_generated(0.85, 0.80, 0.75),
        (PlaceboEffect, MedicalDevices) => TransferConfidence::expert_generated(0.70, 0.65, 0.55),
        (PlaceboEffect, Agrochemical) => TransferConfidence::expert_generated(0.10, 0.05, 0.05),
        // ── StatisticalPower ──
        (StatisticalPower, Biotechnology) => TransferConfidence::expert_generated(0.95, 0.95, 0.90),
        (StatisticalPower, MedicalDevices) => {
            TransferConfidence::expert_generated(0.90, 0.80, 0.65)
        }
        (StatisticalPower, Agrochemical) => TransferConfidence::expert_generated(0.50, 0.40, 0.30),
    }
}

/// Compute full transfer matrix: all primitives × all domains.
#[must_use]
pub fn transfer_matrix() -> Vec<(PharmaPrimitive, TransferDomain, TransferConfidence)> {
    let mut results = Vec::new();
    for &prim in PharmaPrimitive::ALL {
        for &domain in TransferDomain::ALL {
            results.push((prim, domain, lookup_transfer(prim, domain)));
        }
    }
    results
}

/// Find strongest transfer corridors (top N by final score).
#[must_use]
pub fn strongest_transfers(top_n: usize) -> Vec<(PharmaPrimitive, TransferDomain, f64)> {
    let mut matrix: Vec<(PharmaPrimitive, TransferDomain, f64)> = transfer_matrix()
        .into_iter()
        .map(|(p, d, tc)| (p, d, tc.final_score()))
        .collect();
    matrix.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    matrix.truncate(top_n);
    matrix
}

/// Find weakest transfer corridors (bottom N by final score).
#[must_use]
pub fn weakest_transfers(bottom_n: usize) -> Vec<(PharmaPrimitive, TransferDomain, f64)> {
    let mut matrix: Vec<(PharmaPrimitive, TransferDomain, f64)> = transfer_matrix()
        .into_iter()
        .map(|(p, d, tc)| (p, d, tc.final_score()))
        .collect();
    matrix.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
    matrix.truncate(bottom_n);
    matrix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formula_correct() {
        let tc = TransferConfidence::new(0.8, 0.6, 0.4, 1.0);
        let expected = 0.8 * 0.4 + 0.6 * 0.4 + 0.4 * 0.2;
        assert!((tc.raw_score() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn source_modifier_applied() {
        let tc = TransferConfidence::expert_generated(1.0, 1.0, 1.0);
        assert!((tc.final_score() - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn corpus_validated_no_penalty() {
        let tc = TransferConfidence::corpus_validated(1.0, 1.0, 1.0);
        assert!((tc.final_score() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clamping_works() {
        let tc = TransferConfidence::new(1.5, -0.5, 0.5, 2.0);
        assert!((tc.structural - 1.0).abs() < f64::EPSILON);
        assert!(tc.functional.abs() < f64::EPSILON);
        assert!((tc.source_modifier - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn label_ranges() {
        let very_high = TransferConfidence::corpus_validated(1.0, 1.0, 1.0);
        assert_eq!(very_high.label(), "Very High");

        let very_low = TransferConfidence::corpus_validated(0.0, 0.0, 0.0);
        assert_eq!(very_low.label(), "Very Low");
    }

    #[test]
    fn transfer_matrix_has_correct_size() {
        let m = transfer_matrix();
        assert_eq!(m.len(), 24 * 3); // 24 primitives × 3 domains
    }

    #[test]
    fn strongest_returns_top_n() {
        let top5 = strongest_transfers(5);
        assert_eq!(top5.len(), 5);
        // Should be sorted descending
        for w in top5.windows(2) {
            assert!(w[0].2 >= w[1].2);
        }
    }

    #[test]
    fn weakest_returns_bottom_n() {
        let bottom5 = weakest_transfers(5);
        assert_eq!(bottom5.len(), 5);
        // Should be sorted ascending
        for w in bottom5.windows(2) {
            assert!(w[0].2 <= w[1].2);
        }
    }

    #[test]
    fn binding_affinity_high_for_biotech() {
        let tc = lookup_transfer(
            PharmaPrimitive::BindingAffinity,
            TransferDomain::Biotechnology,
        );
        assert!(tc.final_score() > 0.5);
    }

    #[test]
    fn binding_affinity_low_for_medtech() {
        let tc = lookup_transfer(
            PharmaPrimitive::BindingAffinity,
            TransferDomain::MedicalDevices,
        );
        assert!(tc.final_score() < 0.1);
    }

    #[test]
    fn display_includes_all_dimensions() {
        let tc = TransferConfidence::expert_generated(0.9, 0.8, 0.7);
        let s = format!("{tc}");
        assert!(s.contains("S:0.90"));
        assert!(s.contains("F:0.80"));
        assert!(s.contains("C:0.70"));
    }
}
