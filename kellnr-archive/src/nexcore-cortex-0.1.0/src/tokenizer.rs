//! # Tokenizer Wrapper
//!
//! Wraps HuggingFace `tokenizers` for encoding/decoding text.
//!
//! ## T1 Grounding
//! - μ (Mapping): Text ↔ token ID mapping
//! - σ (Sequence): Token sequences

use crate::CortexError;
use std::path::Path;
use std::str::FromStr;

/// Wrapper around HuggingFace tokenizer.
///
/// Tier: T2-P (μ + σ — Mapping + Sequence)
pub struct CortexTokenizer {
    inner: tokenizers::Tokenizer,
}

impl CortexTokenizer {
    /// Load a tokenizer from a JSON file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, CortexError> {
        let inner = tokenizers::Tokenizer::from_file(path.as_ref())
            .map_err(|e| CortexError::TokenizerError(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Load a tokenizer from in-memory bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, CortexError> {
        let json_str = std::str::from_utf8(data)
            .map_err(|e| CortexError::TokenizerError(format!("Invalid UTF-8: {e}")))?;
        let inner = tokenizers::Tokenizer::from_str(json_str).map_err(
            |e: Box<dyn std::error::Error + Send + Sync>| {
                CortexError::TokenizerError(e.to_string())
            },
        )?;
        Ok(Self { inner })
    }

    /// Encode text into token IDs.
    pub fn encode(&self, text: &str) -> Result<Vec<u32>, CortexError> {
        let encoding = self
            .inner
            .encode(text, true)
            .map_err(|e| CortexError::TokenizerError(e.to_string()))?;
        Ok(encoding.get_ids().to_vec())
    }

    /// Decode token IDs back into text.
    pub fn decode(&self, token_ids: &[u32]) -> Result<String, CortexError> {
        self.inner
            .decode(token_ids, true)
            .map_err(|e| CortexError::TokenizerError(e.to_string()))
    }

    /// Get the vocabulary size.
    pub fn vocab_size(&self) -> usize {
        self.inner.get_vocab_size(true)
    }

    /// Get the end-of-sequence token ID if available.
    pub fn eos_token_id(&self) -> Option<u32> {
        self.inner
            .get_added_vocabulary()
            .get_vocab()
            .get("</s>")
            .or_else(|| {
                self.inner
                    .get_added_vocabulary()
                    .get_vocab()
                    .get("<|endoftext|>")
            })
            .copied()
    }
}

#[cfg(test)]
mod tests {
    // Tokenizer tests require actual tokenizer files, so we test the error paths.
    use super::*;

    #[test]
    fn test_from_file_nonexistent() {
        let result = CortexTokenizer::from_file("/nonexistent/tokenizer.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_bytes_invalid_utf8() {
        let result = CortexTokenizer::from_bytes(&[0xFF, 0xFE]);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_bytes_invalid_json() {
        let result = CortexTokenizer::from_bytes(b"not valid json");
        assert!(result.is_err());
    }
}
