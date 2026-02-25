//! Hexadecimal encoding and decoding (RFC 4648 §8, Base16).
//!
//! Zero-dependency replacement for the `hex` crate.
//!
//! # Supply Chain Sovereignty
//!
//! This module has **zero external dependencies**. It replaces the `hex` crate
//! for the `nexcore` ecosystem, eliminating supply chain risk for hex encoding.
//!
//! # Examples
//!
//! ```
//! use nexcore_codec::hex;
//!
//! let encoded = hex::encode(b"Hello");
//! assert_eq!(encoded, "48656c6c6f");
//!
//! let decoded = hex::decode("48656c6c6f").unwrap();
//! assert_eq!(decoded, b"Hello");
//! ```

/// Hex character lookup table (lowercase).
const HEX_CHARS_LOWER: &[u8; 16] = b"0123456789abcdef";

/// Uppercase hex lookup table.
const HEX_CHARS_UPPER: &[u8; 16] = b"0123456789ABCDEF";

/// Look up a nibble (0..=15) in a hex character table.
///
/// `nibble` must be in 0..=15. For `encode` callers this is guaranteed by
/// the bitwise operations `byte >> 4` (gives 0..=15) and `byte & 0x0f`
/// (gives 0..=15).
#[inline]
fn hex_char(table: &[u8; 16], nibble: u8) -> char {
    // SAFETY PROOF: `nibble` is always derived from `byte >> 4` or
    // `byte & 0x0f`, so it is in 0..=15. The table has exactly 16 elements
    // (indices 0..=15), making this indexing always in bounds. All values in
    // the table are printable ASCII (< 128), so casting to `char` is valid.
    #[allow(
        clippy::indexing_slicing,
        reason = "nibble is always byte >> 4 or byte & 0x0f, which is 0..=15; table has 16 elements"
    )]
    #[allow(
        clippy::as_conversions,
        reason = "table bytes are ASCII digits/letters (0-127); casting u8 to char is always valid here"
    )]
    (table[usize::from(nibble)] as char)
}

/// Encode bytes to a lowercase hex string.
///
/// Equivalent to `hex::encode()`.
#[must_use]
pub fn encode(input: impl AsRef<[u8]>) -> String {
    let input = input.as_ref();
    // Each byte expands to exactly 2 hex characters; saturating_mul is
    // sufficient — inputs large enough to overflow usize would OOM first.
    let mut out = String::with_capacity(input.len().saturating_mul(2));
    for &byte in input {
        out.push(hex_char(HEX_CHARS_LOWER, byte >> 4));
        out.push(hex_char(HEX_CHARS_LOWER, byte & 0x0f));
    }
    out
}

/// Encode bytes to an uppercase hex string.
#[must_use]
pub fn encode_upper(input: impl AsRef<[u8]>) -> String {
    let input = input.as_ref();
    let mut out = String::with_capacity(input.len().saturating_mul(2));
    for &byte in input {
        out.push(hex_char(HEX_CHARS_UPPER, byte >> 4));
        out.push(hex_char(HEX_CHARS_UPPER, byte & 0x0f));
    }
    out
}

/// Error returned when decoding an invalid hex string.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Input has odd length (hex requires pairs).
    OddLength,
    /// Invalid hex character at the given byte index.
    InvalidChar { index: usize, byte: u8 },
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OddLength => write!(f, "odd-length hex string"),
            Self::InvalidChar { index, byte } => {
                write!(f, "invalid hex char 0x{byte:02x} at index {index}")
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Decode a hex string to bytes.
///
/// Accepts both uppercase and lowercase hex characters.
/// Returns `Err` on odd length or invalid characters.
///
/// Equivalent to `hex::decode()`.
pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
    let input = input.as_ref();
    if input.len() % 2 != 0 {
        return Err(DecodeError::OddLength);
    }
    let mut out = Vec::with_capacity(input.len() / 2);
    for pair in input.chunks_exact(2) {
        // chunks_exact(2) guarantees pair.len() == 2; indices 0 and 1 are
        // always valid. The `hex_val` function returns 0..=15, so
        // `(high << 4) | low` is at most 0xF0 | 0x0F = 0xFF, which fits in u8.
        #[allow(
            clippy::indexing_slicing,
            reason = "chunks_exact(2) guarantees pair.len() == 2; indices 0 and 1 are always valid"
        )]
        let high = hex_val(pair[0], 0)?;
        #[allow(
            clippy::indexing_slicing,
            reason = "chunks_exact(2) guarantees pair.len() == 2; indices 0 and 1 are always valid"
        )]
        let low = hex_val(pair[1], 1)?;
        // `high` is 0..=15 from hex_val, so `high << 4` is 0..=240.
        // `low` is 0..=15, so `(high << 4) | low` is 0..=255; fits in u8.
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "high is 0..=15 (from hex_val), so high << 4 is 0..=240; OR with low (0..=15) gives 0..=255; no overflow"
        )]
        out.push((high << 4) | low);
    }
    Ok(out)
}

/// Convert a hex ASCII byte to its numeric value (0..=15).
#[inline]
const fn hex_val(byte: u8, offset: usize) -> Result<u8, DecodeError> {
    match byte {
        // Match arm guards prove safety for each arm:
        // b'0'..=b'9': byte >= b'0', so byte - b'0' is in 0..=9.
        // b'a'..=b'f': byte >= b'a', so byte - b'a' is in 0..=5;
        //   adding 10 gives 10..=15, fitting in u8.
        // b'A'..=b'F': byte >= b'A', so byte - b'A' is in 0..=5;
        //   adding 10 gives 10..=15, fitting in u8.
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "match arm guard proves byte >= b'0'; subtraction gives 0..=9 which fits in u8"
        )]
        b'0'..=b'9' => Ok(byte - b'0'),
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "match arm guard proves byte >= b'a' and byte - b'a' <= 5; adding 10 gives 10..=15, fitting in u8"
        )]
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "match arm guard proves byte >= b'A' and byte - b'A' <= 5; adding 10 gives 10..=15, fitting in u8"
        )]
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(DecodeError::InvalidChar {
            index: offset,
            byte,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_empty() {
        assert_eq!(encode(b""), "");
    }

    #[test]
    fn encode_hello() {
        assert_eq!(encode(b"Hello"), "48656c6c6f");
    }

    #[test]
    fn encode_all_bytes() {
        let input: Vec<u8> = (0..=255).collect();
        let encoded = encode(&input);
        assert_eq!(encoded.len(), 512);
        assert!(encoded.starts_with("000102"));
        assert!(encoded.ends_with("fdfeff"));
    }

    #[test]
    fn encode_upper_hello() {
        assert_eq!(encode_upper(b"Hello"), "48656C6C6F");
    }

    #[test]
    fn decode_empty() {
        assert_eq!(decode("").ok(), Some(vec![]));
    }

    #[test]
    fn decode_hello() {
        assert_eq!(decode("48656c6c6f").ok(), Some(b"Hello".to_vec()));
    }

    #[test]
    fn decode_uppercase() {
        assert_eq!(decode("48656C6C6F").ok(), Some(b"Hello".to_vec()));
    }

    #[test]
    fn decode_mixed_case() {
        assert_eq!(decode("48656C6c6F").ok(), Some(b"Hello".to_vec()));
    }

    #[test]
    fn decode_odd_length() {
        assert_eq!(decode("abc"), Err(DecodeError::OddLength));
    }

    #[test]
    fn decode_invalid_char() {
        let err = decode("zz");
        assert!(matches!(err, Err(DecodeError::InvalidChar { .. })));
    }

    #[test]
    fn roundtrip_all_bytes() {
        let input: Vec<u8> = (0..=255).collect();
        let encoded = encode(&input);
        let decoded = decode(&encoded);
        assert_eq!(decoded.ok(), Some(input));
    }

    // RFC 4648 §10 test vectors for Base16
    #[test]
    fn rfc4648_test_vectors() {
        let vectors = [
            ("", ""),
            ("f", "66"),
            ("fo", "666f"),
            ("foo", "666f6f"),
            ("foob", "666f6f62"),
            ("fooba", "666f6f6261"),
            ("foobar", "666f6f626172"),
        ];
        for (input, expected) in vectors {
            assert_eq!(encode(input.as_bytes()), expected, "encode({input:?})");
            assert_eq!(
                decode(expected).ok(),
                Some(input.as_bytes().to_vec()),
                "decode({expected:?})"
            );
        }
    }
}
