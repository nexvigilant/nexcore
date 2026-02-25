#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![forbid(unsafe_code)]

//! nucli — Nucleotide text encoder with exhaustive proofs.
//!
//! Bijective mapping: every byte (0-255) encodes to exactly 4 nucleotides (A,T,G,C).
//! 4^4 = 256 = byte range. Perfect bijection, proven exhaustively.
//!
//! ## Encoding Scheme
//!
//! Each nucleotide carries 2 bits: A=0b00, T=0b01, G=0b10, C=0b11.
//! A byte is split into 4 pairs of 2 bits, MSB first:
//!
//! ```text
//! byte 0x48 = 0b01_00_10_00
//!              T   A   G   A  → "TAGA"
//! ```
//!
//! ## ELL Principles Applied
//!
//! - L1: Exhaustive proof over bounded domain (256 bytes, 256 tetrads)
//! - L2: 4 property classes tested (roundtrip, injectivity, involution, determinism)
//! - L3: Error Display tests from Day 1
//! - L4: Proofs written alongside code

pub mod codec;
pub mod error;
pub mod grounding;
mod proofs;

/// Reverse complement: reverse the strand and swap A↔T, G↔C.
///
/// This is an involution: complement(complement(s)) == s.
pub fn complement(strand: &str) -> String {
    strand
        .chars()
        .rev()
        .map(|c| match c {
            'A' => 'T',
            'T' => 'A',
            'G' => 'C',
            'C' => 'G',
            other => other,
        })
        .collect()
}
