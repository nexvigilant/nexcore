//! Text tokenization — word splitting + TTR
//!
//! Tier: T2-C | Primitives: σ Sequence, N Quantity, μ Mapping

use std::collections::HashMap;

/// Tokenized text with frequency statistics.
#[derive(Debug, Clone)]
pub struct TokenStats {
    pub total_tokens: usize,
    pub unique_tokens: usize,
    pub ttr: f64,
    pub frequencies: HashMap<String, usize>,
    pub tokens: Vec<String>,
}

/// Tokenize text into lowercased words, stripping punctuation.
#[must_use]
pub fn tokenize(text: &str) -> TokenStats {
    let tokens: Vec<String> = text
        .split_whitespace()
        .map(|w| {
            w.chars()
                .filter(|c| c.is_alphanumeric() || *c == '\'')
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect();

    let total_tokens = tokens.len();
    let mut frequencies: HashMap<String, usize> = HashMap::new();
    for token in &tokens {
        *frequencies.entry(token.clone()).or_insert(0) += 1;
    }
    let unique_tokens = frequencies.len();
    let ttr = if total_tokens > 0 {
        unique_tokens as f64 / total_tokens as f64
    } else {
        0.0
    };

    TokenStats {
        total_tokens,
        unique_tokens,
        ttr,
        frequencies,
        tokens,
    }
}

/// Compute TTR deviation from human baseline (0.7).
#[must_use]
pub fn ttr_deviation(ttr: f64) -> f64 {
    (ttr - 0.7).abs()
}
