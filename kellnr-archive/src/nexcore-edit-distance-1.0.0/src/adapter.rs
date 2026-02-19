//! # Domain Adapters (T3 Interpretation Layer)
//!
//! Converts domain-specific sequences into metric-compatible form.
//! The same `EditMetric` produces domain-appropriate results when paired
//! with different adapters.
//!
//! Built-in adapters:
//! - `StringAdapter` — `&str` → `Vec<char>` (Unicode-aware)
//! - `ByteAdapter` — `&[u8]` identity
//! - `DnaAdapter` — nucleotide validation + encoding
//! - `TokenAdapter` — whitespace-split word tokens

use std::fmt;

/// Converts domain-specific input into a sequence of elements for edit distance.
pub trait DomainAdapter: Clone + Send + Sync + fmt::Debug {
    /// Element type produced by this adapter
    type Element: Clone + Eq;

    /// Convert domain input (as string) into element sequence.
    fn encode(&self, input: &str) -> Vec<Self::Element>;

    /// Size of the domain's alphabet (for information-theoretic metrics).
    fn alphabet_size(&self) -> usize;

    /// Human-readable domain name.
    fn domain_name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// StringAdapter: Unicode characters
// ---------------------------------------------------------------------------

/// Adapts Unicode strings to `Vec<char>`. Default for general text.
#[derive(Debug, Clone, Copy, Default)]
pub struct StringAdapter;

impl DomainAdapter for StringAdapter {
    type Element = char;

    fn encode(&self, input: &str) -> Vec<char> {
        input.chars().collect()
    }

    fn alphabet_size(&self) -> usize {
        // Unicode code points (practical upper bound)
        1_114_112
    }

    fn domain_name(&self) -> &str {
        "text/unicode"
    }
}

// ---------------------------------------------------------------------------
// ByteAdapter: raw bytes
// ---------------------------------------------------------------------------

/// Adapts byte slices. Useful for binary comparison or ASCII-only text.
#[derive(Debug, Clone, Copy, Default)]
pub struct ByteAdapter;

impl DomainAdapter for ByteAdapter {
    type Element = u8;

    fn encode(&self, input: &str) -> Vec<u8> {
        input.as_bytes().to_vec()
    }

    fn alphabet_size(&self) -> usize {
        256
    }

    fn domain_name(&self) -> &str {
        "binary/bytes"
    }
}

// ---------------------------------------------------------------------------
// DnaAdapter: nucleotide sequences
// ---------------------------------------------------------------------------

/// Adapts DNA sequences. Validates and normalizes to uppercase {A, C, G, T, N}.
#[derive(Debug, Clone, Copy, Default)]
pub struct DnaAdapter;

impl DomainAdapter for DnaAdapter {
    type Element = u8;

    fn encode(&self, input: &str) -> Vec<u8> {
        input
            .bytes()
            .map(|b| match b {
                b'A' | b'a' => b'A',
                b'C' | b'c' => b'C',
                b'G' | b'g' => b'G',
                b'T' | b't' => b'T',
                _ => b'N', // Unknown/ambiguous
            })
            .collect()
    }

    fn alphabet_size(&self) -> usize {
        5 // A, C, G, T, N
    }

    fn domain_name(&self) -> &str {
        "bioinformatics/dna"
    }
}

// ---------------------------------------------------------------------------
// TokenAdapter: word-level tokens
// ---------------------------------------------------------------------------

/// Adapts text to word tokens (whitespace-split). For word error rate (WER)
/// and sentence-level edit distance.
#[derive(Debug, Clone, Copy, Default)]
pub struct TokenAdapter;

impl DomainAdapter for TokenAdapter {
    type Element = String;

    fn encode(&self, input: &str) -> Vec<String> {
        input.split_whitespace().map(String::from).collect()
    }

    fn alphabet_size(&self) -> usize {
        // Unbounded vocabulary
        usize::MAX
    }

    fn domain_name(&self) -> &str {
        "nlp/tokens"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_adapter_basic() {
        let a = StringAdapter;
        assert_eq!(a.encode("hello"), vec!['h', 'e', 'l', 'l', 'o']);
        assert_eq!(a.domain_name(), "text/unicode");
    }

    #[test]
    fn string_adapter_unicode() {
        let a = StringAdapter;
        assert_eq!(a.encode("こんにちは").len(), 5);
    }

    #[test]
    fn byte_adapter_ascii() {
        let a = ByteAdapter;
        assert_eq!(a.encode("ABC"), vec![65, 66, 67]);
    }

    #[test]
    fn dna_adapter_normalization() {
        let a = DnaAdapter;
        assert_eq!(a.encode("AcGtX"), vec![b'A', b'C', b'G', b'T', b'N']);
        assert_eq!(a.alphabet_size(), 5);
    }

    #[test]
    fn token_adapter_splits() {
        let a = TokenAdapter;
        let tokens = a.encode("the quick brown fox");
        assert_eq!(tokens, vec!["the", "quick", "brown", "fox"]);
        assert_eq!(a.domain_name(), "nlp/tokens");
    }

    #[test]
    fn token_adapter_empty() {
        let a = TokenAdapter;
        assert!(a.encode("").is_empty());
        assert!(a.encode("   ").is_empty());
    }
}
