//! # GroundsTo implementations for the unified skills crate
//!
//! This module re-uses types from sub-modules (core, loader, transfer_confidence,
//! primitive_extractor) and provides groundings for the crate-level `default_registry`
//! concept. Individual sub-module types are grounded in their respective modules,
//! but the unified crate itself contributes the SkillRegistry function.
//!
//! Since the `skills` crate re-exports types from its sub-modules, we ground
//! the key composition types that are unique to this unified crate.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

// Ground the core module types accessible via `skills::core::`
use crate::core::{
    ComplianceLevel, OutputContent, SkillChain, SkillContext, SkillError, SkillMetadata,
    SkillOutput, SkillParallel, SkillRegistry, Trigger,
};

// Ground the loader module types
use crate::loader::{LoadedSkill, SkillFrontmatter, SkillLoader};

// Ground the transfer_confidence module types
use crate::transfer_confidence::{TransferConfidence, TransferConfidenceSkill, TransferScore};

// Ground the primitive_extractor module types
use crate::primitive_extractor::{Primitive, PrimitiveExtractor, PrimitiveTier};

// ---------------------------------------------------------------------------
// core:: types
// ---------------------------------------------------------------------------

impl GroundsTo for SkillMetadata {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

impl GroundsTo for SkillContext {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

impl GroundsTo for ComplianceLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

impl GroundsTo for Trigger {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

impl GroundsTo for SkillOutput {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

impl GroundsTo for OutputContent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Recursion,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

impl GroundsTo for SkillRegistry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

impl GroundsTo for SkillChain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Recursion,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

impl GroundsTo for SkillParallel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

impl GroundsTo for SkillError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// loader:: types
// ---------------------------------------------------------------------------

impl GroundsTo for SkillFrontmatter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

impl GroundsTo for LoadedSkill {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

impl GroundsTo for SkillLoader {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// transfer_confidence:: types
// ---------------------------------------------------------------------------

impl GroundsTo for TransferConfidence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

impl GroundsTo for TransferScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

impl GroundsTo for TransferConfidenceSkill {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// primitive_extractor:: types
// ---------------------------------------------------------------------------

impl GroundsTo for PrimitiveExtractor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

impl GroundsTo for Primitive {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

impl GroundsTo for PrimitiveTier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn compliance_level_is_t1() {
        assert_eq!(ComplianceLevel::tier(), Tier::T1Universal);
    }

    #[test]
    fn skill_metadata_is_t2c() {
        assert_eq!(SkillMetadata::tier(), Tier::T2Composite);
    }

    #[test]
    fn skill_output_dominant_is_mapping() {
        let comp = SkillOutput::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn transfer_score_is_t2c() {
        assert_eq!(TransferScore::tier(), Tier::T2Composite);
    }

    #[test]
    fn primitive_extractor_dominant_is_mapping() {
        let comp = PrimitiveExtractor::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn loaded_skill_is_t2p() {
        assert_eq!(LoadedSkill::tier(), Tier::T2Primitive);
    }

    #[test]
    fn skill_chain_dominant_is_sequence() {
        let comp = SkillChain::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }
}
