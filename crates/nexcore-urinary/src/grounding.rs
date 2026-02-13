//! # GroundsTo implementations for nexcore-urinary types
//!
//! All types from both lib.rs and claude_code.rs ground to Lex Primitiva compositions.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::claude_code::{
    ArtifactRetention, DecisionAuditCleanup, DisposalMethod, LogRotation, RetentionPolicy,
    SessionExpiry, SilentFailureRisk, TelemetryPruning, UrinarySystemHealth, WasteCategory,
};
use crate::{
    Bladder, Excretion, FilterCategory, FiltrationRate, GlomerularFiltration, Nephron,
    Reabsorption, UrinaryHealth,
};

// ============================================================================
// Core urinary types (lib.rs)
// ============================================================================

impl GroundsTo for FilterCategory {
    fn primitive_composition() -> PrimitiveComposition {
        // Σ sum: enumeration over filter domains
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

impl GroundsTo for Nephron {
    fn primitive_composition() -> PrimitiveComposition {
        // ∂ boundary: filtration threshold
        // κ comparison: retention decision
        // N quantity: counts and ages
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.70)
    }
}

impl GroundsTo for GlomerularFiltration {
    fn primitive_composition() -> PrimitiveComposition {
        // ∂ boundary: initial filter threshold
        // N quantity: counts and rates
        // κ comparison: pass/block decision
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
    }
}

impl GroundsTo for Reabsorption {
    fn primitive_composition() -> PrimitiveComposition {
        // κ comparison: reclaim decision criteria
        // N quantity: counts and bytes
        // ∂ boundary: reabsorption threshold
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.75)
    }
}

impl GroundsTo for Excretion {
    fn primitive_composition() -> PrimitiveComposition {
        // ∝ irreversibility: permanent disposal
        // ∅ void: removal from system
        // N quantity: counts and bytes
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Void,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.80)
    }
}

impl GroundsTo for FiltrationRate {
    fn primitive_composition() -> PrimitiveComposition {
        // N quantity: rate measurement
        // ν frequency: throughput
        // κ comparison: current vs target
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

impl GroundsTo for Bladder {
    fn primitive_composition() -> PrimitiveComposition {
        // σ sequence: collection of items
        // ∂ boundary: capacity limits
        // N quantity: counts
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.70)
    }
}

impl GroundsTo for UrinaryHealth {
    fn primitive_composition() -> PrimitiveComposition {
        // ς state: system health state
        // κ comparison: health checks
        // ∂ boundary: health thresholds
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

// ============================================================================
// Claude Code urinary types (claude_code.rs)
// ============================================================================

impl GroundsTo for TelemetryPruning {
    fn primitive_composition() -> PrimitiveComposition {
        // ∝ irreversibility: permanent pruning
        // N quantity: before/after counts
        // ∂ boundary: pruning threshold
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.75)
    }
}

impl GroundsTo for SessionExpiry {
    fn primitive_composition() -> PrimitiveComposition {
        // ∝ irreversibility: session deletion
        // ν frequency: age-based expiry
        // ∃ existence: session lifecycle
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Frequency,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.70)
    }
}

impl GroundsTo for ArtifactRetention {
    fn primitive_composition() -> PrimitiveComposition {
        // π persistence: retention policy
        // ν frequency: age tracking
        // κ comparison: retention decision
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Frequency,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.75)
    }
}

impl GroundsTo for LogRotation {
    fn primitive_composition() -> PrimitiveComposition {
        // σ sequence: log file series
        // N quantity: size tracking
        // ∂ boundary: rotation threshold
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.75)
    }
}

impl GroundsTo for RetentionPolicy {
    fn primitive_composition() -> PrimitiveComposition {
        // κ comparison: retention criteria
        // ∂ boundary: age/count limits
        // ν frequency: periodic enforcement
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.75)
    }
}

impl GroundsTo for DecisionAuditCleanup {
    fn primitive_composition() -> PrimitiveComposition {
        // ∝ irreversibility: audit log pruning
        // N quantity: entry counts
        // κ comparison: keep/prune decision
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.70)
    }
}

impl GroundsTo for WasteCategory {
    fn primitive_composition() -> PrimitiveComposition {
        // Σ sum: enumeration over waste types
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

impl GroundsTo for DisposalMethod {
    fn primitive_composition() -> PrimitiveComposition {
        // Σ sum: enumeration over disposal methods
        // ∝ irreversibility: permanent actions
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Irreversibility])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for SilentFailureRisk {
    fn primitive_composition() -> PrimitiveComposition {
        // ∅ void: silent failure (absence of monitoring)
        // κ comparison: risk assessment
        // N quantity: risk metrics
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Void, 0.75)
    }
}

impl GroundsTo for UrinarySystemHealth {
    fn primitive_composition() -> PrimitiveComposition {
        // ς state: system health state
        // κ comparison: health checks
        // ∂ boundary: health thresholds
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Core types tests
    // ============================================================================

    #[test]
    fn test_filter_category_grounds_to() {
        let comp = FilterCategory::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!(comp.confidence >= 0.85);
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
    }

    #[test]
    fn test_nephron_grounds_to() {
        let comp = Nephron::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!(comp.confidence >= 0.65);
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
    }

    #[test]
    fn test_glomerular_filtration_grounds_to() {
        let comp = GlomerularFiltration::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!(comp.confidence >= 0.70);
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn test_reabsorption_grounds_to() {
        let comp = Reabsorption::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!(comp.confidence >= 0.70);
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn test_excretion_grounds_to() {
        let comp = Excretion::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
        assert!(comp.confidence >= 0.75);
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert!(comp.primitives.contains(&LexPrimitiva::Void));
    }

    #[test]
    fn test_filtration_rate_grounds_to() {
        let comp = FiltrationRate::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!(comp.confidence >= 0.75);
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
    }

    #[test]
    fn test_bladder_grounds_to() {
        let comp = Bladder::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!(comp.confidence >= 0.65);
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn test_urinary_health_grounds_to() {
        let comp = UrinaryHealth::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!(comp.confidence >= 0.60);
        assert!(comp.primitives.contains(&LexPrimitiva::State));
    }

    // ============================================================================
    // Claude Code types tests
    // ============================================================================

    #[test]
    fn test_telemetry_pruning_grounds_to() {
        let comp = TelemetryPruning::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
        assert!(comp.confidence >= 0.70);
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
    }

    #[test]
    fn test_session_expiry_grounds_to() {
        let comp = SessionExpiry::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
        assert!(comp.confidence >= 0.65);
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
    }

    #[test]
    fn test_artifact_retention_grounds_to() {
        let comp = ArtifactRetention::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert!(comp.confidence >= 0.70);
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
    }

    #[test]
    fn test_log_rotation_grounds_to() {
        let comp = LogRotation::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!(comp.confidence >= 0.70);
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
    }

    #[test]
    fn test_retention_policy_grounds_to() {
        let comp = RetentionPolicy::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!(comp.confidence >= 0.70);
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn test_decision_audit_cleanup_grounds_to() {
        let comp = DecisionAuditCleanup::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
        assert!(comp.confidence >= 0.65);
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
    }

    #[test]
    fn test_waste_category_grounds_to() {
        let comp = WasteCategory::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!(comp.confidence >= 0.85);
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
    }

    #[test]
    fn test_disposal_method_grounds_to() {
        let comp = DisposalMethod::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!(comp.confidence >= 0.80);
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
    }

    #[test]
    fn test_silent_failure_risk_grounds_to() {
        let comp = SilentFailureRisk::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Void));
        assert!(comp.confidence >= 0.70);
        assert!(comp.primitives.contains(&LexPrimitiva::Void));
    }

    #[test]
    fn test_urinary_system_health_grounds_to() {
        let comp = UrinarySystemHealth::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!(comp.confidence >= 0.60);
        assert!(comp.primitives.contains(&LexPrimitiva::State));
    }
}
