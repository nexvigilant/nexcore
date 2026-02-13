//! # Lex Primitiva Grounding
//!
//! GroundsTo implementations for all clearance types.
//! Maps each type to its T1 primitive composition.

use crate::access::AccessMode;
use crate::audit::{AuditAction, ClearanceAudit, ClearanceEntry};
use crate::config::ClearanceConfig;
use crate::error::ClearanceError;
use crate::gate::{ClearanceGate, GateResult};
use crate::level::ClassificationLevel;
use crate::policy::ClearancePolicy;
use crate::priority::ClearancePriority;
use crate::tag::{ClassificationTag, TagTarget};
use crate::validator::{CrossBoundaryValidator, ValidationResult};
use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

// ── T1 Pure Primitives ──────────────────────────────────────────────

/// ClassificationLevel: T1, Dominant ς State
impl GroundsTo for ClassificationLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

/// AccessMode: T1, Dominant ς State
impl GroundsTo for AccessMode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

/// ClearancePriority: T1, Dominant κ Comparison
impl GroundsTo for ClearancePriority {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 1.0)
    }
}

/// AuditAction: T1, Dominant σ Sequence
impl GroundsTo for AuditAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Sequence, 1.0)
    }
}

// ── T2-P Cross-Domain Primitives ────────────────────────────────────

/// TagTarget: T2-P, Dominant μ Mapping (μ + λ)
impl GroundsTo for TagTarget {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Location])
            .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// ClearancePolicy: T2-P, Dominant ς State (ς + ∂ + κ)
impl GroundsTo for ClearancePolicy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// ClearanceEntry: T2-P, Dominant π Persistence (π + σ)
impl GroundsTo for ClearanceEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Persistence, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Persistence, 0.85)
    }
}

/// ClearanceError: T2-P, Dominant ∂ Boundary (∂ + Σ)
impl GroundsTo for ClearanceError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// GateResult: T2-P, Dominant ∂ Boundary (∂ + Σ)
impl GroundsTo for GateResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// CrossBoundaryValidator: T2-P, Dominant ∂ Boundary (∂ + κ)
impl GroundsTo for CrossBoundaryValidator {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// ValidationResult: T2-P, Dominant ∂ Boundary (∂ + Σ)
impl GroundsTo for ValidationResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ── T2-C Cross-Domain Composites ────────────────────────────────────

/// ClassificationTag: T2-C, Dominant μ Mapping (μ + ς + π)
impl GroundsTo for ClassificationTag {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// ClearanceConfig: T2-C, Dominant ς State (ς + μ + N + ∂)
impl GroundsTo for ClearanceConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::State, 0.75)
    }
}

// ── T3 Domain-Specific ──────────────────────────────────────────────

/// ClearanceAudit: T3, Dominant π Persistence (π + σ + N + κ)
impl GroundsTo for ClearanceAudit {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.70)
    }
}

/// ClearanceGate: T3, Dominant ∂ Boundary (∂ + κ + ς + σ + → + N)
impl GroundsTo for ClearanceGate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
            LexPrimitiva::Causality,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.65)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn classification_level_is_t1() {
        assert_eq!(ClassificationLevel::tier(), Tier::T1Universal);
        assert_eq!(
            ClassificationLevel::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn access_mode_is_t1() {
        assert_eq!(AccessMode::tier(), Tier::T1Universal);
        assert_eq!(AccessMode::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn clearance_priority_is_t1() {
        assert_eq!(ClearancePriority::tier(), Tier::T1Universal);
        assert_eq!(
            ClearancePriority::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn audit_action_is_t1() {
        assert_eq!(AuditAction::tier(), Tier::T1Universal);
        assert_eq!(
            AuditAction::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn tag_target_is_t2p() {
        assert_eq!(TagTarget::tier(), Tier::T2Primitive);
        assert_eq!(TagTarget::dominant_primitive(), Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn clearance_policy_is_t2p() {
        let tier = ClearancePolicy::tier();
        assert!(tier == Tier::T2Primitive || tier == Tier::T2Composite);
        assert_eq!(
            ClearancePolicy::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn clearance_entry_is_t2p() {
        assert_eq!(ClearanceEntry::tier(), Tier::T2Primitive);
        assert_eq!(
            ClearanceEntry::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn clearance_error_is_t2p() {
        assert_eq!(ClearanceError::tier(), Tier::T2Primitive);
        assert_eq!(
            ClearanceError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn gate_result_is_t2p() {
        assert_eq!(GateResult::tier(), Tier::T2Primitive);
        assert_eq!(
            GateResult::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn cross_boundary_validator_is_t2p() {
        assert_eq!(CrossBoundaryValidator::tier(), Tier::T2Primitive);
        assert_eq!(
            CrossBoundaryValidator::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn validation_result_is_t2p() {
        assert_eq!(ValidationResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn classification_tag_is_t2c() {
        let tier = ClassificationTag::tier();
        assert!(tier == Tier::T2Primitive || tier == Tier::T2Composite);
        assert_eq!(
            ClassificationTag::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn clearance_config_is_t2c() {
        let tier = ClearanceConfig::tier();
        assert!(tier == Tier::T2Composite || tier == Tier::T3DomainSpecific);
        assert_eq!(
            ClearanceConfig::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn clearance_audit_is_t3() {
        let tier = ClearanceAudit::tier();
        assert!(tier == Tier::T2Composite || tier == Tier::T3DomainSpecific);
        assert_eq!(
            ClearanceAudit::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn clearance_gate_is_t3() {
        assert_eq!(ClearanceGate::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            ClearanceGate::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn all_types_have_dominant() {
        assert!(ClassificationLevel::dominant_primitive().is_some());
        assert!(AccessMode::dominant_primitive().is_some());
        assert!(ClearancePriority::dominant_primitive().is_some());
        assert!(AuditAction::dominant_primitive().is_some());
        assert!(TagTarget::dominant_primitive().is_some());
        assert!(ClearancePolicy::dominant_primitive().is_some());
        assert!(ClearanceEntry::dominant_primitive().is_some());
        assert!(ClearanceError::dominant_primitive().is_some());
        assert!(GateResult::dominant_primitive().is_some());
        assert!(CrossBoundaryValidator::dominant_primitive().is_some());
        assert!(ValidationResult::dominant_primitive().is_some());
        assert!(ClassificationTag::dominant_primitive().is_some());
        assert!(ClearanceConfig::dominant_primitive().is_some());
        assert!(ClearanceAudit::dominant_primitive().is_some());
        assert!(ClearanceGate::dominant_primitive().is_some());
    }
}
