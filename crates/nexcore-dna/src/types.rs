//! Core DNA types: Nucleotide, Codon, AminoAcid, Strand, DoubleHelix.
//!
//! These types model the fundamental biological structures of DNA computation.

use crate::error::{DnaError, Result};

// ---------------------------------------------------------------------------
// Nucleotide — ς State (T1)
// ---------------------------------------------------------------------------

/// A single nucleotide base.
///
/// Tier: T1 (ς State)
///
/// Represented as a 2-bit value: A=0b00, T=0b01, G=0b10, C=0b11.
/// RNA uses U in place of T, but the underlying representation is identical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Nucleotide {
    /// Adenine
    A = 0,
    /// Thymine (DNA) / Uracil (RNA)
    T = 1,
    /// Guanine
    G = 2,
    /// Cytosine
    C = 3,
}

impl Nucleotide {
    /// Watson-Crick complement: A↔T, G↔C.
    #[must_use]
    pub fn complement(self) -> Self {
        match self {
            Self::A => Self::T,
            Self::T => Self::A,
            Self::G => Self::C,
            Self::C => Self::G,
        }
    }

    /// Two-bit representation.
    #[must_use]
    pub fn bits(self) -> u8 {
        self as u8
    }

    /// Parse from character (A, T, G, C, U accepted).
    pub fn from_char(ch: char) -> Result<Self> {
        match ch {
            'A' | 'a' => Ok(Self::A),
            'T' | 't' | 'U' | 'u' => Ok(Self::T),
            'G' | 'g' => Ok(Self::G),
            'C' | 'c' => Ok(Self::C),
            _ => Err(DnaError::InvalidBase(ch)),
        }
    }

    /// Construct from 2-bit value (0-3).
    pub fn from_bits(bits: u8) -> Result<Self> {
        match bits & 0b11 {
            0 => Ok(Self::A),
            1 => Ok(Self::T),
            2 => Ok(Self::G),
            3 => Ok(Self::C),
            // The mask ensures this is unreachable, but we avoid unreachable!()
            _ => Err(DnaError::InvalidBase('?')),
        }
    }

    /// Display character (uses U for RNA context when needed externally).
    #[must_use]
    pub fn as_char(self) -> char {
        match self {
            Self::A => 'A',
            Self::T => 'T',
            Self::G => 'G',
            Self::C => 'C',
        }
    }

    /// Display as RNA character (T becomes U).
    #[must_use]
    pub fn as_rna_char(self) -> char {
        match self {
            Self::A => 'A',
            Self::T => 'U',
            Self::G => 'G',
            Self::C => 'C',
        }
    }
}

impl std::fmt::Display for Nucleotide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

// ---------------------------------------------------------------------------
// Codon — σ Sequence + ς State (T2-P)
// ---------------------------------------------------------------------------

/// A triplet of nucleotides encoding one amino acid.
///
/// Tier: T2-P (σ Sequence)
///
/// Index formula: `first * 16 + second * 4 + third` → bijective 0-63.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Codon(pub Nucleotide, pub Nucleotide, pub Nucleotide);

impl Codon {
    /// Unique index in 0..64 range.
    #[must_use]
    pub fn index(&self) -> u8 {
        self.0.bits() * 16 + self.1.bits() * 4 + self.2.bits()
    }

    /// Construct codon from index (0-63).
    pub fn from_index(idx: u8) -> Result<Self> {
        if idx >= 64 {
            return Err(DnaError::InvalidBase('?'));
        }
        let first = Nucleotide::from_bits(idx >> 4)?;
        let second = Nucleotide::from_bits((idx >> 2) & 0b11)?;
        let third = Nucleotide::from_bits(idx & 0b11)?;
        Ok(Self(first, second, third))
    }

    /// Check if this codon decodes to a halt instruction (v3 ISA: indices 33-35).
    #[must_use]
    pub fn is_stop(&self) -> bool {
        // v3 ISA: Halt=33(∂μ), HaltErr=34(∂ς), HaltYield=35(∂ρ)
        matches!(self.index(), 33..=35)
    }

    /// Check if this codon decodes to the entry instruction (v3 ISA: index 32).
    #[must_use]
    pub fn is_start(&self) -> bool {
        // v3 ISA: Entry=32(∂σ) = GAA
        self.index() == 32
    }
}

impl std::fmt::Display for Codon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.0, self.1, self.2)
    }
}

// ---------------------------------------------------------------------------
// AminoAcid — ς State (T1)
// ---------------------------------------------------------------------------

/// The 20 standard amino acids plus Stop signal.
///
/// Tier: T1 (ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AminoAcid {
    Ala,  // A - Alanine
    Arg,  // R - Arginine
    Asn,  // N - Asparagine
    Asp,  // D - Aspartic acid
    Cys,  // C - Cysteine
    Gln,  // Q - Glutamine
    Glu,  // E - Glutamic acid
    Gly,  // G - Glycine
    His,  // H - Histidine
    Ile,  // I - Isoleucine
    Leu,  // L - Leucine
    Lys,  // K - Lysine
    Met,  // M - Methionine (start)
    Phe,  // F - Phenylalanine
    Pro,  // P - Proline
    Ser,  // S - Serine
    Thr,  // T - Threonine
    Trp,  // W - Tryptophan
    Tyr,  // Y - Tyrosine
    Val,  // V - Valine
    Stop, // * - Stop signal
}

impl AminoAcid {
    /// Single-letter IUPAC code.
    #[must_use]
    pub fn code(self) -> char {
        match self {
            Self::Ala => 'A',
            Self::Arg => 'R',
            Self::Asn => 'N',
            Self::Asp => 'D',
            Self::Cys => 'C',
            Self::Gln => 'Q',
            Self::Glu => 'E',
            Self::Gly => 'G',
            Self::His => 'H',
            Self::Ile => 'I',
            Self::Leu => 'L',
            Self::Lys => 'K',
            Self::Met => 'M',
            Self::Phe => 'F',
            Self::Pro => 'P',
            Self::Ser => 'S',
            Self::Thr => 'T',
            Self::Trp => 'W',
            Self::Tyr => 'Y',
            Self::Val => 'V',
            Self::Stop => '*',
        }
    }

    /// Three-letter abbreviation.
    #[must_use]
    pub fn abbrev(self) -> &'static str {
        match self {
            Self::Ala => "Ala",
            Self::Arg => "Arg",
            Self::Asn => "Asn",
            Self::Asp => "Asp",
            Self::Cys => "Cys",
            Self::Gln => "Gln",
            Self::Glu => "Glu",
            Self::Gly => "Gly",
            Self::His => "His",
            Self::Ile => "Ile",
            Self::Leu => "Leu",
            Self::Lys => "Lys",
            Self::Met => "Met",
            Self::Phe => "Phe",
            Self::Pro => "Pro",
            Self::Ser => "Ser",
            Self::Thr => "Thr",
            Self::Trp => "Trp",
            Self::Tyr => "Tyr",
            Self::Val => "Val",
            Self::Stop => "Stp",
        }
    }

    /// Full name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Ala => "Alanine",
            Self::Arg => "Arginine",
            Self::Asn => "Asparagine",
            Self::Asp => "Aspartic acid",
            Self::Cys => "Cysteine",
            Self::Gln => "Glutamine",
            Self::Glu => "Glutamic acid",
            Self::Gly => "Glycine",
            Self::His => "Histidine",
            Self::Ile => "Isoleucine",
            Self::Leu => "Leucine",
            Self::Lys => "Lysine",
            Self::Met => "Methionine",
            Self::Phe => "Phenylalanine",
            Self::Pro => "Proline",
            Self::Ser => "Serine",
            Self::Thr => "Threonine",
            Self::Trp => "Tryptophan",
            Self::Tyr => "Tyrosine",
            Self::Val => "Valine",
            Self::Stop => "Stop",
        }
    }

    /// Chemical property category.
    #[must_use]
    pub fn property(self) -> &'static str {
        match self {
            Self::Ala | Self::Val | Self::Leu | Self::Ile | Self::Pro => "nonpolar",
            Self::Phe | Self::Trp | Self::Met => "nonpolar-aromatic",
            Self::Gly => "special",
            Self::Ser | Self::Thr | Self::Cys | Self::Tyr | Self::Asn | Self::Gln => "polar",
            Self::Asp | Self::Glu => "acidic",
            Self::Lys | Self::Arg | Self::His => "basic",
            Self::Stop => "signal",
        }
    }
}

impl std::fmt::Display for AminoAcid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

// ---------------------------------------------------------------------------
// Strand — σ Sequence + ∃ Existence (T2-P)
// ---------------------------------------------------------------------------

/// A sequence of nucleotides (single-stranded DNA or RNA).
///
/// Tier: T2-P (σ Sequence)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Strand {
    /// The nucleotide bases in order.
    pub bases: Vec<Nucleotide>,
    /// Whether this strand represents RNA (T displayed as U).
    pub is_rna: bool,
}

impl Strand {
    /// Create a new DNA strand from nucleotides.
    #[must_use]
    pub fn new(bases: Vec<Nucleotide>) -> Self {
        Self {
            bases,
            is_rna: false,
        }
    }

    /// Create a new RNA strand from nucleotides.
    #[must_use]
    pub fn new_rna(bases: Vec<Nucleotide>) -> Self {
        Self {
            bases,
            is_rna: true,
        }
    }

    /// Parse a strand from a string of nucleotide characters.
    pub fn parse(s: &str) -> Result<Self> {
        let mut is_rna = false;
        let bases: Result<Vec<_>> = s
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| {
                if c == 'U' || c == 'u' {
                    is_rna = true;
                }
                Nucleotide::from_char(c)
            })
            .collect();
        Ok(Self {
            bases: bases?,
            is_rna,
        })
    }

    /// Number of bases.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bases.len()
    }

    /// Whether the strand is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bases.is_empty()
    }

    /// Extract codons (triplets). Returns error if length not divisible by 3.
    pub fn codons(&self) -> Result<Vec<Codon>> {
        if self.bases.len() % 3 != 0 {
            return Err(DnaError::IncompleteCodon(self.bases.len()));
        }
        Ok(self
            .bases
            .chunks_exact(3)
            .map(|chunk| Codon(chunk[0], chunk[1], chunk[2]))
            .collect())
    }

    /// Get a display string for this strand.
    #[must_use]
    pub fn to_string_repr(&self) -> String {
        if self.is_rna {
            self.bases.iter().map(|n| n.as_rna_char()).collect()
        } else {
            self.bases.iter().map(|n| n.as_char()).collect()
        }
    }
}

impl std::fmt::Display for Strand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_repr())
    }
}

// ---------------------------------------------------------------------------
// DoubleHelix — σ Sequence + κ Comparison + ∃ Existence + μ Mapping (T2-C)
// ---------------------------------------------------------------------------

/// A double-stranded DNA molecule with sense and antisense strands.
///
/// Tier: T2-C (σ + κ + ∃ + μ)
///
/// The antisense strand is the reverse complement of the sense strand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoubleHelix {
    /// 5'→3' sense (coding) strand.
    pub sense: Strand,
    /// 3'→5' antisense (template) strand, stored 5'→3'.
    pub antisense: Strand,
}

impl DoubleHelix {
    /// Create a double helix from a sense strand.
    /// The antisense strand is automatically computed as the reverse complement.
    #[must_use]
    pub fn from_sense(sense: Strand) -> Self {
        let antisense_bases: Vec<Nucleotide> =
            sense.bases.iter().rev().map(|n| n.complement()).collect();
        let antisense = Strand::new(antisense_bases);
        Self { sense, antisense }
    }

    /// Validate that the strands are complementary.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if self.sense.len() != self.antisense.len() {
            return false;
        }
        self.sense
            .bases
            .iter()
            .zip(self.antisense.bases.iter().rev())
            .all(|(s, a)| s.complement() == *a)
    }

    /// Melt (denature) the double helix into two single strands.
    #[must_use]
    pub fn melt(self) -> (Strand, Strand) {
        (self.sense, self.antisense)
    }

    /// Attempt to anneal two strands into a double helix.
    /// Returns None if strands are not complementary.
    #[must_use]
    pub fn anneal(sense: Strand, antisense: Strand) -> Option<Self> {
        let helix = Self { sense, antisense };
        if helix.is_valid() { Some(helix) } else { None }
    }

    /// Number of base pairs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sense.len()
    }

    /// Whether the helix has no base pairs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sense.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nucleotide_complement_involution() {
        // complement(complement(x)) == x for all nucleotides
        for n in [Nucleotide::A, Nucleotide::T, Nucleotide::G, Nucleotide::C] {
            assert_eq!(n.complement().complement(), n);
        }
    }

    #[test]
    fn nucleotide_complement_pairing() {
        assert_eq!(Nucleotide::A.complement(), Nucleotide::T);
        assert_eq!(Nucleotide::T.complement(), Nucleotide::A);
        assert_eq!(Nucleotide::G.complement(), Nucleotide::C);
        assert_eq!(Nucleotide::C.complement(), Nucleotide::G);
    }

    #[test]
    fn nucleotide_from_char() {
        assert_eq!(Nucleotide::from_char('A').ok(), Some(Nucleotide::A));
        assert_eq!(Nucleotide::from_char('t').ok(), Some(Nucleotide::T));
        assert_eq!(Nucleotide::from_char('U').ok(), Some(Nucleotide::T));
        assert!(Nucleotide::from_char('X').is_err());
    }

    #[test]
    fn nucleotide_bits_roundtrip() {
        for n in [Nucleotide::A, Nucleotide::T, Nucleotide::G, Nucleotide::C] {
            assert_eq!(Nucleotide::from_bits(n.bits()).ok(), Some(n));
        }
    }

    #[test]
    fn codon_index_bijective() {
        // All 64 codons map to unique indices
        let mut seen = [false; 64];
        for i in 0u8..64 {
            let codon = Codon::from_index(i);
            assert!(codon.is_ok());
            let codon = codon.ok();
            if let Some(c) = codon {
                let idx = c.index();
                assert_eq!(idx, i, "codon {c} index mismatch: got {idx}, expected {i}");
                assert!(!seen[idx as usize], "duplicate index {idx}");
                seen[idx as usize] = true;
            }
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn codon_start_stop() {
        // v3 ISA: Entry=32(GAA), Halt=33(GAT), HaltErr=34(GAG), HaltYield=35(GAC)
        let entry = Codon(Nucleotide::G, Nucleotide::A, Nucleotide::A); // index 32
        assert!(entry.is_start());
        assert!(!entry.is_stop());

        let halt = Codon(Nucleotide::G, Nucleotide::A, Nucleotide::T); // index 33
        assert!(halt.is_stop());
        assert!(!halt.is_start());

        let halt_err = Codon(Nucleotide::G, Nucleotide::A, Nucleotide::G); // index 34
        assert!(halt_err.is_stop());

        let halt_yield = Codon(Nucleotide::G, Nucleotide::A, Nucleotide::C); // index 35
        assert!(halt_yield.is_stop());
    }

    #[test]
    fn strand_parse_dna() {
        let strand = Strand::parse("ATGCGA");
        assert!(strand.is_ok());
        let s = strand.ok();
        if let Some(s) = s {
            assert_eq!(s.len(), 6);
            assert!(!s.is_rna);
        }
    }

    #[test]
    fn strand_parse_rna() {
        let strand = Strand::parse("AUGCGA");
        assert!(strand.is_ok());
        let s = strand.ok();
        if let Some(s) = s {
            assert!(s.is_rna);
            assert_eq!(s.to_string_repr(), "AUGCGA");
        }
    }

    #[test]
    fn strand_codons() {
        let strand = Strand::parse("ATGCGA");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let codons = s.codons();
            assert!(codons.is_ok());
            if let Some(codons) = codons.ok() {
                assert_eq!(codons.len(), 2);
            }
        }
    }

    #[test]
    fn strand_incomplete_codon_error() {
        let strand = Strand::parse("ATGC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            assert!(s.codons().is_err());
        }
    }

    #[test]
    fn double_helix_from_sense() {
        let sense = Strand::parse("ATGC");
        assert!(sense.is_ok());
        if let Some(s) = sense.ok() {
            let helix = DoubleHelix::from_sense(s);
            assert!(helix.is_valid());
            assert_eq!(helix.len(), 4);
        }
    }

    #[test]
    fn double_helix_melt_anneal() {
        let sense = Strand::parse("ATGCGA");
        assert!(sense.is_ok());
        if let Some(s) = sense.ok() {
            let helix = DoubleHelix::from_sense(s);
            let (sense, antisense) = helix.melt();
            let reannealed = DoubleHelix::anneal(sense, antisense);
            assert!(reannealed.is_some());
        }
    }

    #[test]
    fn double_helix_invalid_anneal() {
        let s1 = Strand::parse("AAAA");
        let s2 = Strand::parse("AAAA");
        if let (Some(s1), Some(s2)) = (s1.ok(), s2.ok()) {
            let result = DoubleHelix::anneal(s1, s2);
            assert!(result.is_none());
        }
    }
}
