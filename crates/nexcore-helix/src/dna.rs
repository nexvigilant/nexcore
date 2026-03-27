//! DNA encoding for helix conservation states.
//!
//! Each conservation check (∂, ς, ∅) encodes as a 3-nucleotide codon:
//! - ∂ → first nucleotide (2 bits: A=0.0-0.25, T=0.25-0.5, G=0.5-0.75, C=0.75-1.0)
//! - ς → second nucleotide
//! - ∅ → third nucleotide
//!
//! 4^3 = 64 codons. 3 are stop codons (UAA, UAG, UGA → TAA, TAG, TGA in DNA).
//! Stop codons fire when any primitive is in the A band (0.0-0.25) — existence collapsing.
//!
//! A sequence of conservation checks encodes as a DNA strand.
//! The heligram reads this strand like a ribosome reads mRNA.
//!
//! Arithmetic on codons:
//! - Addition: per-nucleotide max (strengthening)
//! - Subtraction: per-nucleotide min (weakening)
//! - Complement: A↔T, G↔C (inversion — what was strong becomes weak)
//! - Translation: codon → ExistenceClass (the protein)

use crate::{ConservationInput, ExistenceClass, conservation};

/// A single nucleotide representing a quantized [0,1] value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nucleotide {
    /// A = 0b00 = [0.0, 0.25) — collapsing
    A = 0,
    /// T = 0b01 = [0.25, 0.5) — weak
    T = 1,
    /// G = 0b10 = [0.5, 0.75) — moderate
    G = 2,
    /// C = 0b11 = [0.75, 1.0] — strong
    C = 3,
}

impl Nucleotide {
    /// Quantize a [0,1] value to a nucleotide.
    pub fn from_unit(v: f64) -> Self {
        let clamped = v.clamp(0.0, 1.0);
        if clamped < 0.25 {
            Self::A
        } else if clamped < 0.5 {
            Self::T
        } else if clamped < 0.75 {
            Self::G
        } else {
            Self::C
        }
    }

    /// Decode nucleotide back to midpoint value.
    pub fn to_midpoint(self) -> f64 {
        match self {
            Self::A => 0.125,
            Self::T => 0.375,
            Self::G => 0.625,
            Self::C => 0.875,
        }
    }

    /// Complement: A↔T, G↔C.
    pub fn complement(self) -> Self {
        match self {
            Self::A => Self::T,
            Self::T => Self::A,
            Self::G => Self::C,
            Self::C => Self::G,
        }
    }

    /// As character.
    pub fn as_char(self) -> char {
        match self {
            Self::A => 'A',
            Self::T => 'T',
            Self::G => 'G',
            Self::C => 'C',
        }
    }

    /// From character.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'A' | 'a' => Some(Self::A),
            'T' | 't' => Some(Self::T),
            'G' | 'g' => Some(Self::G),
            'C' | 'c' => Some(Self::C),
            _ => None,
        }
    }

    /// Max of two nucleotides (strengthening).
    pub fn max(self, other: Self) -> Self {
        if (self as u8) >= (other as u8) {
            self
        } else {
            other
        }
    }

    /// Min of two nucleotides (weakening).
    pub fn min(self, other: Self) -> Self {
        if (self as u8) <= (other as u8) {
            self
        } else {
            other
        }
    }
}

/// A codon: 3 nucleotides encoding one conservation check (∂, ς, ∅).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Codon {
    /// ∂ — boundary
    pub boundary: Nucleotide,
    /// ς — state
    pub state: Nucleotide,
    /// ∅ — void
    pub void: Nucleotide,
}

impl Codon {
    /// Encode a conservation input as a codon.
    pub fn encode(input: ConservationInput) -> Self {
        Self {
            boundary: Nucleotide::from_unit(input.boundary),
            state: Nucleotide::from_unit(input.state),
            void: Nucleotide::from_unit(input.void),
        }
    }

    /// Decode codon back to conservation input (midpoint values).
    pub fn decode(self) -> ConservationInput {
        ConservationInput {
            boundary: self.boundary.to_midpoint(),
            state: self.state.to_midpoint(),
            void: self.void.to_midpoint(),
        }
    }

    /// Translate codon to existence class (the "protein").
    pub fn translate(self) -> ExistenceClass {
        conservation(self.decode()).classification
    }

    /// Is this a stop codon? (any primitive in A band = collapsing)
    pub fn is_stop(self) -> bool {
        self.boundary == Nucleotide::A || self.state == Nucleotide::A || self.void == Nucleotide::A
    }

    /// Complement: invert each nucleotide.
    pub fn complement(self) -> Self {
        Self {
            boundary: self.boundary.complement(),
            state: self.state.complement(),
            void: self.void.complement(),
        }
    }

    /// Add two codons: per-nucleotide max (strengthening).
    pub fn add(self, other: Self) -> Self {
        Self {
            boundary: self.boundary.max(other.boundary),
            state: self.state.max(other.state),
            void: self.void.max(other.void),
        }
    }

    /// Subtract: per-nucleotide min (weakening).
    pub fn sub(self, other: Self) -> Self {
        Self {
            boundary: self.boundary.min(other.boundary),
            state: self.state.min(other.state),
            void: self.void.min(other.void),
        }
    }

    /// 6-bit index (0-63) for lookup tables.
    pub fn index(self) -> u8 {
        ((self.boundary as u8) << 4) | ((self.state as u8) << 2) | (self.void as u8)
    }

    /// As 3-character string.
    pub fn as_str(self) -> String {
        format!(
            "{}{}{}",
            self.boundary.as_char(),
            self.state.as_char(),
            self.void.as_char()
        )
    }

    /// Parse from 3-character string.
    pub fn from_str(s: &str) -> Option<Self> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() != 3 {
            return None;
        }
        Some(Self {
            boundary: Nucleotide::from_char(chars[0])?,
            state: Nucleotide::from_char(chars[1])?,
            void: Nucleotide::from_char(chars[2])?,
        })
    }
}

/// A DNA strand: sequence of codons encoding a series of conservation checks.
#[derive(Debug, Clone)]
pub struct Strand {
    pub codons: Vec<Codon>,
}

impl Strand {
    /// Encode a sequence of conservation inputs as a DNA strand.
    pub fn encode(inputs: &[ConservationInput]) -> Self {
        Self {
            codons: inputs.iter().map(|i| Codon::encode(*i)).collect(),
        }
    }

    /// Translate the entire strand: each codon → ExistenceClass.
    /// Stops at the first stop codon (like a ribosome).
    pub fn translate(&self) -> Vec<ExistenceClass> {
        let mut proteins = Vec::new();
        for codon in &self.codons {
            if codon.is_stop() {
                break;
            }
            proteins.push(codon.translate());
        }
        proteins
    }

    /// Translate all codons including stop codons.
    pub fn translate_all(&self) -> Vec<ExistenceClass> {
        self.codons.iter().map(|c| c.translate()).collect()
    }

    /// As DNA string (concatenated codons).
    pub fn as_dna(&self) -> String {
        self.codons.iter().map(|c| c.as_str()).collect()
    }

    /// Complement the entire strand.
    pub fn complement(&self) -> Self {
        Self {
            codons: self.codons.iter().map(|c| c.complement()).collect(),
        }
    }

    /// Add two strands element-wise (pad shorter with AAA).
    pub fn add(&self, other: &Self) -> Self {
        let len = self.codons.len().max(other.codons.len());
        let aaa = Codon {
            boundary: Nucleotide::A,
            state: Nucleotide::A,
            void: Nucleotide::A,
        };
        let codons = (0..len)
            .map(|i| {
                let a = self.codons.get(i).copied().unwrap_or(aaa);
                let b = other.codons.get(i).copied().unwrap_or(aaa);
                a.add(b)
            })
            .collect();
        Self { codons }
    }

    /// Count stop codons in the strand.
    pub fn stop_count(&self) -> usize {
        self.codons.iter().filter(|c| c.is_stop()).count()
    }

    /// Overall health: fraction of non-stop codons.
    pub fn health(&self) -> f64 {
        if self.codons.is_empty() {
            return 0.0;
        }
        let non_stop = self.codons.iter().filter(|c| !c.is_stop()).count();
        non_stop as f64 / self.codons.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nucleotide_quantize() {
        assert_eq!(Nucleotide::from_unit(0.0), Nucleotide::A);
        assert_eq!(Nucleotide::from_unit(0.24), Nucleotide::A);
        assert_eq!(Nucleotide::from_unit(0.3), Nucleotide::T);
        assert_eq!(Nucleotide::from_unit(0.6), Nucleotide::G);
        assert_eq!(Nucleotide::from_unit(0.9), Nucleotide::C);
        assert_eq!(Nucleotide::from_unit(1.0), Nucleotide::C);
    }

    #[test]
    fn nucleotide_complement_involution() {
        for nuc in [Nucleotide::A, Nucleotide::T, Nucleotide::G, Nucleotide::C] {
            assert_eq!(nuc.complement().complement(), nuc);
        }
    }

    #[test]
    fn codon_encode_decode_roundtrip() {
        let input = ConservationInput {
            boundary: 0.9,
            state: 0.6,
            void: 0.8,
        };
        let codon = Codon::encode(input);
        assert_eq!(codon.as_str(), "CGC"); // C(0.9) G(0.6) C(0.8)
        let decoded = codon.decode();
        // Midpoints: C=0.875, G=0.625, C=0.875
        assert!((decoded.boundary - 0.875).abs() < 0.001);
        assert!((decoded.state - 0.625).abs() < 0.001);
        assert!((decoded.void - 0.875).abs() < 0.001);
    }

    #[test]
    fn codon_translate() {
        // CCC = all strong → strong existence
        let strong = Codon {
            boundary: Nucleotide::C,
            state: Nucleotide::C,
            void: Nucleotide::C,
        };
        assert_eq!(strong.translate(), ExistenceClass::Strong);

        // AAA = all collapsing → collapsing
        let collapse = Codon {
            boundary: Nucleotide::A,
            state: Nucleotide::A,
            void: Nucleotide::A,
        };
        assert_eq!(collapse.translate(), ExistenceClass::Collapsing);
    }

    #[test]
    fn codon_is_stop() {
        let stop = Codon {
            boundary: Nucleotide::A,
            state: Nucleotide::C,
            void: Nucleotide::C,
        };
        assert!(stop.is_stop()); // boundary in A band

        let go = Codon {
            boundary: Nucleotide::T,
            state: Nucleotide::T,
            void: Nucleotide::T,
        };
        assert!(!go.is_stop());
    }

    #[test]
    fn codon_arithmetic() {
        let a = Codon {
            boundary: Nucleotide::T,
            state: Nucleotide::G,
            void: Nucleotide::A,
        };
        let b = Codon {
            boundary: Nucleotide::G,
            state: Nucleotide::T,
            void: Nucleotide::C,
        };

        let sum = a.add(b); // per-nucleotide max
        assert_eq!(sum.boundary, Nucleotide::G);
        assert_eq!(sum.state, Nucleotide::G);
        assert_eq!(sum.void, Nucleotide::C);

        let diff = a.sub(b); // per-nucleotide min
        assert_eq!(diff.boundary, Nucleotide::T);
        assert_eq!(diff.state, Nucleotide::T);
        assert_eq!(diff.void, Nucleotide::A);
    }

    #[test]
    fn strand_translate_stops_at_stop_codon() {
        let strand = Strand {
            codons: vec![
                Codon {
                    boundary: Nucleotide::C,
                    state: Nucleotide::C,
                    void: Nucleotide::C,
                }, // strong
                Codon {
                    boundary: Nucleotide::G,
                    state: Nucleotide::G,
                    void: Nucleotide::G,
                }, // moderate
                Codon {
                    boundary: Nucleotide::A,
                    state: Nucleotide::C,
                    void: Nucleotide::C,
                }, // STOP
                Codon {
                    boundary: Nucleotide::C,
                    state: Nucleotide::C,
                    void: Nucleotide::C,
                }, // never reached
            ],
        };
        let proteins = strand.translate();
        assert_eq!(proteins.len(), 2); // stops before the stop codon
        assert_eq!(proteins[0], ExistenceClass::Strong);
    }

    #[test]
    fn nexvigilant_system_encodes() {
        // NexVigilant actual: ∂=0.97, ς=0.14, ∅=0.67
        let input = ConservationInput {
            boundary: 0.97,
            state: 0.14,
            void: 0.67,
        };
        let codon = Codon::encode(input);
        assert_eq!(codon.as_str(), "CAG"); // C(0.97) A(0.14) G(0.67)
        assert!(codon.is_stop()); // A in state = stop codon!
        // Decoded midpoints: 0.875 * 0.125 * 0.625 = 0.068 → weak (≥0.05)
        assert_eq!(codon.translate(), ExistenceClass::Weak);
    }

    #[test]
    fn strand_health() {
        let strand = Strand {
            codons: vec![
                Codon {
                    boundary: Nucleotide::C,
                    state: Nucleotide::C,
                    void: Nucleotide::C,
                },
                Codon {
                    boundary: Nucleotide::A,
                    state: Nucleotide::C,
                    void: Nucleotide::C,
                },
                Codon {
                    boundary: Nucleotide::G,
                    state: Nucleotide::G,
                    void: Nucleotide::G,
                },
            ],
        };
        // 2/3 non-stop = 0.667
        assert!((strand.health() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn product_portfolio_as_strand() {
        // Encode all 5 NexVigilant products as a strand
        let products = vec![
            ConservationInput {
                boundary: 0.95,
                state: 0.85,
                void: 0.7,
            }, // Station: C C G
            ConservationInput {
                boundary: 0.6,
                state: 0.4,
                void: 0.3,
            }, // Nucleus: G T T
            ConservationInput {
                boundary: 0.9,
                state: 0.9,
                void: 0.8,
            }, // Micrograms: C C C
            ConservationInput {
                boundary: 0.85,
                state: 0.7,
                void: 0.6,
            }, // NexCore: C G G
            ConservationInput {
                boundary: 0.3,
                state: 0.15,
                void: 0.2,
            }, // Academy: T A A
        ];
        let strand = Strand::encode(&products);
        assert_eq!(strand.as_dna(), "CCGGTTCCCCGGTAA");
        // Academy=TAA is a stop codon (ς and ∅ both in A band)
        // Ribosome translation stops at Academy
        let proteins = strand.translate();
        assert_eq!(proteins.len(), 4); // Station, Nucleus, Micrograms, NexCore — stops before Academy
        let all = strand.translate_all();
        assert_eq!(all.len(), 5);
        assert_eq!(strand.stop_count(), 1); // only Academy
        // Health: 4/5 non-stop = 80%
        assert!((strand.health() - 0.8).abs() < 0.01);
    }
}
