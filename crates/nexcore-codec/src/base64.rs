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

fn encode_with_alphabet(input: &[u8], alphabet: &[u8; 64], pad: bool) -> String {
    let mut out = String::with_capacity(((input.len() + 2) / 3) * 4);
    let chunks = input.chunks_exact(3);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
        out.push(alphabet[((n >> 18) & 0x3F) as usize] as char);
        out.push(alphabet[((n >> 12) & 0x3F) as usize] as char);
        out.push(alphabet[((n >> 6) & 0x3F) as usize] as char);
        out.push(alphabet[(n & 0x3F) as usize] as char);
    }

    match remainder.len() {
        1 => {
            let n = u32::from(remainder[0]) << 16;
            out.push(alphabet[((n >> 18) & 0x3F) as usize] as char);
            out.push(alphabet[((n >> 12) & 0x3F) as usize] as char);
            if pad {
                out.push('=');
                out.push('=');
            }
        }
        2 => {
            let n = (u32::from(remainder[0]) << 16) | (u32::from(remainder[1]) << 8);
            out.push(alphabet[((n >> 18) & 0x3F) as usize] as char);
            out.push(alphabet[((n >> 12) & 0x3F) as usize] as char);
            out.push(alphabet[((n >> 6) & 0x3F) as usize] as char);
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

    // Strip trailing padding
    let input_len = input.len();
    let pad_count = input.iter().rev().take_while(|&&b| b == b'=').count();
    let data = &input[..input_len - pad_count];

    if data.is_empty() {
        return Ok(Vec::new());
    }

    // Validate length: data + padding should be multiple of 4 if padded,
    // or data length mod 4 should not be 1
    let mod4 = data.len() % 4;
    if mod4 == 1 {
        return Err(DecodeError::InvalidLength);
    }

    let mut out = Vec::with_capacity((data.len() * 3) / 4);
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let bits0 = decode_char(chunk[0], 0, url_safe)?;
        let bits1 = decode_char(chunk[1], 1, url_safe)?;
        let bits2 = decode_char(chunk[2], 2, url_safe)?;
        let bits3 = decode_char(chunk[3], 3, url_safe)?;
        let word = (u32::from(bits0) << 18)
            | (u32::from(bits1) << 12)
            | (u32::from(bits2) << 6)
            | u32::from(bits3);
        out.push((word >> 16) as u8);
        out.push((word >> 8) as u8);
        out.push(word as u8);
    }

    match remainder.len() {
        2 => {
            let bits0 = decode_char(remainder[0], 0, url_safe)?;
            let bits1 = decode_char(remainder[1], 1, url_safe)?;
            let word = (u32::from(bits0) << 18) | (u32::from(bits1) << 12);
            out.push((word >> 16) as u8);
        }
        3 => {
            let bits0 = decode_char(remainder[0], 0, url_safe)?;
            let bits1 = decode_char(remainder[1], 1, url_safe)?;
            let bits2 = decode_char(remainder[2], 2, url_safe)?;
            let word =
                (u32::from(bits0) << 18) | (u32::from(bits1) << 12) | (u32::from(bits2) << 6);
            out.push((word >> 16) as u8);
            out.push((word >> 8) as u8);
        }
        _ => {}
    }

    Ok(out)
}

#[inline]
fn decode_char(byte: u8, index: usize, url_safe: bool) -> Result<u8, DecodeError> {
    match byte {
        b'A'..=b'Z' => Ok(byte - b'A'),
        b'a'..=b'z' => Ok(byte - b'a' + 26),
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
        for len in 0..=64 {
            let input: Vec<u8> = (0..len).map(|i| i as u8).collect();
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
