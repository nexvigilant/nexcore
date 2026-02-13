//! Layer 1: Glyph Intermediate Representation.
//!
//! The 8×2 glyph system maps 8 Lex Primitiva symbols into position-semantic
//! instruction pairs. Two glyphs = 6 bits = 64 opcodes.
//!
//! - P0 (first glyph): Instruction *family* — which hardware unit to activate
//! - P1 (second glyph): Instruction *variant* — which operation within the family
//!
//! This module provides:
//! - `Glyph` enum: 8 primitives (σ μ ς ρ ∂ → κ N)
//! - `GlyphPair`: ordered (P0, P1) pair encoding a single instruction
//! - Current ISA overlay: `Instruction ↔ GlyphPair` bijection (SPEC-v3 §3.5)
//! - Hardware unit dispatch from P0 alone
//! - Hamming distance for error tolerance analysis (SPEC-v3 §8)
//!
//! ## Architecture Note
//!
//! The Glyph IR is a **compile-time abstraction** — it never materializes as a
//! runtime data structure. The VM still executes nucleotide-encoded codons.
//! The glyph layer reinterprets the same 64 opcodes through a primitive lens.
//!
//! Tier: T2-C (μ Mapping + σ Sequence + κ Comparison + ∂ Boundary)

use crate::isa::Instruction;

// ---------------------------------------------------------------------------
// Glyph: 8 primitive symbols
// ---------------------------------------------------------------------------

/// One of 8 primitive glyphs, each carrying 3 bits of information.
///
/// Drawn from the Lex Primitiva. Each glyph maps to a T1 universal primitive
/// and corresponds to a hardware execution unit in the VM.
///
/// Tier: T1 (μ Mapping)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Glyph {
    /// σ Sequence — data flow, ordering (Stack engine, 0 cycles)
    Sigma = 0,
    /// μ Mapping — transformation (ALU, 1 cycle)
    Mu = 1,
    /// ς State — storage, mutation (Memory controller, 1-2 cycles)
    Zeta = 2,
    /// ρ Recursion — loops, self-reference (Math coprocessor, 1-3 cycles)
    Rho = 3,
    /// ∂ Boundary — scope, limits (Lifecycle controller, 0-1 cycles)
    Boundary = 4,
    /// → Causality — control flow (Branch predictor, 0-1 cycles)
    Causality = 5,
    /// κ Comparison — testing, branching (Comparison unit, 1 cycle)
    Kappa = 6,
    /// N Quantity — numbers, constants (Bitwise engine, 1 cycle)
    Quantity = 7,
}

impl Glyph {
    /// All 8 glyphs in index order.
    pub const ALL: [Glyph; 8] = [
        Glyph::Sigma,
        Glyph::Mu,
        Glyph::Zeta,
        Glyph::Rho,
        Glyph::Boundary,
        Glyph::Causality,
        Glyph::Kappa,
        Glyph::Quantity,
    ];

    /// 3-bit index (0-7).
    #[must_use]
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Construct from a 3-bit index (0-7). Returns None if out of range.
    #[must_use]
    pub const fn from_index(idx: u8) -> Option<Glyph> {
        match idx {
            0 => Some(Glyph::Sigma),
            1 => Some(Glyph::Mu),
            2 => Some(Glyph::Zeta),
            3 => Some(Glyph::Rho),
            4 => Some(Glyph::Boundary),
            5 => Some(Glyph::Causality),
            6 => Some(Glyph::Kappa),
            7 => Some(Glyph::Quantity),
            _ => None,
        }
    }

    /// Unicode symbol for display.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Glyph::Sigma => "σ",
            Glyph::Mu => "μ",
            Glyph::Zeta => "ς",
            Glyph::Rho => "ρ",
            Glyph::Boundary => "∂",
            Glyph::Causality => "→",
            Glyph::Kappa => "κ",
            Glyph::Quantity => "N",
        }
    }

    /// Lex Primitiva name.
    #[must_use]
    pub const fn primitive_name(self) -> &'static str {
        match self {
            Glyph::Sigma => "Sequence",
            Glyph::Mu => "Mapping",
            Glyph::Zeta => "State",
            Glyph::Rho => "Recursion",
            Glyph::Boundary => "Boundary",
            Glyph::Causality => "Causality",
            Glyph::Kappa => "Comparison",
            Glyph::Quantity => "Quantity",
        }
    }

    /// VM hardware unit activated by this glyph as P0.
    #[must_use]
    pub const fn hardware_unit(self) -> &'static str {
        match self {
            Glyph::Sigma => "Stack engine",
            Glyph::Mu => "ALU",
            Glyph::Zeta => "Memory controller",
            Glyph::Rho => "Math coprocessor",
            Glyph::Boundary => "Lifecycle controller",
            Glyph::Causality => "Branch predictor",
            Glyph::Kappa => "Comparison unit",
            Glyph::Quantity => "Bitwise engine",
        }
    }

    /// Maximum latency in cycles when this glyph is the P0 family.
    #[must_use]
    pub const fn max_latency_cycles(self) -> u8 {
        match self {
            Glyph::Sigma => 0,     // register operations
            Glyph::Mu => 1,        // single ALU op
            Glyph::Zeta => 2,      // memory access
            Glyph::Rho => 3,       // multi-cycle math
            Glyph::Boundary => 1,  // lifecycle
            Glyph::Causality => 1, // branch
            Glyph::Kappa => 1,     // comparison
            Glyph::Quantity => 1,  // bitwise
        }
    }

    /// Nucleotide pair encoding for this glyph (SPEC-v3 §3.4).
    #[must_use]
    pub const fn nucleotide_pair(self) -> &'static str {
        match self {
            Glyph::Sigma => "AA",
            Glyph::Mu => "AT",
            Glyph::Zeta => "AG",
            Glyph::Rho => "AC",
            Glyph::Boundary => "TA",
            Glyph::Causality => "TT",
            Glyph::Kappa => "TG",
            Glyph::Quantity => "TC",
        }
    }
}

impl core::fmt::Display for Glyph {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

// ---------------------------------------------------------------------------
// GlyphPair: the 8×2 instruction encoding
// ---------------------------------------------------------------------------

/// An ordered pair of glyphs encoding a single VM instruction.
///
/// - `p0`: Family determinative (which hardware unit)
/// - `p1`: Variant selector (which operation within family)
///
/// Together: 3+3 = 6 bits = index in [0, 63].
///
/// Tier: T2-P (σ Sequence + μ Mapping)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphPair {
    /// Position 0: family determinative.
    pub p0: Glyph,
    /// Position 1: variant selector.
    pub p1: Glyph,
}

impl GlyphPair {
    /// Create a new glyph pair.
    #[must_use]
    pub const fn new(p0: Glyph, p1: Glyph) -> Self {
        Self { p0, p1 }
    }

    /// 6-bit glyph index: `p0 * 8 + p1`, range [0, 63].
    #[must_use]
    pub const fn glyph_index(&self) -> u8 {
        self.p0.index() * 8 + self.p1.index()
    }

    /// Construct from a 6-bit index (0-63). Returns None if out of range.
    #[must_use]
    pub const fn from_glyph_index(idx: u8) -> Option<Self> {
        if idx > 63 {
            return None;
        }
        let p0_idx = idx / 8;
        let p1_idx = idx % 8;
        // const fn can't use ? on Option, so manual match
        let p0 = match Glyph::from_index(p0_idx) {
            Some(g) => g,
            None => return None,
        };
        let p1 = match Glyph::from_index(p1_idx) {
            Some(g) => g,
            None => return None,
        };
        Some(Self { p0, p1 })
    }

    /// Family of this instruction (P0 glyph).
    #[must_use]
    pub const fn family(&self) -> Glyph {
        self.p0
    }

    /// Variant within the family (P1 glyph).
    #[must_use]
    pub const fn variant(&self) -> Glyph {
        self.p1
    }

    /// Hardware unit activated by the P0 family glyph.
    #[must_use]
    pub const fn hardware_unit(&self) -> &'static str {
        self.p0.hardware_unit()
    }

    /// Hamming distance to another glyph pair.
    ///
    /// Returns 0-2: how many glyph positions differ.
    /// Distance 1 with same P0 = **within-family mutation** (semantic degradation).
    /// Distance 1 with different P0 = **cross-family mutation** (category change).
    /// Distance 2 = both positions differ.
    #[must_use]
    pub const fn hamming_distance(&self, other: &GlyphPair) -> u8 {
        let d0 = if self.p0.index() != other.p0.index() {
            1
        } else {
            0
        };
        let d1 = if self.p1.index() != other.p1.index() {
            1
        } else {
            0
        };
        d0 + d1
    }

    /// Whether a mutation from `self` to `other` stays within the same family.
    ///
    /// Within-family mutations preserve the semantic category (biological analog:
    /// synonymous mutations in the third codon position).
    #[must_use]
    pub const fn is_within_family_mutation(&self, other: &GlyphPair) -> bool {
        self.p0.index() == other.p0.index() && self.p1.index() != other.p1.index()
    }

    /// Whether a mutation changes the instruction family.
    ///
    /// Cross-family mutations change the hardware unit (biological analog:
    /// nonsynonymous mutations that change the amino acid class).
    #[must_use]
    pub const fn is_cross_family_mutation(&self, other: &GlyphPair) -> bool {
        self.p0.index() != other.p0.index()
    }

    /// Nucleotide encoding: 4-character string (two nucleotide pairs).
    #[must_use]
    pub fn nucleotide_encoding(&self) -> String {
        let mut s = String::with_capacity(4);
        s.push_str(self.p0.nucleotide_pair());
        s.push_str(self.p1.nucleotide_pair());
        s
    }

    /// Short notation: two glyph symbols concatenated (e.g., "σμ", "→κ").
    #[must_use]
    pub fn notation(&self) -> String {
        let mut s = String::with_capacity(8); // unicode chars may be multi-byte
        s.push_str(self.p0.symbol());
        s.push_str(self.p1.symbol());
        s
    }
}

impl core::fmt::Display for GlyphPair {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}{}({})",
            self.p0.symbol(),
            self.p1.symbol(),
            self.glyph_index()
        )
    }
}

// ---------------------------------------------------------------------------
// Current ISA overlay: Instruction ↔ GlyphPair (SPEC-v3 §3.5)
// ---------------------------------------------------------------------------

/// Map a codon index (0-63) to its semantic glyph pair.
///
/// After v3 ISA reclassification, codon_index == glyph_index, making
/// this an identity mapping: `P0 = index / 8`, `P1 = index % 8`.
///
/// SPEC-v3 §3.5: The 8 glyph families are σ(0), μ(1), ς(2), ρ(3),
/// ∂(4), →(5), κ(6), N(7), each containing 8 variant instructions.
#[must_use]
pub fn glyph_for_codon(codon_index: u8) -> GlyphPair {
    let p0 = (codon_index / 8) as usize;
    let p1 = (codon_index % 8) as usize;
    // Clamp to valid glyph range (0-7)
    if p0 < 8 && p1 < 8 {
        GlyphPair::new(Glyph::ALL[p0], Glyph::ALL[p1])
    } else {
        // Fallback for out-of-range indices
        GlyphPair::new(Glyph::Sigma, Glyph::Sigma)
    }
}

/// Map an `Instruction` to its semantic glyph pair.
///
/// Uses `Instruction::to_index()` internally, then the §3.5 overlay.
/// `Lit` pseudo-instructions map to None (they have no fixed glyph encoding).
#[must_use]
pub fn glyph_for_instruction(instr: &Instruction) -> Option<GlyphPair> {
    match instr {
        Instruction::Lit(_) => None, // pseudo-instruction, no fixed glyph
        other => {
            let codon = crate::isa::encode(other)?;
            Some(glyph_for_codon(codon.index()))
        }
    }
}

/// Map a glyph pair back to the Instruction it represents.
///
/// With v3 ISA, glyph_index == codon_index, so `P0*8 + P1 = codon_index`.
#[must_use]
pub fn instruction_for_glyph(pair: &GlyphPair) -> Instruction {
    let codon_index = pair.glyph_index();
    crate::isa::decode_index(codon_index)
}

// ---------------------------------------------------------------------------
// Family analysis
// ---------------------------------------------------------------------------

/// Get all 8 instructions in a glyph family.
#[must_use]
pub fn family_instructions(family: Glyph) -> [(GlyphPair, Instruction); 8] {
    let mut result = [(GlyphPair::new(Glyph::Sigma, Glyph::Sigma), Instruction::Nop); 8];
    for (i, variant) in Glyph::ALL.iter().enumerate() {
        let pair = GlyphPair::new(family, *variant);
        let instr = instruction_for_glyph(&pair);
        result[i] = (pair, instr);
    }
    result
}

/// Count how many within-family mutations are "safe" (same semantic category).
///
/// For a family of 8, there are 8×7 = 56 possible single-position P1 mutations.
/// All of them stay within the family by definition. This is the error tolerance
/// property described in SPEC-v3 §8.2.
#[must_use]
pub const fn within_family_mutation_count() -> usize {
    8 * 7 // 56 possible within-family P1 mutations per family
}

/// Parity check codon: XOR of previous glyph indices (SPEC-v3 §8.3).
///
/// Every 16th codon is a parity codon. This computes the expected parity
/// for a sequence of glyph pair indices.
#[must_use]
pub fn parity_codon(glyph_indices: &[u8]) -> u8 {
    let mut parity: u8 = 0;
    for &idx in glyph_indices {
        parity ^= idx & 0x3F; // mask to 6 bits
    }
    parity
}

/// Verify a parity-protected block of 15 glyphs + 1 check glyph.
#[must_use]
pub fn verify_parity(block: &[u8; 16]) -> bool {
    let computed = parity_codon(&block[..15]);
    computed == (block[15] & 0x3F)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Glyph basic tests ---

    #[test]
    fn glyph_count() {
        assert_eq!(Glyph::ALL.len(), 8);
    }

    #[test]
    fn glyph_indices_0_through_7() {
        for (i, g) in Glyph::ALL.iter().enumerate() {
            assert_eq!(g.index() as usize, i);
        }
    }

    #[test]
    fn glyph_roundtrip() {
        for i in 0..8u8 {
            let g = Glyph::from_index(i);
            assert!(g.is_some());
            assert_eq!(g.map(|g| g.index()), Some(i));
        }
        assert!(Glyph::from_index(8).is_none());
        assert!(Glyph::from_index(255).is_none());
    }

    #[test]
    fn glyph_symbols() {
        assert_eq!(Glyph::Sigma.symbol(), "σ");
        assert_eq!(Glyph::Mu.symbol(), "μ");
        assert_eq!(Glyph::Zeta.symbol(), "ς");
        assert_eq!(Glyph::Rho.symbol(), "ρ");
        assert_eq!(Glyph::Boundary.symbol(), "∂");
        assert_eq!(Glyph::Causality.symbol(), "→");
        assert_eq!(Glyph::Kappa.symbol(), "κ");
        assert_eq!(Glyph::Quantity.symbol(), "N");
    }

    #[test]
    fn glyph_nucleotide_pairs() {
        assert_eq!(Glyph::Sigma.nucleotide_pair(), "AA");
        assert_eq!(Glyph::Quantity.nucleotide_pair(), "TC");
    }

    // --- GlyphPair tests ---

    #[test]
    fn glyph_pair_index_formula() {
        // index = p0 * 8 + p1
        let pair = GlyphPair::new(Glyph::Causality, Glyph::Boundary);
        assert_eq!(pair.glyph_index(), 5 * 8 + 4); // 44
    }

    #[test]
    fn glyph_pair_index_range() {
        // Minimum: σσ = 0
        assert_eq!(GlyphPair::new(Glyph::Sigma, Glyph::Sigma).glyph_index(), 0);
        // Maximum: NN = 63
        assert_eq!(
            GlyphPair::new(Glyph::Quantity, Glyph::Quantity).glyph_index(),
            63
        );
    }

    #[test]
    fn glyph_pair_roundtrip() {
        for idx in 0..64u8 {
            let pair = GlyphPair::from_glyph_index(idx);
            assert!(pair.is_some(), "failed for index {idx}");
            assert_eq!(pair.map(|p| p.glyph_index()), Some(idx));
        }
        assert!(GlyphPair::from_glyph_index(64).is_none());
    }

    #[test]
    fn glyph_pair_all_64_unique() {
        let mut seen = [false; 64];
        for p0 in &Glyph::ALL {
            for p1 in &Glyph::ALL {
                let pair = GlyphPair::new(*p0, *p1);
                let idx = pair.glyph_index() as usize;
                assert!(!seen[idx], "duplicate index {idx}");
                seen[idx] = true;
            }
        }
        assert!(seen.iter().all(|&s| s), "not all 64 indices covered");
    }

    #[test]
    fn glyph_pair_family_and_variant() {
        let pair = GlyphPair::new(Glyph::Kappa, Glyph::Causality);
        assert_eq!(pair.family(), Glyph::Kappa);
        assert_eq!(pair.variant(), Glyph::Causality);
        assert_eq!(pair.hardware_unit(), "Comparison unit");
    }

    #[test]
    fn glyph_pair_notation() {
        let pair = GlyphPair::new(Glyph::Causality, Glyph::Kappa);
        assert_eq!(pair.notation(), "→κ");
    }

    #[test]
    fn glyph_pair_nucleotide_encoding() {
        let pair = GlyphPair::new(Glyph::Sigma, Glyph::Mu);
        assert_eq!(pair.nucleotide_encoding(), "AAAT");
    }

    // --- Hamming distance tests ---

    #[test]
    fn hamming_identical() {
        let a = GlyphPair::new(Glyph::Mu, Glyph::Sigma);
        assert_eq!(a.hamming_distance(&a), 0);
    }

    #[test]
    fn hamming_within_family() {
        let a = GlyphPair::new(Glyph::Mu, Glyph::Sigma); // μσ (Sign)
        let b = GlyphPair::new(Glyph::Mu, Glyph::Rho); // μρ (Max)
        assert_eq!(a.hamming_distance(&b), 1);
        assert!(a.is_within_family_mutation(&b));
        assert!(!a.is_cross_family_mutation(&b));
    }

    #[test]
    fn hamming_cross_family() {
        let a = GlyphPair::new(Glyph::Mu, Glyph::Sigma); // μσ
        let b = GlyphPair::new(Glyph::Causality, Glyph::Sigma); // →σ
        assert_eq!(a.hamming_distance(&b), 1);
        assert!(!a.is_within_family_mutation(&b));
        assert!(a.is_cross_family_mutation(&b));
    }

    #[test]
    fn hamming_max_distance() {
        let a = GlyphPair::new(Glyph::Sigma, Glyph::Sigma); // σσ
        let b = GlyphPair::new(Glyph::Quantity, Glyph::Quantity); // NN
        assert_eq!(a.hamming_distance(&b), 2);
    }

    // --- ISA overlay tests ---

    #[test]
    fn overlay_bijective_64() {
        // Every codon index 0-63 maps to a unique glyph pair
        let mut seen = std::collections::HashSet::new();
        for idx in 0..64u8 {
            let pair = glyph_for_codon(idx);
            assert!(
                seen.insert(pair.glyph_index()),
                "duplicate glyph for codon {idx}"
            );
        }
        assert_eq!(seen.len(), 64);
    }

    #[test]
    fn overlay_instruction_roundtrip() {
        // For every non-Lit instruction: instr → glyph → instr
        for idx in 0..64u8 {
            let instr = crate::isa::decode_index(idx);
            let glyph = glyph_for_instruction(&instr);
            assert!(glyph.is_some(), "no glyph for instruction at index {idx}");
            let back = instruction_for_glyph(
                &glyph.unwrap_or_else(|| GlyphPair::new(Glyph::Sigma, Glyph::Sigma)),
            );
            assert_eq!(
                back, instr,
                "roundtrip failed for index {idx}: {instr:?} → {:?} → {back:?}",
                glyph
            );
        }
    }

    #[test]
    fn overlay_lit_is_none() {
        assert!(glyph_for_instruction(&Instruction::Lit(42)).is_none());
    }

    #[test]
    fn overlay_add_is_mapping_family() {
        // v3 ISA: Add = codon 8 → family μ (Mapping), P1=0 (σ)
        let glyph = glyph_for_instruction(&Instruction::Add);
        assert!(glyph.is_some());
        let pair = glyph.unwrap_or_else(|| GlyphPair::new(Glyph::Sigma, Glyph::Sigma));
        assert_eq!(pair.family(), Glyph::Mu);
        assert_eq!(pair.notation(), "μσ");
    }

    #[test]
    fn overlay_halt_is_boundary_family() {
        // v3 ISA: Halt = codon 33 → family ∂ (Boundary), P1=1 (μ)
        let glyph = glyph_for_instruction(&Instruction::Halt);
        assert!(glyph.is_some());
        let pair = glyph.unwrap_or_else(|| GlyphPair::new(Glyph::Sigma, Glyph::Sigma));
        assert_eq!(pair.family(), Glyph::Boundary);
    }

    #[test]
    fn overlay_min_is_recursion_family() {
        // v3 ISA: Min = codon 27 → family ρ (Recursion), P1=3 (ρ)
        let glyph = glyph_for_instruction(&Instruction::Min);
        assert!(glyph.is_some());
        let pair = glyph.unwrap_or_else(|| GlyphPair::new(Glyph::Sigma, Glyph::Sigma));
        assert_eq!(pair.family(), Glyph::Rho);
    }

    // --- Family analysis tests ---

    #[test]
    fn family_has_8_instructions() {
        for family in &Glyph::ALL {
            let instrs = family_instructions(*family);
            assert_eq!(instrs.len(), 8);
            // All should be in the same family
            for (pair, _instr) in &instrs {
                assert_eq!(pair.family(), *family);
            }
        }
    }

    #[test]
    fn all_8_families_cover_64_instructions() {
        let mut all_instrs = std::collections::HashSet::new();
        for family in &Glyph::ALL {
            for (_pair, instr) in &family_instructions(*family) {
                all_instrs.insert(format!("{instr:?}"));
            }
        }
        assert_eq!(
            all_instrs.len(),
            64,
            "families should cover all 64 instructions"
        );
    }

    // --- Parity tests ---

    #[test]
    fn parity_empty() {
        assert_eq!(parity_codon(&[]), 0);
    }

    #[test]
    fn parity_single() {
        assert_eq!(parity_codon(&[42]), 42);
    }

    #[test]
    fn parity_xor() {
        assert_eq!(parity_codon(&[0b101010, 0b110011]), 0b101010 ^ 0b110011);
    }

    #[test]
    fn parity_verify_valid() {
        let mut block = [0u8; 16];
        for i in 0..15 {
            block[i] = (i as u8) * 3;
        }
        block[15] = parity_codon(&block[..15]);
        assert!(verify_parity(&block));
    }

    #[test]
    fn parity_verify_corrupted() {
        let mut block = [0u8; 16];
        for i in 0..15 {
            block[i] = (i as u8) * 3;
        }
        block[15] = parity_codon(&block[..15]);
        // Corrupt one byte
        block[7] ^= 1;
        assert!(!verify_parity(&block));
    }

    // --- Error tolerance analysis ---

    #[test]
    fn within_family_mutations_count() {
        assert_eq!(within_family_mutation_count(), 56);
    }

    #[test]
    fn error_tolerance_property() {
        // Key property: mutating P1 keeps you in the same family.
        // v3 ISA: Add = codon 8 → family μ (Mapping)
        // Mutating P1 gives other μ family ops: Sub, Mul, Div, Mod, Neg, Abs, Inc
        // All are arithmetic transforms — semantic degradation, not destruction.
        let add_glyph = glyph_for_instruction(&Instruction::Add)
            .unwrap_or_else(|| GlyphPair::new(Glyph::Sigma, Glyph::Sigma));
        assert_eq!(add_glyph.family(), Glyph::Mu);

        // Mutate P1 to each variant — all stay in the Mu/Mapping family
        for variant in &Glyph::ALL {
            let mutated = GlyphPair::new(Glyph::Mu, *variant);
            let mutated_instr = instruction_for_glyph(&mutated);
            assert_eq!(
                mutated.family(),
                Glyph::Mu,
                "mutation produced {mutated_instr:?} outside Mu family"
            );
        }
    }
}
