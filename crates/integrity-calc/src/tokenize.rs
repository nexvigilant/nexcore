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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_empty() {
        let stats = tokenize("");
        assert_eq!(stats.total_tokens, 0);
        assert_eq!(stats.unique_tokens, 0);
        assert_eq!(stats.ttr, 0.0);
    }

    #[test]
    fn tokenize_single_word() {
        let stats = tokenize("hello");
        assert_eq!(stats.total_tokens, 1);
        assert_eq!(stats.unique_tokens, 1);
        assert_eq!(stats.ttr, 1.0);
    }

    #[test]
    fn tokenize_lowercases() {
        let stats = tokenize("Hello WORLD");
        assert!(stats.frequencies.contains_key("hello"));
        assert!(stats.frequencies.contains_key("world"));
    }

    #[test]
    fn tokenize_strips_punctuation() {
        let stats = tokenize("hello, world!");
        assert!(stats.frequencies.contains_key("hello"));
        assert!(stats.frequencies.contains_key("world"));
    }

    #[test]
    fn tokenize_preserves_apostrophes() {
        let stats = tokenize("don't can't");
        assert!(stats.frequencies.contains_key("don't"));
    }

    #[test]
    fn tokenize_repeated_words() {
        let stats = tokenize("the the the cat");
        assert_eq!(stats.total_tokens, 4);
        assert_eq!(stats.unique_tokens, 2);
        assert_eq!(*stats.frequencies.get("the").unwrap_or(&0), 3);
    }

    #[test]
    fn ttr_deviation_at_baseline() {
        assert!((ttr_deviation(0.7)).abs() < 1e-10);
    }

    #[test]
    fn ttr_deviation_above() {
        assert!((ttr_deviation(0.9) - 0.2).abs() < 1e-10);
    }

    #[test]
    fn ttr_deviation_below() {
        assert!((ttr_deviation(0.5) - 0.2).abs() < 1e-10);
    }
}
