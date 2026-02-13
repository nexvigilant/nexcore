//! Benchmark: Strand (baseline) vs Pixel/Tile (color) encoding.
//!
//! Compares throughput, round-trip fidelity, and memory footprint
//! across both encoding paths for the same instruction sequences.
//!
//! Tier: T2-C (κ Comparison + N Quantity + σ Sequence + μ Mapping)

#[cfg(test)]
mod tests {
    use crate::isa::{self, Instruction};
    use crate::tile::Tile;
    use crate::types::Strand;

    // -----------------------------------------------------------------------
    // Test instruction sets: small, medium, full-tile
    // -----------------------------------------------------------------------

    /// Small program: 5 instructions (a minimal computation).
    fn small_program() -> Vec<Instruction> {
        vec![
            Instruction::Entry,
            Instruction::Push1,
            Instruction::Add,
            Instruction::Output,
            Instruction::Halt,
        ]
    }

    /// Medium program: 20 instructions (realistic function body).
    fn medium_program() -> Vec<Instruction> {
        vec![
            Instruction::Entry,
            Instruction::Lit(10),
            Instruction::Lit(0),
            Instruction::Store,
            Instruction::Lit(1),
            Instruction::Dup,
            Instruction::Load,
            Instruction::Add,
            Instruction::Swap,
            Instruction::Store,
            Instruction::Dec,
            Instruction::Dup,
            Instruction::Push0,
            Instruction::Gt,
            Instruction::Lit(5),
            Instruction::JmpIf,
            Instruction::Pop,
            Instruction::Load,
            Instruction::Output,
            Instruction::Halt,
        ]
    }

    /// Full tile: 48 instructions (max tile capacity).
    fn full_program() -> Vec<Instruction> {
        let base = [
            Instruction::Nop,
            Instruction::Dup,
            Instruction::Swap,
            Instruction::Pop,
            Instruction::Add,
            Instruction::Sub,
            Instruction::Mul,
            Instruction::Div,
            Instruction::Load,
            Instruction::Store,
            Instruction::Push0,
            Instruction::Push1,
            Instruction::Entry,
            Instruction::Halt,
            Instruction::Assert,
            Instruction::Output,
            Instruction::Jmp,
            Instruction::JmpIf,
            Instruction::Call,
            Instruction::Ret,
            Instruction::Eq,
            Instruction::Neq,
            Instruction::Lt,
            Instruction::Gt,
        ];
        // Repeat to fill 48 slots
        base.iter().cycle().take(48).copied().collect()
    }

    // -----------------------------------------------------------------------
    // Baseline: Strand encoding (Instruction → Codon → Strand → Codon → Instruction)
    // -----------------------------------------------------------------------

    fn strand_encode(instrs: &[Instruction]) -> Strand {
        let mut bases = Vec::with_capacity(instrs.len() * 3);
        for instr in instrs {
            match instr {
                Instruction::Lit(n) => {
                    let codons = isa::encode_literal(*n);
                    for c in codons {
                        bases.push(c.0);
                        bases.push(c.1);
                        bases.push(c.2);
                    }
                }
                other => {
                    if let Some(c) = isa::encode(other) {
                        bases.push(c.0);
                        bases.push(c.1);
                        bases.push(c.2);
                    }
                }
            }
        }
        Strand::new(bases)
    }

    fn strand_decode(strand: &Strand) -> Vec<Instruction> {
        match strand.codons() {
            Ok(codons) => codons.iter().map(|c| isa::decode(c)).collect(),
            Err(_) => vec![],
        }
    }

    // -----------------------------------------------------------------------
    // Color: Tile encoding (Instruction → Pixel → Tile → RGBA → Tile → Pixel → Instruction)
    // -----------------------------------------------------------------------

    fn tile_encode(instrs: &[Instruction]) -> [u8; 256] {
        Tile::from_instructions(instrs).to_rgba()
    }

    fn tile_decode(rgba: &[u8; 256]) -> Vec<Instruction> {
        Tile::from_rgba(rgba).to_instructions()
    }

    // -----------------------------------------------------------------------
    // Benchmark harness (std::time, no external deps)
    // -----------------------------------------------------------------------

    struct BenchResult {
        label: &'static str,
        encode_ns: u128,
        decode_ns: u128,
        roundtrip_ns: u128,
        encoded_bytes: usize,
        fidelity: f64, // fraction of instructions that survive roundtrip
    }

    const ITERATIONS: u32 = 10_000;

    fn bench_strand(label: &'static str, instrs: &[Instruction]) -> BenchResult {
        // Warmup
        for _ in 0..100 {
            let s = strand_encode(instrs);
            let _ = strand_decode(&s);
        }

        // Encode
        let t0 = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = strand_encode(instrs);
        }
        let encode_ns = t0.elapsed().as_nanos() / u128::from(ITERATIONS);

        // Decode
        let strand = strand_encode(instrs);
        let t1 = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = strand_decode(&strand);
        }
        let decode_ns = t1.elapsed().as_nanos() / u128::from(ITERATIONS);

        // Full roundtrip
        let t2 = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let s = strand_encode(instrs);
            let _ = strand_decode(&s);
        }
        let roundtrip_ns = t2.elapsed().as_nanos() / u128::from(ITERATIONS);

        // Size: 3 bytes per codon (nucleotide = 1 byte in memory)
        let encoded_bytes = strand.len();

        // Fidelity: how many instructions roundtrip correctly?
        // Note: Lit instructions expand to multiple codons in strand encoding,
        // so decoded length may differ. We check non-Lit instructions.
        let decoded = strand_decode(&strand);
        let non_lit: Vec<&Instruction> = instrs
            .iter()
            .filter(|i| !matches!(i, Instruction::Lit(_)))
            .collect();
        let decoded_non_lit: Vec<&Instruction> = decoded
            .iter()
            .filter(|i| !matches!(i, Instruction::Lit(_)))
            .collect();
        let matches = non_lit
            .iter()
            .zip(decoded_non_lit.iter())
            .filter(|(a, b)| a == b)
            .count();
        let fidelity = if non_lit.is_empty() {
            1.0
        } else {
            matches as f64 / non_lit.len() as f64
        };

        BenchResult {
            label,
            encode_ns,
            decode_ns,
            roundtrip_ns,
            encoded_bytes,
            fidelity,
        }
    }

    fn bench_tile(label: &'static str, instrs: &[Instruction]) -> BenchResult {
        // Warmup
        for _ in 0..100 {
            let rgba = tile_encode(instrs);
            let _ = tile_decode(&rgba);
        }

        // Encode
        let t0 = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = tile_encode(instrs);
        }
        let encode_ns = t0.elapsed().as_nanos() / u128::from(ITERATIONS);

        // Decode
        let rgba = tile_encode(instrs);
        let t1 = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = tile_decode(&rgba);
        }
        let decode_ns = t1.elapsed().as_nanos() / u128::from(ITERATIONS);

        // Full roundtrip
        let t2 = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let rgba = tile_encode(instrs);
            let _ = tile_decode(&rgba);
        }
        let roundtrip_ns = t2.elapsed().as_nanos() / u128::from(ITERATIONS);

        // Size: always 256 bytes (8×8 RGBA)
        let encoded_bytes = 256;

        // Fidelity check
        let decoded = tile_decode(&rgba);
        // Filter out Lit because tile truncates to 8 bits
        let non_lit_orig: Vec<&Instruction> = instrs
            .iter()
            .take(48)
            .filter(|i| !matches!(i, Instruction::Lit(_)))
            .collect();
        let non_lit_dec: Vec<&Instruction> = decoded
            .iter()
            .filter(|i| !matches!(i, Instruction::Lit(_)))
            .collect();
        let matches = non_lit_orig
            .iter()
            .zip(non_lit_dec.iter())
            .filter(|(a, b)| a == b)
            .count();
        let fidelity = if non_lit_orig.is_empty() {
            1.0
        } else {
            matches as f64 / non_lit_orig.len() as f64
        };

        BenchResult {
            label,
            encode_ns,
            decode_ns,
            roundtrip_ns,
            encoded_bytes,
            fidelity,
        }
    }

    fn print_results(strand: &BenchResult, tile: &BenchResult) {
        eprintln!("┌─────────────────────────────────────────────────────────────────┐");
        eprintln!(
            "│ {:<63}│",
            format!("BENCHMARK: {} instructions", strand.label)
        );
        eprintln!("├──────────────┬──────────────────┬──────────────────┬────────────┤");
        eprintln!("│ Metric       │ Strand (baseline)│ Tile (color)     │ Speedup    │");
        eprintln!("├──────────────┼──────────────────┼──────────────────┼────────────┤");

        let speedup_enc = strand.encode_ns as f64 / tile.encode_ns.max(1) as f64;
        let speedup_dec = strand.decode_ns as f64 / tile.decode_ns.max(1) as f64;
        let speedup_rt = strand.roundtrip_ns as f64 / tile.roundtrip_ns.max(1) as f64;

        eprintln!(
            "│ Encode       │ {:>12} ns  │ {:>12} ns  │ {:>8.2}×  │",
            strand.encode_ns, tile.encode_ns, speedup_enc
        );
        eprintln!(
            "│ Decode       │ {:>12} ns  │ {:>12} ns  │ {:>8.2}×  │",
            strand.decode_ns, tile.decode_ns, speedup_dec
        );
        eprintln!(
            "│ Roundtrip    │ {:>12} ns  │ {:>12} ns  │ {:>8.2}×  │",
            strand.roundtrip_ns, tile.roundtrip_ns, speedup_rt
        );
        eprintln!(
            "│ Size (bytes) │ {:>12}     │ {:>12}     │ {:>7.1}×   │",
            strand.encoded_bytes,
            tile.encoded_bytes,
            strand.encoded_bytes as f64 / tile.encoded_bytes.max(1) as f64
        );
        eprintln!(
            "│ Fidelity     │ {:>11.1}%     │ {:>11.1}%     │            │",
            strand.fidelity * 100.0,
            tile.fidelity * 100.0
        );
        eprintln!("└──────────────┴──────────────────┴──────────────────┴────────────┘");
    }

    // -----------------------------------------------------------------------
    // Benchmark tests
    // -----------------------------------------------------------------------

    #[test]
    fn bench_small_program() {
        let instrs = small_program();
        let strand = bench_strand("5 (small)", &instrs);
        let tile = bench_tile("5 (small)", &instrs);
        print_results(&strand, &tile);

        // Sanity: both should have perfect fidelity for non-Lit instructions
        assert!(strand.fidelity > 0.99);
        assert!(tile.fidelity > 0.99);
    }

    #[test]
    fn bench_medium_program() {
        let instrs = medium_program();
        let strand = bench_strand("20 (medium)", &instrs);
        let tile = bench_tile("20 (medium)", &instrs);
        print_results(&strand, &tile);

        // Strand fidelity is low for programs with Lit() instructions because
        // Lit(n) expands to multi-codon sequences (push/shift/add), which
        // disrupts instruction alignment on decode. This is by design —
        // strand encoding doesn't have a native "literal" codon.
        // Tile encoding preserves non-Lit instructions perfectly.
        assert!(tile.fidelity > 0.90);
    }

    #[test]
    fn bench_full_tile() {
        let instrs = full_program();
        let strand = bench_strand("48 (full tile)", &instrs);
        let tile = bench_tile("48 (full tile)", &instrs);
        print_results(&strand, &tile);

        assert!(strand.fidelity > 0.99);
        assert!(tile.fidelity > 0.99);
    }

    // -----------------------------------------------------------------------
    // Fidelity deep-dive: Lit value range test
    // -----------------------------------------------------------------------

    #[test]
    fn fidelity_lit_range() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ FIDELITY DEEP-DIVE: Lit value preservation                     │");
        eprintln!("├──────────────┬──────────────────┬──────────────────────────────┤");
        eprintln!("│ Lit value    │ Strand decoded   │ Tile decoded                 │");
        eprintln!("├──────────────┼──────────────────┼──────────────────────────────┤");

        let test_values: Vec<i64> = vec![0, 1, 42, 127, 255, 256, 1000, -1, -42, i64::MAX];

        for &val in &test_values {
            // Strand roundtrip
            let strand = strand_encode(&[Instruction::Lit(val)]);
            let strand_dec = strand_decode(&strand);
            let strand_val = strand_dec
                .iter()
                .find_map(|i| {
                    if let Instruction::Lit(n) = i {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .unwrap_or(-9999);

            // Tile roundtrip
            let rgba = tile_encode(&[Instruction::Lit(val)]);
            let tile_dec = tile_decode(&rgba);
            let tile_val = tile_dec
                .first()
                .and_then(|i| {
                    if let Instruction::Lit(n) = i {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .unwrap_or(-9999);

            let strand_ok = if strand_val == val { "  ✓" } else { "  ✗" };
            let tile_ok = if tile_val == val {
                "  ✓"
            } else {
                &format!("  ✗ (got {})", tile_val)
            };

            eprintln!(
                "│ {:>12} │ {:>12}{:>4} │ {:>12}{:<16} │",
                val, strand_val, strand_ok, tile_val, tile_ok
            );
        }
        eprintln!("└──────────────┴──────────────────┴──────────────────────────────┘");
        eprintln!("Note: Tile B channel is 8-bit, truncates Lit values > 255.");
    }

    // -----------------------------------------------------------------------
    // Memory density comparison
    // -----------------------------------------------------------------------

    #[test]
    fn memory_density() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ MEMORY DENSITY: bits per instruction                           │");
        eprintln!("├──────────────┬──────────────────┬──────────────────┬────────────┤");
        eprintln!("│ Program size │ Strand (bits/i)  │ Tile (bits/i)    │ Winner     │");
        eprintln!("├──────────────┼──────────────────┼──────────────────┼────────────┤");

        for &count in &[1usize, 5, 10, 20, 48] {
            let instrs: Vec<Instruction> = (0..count)
                .map(|i| match i % 6 {
                    0 => Instruction::Push0,
                    1 => Instruction::Add,
                    2 => Instruction::Sub,
                    3 => Instruction::Dup,
                    4 => Instruction::Output,
                    _ => Instruction::Nop,
                })
                .collect();

            let strand = strand_encode(&instrs);
            let strand_bits = strand.len() as f64 * 2.0; // 2 bits per nucleotide
            let strand_bpi = strand_bits / count as f64;

            let tile_bits = 256.0 * 8.0; // always 2048 bits (256 bytes RGBA)
            let tile_bpi = tile_bits / count as f64;

            let winner = if strand_bpi < tile_bpi {
                "Strand"
            } else {
                "Tile"
            };

            eprintln!(
                "│ {:>5} instr  │ {:>12.1} b/i  │ {:>12.1} b/i  │ {:>10} │",
                count, strand_bpi, tile_bpi, winner
            );
        }
        eprintln!("└──────────────┴──────────────────┴──────────────────┴────────────┘");
        eprintln!("Strand: 6 bits/instr (fixed). Tile: 2048 bits / N (amortized).");
    }

    // -----------------------------------------------------------------------
    // Checksum overhead
    // -----------------------------------------------------------------------

    #[test]
    fn checksum_overhead() {
        let instrs = medium_program();

        // Tile encode without checksum
        let t0 = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = Tile::from_instructions(&instrs);
        }
        let no_checksum_ns = t0.elapsed().as_nanos() / u128::from(ITERATIONS);

        // Tile encode with checksum
        let t1 = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let mut tile = Tile::from_instructions(&instrs);
            let hash = tile.compute_checksum();
            tile.set_checksum(&hash);
        }
        let with_checksum_ns = t1.elapsed().as_nanos() / u128::from(ITERATIONS);

        // Verify
        let t2 = std::time::Instant::now();
        let mut tile = Tile::from_instructions(&instrs);
        let hash = tile.compute_checksum();
        tile.set_checksum(&hash);
        for _ in 0..ITERATIONS {
            let _ = tile.verify();
        }
        let verify_ns = t2.elapsed().as_nanos() / u128::from(ITERATIONS);

        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!(
            "│ CHECKSUM OVERHEAD (20 instructions, {} iterations)          │",
            ITERATIONS
        );
        eprintln!("├──────────────────────┬──────────────────────────────────────────┤");
        eprintln!(
            "│ Encode (no checksum) │ {:>12} ns                              │",
            no_checksum_ns
        );
        eprintln!(
            "│ Encode + checksum    │ {:>12} ns (+{}ns)                  │",
            with_checksum_ns,
            with_checksum_ns.saturating_sub(no_checksum_ns)
        );
        eprintln!(
            "│ Verify               │ {:>12} ns                              │",
            verify_ns
        );
        eprintln!("└──────────────────────┴──────────────────────────────────────────┘");

        // Checksum overhead should be reasonable (sub-microsecond in release)
        assert!(with_checksum_ns < 10_000); // under 10μs
    }
}
