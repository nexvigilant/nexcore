//! Bijective byte↔DNA codec.
//!
//! Encoding: each byte → 4 nucleotides (2 bits per nucleotide, MSB first).
//! A=0b00, T=0b01, G=0b10, C=0b11.
//!
//! Domain: 256 bytes ↔ 256 tetrads. Perfect bijection (4^4 = 256).

use crate::error::{NucliError, Result};

// ---------------------------------------------------------------------------
// Nucleotide ↔ 2-bit mapping
// ---------------------------------------------------------------------------

/// Convert 2-bit value to nucleotide character.
const fn bits_to_nuc(bits: u8) -> char {
    match bits & 0b11 {
        0b01 => 'T',
        0b10 => 'G',
        0b11 => 'C',
        _ => 'A', // 0b00; exhaustive after mask, const fn can't panic
    }
}

/// Convert nucleotide character to 2-bit value.
fn nuc_to_bits(ch: char) -> Result<u8> {
    match ch {
        'A' | 'a' => Ok(0b00),
        'T' | 't' => Ok(0b01),
        'G' | 'g' => Ok(0b10),
        'C' | 'c' => Ok(0b11),
        other => Err(NucliError::InvalidNucleotide(other)),
    }
}

// ---------------------------------------------------------------------------
// Encode: bytes → DNA strand
// ---------------------------------------------------------------------------

/// Encode a byte slice as a DNA strand string.
///
/// Each byte becomes 4 nucleotides (MSB first).
/// Returns empty string for empty input.
pub fn encode(data: &[u8]) -> String {
    let mut strand = String::with_capacity(data.len() * 4);
    for &byte in data {
        strand.push(bits_to_nuc((byte >> 6) & 0b11));
        strand.push(bits_to_nuc((byte >> 4) & 0b11));
        strand.push(bits_to_nuc((byte >> 2) & 0b11));
        strand.push(bits_to_nuc(byte & 0b11));
    }
    strand
}

// ---------------------------------------------------------------------------
// Decode: DNA strand → bytes
// ---------------------------------------------------------------------------

/// Decode a DNA strand string back to bytes.
///
/// Strand length must be divisible by 4 (each tetrad = 1 byte).
pub fn decode(strand: &str) -> Result<Vec<u8>> {
    if strand.is_empty() {
        return Err(NucliError::EmptyInput);
    }
    if strand.len() % 4 != 0 {
        return Err(NucliError::IncompleteTetrad(strand.len()));
    }

    let chars: Vec<char> = strand.chars().collect();
    let mut bytes = Vec::with_capacity(chars.len() / 4);

    for chunk in chars.chunks(4) {
        let b3 = nuc_to_bits(chunk[0])?;
        let b2 = nuc_to_bits(chunk[1])?;
        let b1 = nuc_to_bits(chunk[2])?;
        let b0 = nuc_to_bits(chunk[3])?;
        bytes.push((b3 << 6) | (b2 << 4) | (b1 << 2) | b0);
    }

    Ok(bytes)
}

// ---------------------------------------------------------------------------
// Validate
// ---------------------------------------------------------------------------

/// Check that a strand contains only valid nucleotides and has correct length.
pub fn validate(strand: &str) -> Result<()> {
    if strand.is_empty() {
        return Err(NucliError::EmptyInput);
    }
    if strand.len() % 4 != 0 {
        return Err(NucliError::IncompleteTetrad(strand.len()));
    }
    for ch in strand.chars() {
        nuc_to_bits(ch)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_single_byte() {
        // 0x00 = 0b00_00_00_00 → AAAA
        assert_eq!(encode(&[0x00]), "AAAA");
        // 0xFF = 0b11_11_11_11 → CCCC
        assert_eq!(encode(&[0xFF]), "CCCC");
        // 0x48 = 0b01_00_10_00 → TAGA
        assert_eq!(encode(&[0x48]), "TAGA");
    }

    #[test]
    fn decode_single_byte() {
        assert_eq!(decode("AAAA").unwrap(), vec![0x00]);
        assert_eq!(decode("CCCC").unwrap(), vec![0xFF]);
        assert_eq!(decode("TAGA").unwrap(), vec![0x48]);
    }

    #[test]
    fn encode_hello() {
        let strand = encode(b"Hi");
        // H=0x48=TAGA, i=0x69=TGGT
        assert_eq!(strand, "TAGATGGT");
    }

    #[test]
    fn decode_hello() {
        let bytes = decode("TAGATGGT").unwrap();
        assert_eq!(bytes, b"Hi");
    }

    #[test]
    fn roundtrip_ascii() {
        let text = "Hello, World!";
        let strand = encode(text.as_bytes());
        let decoded = decode(&strand).unwrap();
        assert_eq!(decoded, text.as_bytes());
    }

    #[test]
    fn decode_empty_is_error() {
        assert_eq!(decode(""), Err(NucliError::EmptyInput));
    }

    #[test]
    fn decode_incomplete_is_error() {
        assert_eq!(decode("ATG"), Err(NucliError::IncompleteTetrad(3)));
    }

    #[test]
    fn decode_invalid_char_is_error() {
        assert!(matches!(
            decode("AXGT"),
            Err(NucliError::InvalidNucleotide('X'))
        ));
    }

    #[test]
    fn validate_good() {
        assert!(validate("ATGC").is_ok());
        assert!(validate("ATGCATGC").is_ok());
    }

    #[test]
    fn validate_bad_length() {
        assert_eq!(validate("ATG"), Err(NucliError::IncompleteTetrad(3)));
    }

    #[test]
    fn validate_bad_char() {
        assert!(matches!(
            validate("ATXC"),
            Err(NucliError::InvalidNucleotide('X'))
        ));
    }

    #[test]
    fn case_insensitive_decode() {
        assert_eq!(decode("atgc").unwrap(), decode("ATGC").unwrap());
    }
}
