//! # Lex Primitiva Grounding for Ghost Types
//!
//! 16 `GroundsTo` implementations connecting all public ghost types
//! to the T1 primitive bedrock.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::audit::{RedactionAudit, RedactionEntry};
use crate::boundary::{AnonymizationBoundary, BoundaryCheckResult};
use crate::config::{CategoryPolicy, DataCategory, GhostConfig};
use crate::error::GhostError;
use crate::mode::GhostMode;
use crate::priority::DataPrivacyPriority;
use crate::pseudonymize::PseudonymHandle;
use crate::scrubber::{PiiFieldPattern, ScrubAction};
use crate::sensor::{GhostSignal, PiiLeakPattern};

// ── T1: GhostMode (ς State) ───────────────────────────────────────

impl GroundsTo for GhostMode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

// ── T2-P: DataCategory (μ Mapping) ────────────────────────────────

impl GroundsTo for DataCategory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::Mapping, 1.0)
    }
}

// ── T2-C: CategoryPolicy (μ + ς) ──────────────────────────────────

impl GroundsTo for CategoryPolicy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ── T3: GhostConfig (ς + μ + N) ───────────────────────────────────

impl GroundsTo for GhostConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::State, 0.8)
    }
}

// ── T2-P: PseudonymHandle (μ Mapping) ─────────────────────────────

impl GroundsTo for PseudonymHandle {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Mapping, 0.9)
    }
}

// ── T2-C: AnonymizationBoundary (∂ + N) ───────────────────────────

impl GroundsTo for AnonymizationBoundary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Boundary, 0.9)
    }
}

// ── T2-P: BoundaryCheckResult (∂ + κ) ─────────────────────────────

impl GroundsTo for BoundaryCheckResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ── T2-C: RedactionEntry (π + σ) ──────────────────────────────────

impl GroundsTo for RedactionEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Persistence, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Persistence, 0.9)
    }
}

// ── T3: RedactionAudit (π + σ + N) ────────────────────────────────

impl GroundsTo for RedactionAudit {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.85)
    }
}

// ── T2-P: ScrubAction (μ Mapping) ─────────────────────────────────

impl GroundsTo for ScrubAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::Mapping, 1.0)
    }
}

// ── T2-P: PiiFieldPattern (∂ Boundary) ────────────────────────────

impl GroundsTo for PiiFieldPattern {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 1.0)
    }
}

// ── T2-P: PiiLeakPattern (∂ Boundary) ─────────────────────────────

impl GroundsTo for PiiLeakPattern {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 1.0)
    }
}

// ── T3: GhostSignal (∂ + σ + ς) ───────────────────────────────────

impl GroundsTo for GhostSignal {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Sequence,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.8)
    }
}

// ── T2-P: DataPrivacyPriority (κ Comparison) ──────────────────────

impl GroundsTo for DataPrivacyPriority {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 1.0)
    }
}

// ── T2-C: GhostError (∂ + Σ) ──────────────────────────────────────

impl GroundsTo for GhostError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.9)
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn ghost_mode_is_t1() {
        assert_eq!(GhostMode::tier(), Tier::T1Universal);
        assert_eq!(GhostMode::dominant_primitive(), Some(LexPrimitiva::State));
        assert!(GhostMode::is_pure_primitive());
    }

    #[test]
    fn data_category_is_t1() {
        assert_eq!(DataCategory::tier(), Tier::T1Universal);
        assert_eq!(
            DataCategory::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn category_policy_is_t2p() {
        assert_eq!(CategoryPolicy::tier(), Tier::T2Primitive);
        assert_eq!(
            CategoryPolicy::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn ghost_config_is_t2p() {
        assert_eq!(GhostConfig::tier(), Tier::T2Primitive);
        assert_eq!(GhostConfig::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn pseudonym_handle_is_t2p() {
        assert_eq!(PseudonymHandle::tier(), Tier::T2Primitive);
        assert_eq!(
            PseudonymHandle::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn anonymization_boundary_is_t2p() {
        assert_eq!(AnonymizationBoundary::tier(), Tier::T2Primitive);
        assert_eq!(
            AnonymizationBoundary::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn boundary_check_result_is_t2p() {
        assert_eq!(BoundaryCheckResult::tier(), Tier::T2Primitive);
        assert_eq!(
            BoundaryCheckResult::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn redaction_entry_is_t2p() {
        assert_eq!(RedactionEntry::tier(), Tier::T2Primitive);
        assert_eq!(
            RedactionEntry::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn redaction_audit_is_t2p() {
        assert_eq!(RedactionAudit::tier(), Tier::T2Primitive);
        assert_eq!(
            RedactionAudit::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn scrub_action_is_t1() {
        assert_eq!(ScrubAction::tier(), Tier::T1Universal);
        assert_eq!(
            ScrubAction::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn pii_field_pattern_is_t1() {
        assert_eq!(PiiFieldPattern::tier(), Tier::T1Universal);
        assert_eq!(
            PiiFieldPattern::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn pii_leak_pattern_is_t1() {
        assert_eq!(PiiLeakPattern::tier(), Tier::T1Universal);
        assert_eq!(
            PiiLeakPattern::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn ghost_signal_is_t2p() {
        assert_eq!(GhostSignal::tier(), Tier::T2Primitive);
        assert_eq!(
            GhostSignal::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn data_privacy_priority_is_t1() {
        assert_eq!(DataPrivacyPriority::tier(), Tier::T1Universal);
        assert_eq!(
            DataPrivacyPriority::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn ghost_error_is_t2p() {
        assert_eq!(GhostError::tier(), Tier::T2Primitive);
        assert_eq!(
            GhostError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn all_ghost_types_have_dominant() {
        // Verify no type returns None for dominant
        assert!(GhostMode::dominant_primitive().is_some());
        assert!(DataCategory::dominant_primitive().is_some());
        assert!(CategoryPolicy::dominant_primitive().is_some());
        assert!(GhostConfig::dominant_primitive().is_some());
        assert!(PseudonymHandle::dominant_primitive().is_some());
        assert!(AnonymizationBoundary::dominant_primitive().is_some());
        assert!(BoundaryCheckResult::dominant_primitive().is_some());
        assert!(RedactionEntry::dominant_primitive().is_some());
        assert!(RedactionAudit::dominant_primitive().is_some());
        assert!(ScrubAction::dominant_primitive().is_some());
        assert!(PiiFieldPattern::dominant_primitive().is_some());
        assert!(PiiLeakPattern::dominant_primitive().is_some());
        assert!(GhostSignal::dominant_primitive().is_some());
        assert!(DataPrivacyPriority::dominant_primitive().is_some());
        assert!(GhostError::dominant_primitive().is_some());
    }
}
