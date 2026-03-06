//! # NexVigilant Core — statemind
//!
//! DNA chemistry analysis pipeline for brand and word constellation analysis.
//!
//! ## Pipeline Architecture
//!
//! ```text
//!                           ┌─ composition ─── AT/GC ratio, Shannon H
//!                           │
//! char ──→ nucleotide_encode ──→ FFT ──────── spectral entropy, dominant T, energy
//!                           │
//!                           └─ stacking ───── nearest-neighbor ΔG → tension
//!                                     │
//!                                     ▼
//!                               3D projection
//!                            (H, GC%, spectral_entropy)
//!                                     │
//!                     ┌───────────────┼───────────────┐
//!                     ▼               ▼               ▼
//!               pairwise         mutation          k-means
//!               resonance        stability         topology
//!                     │               │               │
//!                     └───────┬───────┘               │
//!                             ▼                       ▼
//!                       ToV d(s) safety          cluster graph
//!                          margin
//! ```
//!
//! ## Primitive Coverage (13/16 Lex Primitiva)
//!
//! | Stage | Module | Dominant T1 |
//! |-------|--------|-------------|
//! | Encode | `nucleotide` | μ Mapping |
//! | Composition | `composition` | N Quantity |
//! | FFT | `spectral` | ν Frequency |
//! | Thermodynamics | `thermodynamics` | → Causality |
//! | Projection | `projection` | λ Location |
//! | Resonance | `resonance` | κ Comparison |
//! | Mutation | `mutation` | ς State |
//! | Clustering | `cluster` | Σ Sum |
//! | Safety | `safety` | ∂ Boundary |
//! | Pipeline | `pipeline` | σ Sequence |
//!
//! ## Key Algorithms
//!
//! - **Encoding:** 2-bit extraction (byte → 4 nucleotides) + ATG/TAA framing
//! - **FFT:** Voss representation (4 binary indicators) → DFT → summed power spectrum
//! - **Thermodynamics:** SantaLucia (1998) nearest-neighbor stacking parameters
//! - **Clustering:** Lloyd's k-means with silhouette-based k selection
//! - **Safety:** ToV d(s) = boundary − (spectral×0.4 + tension×0.3 + AT-richness×0.3)

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod cluster;
pub mod composition;
pub mod grounding;
pub mod mutation;
pub mod nucleotide;
pub mod pipeline;
pub mod projection;
pub mod resonance;
pub mod safety;
pub mod spectral;
pub mod thermodynamics;

// Re-exports for convenience
pub use cluster::{Cluster, ClusterResult, auto_cluster, kmeans};
pub use composition::Composition;
pub use mutation::MutationStability;
pub use nucleotide::{DnaSequence, Nucleotide, encode, encode_raw};
pub use pipeline::{ConstellationAnalysis, DnaAnalysis, MutationRecord, ResonancePair};
pub use projection::Point3D;
pub use resonance::Resonance;
pub use safety::{SafetyLevel, SafetyMargin};
pub use spectral::SpectralProfile;
pub use thermodynamics::Tension;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_pipeline_runs() {
        let a = DnaAnalysis::analyze("NexVigilant");
        assert_eq!(a.word, "NexVigilant");
        assert!(a.nucleotide_count() > 0);
    }

    #[test]
    fn constellation_runs() {
        let ca =
            ConstellationAnalysis::analyze(&["NexVigilant", "vigilance", "patient", "guardian"]);
        assert_eq!(ca.word_count(), 4);
        // C(4,2) = 6 pairwise comparisons
        assert_eq!(ca.resonance_count(), 6);
    }

    #[test]
    fn all_modules_accessible() {
        // Verify re-exports work
        let _n = Nucleotide::A;
        let _p = Point3D::origin();
        let _s = SafetyLevel::Critical;
    }
}
