//! Codon Optimizer: Scoring DNA sequences for translational efficiency.
//!
//! This module provides tools to evaluate DNA sequences based on biological
//! and physical properties like GC content, codon usage bias, and string tension.

use crate::codon_table::CodonTable;
use crate::error::Result;
use crate::ops;
use crate::string_theory;
use crate::types::{AminoAcid, Codon, Strand};
use std::collections::HashMap;

/// Scores for different aspects of codon optimization.
#[derive(Debug, Clone, Default)]
pub struct OptimizationScore {
    /// Overall score (0.0 to 1.0).
    pub total: f64,
    /// GC content score (optimality of G+C ratio).
    pub gc_score: f64,
    /// Tension score (physical stability of the strand).
    pub tension_score: f64,
    /// Usage bias score (alignment with preferred codons).
    pub usage_score: f64,
}

impl std::fmt::Display for OptimizationScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OptimizationScore(total={:.4}, gc={:.4}, tension={:.4}, usage={:.4})",
            self.total, self.gc_score, self.tension_score, self.usage_score
        )
    }
}

pub struct CodonOptimizer {
    table: CodonTable,
    /// Ideal GC content (default 0.5).
    pub target_gc: f64,
    /// Weight for GC content in total score.
    pub gc_weight: f64,
    /// Weight for tension in total score.
    pub tension_weight: f64,
    /// Weight for usage bias in total score.
    pub usage_weight: f64,
}

impl Default for CodonOptimizer {
    fn default() -> Self {
        Self {
            table: CodonTable::standard(),
            target_gc: 0.5,
            gc_weight: 0.4,
            tension_weight: 0.3,
            usage_weight: 0.3,
        }
    }
}

impl CodonOptimizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_table(table: CodonTable) -> Self {
        Self {
            table,
            ..Self::default()
        }
    }

    /// Score a strand for translational efficiency.
    pub fn score(&self, strand: &Strand) -> Result<OptimizationScore> {
        if strand.is_empty() {
            return Ok(OptimizationScore::default());
        }

        // 1. GC Content Score
        // Optimal range is usually 40-60%.
        let actual_gc = ops::gc_content(strand);
        let gc_diff = (actual_gc - self.target_gc).abs();
        // gc_score: 1.0 if perfect, drops to 0.0 if more than 0.4 away
        let gc_score = (1.0 - (gc_diff / 0.4)).clamp(0.0, 1.0);

        // 2. Tension Score
        // High tension or extreme variance can be detrimental.
        // We favor "moderate" tension (normalized to typical range).
        let tens = string_theory::tension(strand);
        // mean_tension is typically between 2.0 (all AT) and 3.0 (all GC)
        // We normalize it: 2.5 is "optimal" mid-point.
        let tension_diff = (tens.mean_tension - 2.5).abs();
        let tension_score = (1.0 - (tension_diff / 0.5)).clamp(0.0, 1.0);

        // 3. Usage Bias Score
        // In this implementation, we'll assume a "standard" bias where
        // codons with higher GC content are slightly penalized if they
        // push the strand out of the 40-60% range, but we'll focus on
        // "homopolymer" avoidance (too many of the same nucleotide in a row).
        let usage_score = self.calculate_usage_score(strand);

        let total = (gc_score * self.gc_weight)
            + (tension_score * self.tension_weight)
            + (usage_score * self.usage_weight);

        Ok(OptimizationScore {
            total,
            gc_score,
            tension_score,
            usage_score,
        })
    }

    fn calculate_usage_score(&self, strand: &Strand) -> f64 {
        if strand.len() < 3 {
            return 1.0;
        }

        // Simple penalty for homopolymers (e.g. AAAA)
        let mut penalty = 0.0;
        let mut run = 1;
        let mut prev = strand.bases[0];

        for &n in &strand.bases[1..] {
            if n == prev {
                run += 1;
                if run >= 4 {
                    penalty += 0.1;
                }
            } else {
                run = 1;
                prev = n;
            }
        }

        (1.0f64 - penalty).clamp(0.0, 1.0)
    }

    /// Suggest an optimized codon for a given amino acid based on context.
    pub fn optimize_codon(&self, aa: AminoAcid, context_gc: f64) -> Codon {
        let synonyms = self.table.codons_for(aa);
        if synonyms.is_empty() {
            // Should not happen for standard AA, but fallback to something
            return Codon(
                crate::types::Nucleotide::A,
                crate::types::Nucleotide::T,
                crate::types::Nucleotide::G,
            );
        }

        // Choose codon that moves GC content toward target
        let mut best_codon = synonyms[0];
        let mut best_dist = f64::MAX;

        for &codon in &synonyms {
            let codon_gc = (if codon.0 == crate::types::Nucleotide::G
                || codon.0 == crate::types::Nucleotide::C
            {
                1.0
            } else {
                0.0
            } + if codon.1 == crate::types::Nucleotide::G
                || codon.1 == crate::types::Nucleotide::C
            {
                1.0
            } else {
                0.0
            } + if codon.2 == crate::types::Nucleotide::G
                || codon.2 == crate::types::Nucleotide::C
            {
                1.0
            } else {
                0.0
            }) / 3.0;

            // Predicting new GC content roughly
            let new_gc = (context_gc + codon_gc) / 2.0;
            let dist = (new_gc - self.target_gc).abs();

            if dist < best_dist {
                best_dist = dist;
                best_codon = codon;
            }
        }

        best_codon
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Strand;

    #[test]
    fn test_optimizer_scores() {
        let optimizer = CodonOptimizer::new();

        // Balanced GC strand
        let balanced = Strand::parse("ATGCATGCATGC").unwrap();
        let s1 = optimizer.score(&balanced).unwrap();
        assert!(s1.total > 0.8);
        assert!((s1.gc_score - 1.0).abs() < 0.1);

        // Extreme GC strand
        let extreme = Strand::parse("GGGGGGGGGGGG").unwrap();
        let s2 = optimizer.score(&extreme).unwrap();
        assert!(s2.gc_score < 0.5);
        assert!(s2.usage_score < 1.0); // homopolymer penalty
    }
}
