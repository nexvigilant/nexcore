//! Evolution Sandbox: Directed evolution of DNA programs.
//!
//! This module provides a sandbox for evolving DNA strands that perform
//! specific computational tasks. It uses genetic algorithms to optimize
//! programs toward a target output or behavior.

use crate::error::Result;
use crate::types::{Codon, Nucleotide, Strand};
use crate::vm::{CodonVM, VmConfig, HaltReason};
use crate::isa::{self, Instruction};
use crate::cortex::{EvolutionConfig, Rng};
use std::fmt;

/// A member of the population in the evolution sandbox.
#[derive(Clone)]
pub struct Specimen {
    pub strand: Strand,
    pub fitness: f64,
    pub output: Vec<i64>,
    pub cycles: u64,
    pub halt_reason: HaltReason,
}

impl fmt::Debug for Specimen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Specimen(fitness={:.4}, output={:?}, cycles={})", self.fitness, self.output, self.cycles)
    }
}

pub struct EvolutionSandbox {
    config: EvolutionConfig,
    vm_config: VmConfig,
}

impl EvolutionSandbox {
    pub fn new(config: EvolutionConfig, vm_config: VmConfig) -> Self {
        Self { config, vm_config }
    }

    /// Evolve a population toward a target output.
    pub fn evolve<F>(&self, target_fitness: F, initial_seeds: &[Strand], seed: u64) -> Result<Specimen>
    where
        F: Fn(&Specimen) -> f64,
    {
        let mut rng = Rng::new(seed);
        let mut population = self.initialize_population(&mut rng, initial_seeds)?;

        let mut best_overall: Option<Specimen> = None;

        for generation_idx in 0..self.config.generations {
            // 1. Evaluate fitness
            for specimen in &mut population {
                specimen.fitness = target_fitness(specimen);
            }

            // 2. Sort by fitness
            population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));

            let current_best = population[0].clone();
            let dominated = match &best_overall {
                None => true,
                Some(prev) => current_best.fitness > prev.fitness,
            };
            if dominated {
                best_overall = Some(current_best.clone());
            }

            if generation_idx % 10 == 0 && self.config.generations > 10 {
                // println!("Gen {}: Best fitness = {:.4}", generation_idx, current_best.fitness);
            }

            // 3. Check for completion (perfect fitness)
            if current_best.fitness >= 1.0 {
                break;
            }

            // 4. Create next generation
            let mut next_gen = Vec::with_capacity(self.config.population_size);

            // Elitism
            for i in 0..self.config.elitism {
                if i < population.len() {
                    next_gen.push(population[i].clone());
                }
            }

            // Fill the rest
            while next_gen.len() < self.config.population_size {
                let parent1 = self.tournament_select(&population, &mut rng);
                let parent2 = self.tournament_select(&population, &mut rng);

                let mut child_strand = if rng.next_f64() < self.config.crossover_rate {
                    self.crossover(&parent1.strand, &parent2.strand, &mut rng)
                } else {
                    parent1.strand.clone()
                };

                self.mutate(&mut child_strand, &mut rng);
                next_gen.push(self.evaluate(child_strand));
            }

            population = next_gen;
        }

        Ok(best_overall.unwrap_or_else(|| Specimen {
            strand: Strand::new(vec![]),
            fitness: 0.0,
            output: vec![],
            cycles: 0,
            halt_reason: HaltReason::Error,
        }))
    }

    /// Seeded Random Walk (Hill-Climbing) Baseline
    ///
    /// Explores the space from a seed using only random mutations,
    /// accepting any change that improves or maintains fitness.
    pub fn random_walk<F>(&self, target_fitness: F, seed: &Strand, budget: usize, rng_seed: u64) -> Specimen 
    where F: Fn(&Specimen) -> f64
    {
        let mut rng = Rng::new(rng_seed);
        let mut best = self.evaluate(seed.clone());
        best.fitness = target_fitness(&best);

        for _ in 0..budget {
            let mut candidate_strand = best.strand.clone();
            self.mutate(&mut candidate_strand, &mut rng);
            let candidate = self.evaluate(candidate_strand);
            let f = target_fitness(&candidate);
            
            if f >= best.fitness {
                best = candidate;
                best.fitness = f;
            }
        }
        best
    }

    fn initialize_population(&self, rng: &mut Rng, seeds: &[Strand]) -> Result<Vec<Specimen>> {
        let mut population = Vec::with_capacity(self.config.population_size);

        // Add seeds
        for seed in seeds {
            population.push(self.evaluate(seed.clone()));
        }

        // Randomly fill the rest
        while population.len() < self.config.population_size {
            let strand = if seeds.is_empty() {
                self.random_strand(rng, 10) // Random 10-codon strand
            } else {
                let mut s = seeds[rng.next_usize(seeds.len())].clone();
                self.mutate(&mut s, rng);
                s
            };
            population.push(self.evaluate(strand));
        }

        Ok(population)
    }

    fn evaluate(&self, strand: Strand) -> Specimen {
        let mut vm = CodonVM::with_config(self.vm_config.clone());
        if vm.load(&strand).is_err() {
            return Specimen {
                strand,
                fitness: 0.0,
                output: vec![],
                cycles: 0,
                halt_reason: HaltReason::Error,
            };
        }

        match vm.execute() {
            Ok(res) => Specimen {
                strand,
                fitness: 0.0,
                output: res.output,
                cycles: res.cycles,
                halt_reason: res.halt_reason,
            },
            Err(_) => Specimen {
                strand,
                fitness: 0.0,
                output: vec![],
                cycles: 0,
                halt_reason: HaltReason::Error,
            },
        }
    }

    fn tournament_select<'a>(&self, population: &'a [Specimen], rng: &mut Rng) -> &'a Specimen {
        let mut best = &population[rng.next_usize(population.len())];
        for _ in 1..self.config.tournament_size {
            let next = &population[rng.next_usize(population.len())];
            if next.fitness > best.fitness {
                best = next;
            }
        }
        best
    }

    fn crossover(&self, a: &Strand, b: &Strand, rng: &mut Rng) -> Strand {
        let len_a = a.bases.len() / 3;
        let len_b = b.bases.len() / 3;
        let min_len = len_a.min(len_b);
        
        if min_len == 0 { return a.clone(); } 
        
        let point = rng.next_usize(min_len);
        let mut bases = Vec::with_capacity(a.bases.len());
        bases.extend_from_slice(&a.bases[..point * 3]);
        bases.extend_from_slice(&b.bases[point * 3..]);
        Strand::new(bases)
    }

    fn mutate(&self, strand: &mut Strand, rng: &mut Rng) {
        for i in 0..strand.bases.len() {
            if rng.next_f64() < self.config.mutation_rate {
                let nucleotides = [Nucleotide::A, Nucleotide::T, Nucleotide::G, Nucleotide::C];
                strand.bases[i] = nucleotides[rng.next_usize(4)];
            }
        }
    }

    fn random_strand(&self, rng: &mut Rng, codons: usize) -> Strand {
        let mut bases = Vec::with_capacity(codons * 3);
        let nucleotides = [Nucleotide::A, Nucleotide::T, Nucleotide::G, Nucleotide::C];
        for _ in 0..codons * 3 {
            bases.push(nucleotides[rng.next_usize(4)]);
        }
        Strand::new(bases)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_basic() {
        let sandbox = EvolutionSandbox::new(EvolutionConfig::default(), VmConfig::default());
        
        // Target: output the number 42
        let target_fitness = |s: &Specimen| {
            if s.output.is_empty() { return 0.0; }
            if s.output[0] == 42 { return 1.0; }
            
            // Partial credit for being close
            1.0 / (1.0 + (s.output[0] - 42).abs() as f64)
        };

        // Seed with a simple program: Entry, Lit 42, Output, Halt
        let seed_source = ".code\n  entry\n  lit 42\n  out\n  halt";
        let seed_prog = crate::asm::assemble(seed_source).unwrap();
        let seed_strand = seed_prog.strand().clone();

        let result = sandbox.evolve(target_fitness, &[seed_strand], 42).unwrap();
        assert!(result.fitness >= 1.0);
    }

    #[test]
    fn experiment_evolve_sequence_and_gc() {
        // Goal: Evolve a program that outputs [10, 20] AND has GC content ~50%
        let sandbox = EvolutionSandbox::new(EvolutionConfig {
            population_size: 200,
            generations: 200,
            mutation_rate: 0.15,
            ..EvolutionConfig::default()
        }, VmConfig::default());

        let target_fitness = |s: &Specimen| {
            // 1. Output score (0.0 to 0.8)
            let mut output_score = 0.0;
            if s.output.len() >= 1 {
                output_score += 0.4 / (1.0 + (s.output[0] - 10).abs() as f64);
            }
            if s.output.len() >= 2 {
                output_score += 0.4 / (1.0 + (s.output[1] - 20).abs() as f64);
            }

            // 2. GC score (0.0 to 0.2)
            let gc = crate::ops::gc_content(&s.strand);
            let gc_score = 0.2 * (1.0 - (gc - 0.5).abs() / 0.5).max(0.0);

            output_score + gc_score
        };

        // Start with some random-ish seeds that at least have an Entry and Halt
        let seed1 = crate::asm::assemble(".code\n entry\n halt").unwrap().strand().clone();
        let seed2 = crate::asm::assemble(".code\n entry\n lit 10\n out\n halt").unwrap().strand().clone();
        let seed3 = crate::asm::assemble(".code\n entry\n lit 10\n out\n lit 20\n out\n halt").unwrap().strand().clone();

        println!("\n--- Starting Sequence + GC Experiment ---");
        let result = sandbox.evolve(target_fitness, &[seed1, seed2, seed3], 123).unwrap();
        
        println!("Experiment Result:");
        println!("  Fitness: {:.4}", result.fitness);
        println!("  Output:  {:?}", result.output);
        println!("  GC:      {:.2}%", crate::ops::gc_content(&result.strand) * 100.0);
        println!("  Strand:  {}", result.strand.to_string_repr());
        
        assert!(result.fitness > 0.5);
    }

    #[test]
    fn experiment_evolve_belief_dna() {
        use crate::storage;

        // Goal: Evolve a "Belief" string toward a target truth
        let target_truth = "DNA is the soul of the machine";
        
        let sandbox = EvolutionSandbox::new(EvolutionConfig {
            population_size: 50,
            generations: 100,
            mutation_rate: 0.05,
            ..EvolutionConfig::default()
        }, VmConfig::default());

        let target_fitness = |s: &Specimen| {
            // Decode the strand back to text
            if let Ok(decoded) = storage::decode_str(&s.strand) {
                // Fitness is similarity to target truth
                crate::lexicon::similarity(&decoded, target_truth)
            } else {
                0.0
            }
        };

        // Seed with a random string of same byte length
        let initial_guess = "Random noise for the machine  ";
        let seed_strand = storage::encode_str(initial_guess);

        println!("\n--- Starting Belief Evolution Experiment ---");
        println!("Target Truth:  \"{}\"", target_truth);
        
        let result = sandbox.evolve(target_fitness, &[seed_strand], 777).unwrap();
        
        let final_belief = storage::decode_str(&result.strand).unwrap_or_default();
        println!("Experiment Result:");
        println!("  Fitness: {:.4}", result.fitness);
        println!("  Belief:  \"{}\"", final_belief);
        
        assert!(result.fitness > 0.3); // Some improvement expected
    }

    #[test]
    fn experiment_rigorous_robustness_analysis_v2() {
        use crate::storage;

        // Parameters
        let target = "DNA is robust";
        let initial = "XXX is broken";
        let pop_size = 100;
        let generations = 100;
        let trials = 10;
        let budget = pop_size * generations;

        println!("\n--- RIGOROUS ROBUSTNESS ANALYSIS V2 ---");
        println!("Target: \"{}\", Budget: {} evaluations", target, budget);

        let sandbox = EvolutionSandbox::new(EvolutionConfig {
            population_size: pop_size,
            generations,
            mutation_rate: 0.1, // More aggressive mutation
            crossover_rate: 0.8,
            elitism: 10,
            ..EvolutionConfig::default()
        }, VmConfig::default());

        let fitness_fn = |s: &Specimen| {
            match storage::decode_str(&s.strand) {
                Ok(decoded) => {
                    // Penalize non-printable/garbage characters (ASCII 32-126)
                    if decoded.chars().any(|c| (c as u32) < 32 || (c as u32) > 126) {
                        return 0.0;
                    }
                    crate::lexicon::similarity(&decoded, target)
                }
                Err(_) => 0.0,
            }
        };

        let mut ga_scores = Vec::new();
        let mut ga_integrity = 0usize;
        let mut srw_scores = Vec::new();
        let mut srw_integrity = 0usize;

        for i in 0..trials {
            let seed = storage::encode_str(initial);
            let trial_seed = (i + 100) as u64;

            // 1. Genetic Algorithm
            let ga_res = sandbox.evolve(fitness_fn, &[seed.clone()], trial_seed).unwrap();
            ga_scores.push(ga_res.fitness);
            if ga_res.fitness > 0.0 { ga_integrity += 1; }

            // 2. Seeded Random Walk (Hill Climbing)
            let srw_res = sandbox.random_walk(fitness_fn, &seed, budget, trial_seed);
            srw_scores.push(srw_res.fitness);
            if srw_res.fitness > 0.0 { srw_integrity += 1; }

            println!("Trial {}: GA={:.4}, SRW={:.4}", i, ga_res.fitness, srw_res.fitness);
        }

        let mean_ga = ga_scores.iter().sum::<f64>() / trials as f64;
        let mean_srw = srw_scores.iter().sum::<f64>() / trials as f64;
        
        let ga_int_rate = (ga_integrity as f64 / trials as f64) * 100.0;
        let srw_int_rate = (srw_integrity as f64 / trials as f64) * 100.0;

        println!("\nFinal Statistics ({} trials):", trials);
        println!("  GA: Mean Fitness={:.4}, Integrity={:.1}%", mean_ga, ga_int_rate);
        println!("  SRW: Mean Fitness={:.4}, Integrity={:.1}%", mean_srw, srw_int_rate);
        
        if mean_srw > 0.0 {
            println!("  Relative Improvement (GA vs SRW): {:.2}%", (mean_ga / mean_srw - 1.0) * 100.0);
        } else {
            println!("  Relative Improvement (GA vs SRW): +inf% (SRW failed)");
        }

        assert!(mean_ga >= mean_srw, "Genetic Algorithm should perform at least as well as Random Walk");
    }
}
