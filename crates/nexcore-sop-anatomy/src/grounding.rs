//! # GroundsTo implementations for nexcore-sop-anatomy types
//!
//! Root primitives: σ (Sequence) + μ (Mapping)
//! The crate is fundamentally about mapping ordered governance structures across domains.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::mapping::{
    AnatomicalAnalog, BioCrateWiring, BodySystem, CodeStructure, CoverageReport, Domain, Priority,
    SectionMapping, SopSection,
};
use crate::reactor::{ChemOp, ChemistryOperation, IronmanPhase};
use crate::transfer::TransferResult;

// ─── Mapping types ─────────────────────────────────────────────────────────

/// SopSection: T2-P (Σ), dominant Σ
///
/// 18-variant sum type representing governance sections.
impl GroundsTo for SopSection {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

/// Priority: T2-P (κ), dominant κ
impl GroundsTo for Priority {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// BodySystem: T2-P (Σ), dominant Σ
impl GroundsTo for BodySystem {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 0.95)
    }
}

/// Domain: T2-P (Σ), dominant Σ
impl GroundsTo for Domain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 0.95)
    }
}

/// AnatomicalAnalog: T3 (μ · κ), dominant μ
impl GroundsTo for AnatomicalAnalog {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// CodeStructure: T3 (μ · ∂), dominant μ
impl GroundsTo for CodeStructure {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// BioCrateWiring: T3 (μ · ∃), dominant μ
impl GroundsTo for BioCrateWiring {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Existence])
            .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// SectionMapping: T3 (σ · μ), dominant σ
impl GroundsTo for SectionMapping {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::Sequence, 0.75)
    }
}

/// CoverageReport: T3 (Σ · N), dominant Σ
impl GroundsTo for CoverageReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

// ─── Reactor types ─────────────────────────────────────────────────────────

/// ChemOp: T2-P (Σ), dominant Σ
impl GroundsTo for ChemOp {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Causality])
            .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

/// ChemistryOperation: T3 (→ · μ), dominant →
impl GroundsTo for ChemistryOperation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Causality, LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

/// IronmanPhase: T2-P (σ), dominant σ
impl GroundsTo for IronmanPhase {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

// ─── Transfer types ────────────────────────────────────────────────────────

/// TransferResult: T3 (μ · κ · →), dominant μ
impl GroundsTo for TransferResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.75)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sop_section_grounding() {
        let comp = SopSection::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn ironman_phase_grounding() {
        let comp = IronmanPhase::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn transfer_result_grounding() {
        let comp = TransferResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }
}
