//! GroundsTo implementations for ORGANIZE pipeline types.
//!
//! Maps each pipeline type to its T1 primitive composition from the Lex Primitiva.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::assign::{AssignedInventory, Assignment};
use crate::config::{FileOp, GroupRule, NamingConfig, OrganizeConfig, RankingConfig};
use crate::enforce::{DriftReport, OrganizeState};
use crate::group::{Group, GroupedInventory};
use crate::integrate::{ExecutedOp, IntegrationPlan};
use crate::name::{NamedInventory, RenameOp};
use crate::observe::{EntryMeta, Inventory};
use crate::pipeline::OrganizeResult2;
use crate::rank::{RankedEntry, RankedInventory, ScoreBreakdown};
use crate::zero_out::{CleanupReport, DuplicateGroup};

// ============================================================================
// Step 1: Observe (∃ Existence)
// ============================================================================

impl GroundsTo for EntryMeta {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Existence, LexPrimitiva::Location])
            .with_dominant(LexPrimitiva::Existence, 0.9)
    }
}

impl GroundsTo for Inventory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

// ============================================================================
// Step 2: Rank (κ Comparison)
// ============================================================================

impl GroundsTo for ScoreBreakdown {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Comparison, 0.9)
    }
}

impl GroundsTo for RankedEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Existence,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

impl GroundsTo for RankedInventory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Sequence,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.8)
    }
}

// ============================================================================
// Step 3: Group (μ Mapping)
// ============================================================================

impl GroundsTo for Group {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

impl GroundsTo for GroupedInventory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.8)
    }
}

// ============================================================================
// Step 4: Assign (→ Causality)
// ============================================================================

impl GroundsTo for Assignment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

impl GroundsTo for AssignedInventory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.8)
    }
}

// ============================================================================
// Step 5: Name (∂ Boundary)
// ============================================================================

impl GroundsTo for RenameOp {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

impl GroundsTo for NamedInventory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.8)
    }
}

// ============================================================================
// Step 6: Integrate (Σ Sum)
// ============================================================================

impl GroundsTo for ExecutedOp {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,
            LexPrimitiva::Causality,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for IntegrationPlan {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Sum, 0.8)
    }
}

// ============================================================================
// Step 7: Zero-out (∅ Void)
// ============================================================================

impl GroundsTo for DuplicateGroup {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Void, 0.85)
    }
}

impl GroundsTo for CleanupReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,
            LexPrimitiva::Sum,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Void, 0.8)
    }
}

// ============================================================================
// Step 8: Enforce (ς State)
// ============================================================================

impl GroundsTo for OrganizeState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Persistence,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

impl GroundsTo for DriftReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Comparison,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::State, 0.8)
    }
}

// ============================================================================
// Config Types
// ============================================================================

impl GroundsTo for OrganizeConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.8)
    }
}

impl GroundsTo for RankingConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Comparison, 0.9)
    }
}

impl GroundsTo for GroupRule {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Mapping, 0.9)
    }
}

impl GroundsTo for FileOp {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Causality])
            .with_dominant(LexPrimitiva::Causality, 1.0)
    }
}

impl GroundsTo for NamingConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Boundary, 0.9)
    }
}

// ============================================================================
// Pipeline Result (T3)
// ============================================================================

impl GroundsTo for OrganizeResult2 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
            LexPrimitiva::Sum,
            LexPrimitiva::Void,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn test_entry_meta_grounds_to_existence() {
        assert_eq!(
            EntryMeta::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
    }

    #[test]
    fn test_ranked_entry_grounds_to_comparison() {
        assert_eq!(
            RankedEntry::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_group_grounds_to_mapping() {
        assert_eq!(Group::dominant_primitive(), Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn test_assignment_grounds_to_causality() {
        assert_eq!(
            Assignment::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn test_rename_op_grounds_to_boundary() {
        assert_eq!(RenameOp::dominant_primitive(), Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn test_executed_op_grounds_to_sum() {
        assert_eq!(ExecutedOp::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn test_duplicate_group_grounds_to_void() {
        assert_eq!(
            DuplicateGroup::dominant_primitive(),
            Some(LexPrimitiva::Void)
        );
    }

    #[test]
    fn test_organize_state_grounds_to_state() {
        assert_eq!(
            OrganizeState::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn test_action_is_pure_causality() {
        assert!(FileOp::is_pure_primitive());
        assert_eq!(FileOp::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_organize_result_is_t3() {
        let tier = OrganizeResult2::tier();
        assert_eq!(tier, Tier::T3DomainSpecific);
    }

    #[test]
    fn test_all_8_steps_have_unique_dominants() {
        let dominants = vec![
            EntryMeta::dominant_primitive(),      // ∃
            RankedEntry::dominant_primitive(),    // κ
            Group::dominant_primitive(),          // μ
            Assignment::dominant_primitive(),     // →
            RenameOp::dominant_primitive(),       // ∂
            ExecutedOp::dominant_primitive(),     // Σ
            DuplicateGroup::dominant_primitive(), // ∅
            OrganizeState::dominant_primitive(),  // ς
        ];

        // All should be Some
        assert!(dominants.iter().all(|d| d.is_some()));

        // All should be unique
        let mut seen = std::collections::HashSet::new();
        for d in &dominants {
            if let Some(p) = d {
                assert!(seen.insert(format!("{p:?}")), "duplicate dominant: {p:?}");
            }
        }
        assert_eq!(seen.len(), 8);
    }
}
