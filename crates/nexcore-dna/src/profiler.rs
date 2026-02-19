//! Mutation Profiler: Exhaustive single-nucleotide sensitivity analysis.
//!
//! This module provides tools to systematically mutate every nucleotide in a
//! DNA program and measure the resulting semantic shift.

use crate::error::Result;
use crate::types::{Nucleotide, Strand};
use crate::vm::{CodonVM, HaltReason, VmConfig, VmResult};
use std::collections::HashMap;

/// Semantic sensitivity of a specific nucleotide position.
#[derive(Debug, Clone, PartialEq)]
pub enum Sensitivity {
    /// No change in behavior (synonymous mutation or non-functional region).
    Neutral,
    /// Change in execution cycles or internal state, but same output buffer.
    Internal,
    /// Change in output buffer (semantic shift).
    Semantic,
    /// Mutation prevents program from starting or causes early crash.
    Lethal,
}

impl Sensitivity {
    /// Numerical score for heatmap generation (0.0 to 1.0).
    pub fn score(&self) -> f64 {
        match self {
            Self::Neutral => 0.0,
            Self::Internal => 0.3,
            Self::Semantic => 0.7,
            Self::Lethal => 1.0,
        }
    }
}

/// A detailed report of mutation sensitivity across a strand.
#[derive(Debug, Clone)]
pub struct ProfileReport {
    /// Sensitivity for each position.
    pub positions: Vec<PositionSensitivity>,
    /// Global summary stats.
    pub summary: ProfileSummary,
}

/// Sensitivity data for a single nucleotide position.
#[derive(Debug, Clone)]
pub struct PositionSensitivity {
    pub index: usize,
    pub original: Nucleotide,
    /// The worst-case sensitivity across all possible mutations at this position.
    pub worst_case: Sensitivity,
    /// Breakdown by specific mutation.
    pub mutations: HashMap<Nucleotide, Sensitivity>,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileSummary {
    pub total_positions: usize,
    pub lethal_count: usize,
    pub semantic_count: usize,
    pub internal_count: usize,
    pub neutral_count: usize,
    pub critical_density: f64,
}

impl ProfileReport {
    /// Generate a text-based heatmap of sensitivity.
    ///
    /// Symbols:
    /// - `.` : Neutral
    /// - `i` : Internal
    /// - `s` : Semantic
    /// - `X` : Lethal
    pub fn to_heatmap(&self) -> String {
        let mut heatmap = String::with_capacity(self.positions.len());
        for pos in &self.positions {
            let symbol = match pos.worst_case {
                Sensitivity::Neutral => '.',
                Sensitivity::Internal => 'i',
                Sensitivity::Semantic => 's',
                Sensitivity::Lethal => 'X',
            };
            heatmap.push(symbol);
        }
        heatmap
    }
}

pub struct MutationProfiler {
    config: VmConfig,
}

impl MutationProfiler {
    pub fn new(config: VmConfig) -> Self {
        Self { config }
    }

    /// Profile a strand by systematically mutating every nucleotide.
    pub fn profile(&self, strand: &Strand) -> Result<ProfileReport> {
        // 1. Get baseline
        let mut vm = CodonVM::with_config(self.config.clone());
        vm.load(strand)?;
        let baseline = vm.execute()?;

        let mut positions = Vec::with_capacity(strand.len());
        let mut summary = ProfileSummary::default();
        summary.total_positions = strand.len();

        // 2. Iterate every position
        for i in 0..strand.len() {
            let original_n = strand.bases[i];
            let mut pos_data = PositionSensitivity {
                index: i,
                original: original_n,
                worst_case: Sensitivity::Neutral,
                mutations: HashMap::new(),
            };

            // 3. Try all 3 other nucleotides
            for &mutant_n in &[Nucleotide::A, Nucleotide::T, Nucleotide::G, Nucleotide::C] {
                if mutant_n == original_n {
                    continue;
                }

                let mut mutant_strand = strand.clone();
                mutant_strand.bases[i] = mutant_n;

                let sensitivity = self.evaluate_mutation(&baseline, &mutant_strand);
                pos_data.mutations.insert(mutant_n, sensitivity.clone());

                // Update worst case
                if sensitivity.score() > pos_data.worst_case.score() {
                    pos_data.worst_case = sensitivity;
                }
            }

            // Update summary
            match pos_data.worst_case {
                Sensitivity::Lethal => summary.lethal_count += 1,
                Sensitivity::Semantic => summary.semantic_count += 1,
                Sensitivity::Internal => summary.internal_count += 1,
                Sensitivity::Neutral => summary.neutral_count += 1,
            }

            positions.push(pos_data);
        }

        summary.critical_density =
            (summary.lethal_count + summary.semantic_count) as f64 / summary.total_positions as f64;

        Ok(ProfileReport { positions, summary })
    }

    fn evaluate_mutation(&self, baseline: &VmResult, mutant_strand: &Strand) -> Sensitivity {
        let mut vm = CodonVM::with_config(self.config.clone());

        // If it fails to load (e.g. parity error if enabled) it's lethal
        if vm.load(mutant_strand).is_err() {
            return Sensitivity::Lethal;
        }

        match vm.execute() {
            Err(_) => Sensitivity::Lethal,
            Ok(result) => {
                if result.halt_reason != baseline.halt_reason
                    && matches!(
                        result.halt_reason,
                        HaltReason::Error | HaltReason::ParityError(_)
                    )
                {
                    return Sensitivity::Lethal;
                }

                if result.output != baseline.output {
                    return Sensitivity::Semantic;
                }

                if result.stack != baseline.stack || result.cycles != baseline.cycles {
                    return Sensitivity::Internal;
                }

                Sensitivity::Neutral
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Strand;

    #[test]
    fn test_profiler_on_simple_program() {
        // A simple program: Entry, Push1, Output, Halt
        // Entry = GAA (index 32)
        // Push1 = TAC (index 19)
        // Output = GTT (index 37)
        // Halt = GAT (index 33)
        let dna = "GAATACGTTGAT";
        let strand = Strand::parse(dna).unwrap();

        let profiler = MutationProfiler::new(VmConfig::default());
        let report = profiler.profile(&strand).unwrap();

        assert_eq!(report.summary.total_positions, 12);

        // Entry codon (0..3) should be critical (Lethal if changed to non-entry)
        for i in 0..3 {
            assert!(matches!(
                report.positions[i].worst_case,
                Sensitivity::Lethal | Sensitivity::Semantic
            ));
        }

        // Halt codon (9..12) should be critical
        for i in 9..12 {
            assert!(matches!(
                report.positions[i].worst_case,
                Sensitivity::Lethal | Sensitivity::Semantic
            ));
        }

        println!(
            "Critical density: {:.2}%",
            report.summary.critical_density * 100.0
        );
    }

    #[test]
    fn test_sensitivity_levels() {
        // GAA (Entry), AAA (Nop), TAC (Push1), GTT (Output), GAT (Halt)
        let dna = "GAAAAATACGTTGAT";
        let strand = Strand::parse(dna).unwrap();

        let profiler = MutationProfiler::new(VmConfig::default());
        let report = profiler.profile(&strand).unwrap();

        // Nop is at index 3..6
        // If we change AAA to TAG (Push0), it should be Internal (stack change, same output)
        let nop_pos = 3;
        let mutations = &report.positions[nop_pos].mutations;

        // Let's see what changing AAA to GGG (Mul) does -> probably Lethal (StackUnderflow)
        // Let's see what changing AAA to TAG (Push0) does

        let push0_n = Nucleotide::G; // AAA -> AAG? No, TAG is 16 = TAA. 
        // Index 16 (TAA) = Load.
        // Index 18 (TAG) = Push0.

        // Wait, if I change AAA (0,0,0) to TAG (1,0,2), that's two mutations.
        // The profiler only does SINGLE nucleotide mutations.
        // AAA -> TAA (1,0,0) = Load. (Index 16)
        // AAA -> GAA (2,0,0) = Entry. (Index 32)
        // AAA -> CAA (3,0,0) = Eq. (Index 48)

        // AAA (0,0,0) -> AAC (0,0,3) = Pop. (Index 3)

        // If we change Nop to Pop, it will StackUnderflow -> Lethal.

        // Let's find a mutation that is Internal.
        // Maybe changing GTT (Output) to something else? No, that's Semantic.

        // If we have: Push1, Push1, Add, Output
        // And we change one Push1 to Push0. Output changes 2 -> 1. Semantic.

        // What about synonymous codons?
        // In the standard genetic code, many codons map to the same AminoAcid.
        // BUT in our ISA, every codon maps to a UNIQUE instruction.
        // So there are NO synonymous codons in the VM sense!
        // This is a key difference from biology.

        // HOWEVER, some instructions might be neutral in specific contexts.
        // e.g. a Nop before an Entry.

        let dna_with_prefix = "TTTGAATACGTTGAT";
        let strand2 = Strand::parse(dna_with_prefix).unwrap();
        let report2 = profiler.profile(&strand2).unwrap();

        // The first 3 bases (TTT) are before the Entry point.
        // None of the single-nucleotide mutations of TTT result in GAA (Entry).
        // TTT mutants: ATT, GTT, CTT, TAT, TGT, TCT, TTA, TTG, TTC.
        // So these should all be Neutral.
        for i in 0..3 {
            assert_eq!(report2.positions[i].worst_case, Sensitivity::Neutral);
        }

        let heatmap = report2.to_heatmap();
        assert_eq!(heatmap.len(), 15);
        assert!(heatmap.starts_with("...")); // Prefix TTT is Neutral
        println!("Heatmap: {}", heatmap);
    }
}
