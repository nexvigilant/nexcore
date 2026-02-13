//! # GroundsTo implementations for nexcore-telemetry-core types
//!
//! Connects telemetry source, snapshot, and operation types
//! to the Lex Primitiva type system.
//!
//! ## Domain Signature
//!
//! - **μ (Mapping)**: tool name → activity type, source → report
//! - **σ (Sequence)**: temporal session streams
//! - **π (Persistence)**: snapshot versioning

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::error::TelemetryError;
use crate::intel::IntelReport;
use crate::types::{
    ActivityType, Operation, ProjectHash, Snapshot, SnapshotType, SourceId, TokenUsage,
};

// ---------------------------------------------------------------------------
// T2-P: Identity and classification types
// ---------------------------------------------------------------------------

/// SourceId: T2-P (λ + ∃), dominant λ
///
/// Unique telemetry source session identifier.
impl GroundsTo for SourceId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,  // λ -- session address
            LexPrimitiva::Existence, // ∃ -- session exists
        ])
        .with_dominant(LexPrimitiva::Location, 0.90)
    }
}

/// ProjectHash: T2-P (λ + μ), dominant λ
///
/// SHA-256 hash of project path. Location-dominant: identity mapping.
impl GroundsTo for ProjectHash {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // λ -- project identity
            LexPrimitiva::Mapping,  // μ -- path → hash
        ])
        .with_dominant(LexPrimitiva::Location, 0.90)
    }
}

/// ActivityType: T2-P (μ + Σ), dominant μ
///
/// Classification of tool operations. Mapping-dominant: name → category.
impl GroundsTo for ActivityType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // μ -- name → type mapping
            LexPrimitiva::Sum,     // Σ -- variant enumeration
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// SnapshotType: T2-P (Σ + ς), dominant Σ
///
/// Artifact type classification: Task, Plan, Walkthrough, Implementation.
impl GroundsTo for SnapshotType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,   // Σ -- one of many variants
            LexPrimitiva::State, // ς -- artifact state category
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// TokenUsage: T2-P (N + Σ), dominant N
///
/// Token usage statistics: input, output, cached, total.
impl GroundsTo for TokenUsage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- token counts
            LexPrimitiva::Sum,      // Σ -- total aggregation
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

// ---------------------------------------------------------------------------
// T2-C / T3: Composed domain types
// ---------------------------------------------------------------------------

/// Operation: T2-C (μ + σ + ς + λ), dominant μ
///
/// A single tool operation. Mapping-dominant: input → result transformation.
impl GroundsTo for Operation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // μ -- args → result
            LexPrimitiva::Sequence, // σ -- temporal order
            LexPrimitiva::State,    // ς -- status (success/failure)
            LexPrimitiva::Location, // λ -- tool name identity
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// Snapshot: T3 (π + ρ + ς + σ + λ + Σ), dominant π
///
/// Versioned artifact snapshot. Persistence-dominant: version history.
impl GroundsTo for Snapshot {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // π -- versioned storage
            LexPrimitiva::Recursion,   // ρ -- version chain
            LexPrimitiva::State,       // ς -- content state
            LexPrimitiva::Sequence,    // σ -- version ordering
            LexPrimitiva::Location,    // λ -- file path
            LexPrimitiva::Sum,         // Σ -- type classification
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

/// IntelReport: T3 (μ + Σ + σ + κ + N + ∂), dominant μ
///
/// Intelligence report generated from telemetry analysis.
/// Mapping-dominant: raw data → actionable intelligence.
impl GroundsTo for IntelReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // μ -- data → intelligence
            LexPrimitiva::Sum,        // Σ -- aggregation of findings
            LexPrimitiva::Sequence,   // σ -- temporal analysis
            LexPrimitiva::Comparison, // κ -- cross-reference
            LexPrimitiva::Quantity,   // N -- metrics
            LexPrimitiva::Boundary,   // ∂ -- governance boundaries
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Error types — ∂ dominant
// ---------------------------------------------------------------------------

/// TelemetryError: T2-P (∂ + ∅), dominant ∂
///
/// Telemetry error variants: not found, invalid format.
impl GroundsTo for TelemetryError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- format constraints
            LexPrimitiva::Void,     // ∅ -- not found
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
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
    fn source_id_is_location_dominant() {
        assert_eq!(SourceId::dominant_primitive(), Some(LexPrimitiva::Location));
    }

    #[test]
    fn activity_type_is_mapping_dominant() {
        assert_eq!(
            ActivityType::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn snapshot_is_persistence_dominant() {
        assert_eq!(
            Snapshot::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
        assert_eq!(Snapshot::tier(), Tier::T3DomainSpecific);
    }

    #[test]
    fn operation_is_mapping_dominant() {
        assert_eq!(Operation::dominant_primitive(), Some(LexPrimitiva::Mapping));
        assert_eq!(Operation::tier(), Tier::T2Composite);
    }

    #[test]
    fn token_usage_is_quantity_dominant() {
        assert_eq!(
            TokenUsage::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn telemetry_error_is_boundary_dominant() {
        assert_eq!(
            TelemetryError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn intel_report_is_t3() {
        assert_eq!(IntelReport::tier(), Tier::T3DomainSpecific);
    }
}
