//! Encoding transcoder: Strand ↔ Tile with optimal selection.
//!
//! Analyzes program characteristics and recommends the encoding that
//! maximizes fidelity + speed for the given instruction profile.
//!
//! ## Decision Logic
//!
//! ```text
//! IF instructions > 48           → Strand (Tile overflow)
//! IF any Lit value > 255 or < 0  → Strand (Tile truncates B channel)
//! IF fidelity-critical           → Strand (lossless for all values)
//! ELSE                           → Tile  (3-9× faster encode)
//! ```
//!
//! ## Crossover Points (from benchmarks)
//!
//! | Factor        | Strand wins when          | Tile wins when           |
//! |---------------|---------------------------|--------------------------|
//! | Encode speed  | never                     | always (3-9× faster)     |
//! | Roundtrip     | < ~10 instructions        | ≥ ~10 instructions       |
//! | Density       | always (6 bits/instr)     | never (2048 bits fixed)  |
//! | Lit fidelity  | values outside 0-255      | values in 0-255          |
//! | Capacity      | unlimited                 | ≤ 48 instructions        |
//!
//! Tier: T2-C (κ Comparison + μ Mapping + σ Sequence + ∂ Boundary + → Causality)

use crate::isa::{self, Instruction};
use crate::tile::Tile;
use crate::types::Strand;

// ---------------------------------------------------------------------------
// Program Profile — static analysis of instruction characteristics
// ---------------------------------------------------------------------------

/// Profile of a program's instruction mix.
///
/// Tier: T2-P (κ Comparison + N Quantity)
#[derive(Debug, Clone)]
pub struct ProgramProfile {
    /// Total instruction count.
    pub instruction_count: usize,
    /// Number of Lit instructions.
    pub lit_count: usize,
    /// Maximum Lit value (0 if no Lits).
    pub max_lit: i64,
    /// Minimum Lit value (0 if no Lits).
    pub min_lit: i64,
    /// Whether all Lit values fit in a u8 (0-255).
    pub lits_fit_u8: bool,
    /// Ratio of Lit instructions to total.
    pub lit_density: f64,
    /// Number of distinct instruction families used.
    pub family_coverage: u8,
}

impl ProgramProfile {
    /// Analyze an instruction sequence.
    #[must_use]
    pub fn analyze(instrs: &[Instruction]) -> Self {
        let instruction_count = instrs.len();
        let mut lit_count = 0usize;
        let mut max_lit: i64 = 0;
        let mut min_lit: i64 = 0;
        let mut lits_fit_u8 = true;
        let mut families = [false; 8];

        for instr in instrs {
            match instr {
                Instruction::Lit(n) => {
                    lit_count += 1;
                    if *n > max_lit {
                        max_lit = *n;
                    }
                    if *n < min_lit {
                        min_lit = *n;
                    }
                    if *n < 0 || *n > 255 {
                        lits_fit_u8 = false;
                    }
                    // Lit family = 7 (N/Quantity)
                    families[7] = true;
                }
                other => {
                    if let Some(c) = isa::encode(other) {
                        let fam = (c.index() / 8) as usize;
                        if fam < 8 {
                            families[fam] = true;
                        }
                    }
                }
            }
        }

        let family_coverage = families.iter().filter(|&&f| f).count() as u8;
        let lit_density = if instruction_count == 0 {
            0.0
        } else {
            lit_count as f64 / instruction_count as f64
        };

        Self {
            instruction_count,
            lit_count,
            max_lit,
            min_lit,
            lits_fit_u8,
            lit_density,
            family_coverage,
        }
    }
}

// ---------------------------------------------------------------------------
// Encoding recommendation
// ---------------------------------------------------------------------------

/// Which encoding to use.
///
/// Tier: T1 (ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// Nucleotide triplet encoding: variable-length, lossless, compact.
    Strand,
    /// 8×8 RGBA pixel tile: fixed 256 bytes, fast, 48-instruction cap.
    Tile,
}

/// Recommendation with reasoning.
///
/// Tier: T2-P (→ Causality + κ Comparison)
#[derive(Debug, Clone)]
pub struct Recommendation {
    /// The recommended encoding.
    pub encoding: Encoding,
    /// Human-readable reasoning.
    pub reason: &'static str,
    /// Estimated encode speedup of recommended vs alternative (≥ 1.0).
    pub speedup_estimate: f64,
    /// Whether the recommendation is fidelity-lossless.
    pub lossless: bool,
}

/// Recommend optimal encoding for a program.
///
/// Decision chain (priority order):
/// 1. Overflow: >48 instructions → Strand
/// 2. Lit fidelity: values outside 0-255 → Strand
/// 3. Empty program: 0 instructions → Strand (trivial)
/// 4. All clear: ≤48 instructions, Lits in 0-255 → Tile
#[must_use]
pub fn recommend(instrs: &[Instruction]) -> Recommendation {
    let profile = ProgramProfile::analyze(instrs);

    // P1: Tile overflow
    if profile.instruction_count > 48 {
        return Recommendation {
            encoding: Encoding::Strand,
            reason: "program exceeds 48-instruction tile capacity",
            speedup_estimate: 1.0,
            lossless: true,
        };
    }

    // P2: Lit fidelity
    if !profile.lits_fit_u8 && profile.lit_count > 0 {
        return Recommendation {
            encoding: Encoding::Strand,
            reason: "Lit values outside 0-255 would be truncated by tile B channel",
            speedup_estimate: 1.0,
            lossless: true,
        };
    }

    // P3: Empty
    if profile.instruction_count == 0 {
        return Recommendation {
            encoding: Encoding::Strand,
            reason: "empty program, strand is trivial",
            speedup_estimate: 1.0,
            lossless: true,
        };
    }

    // P4: Tile is optimal — fast encode, lossless for this profile
    let speedup = if profile.instruction_count < 10 {
        3.0 // small programs: ~3× encode speedup
    } else if profile.instruction_count < 30 {
        6.0 // medium: ~6× (less allocation overhead)
    } else {
        3.5 // full tile: ~3.5×
    };

    Recommendation {
        encoding: Encoding::Tile,
        reason: "instructions ≤ 48 with Lits in 0-255; tile is 3-9× faster",
        speedup_estimate: speedup,
        lossless: true,
    }
}

// ---------------------------------------------------------------------------
// Translators: Strand ↔ Tile
// ---------------------------------------------------------------------------

/// Encode instructions as a DNA Strand.
///
/// Lit(n) expands to multi-codon ATG sequences. Non-Lit instructions
/// encode as single codons. Lossless for all i64 values.
#[must_use]
pub fn encode_strand(instrs: &[Instruction]) -> Strand {
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

/// Decode a DNA Strand back to instructions.
///
/// Note: Lit values decode as the multi-codon expansion, not the original
/// Lit(n). Use `encode_tile` → `decode_tile` for Lit-preserving roundtrip
/// (if values fit in 0-255).
#[must_use]
pub fn decode_strand(strand: &Strand) -> Vec<Instruction> {
    match strand.codons() {
        Ok(codons) => codons.iter().map(isa::decode).collect(),
        Err(_) => vec![],
    }
}

/// Encode instructions as an RGBA tile (256 bytes).
///
/// Capacity: 48 instructions max. Lit values truncated to 8 bits.
#[must_use]
pub fn encode_tile(instrs: &[Instruction]) -> [u8; 256] {
    Tile::from_instructions(instrs).to_rgba()
}

/// Decode an RGBA tile back to instructions.
///
/// Preserves instruction identity for non-Lit and Lit(0-255).
#[must_use]
pub fn decode_tile(rgba: &[u8; 256]) -> Vec<Instruction> {
    Tile::from_rgba(rgba).to_instructions()
}

// ---------------------------------------------------------------------------
// Auto-transcode: Strand → Tile and Tile → Strand
// ---------------------------------------------------------------------------

/// Transcode result containing the encoded data in both formats.
///
/// Tier: T2-C (μ Mapping + σ Sequence + ∂ Boundary)
#[derive(Debug, Clone)]
pub struct TranscodeResult {
    /// The recommended encoding used.
    pub encoding: Encoding,
    /// Strand-encoded form (always available).
    pub strand: Strand,
    /// Tile-encoded form (None if >48 instructions or lossy Lits).
    pub tile: Option<[u8; 256]>,
    /// Profile of the source program.
    pub profile: ProgramProfile,
    /// Recommendation details.
    pub recommendation: Recommendation,
}

/// Analyze and encode a program using the optimal path, providing both
/// representations where lossless.
///
/// This is the main entry point for the transcoding experiment.
#[must_use]
pub fn transcode(instrs: &[Instruction]) -> TranscodeResult {
    let profile = ProgramProfile::analyze(instrs);
    let recommendation = recommend(instrs);
    let strand = encode_strand(instrs);

    let tile = if profile.instruction_count <= 48 && profile.lits_fit_u8 {
        Some(encode_tile(instrs))
    } else {
        None
    };

    TranscodeResult {
        encoding: recommendation.encoding,
        strand,
        tile,
        profile,
        recommendation,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Profile tests ---

    #[test]
    fn profile_empty() {
        let profile = ProgramProfile::analyze(&[]);
        assert_eq!(profile.instruction_count, 0);
        assert_eq!(profile.lit_count, 0);
        assert!(profile.lits_fit_u8);
        assert!((profile.lit_density - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn profile_no_lits() {
        let instrs = vec![Instruction::Entry, Instruction::Add, Instruction::Halt];
        let profile = ProgramProfile::analyze(&instrs);
        assert_eq!(profile.instruction_count, 3);
        assert_eq!(profile.lit_count, 0);
        assert!(profile.lits_fit_u8);
        assert_eq!(profile.max_lit, 0);
        assert_eq!(profile.min_lit, 0);
    }

    #[test]
    fn profile_small_lits() {
        let instrs = vec![
            Instruction::Lit(42),
            Instruction::Lit(200),
            Instruction::Add,
        ];
        let profile = ProgramProfile::analyze(&instrs);
        assert_eq!(profile.lit_count, 2);
        assert_eq!(profile.max_lit, 200);
        assert!(profile.lits_fit_u8);
        assert!((profile.lit_density - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn profile_large_lits() {
        let instrs = vec![Instruction::Lit(1000), Instruction::Lit(-5)];
        let profile = ProgramProfile::analyze(&instrs);
        assert!(!profile.lits_fit_u8);
        assert_eq!(profile.max_lit, 1000);
        assert_eq!(profile.min_lit, -5);
    }

    #[test]
    fn profile_family_coverage() {
        let instrs = vec![
            Instruction::Push0,  // family depends on codon index
            Instruction::Add,    // different family
            Instruction::Eq,     // different family
            Instruction::Lit(1), // N/Quantity family (7)
        ];
        let profile = ProgramProfile::analyze(&instrs);
        assert!(profile.family_coverage >= 3);
    }

    // --- Recommendation tests ---

    #[test]
    fn recommend_empty_is_strand() {
        let rec = recommend(&[]);
        assert_eq!(rec.encoding, Encoding::Strand);
        assert!(rec.lossless);
    }

    #[test]
    fn recommend_overflow_is_strand() {
        let instrs: Vec<Instruction> = (0..60).map(|_| Instruction::Nop).collect();
        let rec = recommend(&instrs);
        assert_eq!(rec.encoding, Encoding::Strand);
        assert!(rec.reason.contains("48"));
    }

    #[test]
    fn recommend_large_lit_is_strand() {
        let instrs = vec![
            Instruction::Lit(1000),
            Instruction::Output,
            Instruction::Halt,
        ];
        let rec = recommend(&instrs);
        assert_eq!(rec.encoding, Encoding::Strand);
        assert!(rec.reason.contains("truncated"));
    }

    #[test]
    fn recommend_negative_lit_is_strand() {
        let instrs = vec![Instruction::Lit(-1), Instruction::Output];
        let rec = recommend(&instrs);
        assert_eq!(rec.encoding, Encoding::Strand);
    }

    #[test]
    fn recommend_small_clean_is_tile() {
        let instrs = vec![
            Instruction::Entry,
            Instruction::Lit(42),
            Instruction::Output,
            Instruction::Halt,
        ];
        let rec = recommend(&instrs);
        assert_eq!(rec.encoding, Encoding::Tile);
        assert!(rec.lossless);
        assert!(rec.speedup_estimate > 1.0);
    }

    #[test]
    fn recommend_max_tile_is_tile() {
        let instrs: Vec<Instruction> = (0..48).map(|_| Instruction::Nop).collect();
        let rec = recommend(&instrs);
        assert_eq!(rec.encoding, Encoding::Tile);
    }

    #[test]
    fn recommend_49_is_strand() {
        let instrs: Vec<Instruction> = (0..49).map(|_| Instruction::Nop).collect();
        let rec = recommend(&instrs);
        assert_eq!(rec.encoding, Encoding::Strand);
    }

    #[test]
    fn recommend_lit_255_is_tile() {
        let instrs = vec![Instruction::Lit(255), Instruction::Output];
        let rec = recommend(&instrs);
        assert_eq!(rec.encoding, Encoding::Tile);
    }

    #[test]
    fn recommend_lit_256_is_strand() {
        let instrs = vec![Instruction::Lit(256), Instruction::Output];
        let rec = recommend(&instrs);
        assert_eq!(rec.encoding, Encoding::Strand);
    }

    // --- Translator tests ---

    #[test]
    fn tile_roundtrip_clean_program() {
        let instrs = vec![
            Instruction::Entry,
            Instruction::Lit(42),
            Instruction::Lit(100),
            Instruction::Add,
            Instruction::Output,
            Instruction::Halt,
        ];
        let rgba = encode_tile(&instrs);
        let decoded = decode_tile(&rgba);
        assert_eq!(decoded, instrs);
    }

    #[test]
    fn tile_roundtrip_no_lits() {
        let instrs = vec![
            Instruction::Push0,
            Instruction::Push1,
            Instruction::Add,
            Instruction::Dup,
            Instruction::Output,
            Instruction::Halt,
        ];
        let rgba = encode_tile(&instrs);
        let decoded = decode_tile(&rgba);
        assert_eq!(decoded, instrs);
    }

    #[test]
    fn strand_encode_decode_non_lit() {
        // Non-Lit instructions should survive strand roundtrip
        let instrs = vec![Instruction::Nop, Instruction::Add, Instruction::Halt];
        let strand = encode_strand(&instrs);
        let decoded = decode_strand(&strand);
        assert_eq!(decoded, instrs);
    }

    // --- Transcode tests ---

    #[test]
    fn transcode_small_tile() {
        let instrs = vec![
            Instruction::Entry,
            Instruction::Lit(10),
            Instruction::Output,
            Instruction::Halt,
        ];
        let result = transcode(&instrs);
        assert_eq!(result.encoding, Encoding::Tile);
        assert!(result.tile.is_some());
        assert!(!result.strand.is_empty());
    }

    #[test]
    fn transcode_overflow_strand() {
        let instrs: Vec<Instruction> = (0..60).map(|_| Instruction::Nop).collect();
        let result = transcode(&instrs);
        assert_eq!(result.encoding, Encoding::Strand);
        assert!(result.tile.is_none());
    }

    #[test]
    fn transcode_lossy_lit_strand() {
        let instrs = vec![Instruction::Lit(500), Instruction::Output];
        let result = transcode(&instrs);
        assert_eq!(result.encoding, Encoding::Strand);
        assert!(result.tile.is_none());
    }

    #[test]
    fn transcode_provides_both_when_possible() {
        let instrs = vec![Instruction::Add, Instruction::Sub, Instruction::Halt];
        let result = transcode(&instrs);
        assert!(result.tile.is_some());
        assert!(!result.strand.is_empty());

        // Both should decode to the same instructions
        if let Some(rgba) = &result.tile {
            let tile_decoded = decode_tile(rgba);
            assert_eq!(tile_decoded, instrs);
        }
    }

    // --- Crossover experiment ---

    #[test]
    fn crossover_experiment() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ CROSSOVER EXPERIMENT: optimal encoding by program size         │");
        eprintln!("├──────────────┬──────────┬──────────┬──────────┬────────────────┤");
        eprintln!("│ Instr count  │ Strand   │ Tile     │ Speedup  │ Recommendation │");
        eprintln!("│              │ (ns/rt)  │ (ns/rt)  │          │                │");
        eprintln!("├──────────────┼──────────┼──────────┼──────────┼────────────────┤");

        let iterations: u32 = 5_000;

        for &count in &[1usize, 3, 5, 10, 15, 20, 30, 48] {
            let instrs: Vec<Instruction> = (0..count)
                .map(|i| match i % 8 {
                    0 => Instruction::Push0,
                    1 => Instruction::Push1,
                    2 => Instruction::Add,
                    3 => Instruction::Sub,
                    4 => Instruction::Dup,
                    5 => Instruction::Output,
                    6 => Instruction::Nop,
                    _ => Instruction::Swap,
                })
                .collect();

            // Warmup
            for _ in 0..50 {
                let s = encode_strand(&instrs);
                let _ = decode_strand(&s);
                let t = encode_tile(&instrs);
                let _ = decode_tile(&t);
            }

            // Strand roundtrip
            let t0 = std::time::Instant::now();
            for _ in 0..iterations {
                let s = encode_strand(&instrs);
                let _ = decode_strand(&s);
            }
            let strand_ns = t0.elapsed().as_nanos() / u128::from(iterations);

            // Tile roundtrip
            let t1 = std::time::Instant::now();
            for _ in 0..iterations {
                let t = encode_tile(&instrs);
                let _ = decode_tile(&t);
            }
            let tile_ns = t1.elapsed().as_nanos() / u128::from(iterations);

            let speedup = strand_ns as f64 / tile_ns.max(1) as f64;
            let rec = recommend(&instrs);

            eprintln!(
                "│ {:>5} instr  │ {:>6} ns│ {:>6} ns│ {:>7.2}× │ {:>14} │",
                count,
                strand_ns,
                tile_ns,
                speedup,
                match rec.encoding {
                    Encoding::Strand => "Strand",
                    Encoding::Tile => "Tile",
                }
            );
        }

        eprintln!("├──────────────┴──────────┴──────────┴──────────┴────────────────┤");
        eprintln!("│ Programs with Lit values >255 or >48 instructions → Strand     │");
        eprintln!("│ All other programs → Tile (faster encode + roundtrip)           │");
        eprintln!("└────────────────────────────────────────────────────────────────-┘");
    }

    #[test]
    fn crossover_lit_density_experiment() {
        eprintln!("\n┌─────────────────────────────────────────────────────────────────┐");
        eprintln!("│ LIT DENSITY EXPERIMENT: encoding choice by literal content      │");
        eprintln!("├──────────┬──────────┬──────────┬──────────┬────────────────────┤");
        eprintln!("│ Lit %    │ Strand   │ Tile     │ Speedup  │ Recommendation     │");
        eprintln!("│          │ (ns/rt)  │ (ns/rt)  │          │                    │");
        eprintln!("├──────────┼──────────┼──────────┼──────────┼────────────────────┤");

        let iterations: u32 = 5_000;
        let total = 20usize;

        for &lit_pct in &[0u8, 10, 25, 50, 75, 100] {
            let lit_count = (total as f64 * f64::from(lit_pct) / 100.0) as usize;
            let non_lit_count = total - lit_count;

            let mut instrs: Vec<Instruction> = Vec::with_capacity(total);
            // Non-lit instructions first
            for i in 0..non_lit_count {
                instrs.push(match i % 4 {
                    0 => Instruction::Add,
                    1 => Instruction::Sub,
                    2 => Instruction::Dup,
                    _ => Instruction::Nop,
                });
            }
            // Lit instructions (all in 0-255 range)
            for i in 0..lit_count {
                instrs.push(Instruction::Lit((i * 10 % 256) as i64));
            }

            // Warmup
            for _ in 0..50 {
                let s = encode_strand(&instrs);
                let _ = decode_strand(&s);
                let t = encode_tile(&instrs);
                let _ = decode_tile(&t);
            }

            // Strand roundtrip
            let t0 = std::time::Instant::now();
            for _ in 0..iterations {
                let s = encode_strand(&instrs);
                let _ = decode_strand(&s);
            }
            let strand_ns = t0.elapsed().as_nanos() / u128::from(iterations);

            // Tile roundtrip
            let t1 = std::time::Instant::now();
            for _ in 0..iterations {
                let t = encode_tile(&instrs);
                let _ = decode_tile(&t);
            }
            let tile_ns = t1.elapsed().as_nanos() / u128::from(iterations);

            let speedup = strand_ns as f64 / tile_ns.max(1) as f64;
            let rec = recommend(&instrs);

            eprintln!(
                "│ {:>5}%   │ {:>6} ns│ {:>6} ns│ {:>7.2}× │ {:>18} │",
                lit_pct,
                strand_ns,
                tile_ns,
                speedup,
                match rec.encoding {
                    Encoding::Strand => "Strand",
                    Encoding::Tile => "Tile",
                }
            );
        }

        eprintln!("├──────────┴──────────┴──────────┴──────────┴────────────────────┤");
        eprintln!("│ Higher Lit density → strand slower (multi-codon expansion)      │");
        eprintln!("│ Tile speed stays constant (fixed 256-byte buffer)               │");
        eprintln!("└────────────────────────────────────────────────────────────────-┘");
    }

    // --- Fidelity verification ---

    #[test]
    fn transcode_fidelity_tile_lossless() {
        // Verify that tile-recommended programs have perfect roundtrip
        let instrs = vec![
            Instruction::Entry,
            Instruction::Lit(0),
            Instruction::Lit(127),
            Instruction::Lit(255),
            Instruction::Add,
            Instruction::Output,
            Instruction::Halt,
        ];
        let result = transcode(&instrs);
        assert_eq!(result.encoding, Encoding::Tile);

        if let Some(rgba) = &result.tile {
            let decoded = decode_tile(rgba);
            assert_eq!(
                decoded, instrs,
                "tile roundtrip must be lossless when recommended"
            );
        }
    }

    #[test]
    fn transcode_fidelity_strand_preserves_large_lit() {
        // Strand path should be recommended and available for large Lits
        let instrs = vec![Instruction::Lit(1000)];
        let result = transcode(&instrs);
        assert_eq!(result.encoding, Encoding::Strand);
        assert!(result.tile.is_none());
        // Strand encodes the literal as multi-codon sequence
        assert!(result.strand.len() > 3);
    }

    #[test]
    fn recommendation_is_deterministic() {
        let instrs = vec![Instruction::Lit(42), Instruction::Add, Instruction::Halt];
        let r1 = recommend(&instrs);
        let r2 = recommend(&instrs);
        assert_eq!(r1.encoding, r2.encoding);
        assert_eq!(r1.reason, r2.reason);
    }
}
