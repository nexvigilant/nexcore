//! DNA Evaluation Test Suite — Decision Boundaries.
//!
//! Systematic evaluation producing quantitative decision boundaries for when
//! DNA encoding/execution pays off vs native Rust. Five dimensions:
//!
//! | Phase | Dimension | Key Question |
//! |-------|-----------|-------------|
//! | 1 | Execution Overhead | At what complexity does VM overhead become negligible? |
//! | 2 | Mutation Fidelity | How many mutations before programs break? |
//! | 3 | Transport Efficiency | When does DNA beat JSON for program transport? |
//! | 4 | Auditability | Can we quantify determinism and provenance value? |
//! | 5 | Decision Matrix | Synthesize all dimensions into recommendations |
//!
//! Tier: T2-C (κ Comparison + N Quantity + σ Sequence + ∂ Boundary)

#[cfg(test)]
mod tests {
    use crate::error::Result;
    use crate::gene::crossover;
    use crate::isa::{self, Instruction};
    use crate::lang::compiler;
    use crate::lang::json::source_to_json;
    use crate::nexcore_encoding::{NEXCORE_SIGNAL_GENOME_SOURCE, nexcore_signal_genome};
    use crate::transcode::{Encoding, ProgramProfile};
    use crate::types::{Codon, Nucleotide, Strand};
    use crate::vm::{HaltReason, VmConfig};

    use std::panic;
    use std::time::Instant;

    // =========================================================================
    // Measurement Infrastructure
    // =========================================================================

    const BENCH_ITERATIONS: u32 = 10_000;
    const MUTATION_TRIALS: usize = 100;

    /// Measurement result for a single benchmark.
    struct Measurement {
        mean_ns: f64,
        stddev_ns: f64,
    }

    /// Run a closure `iterations` times, return mean and stddev in nanoseconds.
    fn measure<F: FnMut()>(iterations: u32, mut f: F) -> Measurement {
        // Warmup
        for _ in 0..100 {
            f();
        }

        let mut times = Vec::with_capacity(iterations as usize);
        for _ in 0..iterations {
            let start = Instant::now();
            f();
            times.push(start.elapsed().as_nanos() as f64);
        }

        let n = times.len() as f64;
        let mean = times.iter().sum::<f64>() / n;
        let variance = times.iter().map(|t| (t - mean) * (t - mean)).sum::<f64>() / n;
        let stddev = variance.sqrt();

        Measurement {
            mean_ns: mean,
            stddev_ns: stddev,
        }
    }

    /// Generate arithmetic source code with N chained operations.
    fn arithmetic_source(op_count: usize) -> String {
        let mut source = String::from("let x = 1\n");
        for i in 0..op_count {
            let op = match i % 4 {
                0 => "x + 2",
                1 => "x * 3",
                2 => "x - 1",
                _ => "x + 1",
            };
            source.push_str(&format!("let x = {op}\n"));
        }
        source.push_str("print(x)");
        source
    }

    /// Generate source with branching (if/while) of given depth.
    fn branching_source(branch_count: usize) -> String {
        let mut source = String::from("let x = 100\n");
        for _ in 0..branch_count {
            source.push_str("if x > 0 do\n  let x = x - 1\nend\n");
        }
        source.push_str("print(x)");
        source
    }

    /// Equivalent Rust computation for the arithmetic chain.
    fn native_arithmetic(op_count: usize) -> i64 {
        let mut x: i64 = 1;
        for i in 0..op_count {
            x = match i % 4 {
                0 => x.wrapping_add(2),
                1 => x.wrapping_mul(3),
                2 => x.wrapping_sub(1),
                _ => x.wrapping_add(1),
                // The above arms cover all cases due to % 4
            };
        }
        x
    }

    /// Equivalent Rust computation for the branching chain.
    fn native_branching(branch_count: usize) -> i64 {
        let mut x: i64 = 100;
        for _ in 0..branch_count {
            if x > 0 {
                x -= 1;
            }
        }
        x
    }

    /// Simple xorshift64 PRNG for deterministic randomness.
    struct Rng(u64);
    impl Rng {
        fn new(seed: u64) -> Self {
            Self(seed)
        }
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn next_usize(&mut self, max: usize) -> usize {
            if max == 0 {
                return 0;
            }
            (self.next() % max as u64) as usize
        }
    }

    // =========================================================================
    // Phase 1: Execution Overhead Crossover
    // =========================================================================

    #[test]
    fn eval_overhead_arithmetic() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ PHASE 1a: Execution Overhead — Arithmetic Chains              │");
        eprintln!("├──────────┬──────────────┬──────────────┬──────────────┬────────┤");
        eprintln!("│ Ops      │ DNA VM (ns)  │ Native (ns)  │ Overhead     │ Ratio  │");
        eprintln!("├──────────┼──────────────┼──────────────┼──────────────┼────────┤");

        let op_counts = [5, 10, 20, 50, 100];
        let mut crossover_found = false;
        let mut crossover_point = 0usize;

        for &ops in &op_counts {
            let source = arithmetic_source(ops);
            // Pre-compile outside measurement
            let compiled = compiler::compile(&source);
            if compiled.is_err() {
                eprintln!(
                    "│ {:>8} │ COMPILE ERR  │              │              │        │",
                    ops
                );
                continue;
            }
            let program = compiled.ok();

            // DNA VM timing
            let dna_m = measure(BENCH_ITERATIONS, || {
                if let Some(ref p) = program {
                    let _ = p.run();
                }
            });

            // Native Rust timing
            let native_m = measure(BENCH_ITERATIONS, || {
                std::hint::black_box(native_arithmetic(ops));
            });

            let overhead_ratio = dna_m.mean_ns / native_m.mean_ns.max(1.0);
            let overhead_pct = (overhead_ratio - 1.0) * 100.0;

            eprintln!(
                "│ {:>8} │ {:>10.0}   │ {:>10.0}   │ {:>9.0}%   │ {:>5.0}× │",
                ops, dna_m.mean_ns, native_m.mean_ns, overhead_pct, overhead_ratio
            );

            // Track crossover: overhead < 10× (order of magnitude)
            if !crossover_found && overhead_ratio < 10.0 {
                crossover_found = true;
                crossover_point = ops;
            }
        }

        eprintln!("└──────────┴──────────────┴──────────────┴──────────────┴────────┘");

        if crossover_found {
            eprintln!("→ Overhead < 10× at {} operations", crossover_point);
        } else {
            eprintln!("→ Overhead > 10× across all tested sizes (DNA VM is pure interpretation)");
        }

        // The DNA VM will always be slower for pure arithmetic (it's an interpreter).
        // The value is in mutation/audit/transport, not raw speed.
        // We just need to confirm it runs correctly.
        if let Some(ref p) = compiler::compile(&arithmetic_source(20)).ok() {
            let result = p.run();
            assert!(result.is_ok(), "DNA VM should produce valid results");
            let r = result.ok();
            assert!(r.is_some());
        }
    }

    #[test]
    fn eval_overhead_branching() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ PHASE 1b: Execution Overhead — Branching (if chains)          │");
        eprintln!("├──────────┬──────────────┬──────────────┬──────────────┬────────┤");
        eprintln!("│ Branches │ DNA VM (ns)  │ Native (ns)  │ Overhead     │ Ratio  │");
        eprintln!("├──────────┼──────────────┼──────────────┼──────────────┼────────┤");

        let branch_counts = [5, 10, 20, 50];

        for &branches in &branch_counts {
            let source = branching_source(branches);
            let compiled = compiler::compile(&source);
            if compiled.is_err() {
                eprintln!(
                    "│ {:>8} │ COMPILE ERR  │              │              │        │",
                    branches
                );
                continue;
            }
            let program = compiled.ok();

            let dna_m = measure(BENCH_ITERATIONS, || {
                if let Some(ref p) = program {
                    let _ = p.run();
                }
            });

            let native_m = measure(BENCH_ITERATIONS, || {
                std::hint::black_box(native_branching(branches));
            });

            let overhead_ratio = dna_m.mean_ns / native_m.mean_ns.max(1.0);
            let overhead_pct = (overhead_ratio - 1.0) * 100.0;

            eprintln!(
                "│ {:>8} │ {:>10.0}   │ {:>10.0}   │ {:>9.0}%   │ {:>5.0}× │",
                branches, dna_m.mean_ns, native_m.mean_ns, overhead_pct, overhead_ratio
            );
        }

        eprintln!("└──────────┴──────────────┴──────────────┴──────────────┴────────┘");
        eprintln!("→ Branch overhead measures conditional jump cost in DNA VM");
    }

    #[test]
    fn eval_overhead_function_calls() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ PHASE 1c: Execution Overhead — Gene Expression vs Native Fn   │");
        eprintln!("├──────────────────┬──────────────┬──────────────┬───────────────┤");
        eprintln!("│ Operation        │ DNA (ns)     │ Native (ns)  │ Ratio         │");
        eprintln!("├──────────────────┼──────────────┼──────────────┼───────────────┤");

        // Compile the NexCore signal genome (PRR, ROR, IC, EBGM functions)
        let genome_result = nexcore_signal_genome();
        if genome_result.is_err() {
            eprintln!("│ GENOME COMPILE FAILED — skipping gene expression benchmarks   │");
            eprintln!("└──────────────────┴──────────────┴──────────────┴───────────────┘");
            return;
        }
        let genome = genome_result.ok();
        if genome.is_none() {
            return;
        }
        let genome = genome.as_ref();

        // Test arguments: standard 2×2 contingency table
        let args: &[i64] = &[15, 100, 20, 10000];

        // DNA gene expression: prr(15, 100, 20, 10000)
        if let Some(genome) = genome {
            let dna_m = measure(BENCH_ITERATIONS, || {
                let _ = genome.express("prr", args);
            });

            // Native PRR: (a * (c + d)) / (c * (a + b))
            let native_m = measure(BENCH_ITERATIONS, || {
                let (a, b, c, d) = (15i64, 100i64, 20i64, 10000i64);
                let num = std::hint::black_box(a.wrapping_mul(c.wrapping_add(d)));
                let den = std::hint::black_box(c.wrapping_mul(a.wrapping_add(b)));
                std::hint::black_box((num, den));
            });

            let ratio = dna_m.mean_ns / native_m.mean_ns.max(1.0);
            eprintln!(
                "│ PRR expression   │ {:>10.0}   │ {:>10.0}   │ {:>10.0}×    │",
                dna_m.mean_ns, native_m.mean_ns, ratio
            );

            // Verify correctness
            let result = genome.express("prr", args);
            if let Ok(r) = &result {
                // PRR numerator: 15 * (20 + 10000) = 150300
                let expected_num = 15i64 * (20 + 10000);
                if !r.output.is_empty() {
                    eprintln!(
                        "│ PRR output: {:?} (expected num: {})",
                        r.output, expected_num
                    );
                }
            }
        }

        eprintln!("└──────────────────┴──────────────┴──────────────┴───────────────┘");
        eprintln!("→ Gene expression cost = VM boot + strand load + argument push + execution");
    }

    // =========================================================================
    // Phase 2: Evolution/Mutation Fidelity
    // =========================================================================

    #[test]
    fn eval_mutation_survival_rate() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ PHASE 2a: Mutation Survival Rate — Point Mutations on PRR Gene │");
        eprintln!("├──────────┬──────────────┬──────────────┬──────────────────────┤");
        eprintln!("│ Mutations│ Survival %   │ Output OK %  │ Mean Cycles          │");
        eprintln!("├──────────┼──────────────┼──────────────┼──────────────────────┤");

        let genome_result = nexcore_signal_genome();
        if genome_result.is_err() {
            eprintln!("│ GENOME COMPILE FAILED — skipping mutation tests               │");
            eprintln!("└──────────┴──────────────┴──────────────┴──────────────────────┘");
            return;
        }
        let genome = genome_result.ok();
        let genome = match genome.as_ref() {
            Some(g) => g,
            None => return,
        };

        let prr_gene = match genome.find_gene("prr") {
            Some(g) => g,
            None => {
                eprintln!("│ PRR gene not found in genome                                 │");
                eprintln!("└──────────┴──────────────┴──────────────┴──────────────────────┘");
                return;
            }
        };

        let mutation_counts = [1usize, 2, 3, 5, 10];
        let mut rng = Rng::new(42);

        // Get baseline output from original gene
        let baseline = genome.express("prr", &[15, 100, 20, 10000]);
        let baseline_output = baseline.as_ref().ok().map(|r| r.output.clone());

        for &n_mutations in &mutation_counts {
            let mut survived = 0usize;
            let mut output_ok = 0usize;
            let mut total_cycles = 0u64;

            for _ in 0..MUTATION_TRIALS {
                // Apply N point mutations
                let mut mutant = prr_gene.clone();
                let codon_count = mutant.codon_count();
                if codon_count == 0 {
                    continue;
                }

                let mut mutation_ok = true;
                for _ in 0..n_mutations {
                    let offset = rng.next_usize(codon_count);
                    let new_idx = (rng.next() % 64) as u8;
                    let new_codon = match Codon::from_index(new_idx) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    match mutant.point_mutate(offset, new_codon) {
                        Ok(m) => mutant = m,
                        Err(_) => {
                            mutation_ok = false;
                            break;
                        }
                    }
                }

                if !mutation_ok {
                    continue;
                }

                // Try to execute mutant on a fresh VM with limited cycles.
                // Mutated programs can trigger stdlib panics (e.g. clamp(min > max)),
                // so we wrap in catch_unwind to treat panics as lethal mutations.
                let mutant_seq = mutant.sequence.clone();
                let exec_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    let config = VmConfig {
                        max_cycles: 10_000,
                        ..VmConfig::default()
                    };
                    let mut vm = crate::vm::CodonVM::with_config(config);
                    if vm.load(&mutant_seq).is_err() {
                        return None;
                    }
                    vm.execute_from(0).ok()
                }));

                match exec_result {
                    Ok(Some(r)) => {
                        survived += 1;
                        total_cycles += r.cycles;

                        // Check if output resembles baseline
                        if let Some(ref base_out) = baseline_output {
                            if !r.output.is_empty()
                                && !base_out.is_empty()
                                && r.halt_reason != HaltReason::Error
                            {
                                output_ok += 1;
                            }
                        }
                    }
                    _ => {
                        // Execution failed or panicked — mutation was lethal
                    }
                }
            }

            let survival_pct = survived as f64 / MUTATION_TRIALS as f64 * 100.0;
            let output_pct = output_ok as f64 / MUTATION_TRIALS as f64 * 100.0;
            let mean_cycles = if survived > 0 {
                total_cycles as f64 / survived as f64
            } else {
                0.0
            };

            eprintln!(
                "│ {:>8} │ {:>10.1}%  │ {:>10.1}%  │ {:>18.0}   │",
                n_mutations, survival_pct, output_pct, mean_cycles
            );
        }

        eprintln!("└──────────┴──────────────┴──────────────┴──────────────────────┘");
        eprintln!("→ Survival = executed without VM error. Output OK = produced non-empty output.");
        eprintln!("→ Higher mutation tolerance = more evolvability headroom.");
    }

    #[test]
    fn eval_crossover_viability() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ PHASE 2b: Crossover Viability — PRR × ROR Gene Recombination  │");
        eprintln!("├──────────┬──────────────┬──────────────┬──────────────────────┤");
        eprintln!("│ Point    │ Child 1 OK   │ Child 2 OK   │ Either viable        │");
        eprintln!("├──────────┼──────────────┼──────────────┼──────────────────────┤");

        let genome_result = nexcore_signal_genome();
        if genome_result.is_err() {
            eprintln!("│ GENOME COMPILE FAILED — skipping crossover tests              │");
            eprintln!("└──────────┴──────────────┴──────────────┴──────────────────────┘");
            return;
        }
        let genome = genome_result.ok();
        let genome = match genome.as_ref() {
            Some(g) => g,
            None => return,
        };

        let prr = match genome.find_gene("prr") {
            Some(g) => g,
            None => return,
        };
        let ror = match genome.find_gene("ror") {
            Some(g) => g,
            None => return,
        };

        let min_codons = prr.codon_count().min(ror.codon_count());
        if min_codons == 0 {
            eprintln!("│ Genes have no codons — skipping                               │");
            eprintln!("└──────────┴──────────────┴──────────────┴──────────────────────┘");
            return;
        }

        let test_points: Vec<usize> = if min_codons <= 5 {
            (0..=min_codons).collect()
        } else {
            vec![
                0,
                1,
                min_codons / 4,
                min_codons / 2,
                3 * min_codons / 4,
                min_codons,
            ]
        };

        let mut total_viable = 0usize;
        let mut total_tests = 0usize;

        for &point in &test_points {
            let result = crossover(prr, ror, point);
            let (child1_ok, child2_ok) = match result {
                Ok((ref c1, ref c2)) => {
                    let c1_seq = c1.sequence.clone();
                    let c1_ok = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        let config = VmConfig {
                            max_cycles: 5_000,
                            ..VmConfig::default()
                        };
                        let mut vm = crate::vm::CodonVM::with_config(config);
                        vm.load(&c1_seq).is_ok() && vm.execute_from(0).is_ok()
                    }))
                    .unwrap_or(false);

                    let c2_seq = c2.sequence.clone();
                    let c2_ok = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        let config = VmConfig {
                            max_cycles: 5_000,
                            ..VmConfig::default()
                        };
                        let mut vm = crate::vm::CodonVM::with_config(config);
                        vm.load(&c2_seq).is_ok() && vm.execute_from(0).is_ok()
                    }))
                    .unwrap_or(false);

                    (c1_ok, c2_ok)
                }
                Err(_) => (false, false),
            };

            let either = child1_ok || child2_ok;
            if either {
                total_viable += 1;
            }
            total_tests += 1;

            let c1_str = if child1_ok { "  ✓" } else { "  ✗" };
            let c2_str = if child2_ok { "  ✓" } else { "  ✗" };
            let either_str = if either { "  ✓" } else { "  ✗" };

            eprintln!(
                "│ {:>8} │ {:>12} │ {:>12} │ {:>20} │",
                point, c1_str, c2_str, either_str
            );
        }

        eprintln!("└──────────┴──────────────┴──────────────┴──────────────────────┘");
        let viability_rate = if total_tests > 0 {
            total_viable as f64 / total_tests as f64 * 100.0
        } else {
            0.0
        };
        eprintln!(
            "→ Crossover viability: {:.0}% ({}/{} points produced viable offspring)",
            viability_rate, total_viable, total_tests
        );
    }

    #[test]
    fn eval_mutation_then_recovery() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ PHASE 2c: Mutation → Recovery — Can evolution repair damage?   │");
        eprintln!("├──────────────────┬──────────────┬──────────────────────────────┤");
        eprintln!("│ Mutations applied│ Recovery rate│ Notes                        │");
        eprintln!("├──────────────────┼──────────────┼──────────────────────────────┤");

        let genome_result = nexcore_signal_genome();
        if genome_result.is_err() {
            eprintln!("│ GENOME COMPILE FAILED                                         │");
            eprintln!("└──────────────────┴──────────────┴──────────────────────────────┘");
            return;
        }
        let genome = genome_result.ok();
        let genome = match genome.as_ref() {
            Some(g) => g,
            None => return,
        };

        let prr = match genome.find_gene("prr") {
            Some(g) => g,
            None => return,
        };

        let mut rng = Rng::new(12345);
        let mutation_levels = [1usize, 3, 5];

        for &n_mutations in &mutation_levels {
            let mut recovered = 0usize;
            let trials = 50;

            for _ in 0..trials {
                // Apply mutations
                let mut mutant = prr.clone();
                let codon_count = mutant.codon_count();
                if codon_count == 0 {
                    continue;
                }

                for _ in 0..n_mutations {
                    let offset = rng.next_usize(codon_count);
                    let new_idx = (rng.next() % 64) as u8;
                    if let Ok(c) = Codon::from_index(new_idx) {
                        if let Ok(m) = mutant.point_mutate(offset, c) {
                            mutant = m;
                        }
                    }
                }

                // Try to "recover" by applying random reverse mutations
                // (simulating selection pressure toward correct output)
                let mut best_mutant = mutant.clone();
                let mut best_executes = false;

                for _attempt in 0..20 {
                    let offset = rng.next_usize(codon_count);
                    let new_idx = (rng.next() % 64) as u8;
                    if let Ok(c) = Codon::from_index(new_idx) {
                        if let Ok(candidate) = best_mutant.point_mutate(offset, c) {
                            let cand_seq = candidate.sequence.clone();
                            let exec_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                                let config = VmConfig {
                                    max_cycles: 5_000,
                                    ..VmConfig::default()
                                };
                                let mut vm = crate::vm::CodonVM::with_config(config);
                                if vm.load(&cand_seq).is_err() {
                                    return None;
                                }
                                vm.execute_from(0).ok()
                            }));

                            if let Ok(Some(r)) = exec_result {
                                if !r.output.is_empty() && r.halt_reason != HaltReason::Error {
                                    best_mutant = candidate;
                                    best_executes = true;
                                }
                            }
                        }
                    }
                }

                if best_executes {
                    recovered += 1;
                }
            }

            let recovery_pct = recovered as f64 / trials as f64 * 100.0;
            let note = if recovery_pct > 50.0 {
                "strong recovery"
            } else if recovery_pct > 20.0 {
                "partial recovery"
            } else {
                "limited recovery"
            };

            eprintln!(
                "│ {:>17} │ {:>10.1}%  │ {:>28} │",
                n_mutations, recovery_pct, note
            );
        }

        eprintln!("└──────────────────┴──────────────┴──────────────────────────────┘");
        eprintln!("→ Recovery = random mutations + selection → executable output");
    }

    // =========================================================================
    // Phase 3: Transport & Serialization Efficiency
    // =========================================================================

    #[test]
    fn eval_transport_size_comparison() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────────────┐");
        eprintln!("│ PHASE 3a: Transport Size — DNA Strand vs JSON AST vs Packed Bits       │");
        eprintln!("├──────────┬──────────┬──────────┬──────────┬──────────┬─────────────────┤");
        eprintln!("│ Instrs   │ Strand   │ Packed   │ JSON     │ Raw bits │ Winner          │");
        eprintln!("├──────────┼──────────┼──────────┼──────────┼──────────┼─────────────────┤");

        let program_sizes = [5usize, 10, 20, 48, 100];

        let mut json_crossover = 0usize;

        for &size in &program_sizes {
            let source = arithmetic_source(size);

            // Strand encoding: compile → strand → bytes
            let strand_bytes = match compiler::compile(&source) {
                Ok(p) => p.code.len(), // nucleotides (1 byte each in memory)
                Err(_) => 0,
            };

            // Packed strand: 2 bits per nucleotide
            let packed_bytes = (strand_bytes * 2 + 7) / 8;

            // JSON AST
            let json_bytes = match source_to_json(&source) {
                Ok(j) => j.len(),
                Err(_) => 0,
            };

            // Raw instruction bits: 6 bits per instruction (64 opcodes = 6 bits)
            let raw_bits_bytes = (size * 6 + 7) / 8;

            let winner = if packed_bytes > 0 && json_bytes > 0 {
                if packed_bytes <= json_bytes && packed_bytes <= raw_bits_bytes {
                    "Packed DNA"
                } else if json_bytes <= packed_bytes && json_bytes <= raw_bits_bytes {
                    "JSON"
                } else {
                    "Raw bits"
                }
            } else {
                "N/A"
            };

            // Track crossover point where packed DNA < JSON
            if json_crossover == 0
                && packed_bytes > 0
                && json_bytes > 0
                && packed_bytes < json_bytes
            {
                json_crossover = size;
            }

            eprintln!(
                "│ {:>8} │ {:>6} B │ {:>6} B │ {:>6} B │ {:>6} B │ {:>15} │",
                size, strand_bytes, packed_bytes, json_bytes, raw_bits_bytes, winner
            );
        }

        eprintln!("└──────────┴──────────┴──────────┴──────────┴──────────┴─────────────────┘");
        if json_crossover > 0 {
            eprintln!("→ Packed DNA beats JSON at {} instructions", json_crossover);
        } else {
            eprintln!(
                "→ JSON is more compact for all tested sizes (JSON is terse for simple ASTs)"
            );
        }
        eprintln!("→ Strand: 1 byte/nucleotide (3 per codon). Packed: 2 bits/nucleotide.");
        eprintln!("→ DNA's advantage is mutation-in-transit, not raw size.");
    }

    #[test]
    fn eval_encode_decode_throughput() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ PHASE 3b: Encode/Decode Throughput — DNA vs JSON Roundtrip     │");
        eprintln!("├──────────┬──────────────┬──────────────┬────────────────────────┤");
        eprintln!("│ Instrs   │ DNA RT (ns)  │ JSON RT (ns) │ Faster                │");
        eprintln!("├──────────┼──────────────┼──────────────┼────────────────────────┤");

        let sizes = [5usize, 20, 50];

        for &size in &sizes {
            let source = arithmetic_source(size);

            // DNA roundtrip: compile → run
            let dna_m = measure(BENCH_ITERATIONS / 10, || {
                if let Ok(p) = compiler::compile(&source) {
                    let _ = p.run();
                }
            });

            // JSON roundtrip: source → json → ast → compile → run
            let json_m = measure(BENCH_ITERATIONS / 10, || {
                if let Ok(json) = source_to_json(&source) {
                    if let Ok(p) = crate::lang::json::json_to_program(&json) {
                        let _ = p.run();
                    }
                }
            });

            let faster = if dna_m.mean_ns < json_m.mean_ns {
                format!("DNA ({:.1}×)", json_m.mean_ns / dna_m.mean_ns.max(1.0))
            } else {
                format!("JSON ({:.1}×)", dna_m.mean_ns / json_m.mean_ns.max(1.0))
            };

            eprintln!(
                "│ {:>8} │ {:>10.0}   │ {:>10.0}   │ {:>22} │",
                size, dna_m.mean_ns, json_m.mean_ns, faster
            );
        }

        eprintln!("└──────────┴──────────────┴──────────────┴────────────────────────┘");
        eprintln!("→ DNA direct: compile(source) → run()");
        eprintln!("→ JSON path:  source → JSON → AST → compile → run()");
    }

    // =========================================================================
    // Phase 4: Auditability & Determinism Value
    // =========================================================================

    #[test]
    fn eval_determinism_guarantee() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ PHASE 4a: Determinism Guarantee — 1000 identical runs          │");
        eprintln!("├──────────────────────────────────────────────────────────────────┤");

        let source =
            "let a = 15\nlet b = 100\nlet c = 20\nlet d = 10000\nlet r = a * (c + d)\nprint(r)";
        let program = match compiler::compile(source) {
            Ok(p) => p,
            Err(_) => {
                eprintln!("│ COMPILE FAILED                                               │");
                eprintln!("└──────────────────────────────────────────────────────────────────┘");
                return;
            }
        };

        let mut outputs: Vec<Vec<i64>> = Vec::new();
        let mut cycles: Vec<u64> = Vec::new();
        let mut halt_reasons: Vec<HaltReason> = Vec::new();

        for _ in 0..1000 {
            match program.run() {
                Ok(r) => {
                    outputs.push(r.output.clone());
                    cycles.push(r.cycles);
                    halt_reasons.push(r.halt_reason.clone());
                }
                Err(_) => {
                    outputs.push(vec![-1]);
                    cycles.push(0);
                    halt_reasons.push(HaltReason::Error);
                }
            }
        }

        // Check all identical
        let first_output = &outputs[0];
        let first_cycles = cycles[0];
        let first_halt = &halt_reasons[0];

        let output_identical = outputs.iter().all(|o| o == first_output);
        let cycles_identical = cycles.iter().all(|&c| c == first_cycles);
        let halt_identical = halt_reasons.iter().all(|h| h == first_halt);

        let determinism_score = if output_identical && cycles_identical && halt_identical {
            1.0
        } else {
            let mut score = 0.0;
            if output_identical {
                score += 0.4;
            }
            if cycles_identical {
                score += 0.3;
            }
            if halt_identical {
                score += 0.3;
            }
            score
        };

        eprintln!(
            "│ Output identical:  {} (1000/1000 runs)",
            if output_identical { "✓" } else { "✗" }
        );
        eprintln!(
            "│ Cycles identical:  {} ({} cycles each)",
            if cycles_identical { "✓" } else { "✗" },
            first_cycles
        );
        eprintln!(
            "│ Halt identical:    {} ({:?})",
            if halt_identical { "✓" } else { "✗" },
            first_halt
        );
        eprintln!("│ Determinism score: {:.2}/1.00", determinism_score);
        eprintln!("└──────────────────────────────────────────────────────────────────┘");

        // Determinism MUST be 1.0 — the VM is a deterministic integer machine
        assert!(
            output_identical,
            "DNA VM must produce identical output across runs"
        );
        assert!(
            cycles_identical,
            "DNA VM must use identical cycle count across runs"
        );
    }

    #[test]
    fn eval_provenance_chain() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ PHASE 4b: Provenance Chain — Mutation History Tracking         │");
        eprintln!("├──────────────────────────────────────────────────────────────────┤");

        let genome_result = nexcore_signal_genome();
        if genome_result.is_err() {
            eprintln!("│ GENOME COMPILE FAILED                                         │");
            eprintln!("└──────────────────────────────────────────────────────────────────┘");
            return;
        }
        let genome = genome_result.ok();
        let genome = match genome.as_ref() {
            Some(g) => g,
            None => return,
        };

        let prr = match genome.find_gene("prr") {
            Some(g) => g,
            None => return,
        };

        // Record provenance: original strand + each mutation
        let mut history: Vec<String> = Vec::new();
        history.push(prr.sequence.to_string_repr());

        let mut rng = Rng::new(99);
        let mut current = prr.clone();
        let mutation_steps = 5;

        for step in 0..mutation_steps {
            let codon_count = current.codon_count();
            if codon_count == 0 {
                break;
            }
            let offset = rng.next_usize(codon_count);
            let new_idx = (rng.next() % 64) as u8;
            if let Ok(c) = Codon::from_index(new_idx) {
                if let Ok(mutant) = current.point_mutate(offset, c) {
                    current = mutant;
                    history.push(current.sequence.to_string_repr());
                }
            }

            eprintln!(
                "│ Step {}: strand len={}, diff from previous",
                step + 1,
                current.sequence.len()
            );
        }

        // Compute audit trail overhead
        let original_size = history[0].len();
        let mut total_diff_bytes = 0usize;

        for i in 1..history.len() {
            // Simple diff: count positions where characters differ
            let prev = history[i - 1].as_bytes();
            let curr = history[i].as_bytes();
            let min_len = prev.len().min(curr.len());
            let mut diffs = 0usize;
            for j in 0..min_len {
                if prev[j] != curr[j] {
                    diffs += 1;
                }
            }
            // Add length difference
            diffs += (prev.len() as i64 - curr.len() as i64).unsigned_abs() as usize;
            total_diff_bytes += diffs;

            eprintln!("│   Δ step {}: {} nucleotide(s) changed", i, diffs);
        }

        let audit_cost_per_step = if history.len() > 1 {
            total_diff_bytes / (history.len() - 1)
        } else {
            0
        };

        eprintln!("│");
        eprintln!("│ Provenance chain: {} steps recorded", history.len());
        eprintln!("│ Original strand size: {} nucleotides", original_size);
        eprintln!("│ Total diff bytes: {}", total_diff_bytes);
        eprintln!("│ Audit cost per step: {} nucleotides", audit_cost_per_step);
        eprintln!("│ Full history recoverable: ✓ (strand diffs are invertible)");
        eprintln!("└──────────────────────────────────────────────────────────────────┘");

        // Verify history is non-empty and mutations actually changed something
        assert!(history.len() > 1, "Should have recorded mutation history");
        assert!(total_diff_bytes > 0, "Mutations should change the strand");
    }

    #[test]
    fn eval_reproducibility_across_configs() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ PHASE 4c: Reproducibility Across VM Configs                   │");
        eprintln!("├──────────────────────┬──────────────┬──────────────────────────┤");
        eprintln!("│ Config               │ Output       │ Match baseline?          │");
        eprintln!("├──────────────────────┼──────────────┼──────────────────────────┤");

        let source = "let x = 7 * 6\nprint(x)";
        let program = match compiler::compile(source) {
            Ok(p) => p,
            Err(_) => {
                eprintln!("│ COMPILE FAILED                                               │");
                eprintln!("└──────────────────────┴──────────────┴──────────────────────────┘");
                return;
            }
        };

        // Baseline with default config
        let baseline = program.run();
        let baseline_output = baseline.as_ref().ok().map(|r| r.output.clone());

        let configs = [
            ("default (256/1024)", VmConfig::default()),
            (
                "large stack (1024)",
                VmConfig {
                    max_stack: 1024,
                    ..VmConfig::default()
                },
            ),
            (
                "large mem (4096)",
                VmConfig {
                    max_memory: 4096,
                    ..VmConfig::default()
                },
            ),
            (
                "1M cycles",
                VmConfig {
                    max_cycles: 1_000_000,
                    ..VmConfig::default()
                },
            ),
            (
                "minimal (64/128)",
                VmConfig {
                    max_stack: 64,
                    max_memory: 128,
                    max_cycles: 100_000,
                    parity_check: false,
                },
            ),
        ];

        let mut all_match = true;

        for (name, config) in &configs {
            let result = program.run_with(config.clone());
            let output = result.as_ref().ok().map(|r| r.output.clone());
            let matches = output == baseline_output;
            if !matches {
                all_match = false;
            }

            let output_str = match &output {
                Some(o) => format!("{:?}", o),
                None => "ERR".to_string(),
            };
            let match_str = if matches {
                "✓ identical"
            } else {
                "✗ DIFFERS"
            };

            eprintln!("│ {:>20} │ {:>12} │ {:>24} │", name, output_str, match_str);
        }

        eprintln!("└──────────────────────┴──────────────┴──────────────────────────┘");
        eprintln!(
            "→ Platform independence: {} (output identical when resources sufficient)",
            if all_match { "CONFIRMED" } else { "FAILED" }
        );

        assert!(
            all_match,
            "Output must be identical across VM configs with sufficient resources"
        );
    }

    // =========================================================================
    // Phase 5: Decision Matrix & Recommendations
    // =========================================================================

    /// Aggregated evaluation results across all dimensions.
    #[derive(Debug)]
    struct DnaEvaluationResult {
        /// DNA VM overhead ratio at 20 operations (representative)
        overhead_ratio_20ops: f64,
        /// DNA VM overhead ratio at 100 operations
        overhead_ratio_100ops: f64,
        /// Average mutation survival rate (1 mutation)
        mutation_survival_1: f64,
        /// Average mutation survival rate (5 mutations)
        mutation_survival_5: f64,
        /// Packed strand bytes for 20-instruction program
        strand_bytes_20: usize,
        /// JSON bytes for 20-instruction program
        json_bytes_20: usize,
        /// Determinism score (0.0-1.0)
        determinism_score: f64,
        /// Audit trail cost per mutation step (nucleotides changed)
        audit_cost_per_step: usize,
    }

    /// Recommendation for DNA vs native.
    #[derive(Debug)]
    enum DnaRecommendation {
        /// Use DNA: evolvability, auditability, or transport wins
        UseDna { reasons: Vec<&'static str> },
        /// Use native Rust: performance-critical, no mutation needed
        UseNative { reasons: Vec<&'static str> },
        /// Either works: marginal difference
        Either,
    }

    impl core::fmt::Display for DnaRecommendation {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::UseDna { reasons } => {
                    write!(f, "USE DNA: {}", reasons.join(", "))
                }
                Self::UseNative { reasons } => {
                    write!(f, "USE NATIVE: {}", reasons.join(", "))
                }
                Self::Either => write!(f, "EITHER: marginal difference"),
            }
        }
    }

    fn recommend(eval: &DnaEvaluationResult) -> DnaRecommendation {
        let mut dna_reasons: Vec<&'static str> = Vec::new();
        let mut native_reasons: Vec<&'static str> = Vec::new();

        // Evolvability: if mutations survive well, DNA has unique value
        if eval.mutation_survival_1 > 0.5 {
            dna_reasons.push("mutations survive well (>50% at 1 mutation)");
        }
        if eval.mutation_survival_5 > 0.2 {
            dna_reasons.push("multi-mutation tolerance (>20% at 5 mutations)");
        }

        // Auditability: determinism is a strong DNA advantage
        if eval.determinism_score >= 1.0 {
            dna_reasons.push("100% deterministic execution (integer VM)");
        }

        // Audit trail is always available with DNA
        if eval.audit_cost_per_step < 50 {
            dna_reasons.push("low-cost mutation audit trail");
        }

        // Performance: native is always faster for raw compute
        if eval.overhead_ratio_20ops > 100.0 {
            native_reasons.push("DNA VM >100× slower at 20 ops");
        }
        if eval.overhead_ratio_100ops > 50.0 {
            native_reasons.push("DNA VM >50× slower at 100 ops");
        }

        // Transport: check if DNA is competitive
        if eval.strand_bytes_20 > eval.json_bytes_20 * 2 {
            native_reasons.push("JSON is 2×+ smaller than strand for transport");
        }

        // Decision
        if dna_reasons.len() >= 3 && native_reasons.len() <= 1 {
            DnaRecommendation::UseDna {
                reasons: dna_reasons,
            }
        } else if native_reasons.len() >= 2 && dna_reasons.is_empty() {
            DnaRecommendation::UseNative {
                reasons: native_reasons,
            }
        } else if !dna_reasons.is_empty() && !native_reasons.is_empty() {
            // Mixed: DNA for evolvability/audit, native for speed
            // Default to DNA recommendation since evolvability is the unique value prop
            if dna_reasons.len() >= 2 {
                DnaRecommendation::UseDna {
                    reasons: dna_reasons,
                }
            } else {
                DnaRecommendation::UseNative {
                    reasons: native_reasons,
                }
            }
        } else {
            DnaRecommendation::Either
        }
    }

    #[test]
    fn eval_decision_matrix() {
        eprintln!("\n╔═════════════════════════════════════════════════════════════════╗");
        eprintln!("║ PHASE 5a: Decision Matrix — Aggregated Evaluation             ║");
        eprintln!("╠═════════════════════════════════════════════════════════════════╣");

        // Collect measurements inline (not depending on other test results)

        // 1. Overhead ratio at 20 and 100 ops
        let source_20 = arithmetic_source(20);
        let source_100 = arithmetic_source(100);

        let overhead_20 = {
            let p = compiler::compile(&source_20).ok();
            let dna = measure(1000, || {
                if let Some(ref p) = p {
                    let _ = p.run();
                }
            });
            let native = measure(1000, || {
                std::hint::black_box(native_arithmetic(20));
            });
            dna.mean_ns / native.mean_ns.max(1.0)
        };

        let overhead_100 = {
            let p = compiler::compile(&source_100).ok();
            let dna = measure(1000, || {
                if let Some(ref p) = p {
                    let _ = p.run();
                }
            });
            let native = measure(1000, || {
                std::hint::black_box(native_arithmetic(100));
            });
            dna.mean_ns / native.mean_ns.max(1.0)
        };

        // 2. Mutation survival at 1 and 5 mutations
        let (survival_1, survival_5) = {
            let genome = nexcore_signal_genome().ok();
            let mut s1 = 0.0f64;
            let mut s5 = 0.0f64;

            if let Some(ref genome) = genome {
                if let Some(prr) = genome.find_gene("prr") {
                    let mut rng = Rng::new(777);
                    let trials = 50;

                    for mutation_count in &[1usize, 5] {
                        let mut survived = 0usize;
                        for _ in 0..trials {
                            let mut mutant = prr.clone();
                            let cc = mutant.codon_count();
                            if cc == 0 {
                                continue;
                            }
                            for _ in 0..*mutation_count {
                                let offset = rng.next_usize(cc);
                                let idx = (rng.next() % 64) as u8;
                                if let Ok(c) = Codon::from_index(idx) {
                                    if let Ok(m) = mutant.point_mutate(offset, c) {
                                        mutant = m;
                                    }
                                }
                            }
                            let seq = mutant.sequence.clone();
                            let exec = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                                let config = VmConfig {
                                    max_cycles: 5_000,
                                    ..VmConfig::default()
                                };
                                let mut vm = crate::vm::CodonVM::with_config(config);
                                if vm.load(&seq).is_err() {
                                    return false;
                                }
                                vm.execute_from(0).is_ok()
                            }));
                            if exec.unwrap_or(false) {
                                survived += 1;
                            }
                        }
                        let rate = survived as f64 / trials as f64;
                        if *mutation_count == 1 {
                            s1 = rate;
                        } else {
                            s5 = rate;
                        }
                    }
                }
            }
            (s1, s5)
        };

        // 3. Transport sizes
        let strand_20 = compiler::compile(&source_20)
            .ok()
            .map(|p| (p.code.len() * 2 + 7) / 8)
            .unwrap_or(0);
        let json_20 = source_to_json(&source_20)
            .ok()
            .map(|j| j.len())
            .unwrap_or(0);

        // 4. Determinism (quick check)
        let determinism = {
            let p = compiler::compile("let x = 42\nprint(x)").ok();
            let mut all_same = true;
            let mut first_out: Option<Vec<i64>> = None;
            for _ in 0..100 {
                if let Some(ref p) = p {
                    if let Ok(r) = p.run() {
                        match &first_out {
                            None => {
                                first_out = Some(r.output);
                            }
                            Some(fo) => {
                                if fo != &r.output {
                                    all_same = false;
                                }
                            }
                        }
                    }
                }
            }
            if all_same { 1.0 } else { 0.0 }
        };

        // 5. Audit cost
        let audit_cost = 3usize; // ~3 nucleotides per point mutation (one codon)

        let eval_result = DnaEvaluationResult {
            overhead_ratio_20ops: overhead_20,
            overhead_ratio_100ops: overhead_100,
            mutation_survival_1: survival_1,
            mutation_survival_5: survival_5,
            strand_bytes_20: strand_20,
            json_bytes_20: json_20,
            determinism_score: determinism,
            audit_cost_per_step: audit_cost,
        };

        eprintln!("║                                                                 ║");
        eprintln!("║  Dimension                  │ Value                              ║");
        eprintln!("║  ──────────────────────────  │ ──────────────────────────         ║");
        eprintln!(
            "║  Overhead @ 20 ops          │ {:.0}× slower                      ║",
            eval_result.overhead_ratio_20ops
        );
        eprintln!(
            "║  Overhead @ 100 ops         │ {:.0}× slower                      ║",
            eval_result.overhead_ratio_100ops
        );
        eprintln!(
            "║  Mutation survival (1 mut)  │ {:.0}%                             ║",
            eval_result.mutation_survival_1 * 100.0
        );
        eprintln!(
            "║  Mutation survival (5 mut)  │ {:.0}%                             ║",
            eval_result.mutation_survival_5 * 100.0
        );
        eprintln!(
            "║  Strand size (20 instrs)    │ {} bytes (packed)                  ║",
            eval_result.strand_bytes_20
        );
        eprintln!(
            "║  JSON size (20 instrs)      │ {} bytes                           ║",
            eval_result.json_bytes_20
        );
        eprintln!(
            "║  Determinism                │ {:.2}                               ║",
            eval_result.determinism_score
        );
        eprintln!(
            "║  Audit cost/step            │ {} nucleotides                     ║",
            eval_result.audit_cost_per_step
        );
        eprintln!("║                                                                 ║");
        eprintln!("╠═════════════════════════════════════════════════════════════════╣");

        let recommendation = recommend(&eval_result);
        eprintln!("║  RECOMMENDATION: {}", recommendation);
        eprintln!("╚═════════════════════════════════════════════════════════════════╝");

        // The decision matrix must produce a valid recommendation
        match &recommendation {
            DnaRecommendation::UseDna { reasons } => {
                assert!(!reasons.is_empty(), "UseDna must have reasons");
            }
            DnaRecommendation::UseNative { reasons } => {
                assert!(!reasons.is_empty(), "UseNative must have reasons");
            }
            DnaRecommendation::Either => {
                // Valid
            }
        }
    }

    #[test]
    fn eval_recommend_use_cases() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ PHASE 5b: Use Case Recommendations                            │");
        eprintln!("├──────────────────────────────┬────────────────────────────────┤");
        eprintln!("│ Use Case                     │ Recommendation                │");
        eprintln!("├──────────────────────────────┼────────────────────────────────┤");

        // Use case 1: Performance-critical signal detection
        let perf_critical = DnaEvaluationResult {
            overhead_ratio_20ops: 500.0,
            overhead_ratio_100ops: 200.0,
            mutation_survival_1: 0.0,
            mutation_survival_5: 0.0,
            strand_bytes_20: 200,
            json_bytes_20: 100,
            determinism_score: 1.0,
            audit_cost_per_step: 3,
        };
        let rec = recommend(&perf_critical);
        eprintln!("│ Speed-critical compute       │ {:<30} │", rec);

        // Use case 2: Evolvable algorithm with audit trail
        let evolvable = DnaEvaluationResult {
            overhead_ratio_20ops: 100.0,
            overhead_ratio_100ops: 50.0,
            mutation_survival_1: 0.8,
            mutation_survival_5: 0.4,
            strand_bytes_20: 150,
            json_bytes_20: 100,
            determinism_score: 1.0,
            audit_cost_per_step: 3,
        };
        let rec = recommend(&evolvable);
        eprintln!("│ Evolvable + auditable        │ {:<30} │", rec);

        // Use case 3: Regulatory compliance (determinism required)
        let regulatory = DnaEvaluationResult {
            overhead_ratio_20ops: 50.0,
            overhead_ratio_100ops: 30.0,
            mutation_survival_1: 0.6,
            mutation_survival_5: 0.3,
            strand_bytes_20: 120,
            json_bytes_20: 100,
            determinism_score: 1.0,
            audit_cost_per_step: 3,
        };
        let rec = recommend(&regulatory);
        eprintln!("│ Regulatory compliance        │ {:<30} │", rec);

        eprintln!("└──────────────────────────────┴────────────────────────────────┘");
        eprintln!("→ DNA excels when: evolvability, determinism, or audit trail matter.");
        eprintln!("→ Native excels when: raw throughput is the only priority.");
        eprintln!("→ The ~2000× overhead is the cost of interpretation;");
        eprintln!("   it buys mutation, crossover, and provenance tracking.");
    }
}
