//! GroundsTo implementations for all public types.
//!
//! Every public type in nexcore-transform is grounded to its
//! T1 primitive composition via the Lex Primitiva system.

use nexcore_lex_primitiva::prelude::*;

use crate::annotation::{ConceptAnnotation, ConceptOccurrence};
use crate::fidelity::FidelityReport;
use crate::ledger::{LedgerEntry, LedgerSummary, TransferLedger};
use crate::mapping::{ConceptMapping, MappingMethod, MappingTable};
use crate::plan::{ParagraphInstruction, TransformationPlan};
use crate::profile::{ConceptBridge, DomainProfile, RhetoricalRole};
use crate::segment::{Paragraph, SourceText};

// ═══════════════════════════════════════════════════════════════════════════
// SEGMENT TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Tier: T2-P | Dominant: sigma (Sequence)
impl GroundsTo for Paragraph {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // ordered text
            LexPrimitiva::Quantity, // word count + index
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.95)
    }
}

/// Tier: T2-C | Dominant: sigma (Sequence)
impl GroundsTo for SourceText {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // paragraph ordering
            LexPrimitiva::Quantity, // total words
            LexPrimitiva::Mapping,  // title -> paragraphs
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.90)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PROFILE TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Tier: T2-P | Dominant: mu (Mapping)
impl GroundsTo for ConceptBridge {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // generic -> specific
            LexPrimitiva::Quantity, // confidence
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

/// Tier: T2-P | Dominant: kappa (Comparison)
impl GroundsTo for RhetoricalRole {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // classification
            LexPrimitiva::Sum,        // variant selection
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// Tier: T2-C | Dominant: mu (Mapping)
impl GroundsTo for DomainProfile {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // vocabulary + bridges
            LexPrimitiva::Sequence,   // ordered vocabulary
            LexPrimitiva::Comparison, // rhetorical classification
            LexPrimitiva::Boundary,   // domain boundary
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ANNOTATION TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Tier: T2-P | Dominant: mu (Mapping)
impl GroundsTo for ConceptOccurrence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // term identification
            LexPrimitiva::Existence, // has_bridge flag
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// Tier: T2-C | Dominant: mu (Mapping)
impl GroundsTo for ConceptAnnotation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // concept occurrences
            LexPrimitiva::Sequence, // ordered concepts
            LexPrimitiva::Quantity, // paragraph index
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.88)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MAPPING TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Tier: T2-P | Dominant: kappa (Comparison)
impl GroundsTo for MappingMethod {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // method classification
            LexPrimitiva::Sum,        // variant selection
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// Tier: T2-C | Dominant: mu (Mapping)
impl GroundsTo for ConceptMapping {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // source -> target
            LexPrimitiva::Quantity,   // confidence
            LexPrimitiva::Comparison, // method
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.88)
    }
}

/// Tier: T2-C | Dominant: sigma (Sequence)
impl GroundsTo for MappingTable {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // ordered mappings
            LexPrimitiva::Mapping,  // individual mappings
            LexPrimitiva::Quantity, // aggregate stats
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PLAN TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Tier: T2-C | Dominant: sigma (Sequence)
impl GroundsTo for ParagraphInstruction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // ordered replacements
            LexPrimitiva::Mapping,    // concept replacements
            LexPrimitiva::Comparison, // rhetorical role
            LexPrimitiva::Quantity,   // word count
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.82)
    }
}

/// Tier: T3 | Dominant: sigma (Sequence)
impl GroundsTo for TransformationPlan {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,    // paragraph pipeline
            LexPrimitiva::Mapping,     // concept mappings
            LexPrimitiva::Comparison,  // rhetorical classification
            LexPrimitiva::Quantity,    // metrics
            LexPrimitiva::Boundary,    // domain boundaries
            LexPrimitiva::Persistence, // plan as artifact
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LEDGER TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Tier: T2-P | Dominant: mu (Mapping)
impl GroundsTo for LedgerEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // source -> target
            LexPrimitiva::Quantity, // confidence
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

/// Tier: T2-P | Dominant: N (Quantity)
impl GroundsTo for LedgerSummary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // counts + confidence
            LexPrimitiva::Sum,      // aggregation
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.92)
    }
}

/// Tier: T2-C | Dominant: sigma (Sequence)
impl GroundsTo for TransferLedger {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // ordered entries
            LexPrimitiva::Mapping,  // individual entries
            LexPrimitiva::Quantity, // summary stats
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FIDELITY TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Tier: T2-C | Dominant: kappa (Comparison)
impl GroundsTo for FidelityReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // scoring comparisons
            LexPrimitiva::Quantity,   // numeric metrics
            LexPrimitiva::Sequence,   // per-paragraph coverage
            LexPrimitiva::Mapping,    // plan reference
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_types_have_dominant() {
        assert!(Paragraph::dominant_primitive().is_some());
        assert!(SourceText::dominant_primitive().is_some());
        assert!(ConceptBridge::dominant_primitive().is_some());
        assert!(RhetoricalRole::dominant_primitive().is_some());
        assert!(DomainProfile::dominant_primitive().is_some());
        assert!(ConceptOccurrence::dominant_primitive().is_some());
        assert!(ConceptAnnotation::dominant_primitive().is_some());
        assert!(MappingMethod::dominant_primitive().is_some());
        assert!(ConceptMapping::dominant_primitive().is_some());
        assert!(MappingTable::dominant_primitive().is_some());
        assert!(ParagraphInstruction::dominant_primitive().is_some());
        assert!(TransformationPlan::dominant_primitive().is_some());
        assert!(LedgerEntry::dominant_primitive().is_some());
        assert!(LedgerSummary::dominant_primitive().is_some());
        assert!(TransferLedger::dominant_primitive().is_some());
        assert!(FidelityReport::dominant_primitive().is_some());
    }

    #[test]
    fn test_tier_classifications() {
        // T2-P types (2-3 unique primitives)
        assert!(matches!(Paragraph::tier(), Tier::T2Primitive));
        assert!(matches!(ConceptBridge::tier(), Tier::T2Primitive));
        assert!(matches!(RhetoricalRole::tier(), Tier::T2Primitive));
        assert!(matches!(ConceptOccurrence::tier(), Tier::T2Primitive));
        assert!(matches!(MappingMethod::tier(), Tier::T2Primitive));
        assert!(matches!(LedgerEntry::tier(), Tier::T2Primitive));
        assert!(matches!(LedgerSummary::tier(), Tier::T2Primitive));
        assert!(matches!(SourceText::tier(), Tier::T2Primitive)); // 3 primitives
        assert!(matches!(ConceptAnnotation::tier(), Tier::T2Primitive)); // 3 primitives
        assert!(matches!(ConceptMapping::tier(), Tier::T2Primitive)); // 3 primitives
        assert!(matches!(MappingTable::tier(), Tier::T2Primitive)); // 3 primitives
        assert!(matches!(TransferLedger::tier(), Tier::T2Primitive)); // 3 primitives

        // T2-C types (4-5 unique primitives)
        assert!(matches!(DomainProfile::tier(), Tier::T2Composite));
        assert!(matches!(ParagraphInstruction::tier(), Tier::T2Composite));
        assert!(matches!(FidelityReport::tier(), Tier::T2Composite));

        // T3 types (6+ unique primitives)
        assert!(matches!(TransformationPlan::tier(), Tier::T3DomainSpecific));
    }

    #[test]
    fn test_dominant_primitives_match_spec() {
        assert_eq!(
            Paragraph::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            ConceptBridge::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        assert_eq!(
            RhetoricalRole::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
        assert_eq!(
            DomainProfile::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        assert_eq!(
            MappingTable::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            TransformationPlan::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            FidelityReport::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_confidence_values_positive() {
        let types_compositions = vec![
            Paragraph::primitive_composition(),
            SourceText::primitive_composition(),
            ConceptBridge::primitive_composition(),
            DomainProfile::primitive_composition(),
            ConceptAnnotation::primitive_composition(),
            ConceptMapping::primitive_composition(),
            MappingTable::primitive_composition(),
            TransformationPlan::primitive_composition(),
            TransferLedger::primitive_composition(),
            FidelityReport::primitive_composition(),
        ];
        for comp in &types_compositions {
            assert!(comp.confidence > 0.0, "confidence should be positive");
            assert!(comp.confidence <= 1.0, "confidence should be <= 1.0");
        }
    }
}
