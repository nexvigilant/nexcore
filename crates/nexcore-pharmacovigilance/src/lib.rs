//! # NexVigilant Core — pharmacovigilance
//!
//! WHO-grounded pharmacovigilance taxonomy encoded as typed Rust.
//!
//! ## WHO Definition
//!
//! > "The science and activities relating to the **detection**, **assessment**,
//! > **understanding**, and **prevention** of adverse effects or any other
//! > drug-related problem."
//!
//! ## Taxonomy (120+ concepts)
//!
//! | Tier | Count | Examples |
//! |------|-------|---------|
//! | T1 | 16 | Lex Primitiva symbols (σ, μ, ς, ...) |
//! | T2-P | 22 | `Threshold`, `Harm`, `Severity`, `Sensitivity` |
//! | T2-C | 20 | `Signal`, `AdverseEvent`, `Seriousness`, `BenefitRiskEvaluation` |
//! | T3 | 66+ | `Prr`, `Naranjo`, `BradfordHill`, `Rems`, `FAERS`, `Hy'sLaw` |
//!
//! ## Chomsky Grammar Classification
//!
//! The WHO definition's 4 verbs ascend the Chomsky hierarchy:
//!
//! | WHO Verb | Level | Automaton |
//! |----------|-------|-----------|
//! | Detection | Type-3 | Finite Automaton |
//! | Assessment | Type-2 | Pushdown Automaton |
//! | Understanding | Type-1 | Linear Bounded Automaton |
//! | Prevention | Type-0 | Turing Machine |
//!
//! ## Transfer Confidence
//!
//! `TC = structural × 0.4 + functional × 0.4 + contextual × 0.2`
//!
//! | Target | Score | Label |
//! |--------|-------|-------|
//! | Regulatory Affairs | 0.91 | Very High |
//! | Clinical Trials | 0.86 | Very High |
//! | Epidemiology | 0.82 | High |
//! | Health Economics | 0.61 | Moderate |
//!
//! ## Signature Primitive
//!
//! **κ (Comparison)** dominates: 34/120 concepts contain it.
//! PV is fundamentally *systematic comparison*.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod analytics;
pub mod assessment;
pub mod chomsky;
pub mod composites;
pub mod detection;
pub mod grounding;
pub mod lex;
pub mod prevention;
pub mod primitives;
pub mod regulatory;
pub mod transfer;
pub mod understanding;

// Re-exports for convenience
pub use analytics::{AnalyticsConcept, SafetyCommsConcept, SpecialPopulationConcept};
pub use assessment::AssessmentConcept;
pub use chomsky::{ChomskyLevel, PvSubsystem, who_pillar_complexity};
pub use composites::PvComposite;
pub use detection::DetectionConcept;
pub use lex::{LexSymbol, PrimitiveComposition, Tier};
pub use prevention::{PreventionConcept, ScopeConcept};
pub use primitives::PvPrimitive;
pub use regulatory::{InfrastructureConcept, OperationsConcept, RegulatoryConcept};
pub use transfer::{
    TransferConfidence, TransferDomain, lookup_transfer, strongest_transfer, transfer_matrix,
    weakest_transfer,
};
pub use understanding::UnderstandingConcept;

/// Summary of the full taxonomy.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaxonomySummary {
    /// T1 universal primitives (Lex Primitiva).
    pub t1: usize,
    /// T2-P cross-domain primitives.
    pub t2p: usize,
    /// T2-C cross-domain composites.
    pub t2c: usize,
    /// T3 domain-specific concepts.
    pub t3: usize,
    /// Total concepts.
    pub total: usize,
}

/// Compute the taxonomy summary.
#[must_use]
pub fn taxonomy_summary() -> TaxonomySummary {
    let t1 = LexSymbol::ALL.len();
    let t2p = PvPrimitive::ALL.len();
    let t2c = PvComposite::ALL.len();
    let t3 = DetectionConcept::ALL.len()
        + AssessmentConcept::ALL.len()
        + UnderstandingConcept::ALL.len()
        + PreventionConcept::ALL.len()
        + ScopeConcept::ALL.len()
        + RegulatoryConcept::ALL.len()
        + InfrastructureConcept::ALL.len()
        + OperationsConcept::ALL.len()
        + AnalyticsConcept::ALL.len()
        + SafetyCommsConcept::ALL.len()
        + SpecialPopulationConcept::ALL.len();
    let total = t1 + t2p + t2c + t3;
    TaxonomySummary {
        t1,
        t2p,
        t2c,
        t3,
        total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taxonomy_t1_is_16() {
        assert_eq!(taxonomy_summary().t1, 16);
    }

    #[test]
    fn taxonomy_t2p_is_22() {
        assert_eq!(taxonomy_summary().t2p, 22);
    }

    #[test]
    fn taxonomy_t2c_is_20() {
        assert_eq!(taxonomy_summary().t2c, 20);
    }

    #[test]
    fn taxonomy_t3_at_least_90() {
        let t3 = taxonomy_summary().t3;
        assert!(t3 >= 90, "Expected 90+ T3 concepts, got {t3}");
    }

    #[test]
    fn taxonomy_total_at_least_148() {
        let total = taxonomy_summary().total;
        assert!(total >= 148, "Expected 148+ total concepts, got {total}");
    }

    #[test]
    fn who_pillars_complete() {
        let pillars = who_pillar_complexity();
        assert_eq!(pillars.len(), 4);
        assert_eq!(pillars[0].0, "Detection");
        assert_eq!(pillars[3].0, "Prevention");
    }
}
