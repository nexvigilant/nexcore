//! # Compression Utilities
//!
//! Gzip compression and decompression.

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::io::{Read, Write};

use super::super::error::{FoundationError, FoundationResult};

/// Compress data using gzip
///
/// # Errors
///
/// Returns an error if compression fails.
pub fn compress_gzip(data: &[u8]) -> FoundationResult<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish().map_err(FoundationError::Io)
}

/// Decompress gzip data
///
/// # Errors
///
/// Returns an error if decompression fails.
pub fn decompress_gzip(data: &[u8]) -> FoundationResult<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

/// Compress a string using gzip and return base64-encoded result
///
/// # Errors
///
/// Returns an error if compression fails.
pub fn compress_string_to_base64(input: &str) -> FoundationResult<String> {
    let compressed = compress_gzip(input.as_bytes())?;
    Ok(nexcore_codec::base64::encode(&compressed))
}

/// Decompress a base64-encoded gzip string
///
/// # Errors
///
/// Returns an error if decompression or base64 decoding fails.
pub fn decompress_base64_to_string(input: &str) -> FoundationResult<String> {
    let decoded = nexcore_codec::base64::decode(input)
        .map_err(|e| FoundationError::Serialization(e.to_string()))?;
    let decompressed = decompress_gzip(&decoded)?;
    String::from_utf8(decompressed).map_err(|e| FoundationError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let original = b"Hello, World! This is a test of gzip compression.";
        let compressed = compress_gzip(original).unwrap();
        let decompressed = decompress_gzip(&compressed).unwrap();
        assert_eq!(original.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_compression_reduces_size() {
        let original = "x".repeat(1000);
        let compressed = compress_gzip(original.as_bytes()).unwrap();
        assert!(compressed.len() < original.len());
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = "Hello, NexCore!";
        let encoded = compress_string_to_base64(original).unwrap();
        let decoded = decompress_base64_to_string(&encoded).unwrap();
        assert_eq!(original, decoded);
    }
}
