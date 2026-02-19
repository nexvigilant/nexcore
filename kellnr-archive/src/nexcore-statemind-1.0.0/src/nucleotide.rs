//! Stage 1: Character → Nucleotide Encoding.
//!
//! Surjective mapping from UTF-8 bytes to quaternary DNA alphabet.
//! Each byte → 4 nucleotides (2-bit extraction). Framed with ATG start
//! and TAA stop codons for biological authenticity.
//!
//! Tier: T1 | Dominant: μ (Mapping) — deterministic char→base function.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The four DNA nucleotide bases.
///
/// Tier: T1 | Grounds to: μ (Mapping) — alphabet of 4 symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Nucleotide {
    /// Adenine — purine, pairs with Thymine.
    A,
    /// Thymine — pyrimidine, pairs with Adenine.
    T,
    /// Guanine — purine, pairs with Cytosine.
    G,
    /// Cytosine — pyrimidine, pairs with Guanine.
    C,
}

impl Nucleotide {
    /// All four bases in canonical order.
    pub const ALL: [Self; 4] = [Self::A, Self::T, Self::G, Self::C];

    /// Watson-Crick complement.
    #[must_use]
    pub const fn complement(self) -> Self {
        match self {
            Self::A => Self::T,
            Self::T => Self::A,
            Self::G => Self::C,
            Self::C => Self::G,
        }
    }

    /// Whether this is a purine (A or G — larger, two-ring).
    #[must_use]
    pub const fn is_purine(self) -> bool {
        matches!(self, Self::A | Self::G)
    }

    /// Whether this is a pyrimidine (T or C — smaller, one-ring).
    #[must_use]
    pub const fn is_pyrimidine(self) -> bool {
        matches!(self, Self::T | Self::C)
    }

    /// Single-character representation.
    #[must_use]
    pub const fn as_char(self) -> char {
        match self {
            Self::A => 'A',
            Self::T => 'T',
            Self::G => 'G',
            Self::C => 'C',
        }
    }
}

impl fmt::Display for Nucleotide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

/// A DNA sequence — ordered collection of nucleotides.
///
/// Tier: T2-P | Grounds to: σ (Sequence) + μ (Mapping).
#[derive(Debug, Clone, Serialize)]
pub struct DnaSequence {
    bases: Vec<Nucleotide>,
}

impl DnaSequence {
    /// Create a sequence from a vector of nucleotides.
    #[must_use]
    pub fn new(bases: Vec<Nucleotide>) -> Self {
        Self { bases }
    }

    /// Number of nucleotides.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bases.len()
    }

    /// Whether the sequence is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bases.is_empty()
    }

    /// Access the underlying bases.
    #[must_use]
    pub fn bases(&self) -> &[Nucleotide] {
        &self.bases
    }

    /// Render as a nucleotide string (e.g., "ATGCCG...TAA").
    #[must_use]
    pub fn to_string_repr(&self) -> String {
        self.bases.iter().map(|b| b.as_char()).collect()
    }

    /// Count occurrences of a specific base.
    #[must_use]
    pub fn count(&self, base: Nucleotide) -> usize {
        self.bases.iter().filter(|&&b| b == base).count()
    }

    /// GC content as a fraction [0.0, 1.0].
    #[must_use]
    pub fn gc_content(&self) -> f64 {
        if self.bases.is_empty() {
            return 0.0;
        }
        (self.count(Nucleotide::G) + self.count(Nucleotide::C)) as f64 / self.bases.len() as f64
    }

    /// AT content as a fraction [0.0, 1.0].
    #[must_use]
    pub fn at_content(&self) -> f64 {
        if self.bases.is_empty() {
            return 0.0;
        }
        (self.count(Nucleotide::A) + self.count(Nucleotide::T)) as f64 / self.bases.len() as f64
    }
}

impl fmt::Display for DnaSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({} nt)", self.to_string_repr(), self.len())
    }
}

/// Start codon — ATG (methionine, universal start signal).
pub const START_CODON: [Nucleotide; 3] = [Nucleotide::A, Nucleotide::T, Nucleotide::G];

/// Stop codon — TAA (ochre, most common stop signal).
pub const STOP_CODON: [Nucleotide; 3] = [Nucleotide::T, Nucleotide::A, Nucleotide::A];

/// Map 2 bits to a nucleotide.
///
/// Encoding: 00→A, 01→C, 10→G, 11→T
#[must_use]
const fn from_bits(bits: u8) -> Nucleotide {
    match bits & 0x03 {
        0 => Nucleotide::A,
        1 => Nucleotide::C,
        2 => Nucleotide::G,
        _ => Nucleotide::T, // 3 — exhaustive by mask
    }
}

/// Encode a string into a DNA sequence.
///
/// Each byte → 4 nucleotides (2-bit extraction from MSB to LSB).
/// Framed with ATG start codon and TAA stop codon.
///
/// For input of length n: output length = 4n + 6.
///
/// # Examples
///
/// "A" (ASCII 65 = 0b01000001) → ATG + CAAC + TAA = 10 nucleotides
#[must_use]
pub fn encode(input: &str) -> DnaSequence {
    let mut bases = Vec::with_capacity(input.len() * 4 + 6);

    // Start codon
    bases.extend_from_slice(&START_CODON);

    // Encode each byte as 4 nucleotides
    for &byte in input.as_bytes() {
        bases.push(from_bits((byte >> 6) & 0x03));
        bases.push(from_bits((byte >> 4) & 0x03));
        bases.push(from_bits((byte >> 2) & 0x03));
        bases.push(from_bits(byte & 0x03));
    }

    // Stop codon
    bases.extend_from_slice(&STOP_CODON);

    DnaSequence::new(bases)
}

/// Encode without start/stop codons (raw coding region only).
#[must_use]
pub fn encode_raw(input: &str) -> DnaSequence {
    let mut bases = Vec::with_capacity(input.len() * 4);
    for &byte in input.as_bytes() {
        bases.push(from_bits((byte >> 6) & 0x03));
        bases.push(from_bits((byte >> 4) & 0x03));
        bases.push(from_bits((byte >> 2) & 0x03));
        bases.push(from_bits(byte & 0x03));
    }
    DnaSequence::new(bases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_length() {
        let seq = encode("NexVigilant");
        // 11 chars × 4 + 6 (start + stop) = 50
        assert_eq!(seq.len(), 50);
    }

    #[test]
    fn encode_raw_length() {
        let seq = encode_raw("NexVigilant");
        assert_eq!(seq.len(), 44); // 11 × 4
    }

    #[test]
    fn starts_with_atg() {
        let seq = encode("test");
        assert_eq!(seq.bases()[0], Nucleotide::A);
        assert_eq!(seq.bases()[1], Nucleotide::T);
        assert_eq!(seq.bases()[2], Nucleotide::G);
    }

    #[test]
    fn ends_with_taa() {
        let seq = encode("test");
        let n = seq.len();
        assert_eq!(seq.bases()[n - 3], Nucleotide::T);
        assert_eq!(seq.bases()[n - 2], Nucleotide::A);
        assert_eq!(seq.bases()[n - 1], Nucleotide::A);
    }

    #[test]
    fn complement_involution() {
        for base in Nucleotide::ALL {
            assert_eq!(base.complement().complement(), base);
        }
    }

    #[test]
    fn purine_pyrimidine_partition() {
        for base in Nucleotide::ALL {
            assert_ne!(base.is_purine(), base.is_pyrimidine());
        }
    }

    #[test]
    fn deterministic_encoding() {
        let a = encode("NexVigilant");
        let b = encode("NexVigilant");
        assert_eq!(a.bases(), b.bases());
    }

    #[test]
    fn gc_at_sum_to_one() {
        let seq = encode("NexVigilant");
        let sum = seq.gc_content() + seq.at_content();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn empty_string_encodes() {
        let seq = encode("");
        assert_eq!(seq.len(), 6); // just start + stop codons
    }
}
