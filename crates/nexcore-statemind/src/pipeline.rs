//! Stage 10: Analysis Pipeline Orchestration.
//!
//! Top-level API combining all stages into `DnaAnalysis` (single word)
//! and `ConstellationAnalysis` (word constellation with pairwise comparisons).
//!
//! Tier: T3 | Dominant: σ (Sequence) — ordered pipeline execution.

use crate::cluster::{ClusterResult, auto_cluster};
use crate::composition::Composition;
use crate::mutation::MutationStability;
use crate::nucleotide::{DnaSequence, encode};
use crate::projection::Point3D;
use crate::resonance::Resonance;
use crate::safety::SafetyMargin;
use crate::spectral::SpectralProfile;
use crate::thermodynamics::Tension;
use serde::Serialize;

/// Complete DNA chemistry analysis of a single word.
///
/// Pipeline: encode → composition → spectral → thermodynamics → project → safety.
///
/// Tier: T3 | Grounds to: σ (Sequence) + μ (Mapping) + N (Quantity)
///         + ν (Frequency) + → (Causality) + λ (Location) + ∂ (Boundary).
#[derive(Debug, Clone, Serialize)]
pub struct DnaAnalysis {
    /// The analyzed word.
    pub word: String,
    /// Encoded DNA sequence.
    pub sequence: DnaSequence,
    /// Nucleotide composition statistics.
    pub composition: Composition,
    /// Spectral frequency profile.
    pub spectral: SpectralProfile,
    /// Thermodynamic tension.
    pub tension: Tension,
    /// 3D position in statemind space.
    pub projection: Point3D,
    /// Theory of Vigilance safety margin.
    pub safety: SafetyMargin,
}

impl DnaAnalysis {
    /// Run the full analysis pipeline on a single word.
    #[must_use]
    pub fn analyze(word: &str) -> Self {
        let sequence = encode(word);
        let composition = Composition::analyze(&sequence);
        let spectral = SpectralProfile::analyze(&sequence);
        let tension = Tension::analyze(&sequence);
        let projection = Point3D::from_features(&composition, &spectral);
        let safety = SafetyMargin::compute(&projection, &tension);

        Self {
            word: word.to_string(),
            sequence,
            composition,
            spectral,
            tension,
            projection,
            safety,
        }
    }

    /// Sequence length in nucleotides.
    #[must_use]
    pub fn nucleotide_count(&self) -> usize {
        self.sequence.len()
    }
}

impl std::fmt::Display for DnaAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} → {} nt | H={:.3} | GC={:.3} | E={:.3} | T={:.3} | {} | {}",
            self.word,
            self.sequence.len(),
            self.composition.shannon_entropy,
            self.composition.gc_ratio,
            self.spectral.total_energy,
            self.tension.normalized,
            self.projection,
            self.safety,
        )
    }
}

/// Pairwise resonance record.
#[derive(Debug, Clone, Serialize)]
pub struct ResonancePair {
    /// First word.
    pub word_a: String,
    /// Second word.
    pub word_b: String,
    /// Resonance between them.
    pub resonance: Resonance,
}

/// Mutation stability record.
#[derive(Debug, Clone, Serialize)]
pub struct MutationRecord {
    /// The word analyzed.
    pub word: String,
    /// Mutation stability profile.
    pub stability: MutationStability,
}

/// Full constellation analysis of multiple words.
///
/// Combines single-word analyses with pairwise resonance,
/// mutation stability, and cluster topology.
///
/// Tier: T3 | Grounds to: σ (Sequence) + κ (Comparison) + Σ (Sum)
///         + λ (Location) + ∂ (Boundary) + ς (State) + ∃ (Existence).
#[derive(Debug, Clone, Serialize)]
pub struct ConstellationAnalysis {
    /// Individual word analyses.
    pub analyses: Vec<DnaAnalysis>,
    /// Pairwise resonance comparisons.
    pub resonances: Vec<ResonancePair>,
    /// Mutation stability for each word.
    pub mutations: Vec<MutationRecord>,
    /// Cluster topology.
    pub clusters: ClusterResult,
}

impl ConstellationAnalysis {
    /// Analyze a constellation of words.
    ///
    /// Runs full pipeline on each word, then computes pairwise resonances,
    /// mutation stability, and cluster topology.
    #[must_use]
    pub fn analyze(words: &[&str]) -> Self {
        let analyses: Vec<DnaAnalysis> = words.iter().map(|w| DnaAnalysis::analyze(w)).collect();

        // Pairwise resonances
        let mut resonances = Vec::new();
        for i in 0..analyses.len() {
            for j in (i + 1)..analyses.len() {
                let r = Resonance::compare(&analyses[i].spectral, &analyses[j].spectral);
                resonances.push(ResonancePair {
                    word_a: analyses[i].word.clone(),
                    word_b: analyses[j].word.clone(),
                    resonance: r,
                });
            }
        }

        // Mutation stability (skip for large constellations to avoid O(n²·m) cost)
        let mutations: Vec<MutationRecord> = if words.len() <= 10 {
            analyses
                .iter()
                .map(|a| MutationRecord {
                    word: a.word.clone(),
                    stability: MutationStability::analyze(&a.sequence, &a.projection),
                })
                .collect()
        } else {
            Vec::new()
        };

        // Clustering
        let points: Vec<(String, Point3D)> = analyses
            .iter()
            .map(|a| (a.word.clone(), a.projection.clone()))
            .collect();
        let clusters = auto_cluster(&points);

        Self {
            analyses,
            resonances,
            mutations,
            clusters,
        }
    }

    /// Number of words in the constellation.
    #[must_use]
    pub fn word_count(&self) -> usize {
        self.analyses.len()
    }

    /// Number of pairwise resonance comparisons.
    #[must_use]
    pub fn resonance_count(&self) -> usize {
        self.resonances.len()
    }

    /// Find the resonance between two specific words, if computed.
    #[must_use]
    pub fn find_resonance(&self, word_a: &str, word_b: &str) -> Option<&Resonance> {
        self.resonances.iter().find_map(|rp| {
            if (rp.word_a == word_a && rp.word_b == word_b)
                || (rp.word_a == word_b && rp.word_b == word_a)
            {
                Some(&rp.resonance)
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_word_analysis() {
        let a = DnaAnalysis::analyze("NexVigilant");
        assert!(!a.word.is_empty());
        assert!(a.nucleotide_count() > 0);
        assert!(a.composition.shannon_entropy > 0.0);
    }

    #[test]
    fn deterministic_analysis() {
        let a = DnaAnalysis::analyze("NexVigilant");
        let b = DnaAnalysis::analyze("NexVigilant");
        assert_eq!(a.sequence.bases(), b.sequence.bases());
        assert!((a.composition.shannon_entropy - b.composition.shannon_entropy).abs() < 1e-10);
        assert!((a.spectral.total_energy - b.spectral.total_energy).abs() < 1e-10);
    }

    #[test]
    fn constellation_analysis() {
        let ca = ConstellationAnalysis::analyze(&["NexVigilant", "vigilance", "patient"]);
        assert_eq!(ca.word_count(), 3);
        // C(3,2) = 3 pairwise resonances
        assert_eq!(ca.resonance_count(), 3);
        assert!(!ca.clusters.clusters.is_empty());
    }

    #[test]
    fn find_resonance_works() {
        let ca = ConstellationAnalysis::analyze(&["alpha", "beta", "gamma"]);
        let r = ca.find_resonance("alpha", "beta");
        assert!(r.is_some());
        // Symmetric lookup
        let r2 = ca.find_resonance("beta", "alpha");
        assert!(r2.is_some());
    }

    #[test]
    fn resonance_count_formula() {
        for n in 2..=5 {
            let words: Vec<String> = (0..n).map(|i| format!("word{i}")).collect();
            let word_refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
            let ca = ConstellationAnalysis::analyze(&word_refs);
            let expected = n * (n - 1) / 2; // C(n, 2)
            assert_eq!(
                ca.resonance_count(),
                expected,
                "C({n}, 2) should be {expected}"
            );
        }
    }

    #[test]
    fn display_includes_all_fields() {
        let a = DnaAnalysis::analyze("test");
        let display = format!("{a}");
        assert!(display.contains("test"));
        assert!(display.contains("nt"));
    }
}
