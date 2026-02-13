//! # GroundsTo implementations for nexcore-integumentary types

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    AuthResult, CoolingAction, IntegumentaryError, SensorKind, SensorReading, Shield,
    SkinCondition, WoundRepair,
    claude_code::{
        IntegumentaryHealth, PermissionCascade, PermissionDecision, PermissionRule, RiskLevel,
        RuleOrigin, SandboxLayer, Scar, ScarringMechanism, ScopedSetting, SettingsPrecedence,
        SettingsScope,
    },
};

impl GroundsTo for AuthResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // partial -- auth is a boundary gate
            LexPrimitiva::Existence, // exists -- user identity verification
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

impl GroundsTo for SensorKind {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- 4-variant sensor classification
            LexPrimitiva::Comparison, // kappa -- sensors compare against thresholds
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for SensorReading {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- numeric reading value
            LexPrimitiva::Comparison, // kappa -- reading vs normal range
            LexPrimitiva::Mapping,    // mu -- raw reading -> status
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.70)
    }
}

impl GroundsTo for SkinCondition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- current skin state
            LexPrimitiva::Product,  // times -- composite of all sensor readings
            LexPrimitiva::Boundary, // partial -- skin IS the boundary
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

impl GroundsTo for Shield {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- shield IS boundary protection
            LexPrimitiva::Product,  // times -- composite of auth + validation + monitoring
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

impl GroundsTo for WoundRepair {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- wound -> repair action
            LexPrimitiva::Boundary,  // partial -- restoring boundary integrity
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

impl GroundsTo for CoolingAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- heat -> cooling response
            LexPrimitiva::Quantity,  // N -- load reduction amount
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

impl GroundsTo for IntegumentaryError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- boundary system failure
            LexPrimitiva::Sum,      // Sigma -- error variants
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ============================================================================
// Claude Code Infrastructure Types (§2 alignment)
// ============================================================================

impl GroundsTo for PermissionDecision {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- 3-variant decision (Deny/Ask/Allow)
            LexPrimitiva::Boundary, // partial -- permission IS boundary control
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for PermissionCascade {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- the epidermis IS the boundary
            LexPrimitiva::Sequence,   // sigma -- ordered evaluation (deny→ask→allow)
            LexPrimitiva::Comparison, // kappa -- pattern matching against requests
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.70)
    }
}

impl GroundsTo for SettingsScope {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- 5-variant scope classification
            LexPrimitiva::Location, // lambda -- where in the dermis stack
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

impl GroundsTo for ScopedSetting {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- key→value mapping
            LexPrimitiva::Location, // lambda -- scoped to a dermis layer
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

impl GroundsTo for SettingsPrecedence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered precedence evaluation
            LexPrimitiva::Mapping,  // mu -- key→value resolution
            LexPrimitiva::Boundary, // partial -- dermis as inspection layer
            LexPrimitiva::Location, // lambda -- scope determines location
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.65)
    }
}

impl GroundsTo for SandboxLayer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- hypodermis IS insulation boundary
            LexPrimitiva::State,    // varsigma -- active/inactive state
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

impl GroundsTo for RiskLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- 3-variant risk classification
            LexPrimitiva::Comparison, // kappa -- threshold comparison
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for Scar {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,       // arrow -- incident→deny_rule causal chain
            LexPrimitiva::Persistence,     // pi -- scars are permanent
            LexPrimitiva::Irreversibility, // proportional -- scarring is one-way
        ])
        .with_dominant(LexPrimitiva::Causality, 0.70)
    }
}

impl GroundsTo for ScarringMechanism {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,        // sigma -- accumulated over time
            LexPrimitiva::Irreversibility, // proportional -- scars don't heal
            LexPrimitiva::Boundary,        // partial -- reinforces the boundary
            LexPrimitiva::Persistence,     // pi -- persisted across sessions
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.60)
    }
}

impl GroundsTo for PermissionRule {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // mu -- pattern maps to decision
            LexPrimitiva::Boundary,  // partial -- permission boundary enforcement
            LexPrimitiva::Causality, // arrow -- rule origin causes decision behavior
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.75)
    }
}

impl GroundsTo for RuleOrigin {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,             // sigma -- 3-variant enum (Manual/Adaptive/Managed)
            LexPrimitiva::Irreversibility, // proportional -- adaptive scars are permanent
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for IntegumentaryHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- current health state
            LexPrimitiva::Comparison, // kappa -- health checks against expectations
            LexPrimitiva::Boundary,   // partial -- health of boundary system
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn auth_result_is_boundary() {
        assert_eq!(
            AuthResult::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(AuthResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn sensor_kind_is_sum() {
        assert_eq!(SensorKind::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn sensor_reading_is_quantity() {
        assert_eq!(
            SensorReading::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn skin_condition_is_state() {
        assert_eq!(
            SkinCondition::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn shield_is_boundary() {
        assert_eq!(Shield::dominant_primitive(), Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn wound_repair_is_causality() {
        assert_eq!(
            WoundRepair::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn all_types_have_dominant() {
        assert!(AuthResult::dominant_primitive().is_some());
        assert!(SensorKind::dominant_primitive().is_some());
        assert!(SensorReading::dominant_primitive().is_some());
        assert!(SkinCondition::dominant_primitive().is_some());
        assert!(Shield::dominant_primitive().is_some());
        assert!(WoundRepair::dominant_primitive().is_some());
        assert!(CoolingAction::dominant_primitive().is_some());
        assert!(IntegumentaryError::dominant_primitive().is_some());
    }

    // ====================================================================
    // Claude Code infrastructure type grounding tests
    // ====================================================================

    #[test]
    fn permission_decision_is_sum() {
        assert_eq!(
            PermissionDecision::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(PermissionDecision::tier(), Tier::T2Primitive);
    }

    #[test]
    fn permission_cascade_is_boundary() {
        assert_eq!(
            PermissionCascade::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn settings_scope_is_sum() {
        assert_eq!(SettingsScope::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn settings_precedence_is_sequence() {
        assert_eq!(
            SettingsPrecedence::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        // 4 primitives → T2-C
        assert_eq!(SettingsPrecedence::tier(), Tier::T2Composite);
    }

    #[test]
    fn sandbox_is_boundary() {
        assert_eq!(
            SandboxLayer::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn scar_is_causality() {
        assert_eq!(Scar::dominant_primitive(), Some(LexPrimitiva::Causality));
    }

    #[test]
    fn scarring_mechanism_is_sequence() {
        assert_eq!(
            ScarringMechanism::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        // 4 primitives → T2-C
        assert_eq!(ScarringMechanism::tier(), Tier::T2Composite);
    }

    #[test]
    fn integumentary_health_is_state() {
        assert_eq!(
            IntegumentaryHealth::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn permission_rule_is_mapping() {
        assert_eq!(
            PermissionRule::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        assert_eq!(PermissionRule::tier(), Tier::T2Primitive);
    }

    #[test]
    fn rule_origin_is_sum() {
        assert_eq!(RuleOrigin::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(RuleOrigin::tier(), Tier::T2Primitive);
    }

    #[test]
    fn all_claude_code_types_have_dominant() {
        assert!(PermissionRule::dominant_primitive().is_some());
        assert!(PermissionDecision::dominant_primitive().is_some());
        assert!(RuleOrigin::dominant_primitive().is_some());
        assert!(PermissionCascade::dominant_primitive().is_some());
        assert!(SettingsScope::dominant_primitive().is_some());
        assert!(ScopedSetting::dominant_primitive().is_some());
        assert!(SettingsPrecedence::dominant_primitive().is_some());
        assert!(SandboxLayer::dominant_primitive().is_some());
        assert!(RiskLevel::dominant_primitive().is_some());
        assert!(Scar::dominant_primitive().is_some());
        assert!(ScarringMechanism::dominant_primitive().is_some());
        assert!(IntegumentaryHealth::dominant_primitive().is_some());
    }
}
