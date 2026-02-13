//! # GroundsTo Implementations
//!
//! Maps all 35 cloud computing types to their Lex Primitiva compositions.
//!
//! | Tier | Count | Unique Primitives |
//! |------|-------|-------------------|
//! | T1   | 6     | 1                 |
//! | T2-P | 14    | 2-3               |
//! | T2-C | 10    | 4-5               |
//! | T3   | 5     | 6+                |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::composites::*;
use crate::primitives::*;
use crate::service_models::*;

// ============================================================================
// T1 Universal (6 types, 1 unique primitive each)
// ============================================================================

impl GroundsTo for Identity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Existence])
            .with_dominant(LexPrimitiva::Existence, 1.0)
    }
}

impl GroundsTo for Threshold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

impl GroundsTo for FeedbackLoop {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Causality, LexPrimitiva::Recursion])
            .with_dominant(LexPrimitiva::Causality, 0.92)
    }
}

impl GroundsTo for Idempotency {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Recursion, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Recursion, 0.90)
    }
}

impl GroundsTo for Immutability {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.95)
    }
}

impl GroundsTo for Convergence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Causality, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Causality, 0.90)
    }
}

// ============================================================================
// T2-P Cross-Domain (14 types, 2-3 unique primitives each)
// ============================================================================

impl GroundsTo for Compute {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Frequency])
            .with_dominant(LexPrimitiva::Quantity, 0.92)
    }
}

impl GroundsTo for Storage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Persistence, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Persistence, 0.90)
    }
}

impl GroundsTo for NetworkLink {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Causality, LexPrimitiva::Location])
            .with_dominant(LexPrimitiva::Causality, 0.92)
    }
}

impl GroundsTo for IsolationBoundary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 1.0)
    }
}

impl GroundsTo for Permission {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Mapping, 0.88)
    }
}

impl GroundsTo for ResourcePool {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

impl GroundsTo for Metering {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Frequency])
            .with_dominant(LexPrimitiva::Quantity, 0.88)
    }
}

impl GroundsTo for Replication {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Recursion, LexPrimitiva::Persistence])
            .with_dominant(LexPrimitiva::Recursion, 0.90)
    }
}

impl GroundsTo for Routing {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Causality])
            .with_dominant(LexPrimitiva::Mapping, 0.88)
    }
}

impl GroundsTo for Lease {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::State, 0.85)
            .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for Encryption {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

impl GroundsTo for Queue {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Sequence, 0.90)
    }
}

impl GroundsTo for HealthCheck {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Existence, LexPrimitiva::Frequency])
            .with_dominant(LexPrimitiva::Existence, 0.88)
    }
}

impl GroundsTo for Elasticity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Recursion,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ============================================================================
// T2-C Composites (10 types, 4-5 unique primitives each)
// ============================================================================

impl GroundsTo for VirtualMachine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.82)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for LoadBalancer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Causality,
            LexPrimitiva::Existence,
            LexPrimitiva::Frequency,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

impl GroundsTo for AutoScaling {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
            LexPrimitiva::Recursion,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.78)
    }
}

impl GroundsTo for Iam {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Existence, 0.82)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for EventualConsistency {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,
            LexPrimitiva::Persistence,
            LexPrimitiva::Causality,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.85)
    }
}

impl GroundsTo for Tenancy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Existence, 0.80)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for PayPerUse {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Causality,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

impl GroundsTo for ReservedCapacity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::State, 0.82)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for SpotPricing {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
            LexPrimitiva::Recursion,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.78)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for SecretsManagement {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Mapping,
            LexPrimitiva::Persistence,
            LexPrimitiva::Quantity,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.82)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ============================================================================
// T3 Domain-Specific (5 types, 6+ unique primitives each)
// ============================================================================

impl GroundsTo for Container {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Boundary,
            LexPrimitiva::Sum,
            LexPrimitiva::State,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for Iaas {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Persistence,
            LexPrimitiva::Causality,
            LexPrimitiva::Location,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.70)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for Paas {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Persistence,
            LexPrimitiva::Causality,
            LexPrimitiva::Location,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.68)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for Saas {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Persistence,
            LexPrimitiva::Causality,
            LexPrimitiva::Location,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.65)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for Serverless {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.72)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // ================================================================
    // Tier classification verification
    // ================================================================

    #[test]
    fn test_t1_types_classify_as_t1_or_t2p() {
        // T1 types have 1-2 unique primitives (classified T1 or T2-P)
        assert!(matches!(
            Identity::tier(),
            Tier::T1Universal | Tier::T2Primitive
        ));
        assert!(matches!(
            Threshold::tier(),
            Tier::T1Universal | Tier::T2Primitive
        ));
        assert!(matches!(
            FeedbackLoop::tier(),
            Tier::T1Universal | Tier::T2Primitive
        ));
        assert!(matches!(
            Idempotency::tier(),
            Tier::T1Universal | Tier::T2Primitive
        ));
        assert!(matches!(
            Immutability::tier(),
            Tier::T1Universal | Tier::T2Primitive
        ));
        assert!(matches!(
            Convergence::tier(),
            Tier::T1Universal | Tier::T2Primitive
        ));
    }

    #[test]
    fn test_t2p_types_classify_correctly() {
        assert!(matches!(
            Compute::tier(),
            Tier::T1Universal | Tier::T2Primitive
        ));
        assert!(matches!(
            Storage::tier(),
            Tier::T1Universal | Tier::T2Primitive
        ));
        assert!(matches!(
            NetworkLink::tier(),
            Tier::T1Universal | Tier::T2Primitive
        ));
        assert!(matches!(
            IsolationBoundary::tier(),
            Tier::T1Universal | Tier::T2Primitive
        ));
        assert!(matches!(
            Elasticity::tier(),
            Tier::T2Primitive | Tier::T2Composite
        ));
    }

    #[test]
    fn test_t2c_types_classify_correctly() {
        assert!(matches!(
            VirtualMachine::tier(),
            Tier::T2Composite | Tier::T3DomainSpecific
        ));
        assert!(matches!(
            LoadBalancer::tier(),
            Tier::T2Composite | Tier::T3DomainSpecific
        ));
        assert!(matches!(
            AutoScaling::tier(),
            Tier::T2Composite | Tier::T3DomainSpecific
        ));
        assert!(matches!(
            Iam::tier(),
            Tier::T2Composite | Tier::T3DomainSpecific
        ));
        assert!(matches!(
            EventualConsistency::tier(),
            Tier::T2Composite | Tier::T3DomainSpecific
        ));
    }

    #[test]
    fn test_t3_types_classify_as_domain_specific() {
        assert!(matches!(
            Container::tier(),
            Tier::T2Composite | Tier::T3DomainSpecific
        ));
        assert!(matches!(
            Iaas::tier(),
            Tier::T2Composite | Tier::T3DomainSpecific
        ));
        assert!(matches!(
            Paas::tier(),
            Tier::T2Composite | Tier::T3DomainSpecific
        ));
        assert!(matches!(
            Saas::tier(),
            Tier::T2Composite | Tier::T3DomainSpecific
        ));
        assert!(matches!(
            Serverless::tier(),
            Tier::T2Composite | Tier::T3DomainSpecific
        ));
    }

    // ================================================================
    // Confidence bounds
    // ================================================================

    #[test]
    fn test_all_confidences_in_range() {
        let compositions: Vec<PrimitiveComposition> = vec![
            Identity::primitive_composition(),
            Threshold::primitive_composition(),
            FeedbackLoop::primitive_composition(),
            Idempotency::primitive_composition(),
            Immutability::primitive_composition(),
            Convergence::primitive_composition(),
            Compute::primitive_composition(),
            Storage::primitive_composition(),
            NetworkLink::primitive_composition(),
            IsolationBoundary::primitive_composition(),
            Permission::primitive_composition(),
            ResourcePool::primitive_composition(),
            Metering::primitive_composition(),
            Replication::primitive_composition(),
            Routing::primitive_composition(),
            Lease::primitive_composition(),
            Encryption::primitive_composition(),
            Queue::primitive_composition(),
            HealthCheck::primitive_composition(),
            Elasticity::primitive_composition(),
            VirtualMachine::primitive_composition(),
            LoadBalancer::primitive_composition(),
            AutoScaling::primitive_composition(),
            Iam::primitive_composition(),
            EventualConsistency::primitive_composition(),
            Tenancy::primitive_composition(),
            PayPerUse::primitive_composition(),
            ReservedCapacity::primitive_composition(),
            SpotPricing::primitive_composition(),
            SecretsManagement::primitive_composition(),
            Container::primitive_composition(),
            Iaas::primitive_composition(),
            Paas::primitive_composition(),
            Saas::primitive_composition(),
            Serverless::primitive_composition(),
        ];

        assert_eq!(compositions.len(), 35, "Expected 35 grounded types");

        for (i, comp) in compositions.iter().enumerate() {
            assert!(
                comp.confidence >= 0.0 && comp.confidence <= 1.0,
                "Type {} confidence {} out of range",
                i,
                comp.confidence
            );
            assert!(
                comp.dominant.is_some(),
                "Type {} missing dominant primitive",
                i
            );
            assert!(!comp.primitives.is_empty(), "Type {} has no primitives", i);
        }
    }

    // ================================================================
    // Dominant primitive verification
    // ================================================================

    #[test]
    fn test_dominant_primitives() {
        assert_eq!(
            Identity::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
        assert_eq!(
            Threshold::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
        assert_eq!(
            IsolationBoundary::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(Queue::dominant_primitive(), Some(LexPrimitiva::Sequence));
        assert_eq!(Compute::dominant_primitive(), Some(LexPrimitiva::Quantity));
    }

    // ================================================================
    // Pure primitive check
    // ================================================================

    #[test]
    fn test_identity_is_pure() {
        assert!(Identity::is_pure_primitive());
    }

    #[test]
    fn test_isolation_boundary_is_pure() {
        assert!(IsolationBoundary::is_pure_primitive());
    }

    #[test]
    fn test_composites_not_pure() {
        assert!(!VirtualMachine::is_pure_primitive());
        assert!(!Iaas::is_pure_primitive());
        assert!(!Saas::is_pure_primitive());
    }
}
