//! Base64 encoding and decoding (RFC 4648 §4 + §5).
//!
//! Zero-dependency replacement for the `base64` crate.
//!
//! # Supply Chain Sovereignty
//!
//! This module has **zero external dependencies**. It replaces the `base64` crate
//! for the `nexcore` ecosystem.
//!
//! # Alphabets
//!
//! - **Standard** (§4): `A-Z a-z 0-9 + /` with `=` padding
//! - **URL-safe** (§5): `A-Z a-z 0-9 - _` with optional padding
//!
//! # Examples
//!
//! ```
//! use nexcore_codec::base64;
//!
//! let encoded = base64::encode(b"Hello, World!");
//! assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");
//!
//! let decoded = base64::decode("SGVsbG8sIFdvcmxkIQ==").unwrap();
//! assert_eq!(decoded, b"Hello, World!");
//! ```

/// Standard Base64 alphabet (RFC 4648 §4).
const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// URL-safe Base64 alphabet (RFC 4648 §5).
const URL_SAFE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode bytes using standard Base64 with `=` padding.
#[must_use]
pub fn encode(input: impl AsRef<[u8]>) -> String {
    encode_with_alphabet(input.as_ref(), STANDARD, true)
}

/// Decode a standard Base64 string (with or without padding).
pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
    decode_with_alphabet(input.as_ref(), false)
}

/// Encode bytes using URL-safe Base64 without padding.
#[must_use]
pub fn encode_url_safe_no_pad(input: impl AsRef<[u8]>) -> String {
    encode_with_alphabet(input.as_ref(), URL_SAFE, false)
}

/// Decode a URL-safe Base64 string (without padding).
pub fn decode_url_safe_no_pad(input: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
    decode_with_alphabet(input.as_ref(), true)
}

/// Encode bytes using URL-safe Base64 with `=` padding.
#[must_use]
pub fn encode_url_safe(input: impl AsRef<[u8]>) -> String {
    encode_with_alphabet(input.as_ref(), URL_SAFE, true)
}

/// Error returned when decoding an invalid Base64 string.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Invalid character encountered.
    InvalidChar { index: usize, byte: u8 },
    /// Input length is invalid (not a multiple of 4 when padded).
    InvalidLength,
    /// Invalid padding.
    InvalidPadding,
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidChar { index, byte } => {
                write!(f, "invalid base64 char 0x{byte:02x} at index {index}")
            }
            Self::InvalidLength => write!(f, "invalid base64 length"),
            Self::InvalidPadding => write!(f, "invalid base64 padding"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Look up a byte in the alphabet table.
///
/// The index `idx` is always in 0..64 because it is derived from a `u32`
/// masked with `0x3F` before calling this function.
#[inline]
fn alphabet_char(alphabet: &[u8; 64], idx: u32) -> char {
    // SAFETY PROOF: `idx` is always `(n >> k) & 0x3F`, which is at most 63.
    // The alphabet array has exactly 64 elements (indices 0..=63), so this
    // index is always in bounds. All alphabet bytes are printable ASCII, so
    // casting to `char` is always valid (all values < 128).
    #[allow(
        clippy::indexing_slicing,
        reason = "idx is always (bits >> k) & 0x3F which is at most 63; alphabet has 64 elements"
    )]
    #[allow(
        clippy::as_conversions,
        reason = "alphabet bytes are ASCII (0-127); casting u8 to char is always valid here"
    )]
    (alphabet[idx as usize] as char)
}

fn encode_with_alphabet(input: &[u8], alphabet: &[u8; 64], pad: bool) -> String {
    // Capacity: ceil(n / 3) * 4. Use saturating arithmetic — inputs large
    // enough to overflow usize would OOM long before reaching this point.
    let capacity = input
        .len()
        .saturating_add(2)
        .checked_div(3)
        .unwrap_or(0)
        .saturating_mul(4);
    let mut out = String::with_capacity(capacity);
    let chunks = input.chunks_exact(3);
    let remainder = chunks.remainder();

    for chunk in chunks {
        // chunks_exact(3) guarantees chunk has exactly 3 elements.
        #[allow(
            clippy::indexing_slicing,
            reason = "chunks_exact(3) guarantees chunk.len() == 3; indices 0, 1, 2 are always valid"
        )]
        let n = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
        out.push(alphabet_char(alphabet, (n >> 18) & 0x3F));
        out.push(alphabet_char(alphabet, (n >> 12) & 0x3F));
        out.push(alphabet_char(alphabet, (n >> 6) & 0x3F));
        out.push(alphabet_char(alphabet, n & 0x3F));
    }

    match remainder.len() {
        1 => {
            // chunks_exact remainder with len == 1: index 0 is valid.
            #[allow(
                clippy::indexing_slicing,
                reason = "remainder.len() == 1 is proven by the match arm; index 0 is always valid"
            )]
            let n = u32::from(remainder[0]) << 16;
            out.push(alphabet_char(alphabet, (n >> 18) & 0x3F));
            out.push(alphabet_char(alphabet, (n >> 12) & 0x3F));
            if pad {
                out.push('=');
                out.push('=');
            }
        }
        2 => {
            // chunks_exact remainder with len == 2: indices 0 and 1 are valid.
            #[allow(
                clippy::indexing_slicing,
                reason = "remainder.len() == 2 is proven by the match arm; indices 0 and 1 are always valid"
            )]
            let n = (u32::from(remainder[0]) << 16) | (u32::from(remainder[1]) << 8);
            out.push(alphabet_char(alphabet, (n >> 18) & 0x3F));
            out.push(alphabet_char(alphabet, (n >> 12) & 0x3F));
            out.push(alphabet_char(alphabet, (n >> 6) & 0x3F));
            if pad {
                out.push('=');
            }
        }
        _ => {}
    }

    out
}

fn decode_with_alphabet(input: &[u8], url_safe: bool) -> Result<Vec<u8>, DecodeError> {
    // Strip whitespace and padding
    let input: Vec<u8> = input
        .iter()
        .copied()
        .filter(|&b| b != b'\n' && b != b'\r' && b != b' ' && b != b'\t')
        .collect();

    // Strip trailing padding. `pad_count` is bounded by `input.len()` because
    // `take_while` cannot yield more elements than the iterator contains.
    let input_len = input.len();
    let pad_count = input.iter().rev().take_while(|&&b| b == b'=').count();
    // pad_count is produced by `take_while` on `input.iter()`, which cannot
    // yield more elements than the iterator contains, so `pad_count <= input_len`.
    // The subtraction therefore cannot underflow, and the slice is always in bounds.
    let data_len = input_len.saturating_sub(pad_count);
    // `data_len <= input_len` by construction; `.get(..)` returns `None` only
    // if `data_len > input.len()`, which is impossible, so `unwrap_or` with an
    // empty slice is the safe fallback that can never actually be reached.
    let data: &[u8] = input.get(..data_len).unwrap_or(&[]);

    if data.is_empty() {
        return Ok(Vec::new());
    }

    // Validate length: data length mod 4 must not be 1 (would be incomplete group).
    let mod4 = data.len() % 4;
    if mod4 == 1 {
        return Err(DecodeError::InvalidLength);
    }

    // Capacity: floor(n * 3 / 4). Use saturating to avoid overflow on huge
    // inputs — such inputs would OOM before reaching this point.
    let capacity = data.len().saturating_mul(3).checked_div(4).unwrap_or(0);
    let mut out = Vec::with_capacity(capacity);
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        // chunks_exact(4) guarantees chunk has exactly 4 elements.
        #[allow(
            clippy::indexing_slicing,
            reason = "chunks_exact(4) guarantees chunk.len() == 4; indices 0-3 are always valid"
        )]
        {
            let bits0 = decode_char(chunk[0], 0, url_safe)?;
            let bits1 = decode_char(chunk[1], 1, url_safe)?;
            let bits2 = decode_char(chunk[2], 2, url_safe)?;
            let bits3 = decode_char(chunk[3], 3, url_safe)?;
            let word = (u32::from(bits0) << 18)
                | (u32::from(bits1) << 12)
                | (u32::from(bits2) << 6)
                | u32::from(bits3);
            // Each shift extracts an 8-bit field from a 24-bit word; the
            // truncating cast to u8 is the intended operation.
            #[allow(
                clippy::as_conversions,
                reason = "extracting 8-bit fields from a 24-bit Base64 word; truncation is the correct semantic"
            )]
            {
                out.push((word >> 16) as u8);
                out.push((word >> 8) as u8);
                out.push(word as u8);
            }
        }
    }

    match remainder.len() {
        2 => {
            // remainder.len() == 2: indices 0 and 1 are valid.
            #[allow(
                clippy::indexing_slicing,
                reason = "remainder.len() == 2 is proven by the match arm; indices 0 and 1 are always valid"
            )]
            {
                let bits0 = decode_char(remainder[0], 0, url_safe)?;
                let bits1 = decode_char(remainder[1], 1, url_safe)?;
                let word = (u32::from(bits0) << 18) | (u32::from(bits1) << 12);
                // Extracting the top 8 bits of the 24-bit word.
                #[allow(
                    clippy::as_conversions,
                    reason = "extracting 8-bit field from 24-bit Base64 word; truncation is the correct semantic"
                )]
                out.push((word >> 16) as u8);
            }
        }
        3 => {
            // remainder.len() == 3: indices 0, 1, and 2 are valid.
            #[allow(
                clippy::indexing_slicing,
                reason = "remainder.len() == 3 is proven by the match arm; indices 0, 1, and 2 are always valid"
            )]
            {
                let bits0 = decode_char(remainder[0], 0, url_safe)?;
                let bits1 = decode_char(remainder[1], 1, url_safe)?;
                let bits2 = decode_char(remainder[2], 2, url_safe)?;
                let word =
                    (u32::from(bits0) << 18) | (u32::from(bits1) << 12) | (u32::from(bits2) << 6);
                // Extracting 8-bit fields from a 24-bit Base64 word.
                #[allow(
                    clippy::as_conversions,
                    reason = "extracting 8-bit fields from 24-bit Base64 word; truncation is the correct semantic"
                )]
                {
                    out.push((word >> 16) as u8);
                    out.push((word >> 8) as u8);
                }
            }
        }
        _ => {}
    }

    Ok(out)
}

#[inline]
fn decode_char(byte: u8, index: usize, url_safe: bool) -> Result<u8, DecodeError> {
    match byte {
        // Match arm guards prove the subtraction cannot underflow:
        // b'A'..=b'Z' guarantees byte >= b'A', so byte - b'A' is in 0..=25.
        // b'a'..=b'z' guarantees byte >= b'a', so byte - b'a' is in 0..=25;
        //   adding 26 gives 26..=51, which fits in u8.
        // b'0'..=b'9' guarantees byte >= b'0', so byte - b'0' is in 0..=9;
        //   adding 52 gives 52..=61, which fits in u8.
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "match arm guards prove byte >= b'A'; subtraction cannot underflow; result fits in u8"
        )]
        b'A'..=b'Z' => Ok(byte - b'A'),
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "match arm guards prove byte >= b'a' and byte - b'a' <= 25; adding 26 gives at most 51, fitting in u8"
        )]
        b'a'..=b'z' => Ok(byte - b'a' + 26),
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "match arm guards prove byte >= b'0' and byte - b'0' <= 9; adding 52 gives at most 61, fitting in u8"
        )]
        b'0'..=b'9' => Ok(byte - b'0' + 52),
        b'+' if !url_safe => Ok(62),
        b'/' if !url_safe => Ok(63),
        b'-' if url_safe => Ok(62),
        b'_' if url_safe => Ok(63),
        _ => Err(DecodeError::InvalidChar { index, byte }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 4648 §10 test vectors
    #[test]
    fn rfc4648_test_vectors() {
        let vectors = [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
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

    #[test]
    fn encode_empty() {
        assert_eq!(encode(b""), "");
    }

    #[test]
    fn encode_hello_world() {
        assert_eq!(encode(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn decode_hello_world() {
        assert_eq!(
            decode("SGVsbG8sIFdvcmxkIQ==").ok(),
            Some(b"Hello, World!".to_vec())
        );
    }

    #[test]
    fn decode_without_padding() {
        // Decoder should handle missing padding gracefully
        assert_eq!(
            decode("SGVsbG8sIFdvcmxkIQ").ok(),
            Some(b"Hello, World!".to_vec())
        );
    }

    #[test]
    fn url_safe_encode() {
        // Standard: uses + and /
        let input = [0xFF, 0xFE, 0xFD];
        let standard = encode(&input);
        assert!(standard.contains('+') || standard.contains('/') || !standard.contains('-'));

        // URL-safe: uses - and _
        let url = encode_url_safe_no_pad(&input);
        assert!(!url.contains('+'));
        assert!(!url.contains('/'));
        assert!(!url.contains('='));
    }

    #[test]
    fn url_safe_roundtrip() {
        let input = b"Hello, World! This is a test of URL-safe base64.";
        let encoded = encode_url_safe_no_pad(input);
        let decoded = decode_url_safe_no_pad(&encoded);
        assert_eq!(decoded.ok(), Some(input.to_vec()));
    }

    #[test]
    fn decode_invalid_char() {
        let err = decode("!!!!");
        assert!(matches!(err, Err(DecodeError::InvalidChar { .. })));
    }

    #[test]
    fn decode_invalid_length() {
        // Single char is invalid (mod 4 == 1)
        let err = decode("A");
        assert!(matches!(err, Err(DecodeError::InvalidLength)));
    }

    #[test]
    fn roundtrip_all_byte_values() {
        let input: Vec<u8> = (0..=255).collect();
        let encoded = encode(&input);
        let decoded = decode(&encoded);
        assert_eq!(decoded.ok(), Some(input));
    }

    #[test]
    fn roundtrip_various_lengths() {
        for len in 0..=64_u8 {
            let input: Vec<u8> = (0..len).collect();
            let encoded = encode(&input);
            let decoded = decode(&encoded);
            assert_eq!(decoded.ok(), Some(input), "roundtrip failed for len={len}");
        }
    }

    #[test]
    fn decode_with_whitespace() {
        let encoded = "SGVs\nbG8s\nIFdv\ncmxk\nIQ==";
        assert_eq!(decode(encoded).ok(), Some(b"Hello, World!".to_vec()));
    }
}
