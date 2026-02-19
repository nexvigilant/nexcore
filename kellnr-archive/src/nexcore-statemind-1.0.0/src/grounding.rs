//! # GroundsTo implementations for nexcore-statemind types
//!
//! Connects DNA chemistry analysis types to the Lex Primitiva type system.
//!
//! ## Domain Signature
//!
//! This crate covers 13/16 Lex Primitiva. Each pipeline stage
//! has a unique dominant primitive (see crate-level docs).

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::cluster::{Cluster, ClusterResult};
use crate::composition::Composition;
use crate::mutation::MutationStability;
use crate::nucleotide::{DnaSequence, Nucleotide};
use crate::pipeline::{ConstellationAnalysis, DnaAnalysis, MutationRecord, ResonancePair};
use crate::projection::Point3D;
use crate::resonance::Resonance;
use crate::safety::{SafetyLevel, SafetyMargin};
use crate::spectral::SpectralProfile;
use crate::thermodynamics::Tension;

// ---------------------------------------------------------------------------
// T1/T2-P: Encoding and primitive types
// ---------------------------------------------------------------------------

/// Nucleotide: T1 (Σ), dominant Σ
///
/// One of four DNA bases: A, T, G, C. Pure sum type.
impl GroundsTo for Nucleotide {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one of four variants
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// DnaSequence: T2-P (σ + Σ), dominant σ
///
/// Ordered sequence of nucleotides.
impl GroundsTo for DnaSequence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ -- ordered nucleotides
            LexPrimitiva::Sum,      // Σ -- nucleotide variants
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.90)
    }
}

// ---------------------------------------------------------------------------
// T2-P: Analysis result types
// ---------------------------------------------------------------------------

/// Composition: T2-P (N + ∝), dominant N
///
/// Nucleotide composition: AT/GC ratio, Shannon entropy.
impl GroundsTo for Composition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,        // N -- ratios, entropy value
            LexPrimitiva::Irreversibility, // ∝ -- normalization
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// SpectralProfile: T2-P (ν + N), dominant ν
///
/// FFT spectral analysis: dominant period, spectral entropy, energy.
impl GroundsTo for SpectralProfile {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // ν -- spectral frequencies
            LexPrimitiva::Quantity,  // N -- energy, entropy values
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.90)
    }
}

/// Tension: T2-P (→ + N), dominant →
///
/// Nearest-neighbor stacking thermodynamics: deltaG tension.
impl GroundsTo for Tension {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → -- thermodynamic causation
            LexPrimitiva::Quantity,  // N -- deltaG values
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// Point3D: T2-P (λ + N), dominant λ
///
/// 3D position in statemind space (H, GC%, spectral_entropy).
impl GroundsTo for Point3D {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // λ -- position in space
            LexPrimitiva::Quantity, // N -- coordinate values
        ])
        .with_dominant(LexPrimitiva::Location, 0.90)
    }
}

/// Resonance: T2-P (κ + N), dominant κ
///
/// Pairwise resonance between two DNA analyses.
impl GroundsTo for Resonance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- pairwise comparison
            LexPrimitiva::Quantity,   // N -- distance values
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// MutationStability: T2-P (ς + N), dominant ς
///
/// Mutation stability analysis: state changes under perturbation.
impl GroundsTo for MutationStability {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- pre/post mutation state
            LexPrimitiva::Quantity, // N -- stability scores
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// SafetyLevel: T2-P (∂ + κ), dominant ∂
///
/// Safety classification: Safe, Caution, Warning, Critical.
impl GroundsTo for SafetyLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- safety thresholds
            LexPrimitiva::Comparison, // κ -- level classification
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// SafetyMargin: T2-P (∂ + N), dominant ∂
///
/// Theory of Vigilance safety margin: boundary - weighted components.
impl GroundsTo for SafetyMargin {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- safety boundary
            LexPrimitiva::Quantity, // N -- margin value
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// ---------------------------------------------------------------------------
// T2-C: Clustering
// ---------------------------------------------------------------------------

/// Cluster: T2-C (Σ + λ + N + κ), dominant Σ
///
/// A k-means cluster: centroid + member points.
impl GroundsTo for Cluster {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- member aggregation
            LexPrimitiva::Location,   // λ -- centroid position
            LexPrimitiva::Quantity,   // N -- member count
            LexPrimitiva::Comparison, // κ -- nearest centroid assignment
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// ClusterResult: T2-C (Σ + κ + N + σ), dominant Σ
///
/// Result of k-means with silhouette-based k selection.
impl GroundsTo for ClusterResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- aggregation across clusters
            LexPrimitiva::Comparison, // κ -- silhouette evaluation
            LexPrimitiva::Quantity,   // N -- k, silhouette score
            LexPrimitiva::Sequence,   // σ -- ordered clusters
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

// ---------------------------------------------------------------------------
// T3: Pipeline types
// ---------------------------------------------------------------------------

/// DnaAnalysis: T3 (σ + μ + N + ν + → + λ + ∂), dominant σ
///
/// Full pipeline analysis of a single word. Sequence-dominant: ordered stages.
impl GroundsTo for DnaAnalysis {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ -- pipeline stage ordering
            LexPrimitiva::Mapping,   // μ -- word → DNA encoding
            LexPrimitiva::Quantity,  // N -- numeric results
            LexPrimitiva::Frequency, // ν -- spectral analysis
            LexPrimitiva::Causality, // → -- thermodynamic tension
            LexPrimitiva::Location,  // λ -- 3D projection
            LexPrimitiva::Boundary,  // ∂ -- safety margin
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// ConstellationAnalysis: T3 (κ + σ + Σ + μ + λ + N + ∂), dominant κ
///
/// Multi-word pairwise comparison. Comparison-dominant: resonance matrix.
impl GroundsTo for ConstellationAnalysis {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- pairwise resonance
            LexPrimitiva::Sequence,   // σ -- word list, pipeline
            LexPrimitiva::Sum,        // Σ -- clustering
            LexPrimitiva::Mapping,    // μ -- word → analysis
            LexPrimitiva::Location,   // λ -- constellation positions
            LexPrimitiva::Quantity,   // N -- scores
            LexPrimitiva::Boundary,   // ∂ -- safety thresholds
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// ResonancePair: T2-P (κ + N), dominant κ
///
/// A single pairwise resonance comparison result.
impl GroundsTo for ResonancePair {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- pairwise comparison
            LexPrimitiva::Quantity,   // N -- distance value
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// MutationRecord: T2-C (ς + μ + N + →), dominant ς
///
/// Record of a single mutation experiment.
impl GroundsTo for MutationRecord {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // ς -- before/after state
            LexPrimitiva::Mapping,   // μ -- char → nucleotide
            LexPrimitiva::Quantity,  // N -- delta values
            LexPrimitiva::Causality, // → -- mutation → effect
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
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
    fn nucleotide_is_t1() {
        assert_eq!(Nucleotide::tier(), Tier::T1Universal);
        assert_eq!(Nucleotide::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn dna_sequence_is_sequence_dominant() {
        assert_eq!(
            DnaSequence::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn spectral_is_frequency_dominant() {
        assert_eq!(
            SpectralProfile::dominant_primitive(),
            Some(LexPrimitiva::Frequency)
        );
    }

    #[test]
    fn tension_is_causality_dominant() {
        assert_eq!(Tension::dominant_primitive(), Some(LexPrimitiva::Causality));
    }

    #[test]
    fn point3d_is_location_dominant() {
        assert_eq!(Point3D::dominant_primitive(), Some(LexPrimitiva::Location));
    }

    #[test]
    fn safety_level_is_boundary_dominant() {
        assert_eq!(
            SafetyLevel::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn dna_analysis_is_t3() {
        assert_eq!(DnaAnalysis::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            DnaAnalysis::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn constellation_is_comparison_dominant() {
        assert_eq!(
            ConstellationAnalysis::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn cluster_is_sum_dominant() {
        assert_eq!(Cluster::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn resonance_is_comparison_dominant() {
        assert_eq!(
            Resonance::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }
}
