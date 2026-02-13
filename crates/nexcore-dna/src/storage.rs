//! DNA data storage: encode/decode bytes ↔ nucleotide sequences.
//!
//! Encoding scheme: 2 bits per nucleotide, 4 nucleotides per byte.
//! A 4-nucleotide header encodes the original byte length (up to 256^2 = 65536).
//! Format: [length_high_byte as 4 nucs][length_low_byte as 4 nucs][data nucs...]

use crate::error::Result;
use crate::types::{Nucleotide, Strand};

/// Encode a byte slice into a DNA strand.
///
/// Each byte maps to 4 nucleotides (2 bits each).
/// Prefixed with 8 nucleotides encoding the length as a u32 (4 bytes → 16 nucs).
///
/// Tier: T2-P (σ Sequence + μ Mapping + π Persistence)
pub fn encode(data: &[u8]) -> Strand {
    let len = data.len() as u32;
    let len_bytes = len.to_be_bytes();

    let capacity = 16 + data.len() * 4; // 4 bytes * 4 nucs + data * 4 nucs
    let mut bases = Vec::with_capacity(capacity);

    // Encode length header (4 bytes = 16 nucleotides)
    for &b in &len_bytes {
        encode_byte(b, &mut bases);
    }

    // Encode data
    for &b in data {
        encode_byte(b, &mut bases);
    }

    Strand::new(bases)
}

/// Decode a DNA strand back into bytes.
///
/// Reverses the encoding: reads the length header, then decodes that many bytes.
pub fn decode(strand: &Strand) -> Result<Vec<u8>> {
    if strand.bases.len() < 16 {
        // Not enough for length header
        return Ok(Vec::new());
    }

    // Decode length header (4 bytes = 16 nucleotides)
    let len_bytes = [
        decode_byte(&strand.bases[0..4]),
        decode_byte(&strand.bases[4..8]),
        decode_byte(&strand.bases[8..12]),
        decode_byte(&strand.bases[12..16]),
    ];
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut result = Vec::with_capacity(len);
    let data_start = 16;

    for i in 0..len {
        let offset = data_start + i * 4;
        if offset + 4 > strand.bases.len() {
            break;
        }
        result.push(decode_byte(&strand.bases[offset..offset + 4]));
    }

    Ok(result)
}

/// Encode a string as DNA.
pub fn encode_str(s: &str) -> Strand {
    encode(s.as_bytes())
}

/// Decode DNA back to a UTF-8 string.
pub fn decode_str(strand: &Strand) -> Result<String> {
    let bytes = decode(strand)?;
    String::from_utf8(bytes).map_err(|_| crate::error::DnaError::InvalidBase('?'))
}

// --- Internal helpers ---

fn encode_byte(byte: u8, bases: &mut Vec<Nucleotide>) {
    // High bits first: bits 7-6, 5-4, 3-2, 1-0
    for shift in (0..4).rev() {
        let bits = (byte >> (shift * 2)) & 0b11;
        // from_bits with masked input always succeeds
        if let Ok(nuc) = Nucleotide::from_bits(bits) {
            bases.push(nuc);
        }
    }
}

fn decode_byte(bases: &[Nucleotide]) -> u8 {
    let mut byte: u8 = 0;
    for (i, &nuc) in bases.iter().enumerate().take(4) {
        let shift = (3 - i) * 2;
        byte |= nuc.bits() << shift;
    }
    byte
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_empty() {
        let encoded = encode(&[]);
        let decoded = decode(&encoded);
        assert!(decoded.is_ok());
        if let Some(d) = decoded.ok() {
            assert!(d.is_empty());
        }
    }

    #[test]
    fn encode_decode_single_byte() {
        let data = [0x42u8]; // 'B'
        let encoded = encode(&data);
        let decoded = decode(&encoded);
        assert!(decoded.is_ok());
        if let Some(d) = decoded.ok() {
            assert_eq!(d, data);
        }
    }

    #[test]
    fn encode_decode_all_byte_values() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = encode(&data);
        let decoded = decode(&encoded);
        assert!(decoded.is_ok());
        if let Some(d) = decoded.ok() {
            assert_eq!(d, data);
        }
    }

    #[test]
    fn encode_decode_string() {
        let msg = "Hello, DNA!";
        let encoded = encode_str(msg);
        let decoded = decode_str(&encoded);
        assert!(decoded.is_ok());
        if let Some(d) = decoded.ok() {
            assert_eq!(d, msg);
        }
    }

    #[test]
    fn encode_decode_roundtrip_many() {
        for data in [
            vec![],
            vec![0u8],
            vec![255u8],
            vec![0, 1, 2, 3],
            vec![0xDE, 0xAD, 0xBE, 0xEF],
            b"NexCore DNA".to_vec(),
        ] {
            let encoded = encode(&data);
            let decoded = decode(&encoded);
            assert!(decoded.is_ok());
            if let Some(d) = decoded.ok() {
                assert_eq!(d, data, "roundtrip failed for {data:?}");
            }
        }
    }

    #[test]
    fn encoding_uses_4_nucs_per_byte() {
        let data = [0x00, 0xFF];
        let encoded = encode(&data);
        // 16 nucs header + 2 bytes * 4 nucs = 24
        assert_eq!(encoded.len(), 24);
    }

    #[test]
    fn byte_encoding_correctness() {
        // 0b00_01_10_11 = 0x1B → A, T, G, C
        let data = [0b00_01_10_11u8];
        let encoded = encode(&data);
        // Skip 16-nuc header, check data nucleotides
        assert_eq!(encoded.bases[16], Nucleotide::A);
        assert_eq!(encoded.bases[17], Nucleotide::T);
        assert_eq!(encoded.bases[18], Nucleotide::G);
        assert_eq!(encoded.bases[19], Nucleotide::C);
    }
}
