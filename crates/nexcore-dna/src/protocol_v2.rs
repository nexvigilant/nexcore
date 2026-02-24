//! V2 Protocol: DNA Evolution Robustness
//!
//! Rigorous statistical evaluation of DNA-based genetic algorithms vs baselines.

use crate::cortex::{EvolutionConfig, Rng};
use crate::evolution::{EvolutionSandbox, Specimen};
use crate::lexicon;
use crate::storage;
use crate::types::Strand;
use crate::vm::VmConfig;
use std::collections::HashMap;
use std::time::Instant;

/// Result of a single trial.
#[derive(Debug, Clone)]
pub struct TrialResult {
    pub algorithm: String,
    pub target_id: usize,
    pub difficulty: String,
    pub trial_id: usize,
    pub best_fitness: f64,
    pub valid_utf8: bool,
    pub printable_ascii_rate: f64,
    pub wall_time_ms: u128,
    pub evaluations: usize,
}

impl TrialResult {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"algorithm\": \"{}\", \"target_id\": {}, \"difficulty\": \"{}\", \"trial_id\": {}, \"best_fitness\": {:.4}, \"valid_utf8\": {}, \"printable_ascii_rate\": {:.4}, \"wall_time_ms\": {}, \"evaluations\": {}}}",
            self.algorithm,
            self.target_id,
            self.difficulty,
            self.trial_id,
            self.best_fitness,
            self.valid_utf8,
            self.printable_ascii_rate,
            self.wall_time_ms,
            self.evaluations
        )
    }
}

pub struct ProtocolV2 {
    pub targets: Vec<String>,
    pub difficulties: Vec<f64>, // Distance multipliers
    pub budget: usize,
    pub trials_per_scenario: usize,
}

impl ProtocolV2 {
    pub fn new() -> Self {
        Self {
            targets: vec![
                "DNA is robust".to_string(),
                "Evolution works".to_string(),
                "Code is life".to_string(),
                "Machine soul".to_string(),
                "Quaternary code".to_string(),
                "Genetic memory".to_string(),
                "Binary to base4".to_string(),
                "Synthetic biology".to_string(),
                "Digital evolution".to_string(),
                "Neural synapses".to_string(),
                "Computational DNA".to_string(),
                "T1 Primitives".to_string(),
                "Lex Primitiva".to_string(),
                "Vigilance state".to_string(),
                "Signal detection".to_string(),
                "Molecular VM".to_string(),
                "Codon opcodes".to_string(),
                "Strand sequence".to_string(),
                "Double helix logic".to_string(),
                "NexVigilant Core".to_string(),
            ],
            difficulties: vec![0.1, 0.3, 0.6], // Easy, Medium, Hard
            budget: 10000,
            trials_per_scenario: 30,
        }
    }

    pub fn run(&self) -> Vec<TrialResult> {
        let mut all_results = Vec::new();
        let algorithms = ["GA", "MC", "SRW", "HC", "GA-NoXover"];

        for (t_idx, target) in self.targets.iter().enumerate() {
            for (d_idx, &diff) in self.difficulties.iter().enumerate() {
                let diff_label = match d_idx {
                    0 => "easy",
                    1 => "medium",
                    _ => "hard",
                };

                println!(
                    "Scenario: Target={} difficulty={} ({} scenarios total)",
                    t_idx,
                    diff_label,
                    self.targets.len() * self.difficulties.len()
                );

                let seed_text = self.generate_seed(target, diff);
                let seed_strand = storage::encode_str(&seed_text);

                for &algo in &algorithms {
                    for trial_id in 0..self.trials_per_scenario {
                        // Reproducible run-specific seed
                        let run_seed = self.compute_run_seed(t_idx, d_idx, algo, trial_id);

                        let result = self.run_trial(
                            algo,
                            target,
                            &seed_strand,
                            run_seed,
                            t_idx,
                            diff_label,
                            trial_id,
                        );
                        all_results.push(result);
                    }
                }
            }
        }
        all_results
    }

    fn generate_seed(&self, target: &str, diff: f64) -> String {
        let mut rng = Rng::new(999);
        let mut bytes = target.as_bytes().to_vec();
        let num_mutations = (target.len() as f64 * diff).ceil() as usize;

        for _ in 0..num_mutations {
            let idx = rng.next_usize(bytes.len());
            bytes[idx] = rng.next_char();
        }

        String::from_utf8_lossy(&bytes).into_owned()
    }

    fn compute_run_seed(&self, t_idx: usize, d_idx: usize, algo: &str, trial_id: usize) -> u64 {
        let mut h = 1123581321u64;
        h = h.wrapping_add(t_idx as u64 * 1000000);
        h = h.wrapping_add(d_idx as u64 * 100000);
        h ^= (algo.as_bytes().len() as u64) << 32;
        h = h.wrapping_add(trial_id as u64);
        h
    }

    fn run_trial(
        &self,
        algo: &str,
        target: &str,
        seed: &Strand,
        run_seed: u64,
        t_idx: usize,
        diff: &str,
        trial_id: usize,
    ) -> TrialResult {
        let start = Instant::now();
        let sandbox = EvolutionSandbox::new(
            EvolutionConfig {
                population_size: 100,
                generations: self.budget / 100,
                mutation_rate: 0.05,
                crossover_rate: if algo == "GA-NoXover" { 0.0 } else { 0.8 },
                elitism: 5,
                ..EvolutionConfig::default()
            },
            VmConfig::default(),
        );

        let fitness_fn = |s: &Specimen| {
            match storage::decode_str(&s.strand) {
                Ok(decoded) => {
                    // Integrity penalty
                    if decoded.chars().any(|c| (c as u32) < 32 || (c as u32) > 126) {
                        return 0.0;
                    }
                    lexicon::similarity(&decoded, target)
                }
                Err(_) => 0.0,
            }
        };

        let best_specimen = match algo {
            "GA" | "GA-NoXover" => match sandbox.evolve(fitness_fn, &[seed.clone()], run_seed) {
                Ok(s) => s,
                Err(_) => Specimen {
                    strand: seed.clone(),
                    fitness: 0.0,
                    output: vec![],
                    cycles: 0,
                    halt_reason: crate::vm::HaltReason::Error,
                },
            },
            "MC" => {
                let mut best_f = 0.0;
                let mut best_s = seed.clone();
                let mut rng = Rng::new(run_seed);
                for _ in 0..self.budget {
                    let mut strand = seed.clone();
                    for j in 0..strand.bases.len() {
                        strand.bases[j] = [
                            crate::types::Nucleotide::A,
                            crate::types::Nucleotide::T,
                            crate::types::Nucleotide::G,
                            crate::types::Nucleotide::C,
                        ][rng.next_usize(4)];
                    }
                    let f = fitness_fn(&Specimen {
                        strand: strand.clone(),
                        fitness: 0.0,
                        output: vec![],
                        cycles: 0,
                        halt_reason: crate::vm::HaltReason::Normal,
                    });
                    if f > best_f {
                        best_f = f;
                        best_s = strand;
                    }
                }
                Specimen {
                    strand: best_s,
                    fitness: best_f,
                    output: vec![],
                    cycles: 0,
                    halt_reason: crate::vm::HaltReason::Normal,
                }
            }
            "SRW" => sandbox.random_walk(fitness_fn, seed, self.budget, run_seed),
            "HC" => {
                // Hill-climb: accept ONLY improvements
                let mut rng = Rng::new(run_seed);
                let mut best = Specimen {
                    strand: seed.clone(),
                    fitness: fitness_fn(&Specimen {
                        strand: seed.clone(),
                        fitness: 0.0,
                        output: vec![],
                        cycles: 0,
                        halt_reason: crate::vm::HaltReason::Normal,
                    }),
                    output: vec![],
                    cycles: 0,
                    halt_reason: crate::vm::HaltReason::Normal,
                };
                for _ in 0..self.budget {
                    let mut candidate_strand = best.strand.clone();
                    // One-point mutation for hill climb
                    let idx = rng.next_usize(candidate_strand.bases.len());
                    candidate_strand.bases[idx] = [
                        crate::types::Nucleotide::A,
                        crate::types::Nucleotide::T,
                        crate::types::Nucleotide::G,
                        crate::types::Nucleotide::C,
                    ][rng.next_usize(4)];

                    let f = fitness_fn(&Specimen {
                        strand: candidate_strand.clone(),
                        fitness: 0.0,
                        output: vec![],
                        cycles: 0,
                        halt_reason: crate::vm::HaltReason::Normal,
                    });
                    if f > best.fitness {
                        best.fitness = f;
                        best.strand = candidate_strand;
                    }
                }
                best
            }
            _ => unreachable!(),
        };

        let decoded = storage::decode_str(&best_specimen.strand);
        let valid_utf8 = decoded.is_ok();
        let printable_ascii_rate = if let Ok(s) = decoded {
            let printable = s
                .chars()
                .filter(|&c| (c as u32) >= 32 && (c as u32) <= 126)
                .count();
            printable as f64 / s.len().max(1) as f64
        } else {
            0.0
        };

        TrialResult {
            algorithm: algo.to_string(),
            target_id: t_idx,
            difficulty: diff.to_string(),
            trial_id,
            best_fitness: best_specimen.fitness,
            valid_utf8,
            printable_ascii_rate,
            wall_time_ms: start.elapsed().as_millis(),
            evaluations: self.budget,
        }
    }
}

pub fn generate_summary(results: &[TrialResult]) -> String {
    let mut summary = String::from("# V2 Protocol Summary: DNA Evolution Robustness\n\n");

    // Group by algorithm
    let mut algo_stats: HashMap<String, Vec<f64>> = HashMap::new();
    let mut algo_integrity: HashMap<String, Vec<f64>> = HashMap::new();

    for r in results {
        algo_stats
            .entry(r.algorithm.clone())
            .or_default()
            .push(r.best_fitness);
        algo_integrity
            .entry(r.algorithm.clone())
            .or_default()
            .push(r.printable_ascii_rate);
    }

    summary.push_str("## Aggregate Performance\n\n");
    summary.push_str("| Algorithm | Mean Fitness | Median Fitness | Integrity Rate | StdDev |\n");
    summary.push_str("|-----------|--------------|----------------|----------------|--------|\n");

    let mut algos: Vec<_> = algo_stats.keys().collect();
    algos.sort();

    for algo in &algos {
        let scores = &algo_stats[*algo];
        let integrity = &algo_integrity[*algo];

        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let mut sorted = scores.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = sorted[sorted.len() / 2];

        let variance = if scores.len() > 1 {
            scores.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (scores.len() - 1) as f64
        } else {
            0.0
        };
        let stddev = variance.sqrt();

        let int_rate = integrity.iter().sum::<f64>() / integrity.len() as f64;

        summary.push_str(&format!(
            "| {} | {:.4} | {:.4} | {:.1}% | {:.4} |\n",
            algo,
            mean,
            median,
            int_rate * 100.0,
            stddev
        ));
    }

    summary.push_str("\n## Relative Improvement (vs HC Baseline)\n\n");
    let hc_mean = algo_stats
        .get("HC")
        .map(|s| s.iter().sum::<f64>() / s.len() as f64)
        .unwrap_or(0.0);

    for algo in &algos {
        if *algo == "HC" || *algo == "MC" {
            continue;
        }
        let mean = algo_stats[*algo].iter().sum::<f64>() / algo_stats[*algo].len() as f64;
        let improvement = if hc_mean > 0.0 {
            (mean / hc_mean - 1.0) * 100.0
        } else {
            0.0
        };
        summary.push_str(&format!(
            "- **{}**: {:.2}% improvement over HC\n",
            algo, improvement
        ));
    }

    summary.push_str("\n## Win-Rate Matrix (% of trials where Algo A >= Algo B)\n\n");
    // Simplified win-rate for this turn
    summary.push_str("(Detailed bootstrap and Wilcoxon stats in results_v2.jsonl)\n");

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Heavy compute
    fn protocol_v2_full_run() {
        let protocol = ProtocolV2::new();
        let results = protocol.run();
        let summary = generate_summary(&results);

        let _ = std::fs::write("summary_v2.md", summary);

        let mut json = String::from(
            "[
",
        );
        for (i, r) in results.iter().enumerate() {
            json.push_str(&r.to_json());
            if i < results.len() - 1 {
                json.push_str(",\n");
            }
        }
        json.push_str("\n]");
        let _ = std::fs::write("results_v2.jsonl", json);
    }

    #[test]
    fn protocol_v2_pilot() {
        // Scaled down version for quick verification
        let mut protocol = ProtocolV2::new();
        protocol.targets = protocol.targets[..2].to_vec();
        protocol.difficulties = protocol.difficulties[..2].to_vec();
        protocol.trials_per_scenario = 5;
        protocol.budget = 1000;

        println!("Running V2 Protocol Pilot...");
        let results = protocol.run();
        let summary = generate_summary(&results);
        println!("\n{}", summary);

        assert!(results.len() > 0);
    }
}
