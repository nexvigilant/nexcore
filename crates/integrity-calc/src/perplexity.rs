//! Perplexity variance — per-sentence entropy variance
//!
//! Tier: T2-C | Primitives: ν Frequency, κ Comparison

use crate::entropy::shannon_entropy;

/// Perplexity variance result.
#[derive(Debug, Clone)]
pub struct PerplexityResult {
    pub mean_entropy: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub sentence_count: usize,
    pub sentence_entropies: Vec<f64>,
}

/// Split text into sentences (simple heuristic).
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if (ch == '.' || ch == '!' || ch == '?') && current.split_whitespace().count() > 1 {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current = String::new();
        }
    }

    let trimmed = current.trim().to_string();
    if trimmed.split_whitespace().count() > 1 {
        sentences.push(trimmed);
    }

    sentences
}

/// Tokenize a sentence into lowercased words.
fn sentence_tokens(sentence: &str) -> Vec<String> {
    sentence
        .split_whitespace()
        .map(|w| {
            w.chars()
                .filter(|c| c.is_alphanumeric() || *c == '\'')
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect()
}

/// Analyze perplexity variance across sentences.
pub fn perplexity_variance(text: &str) -> PerplexityResult {
    let sentences = split_sentences(text);

    if sentences.is_empty() {
        return PerplexityResult {
            mean_entropy: 0.0,
            variance: 0.0,
            std_dev: 0.0,
            sentence_count: 0,
            sentence_entropies: vec![],
        };
    }

    let sentence_entropies: Vec<f64> = sentences
        .iter()
        .map(|s| {
            let tokens = sentence_tokens(s);
            shannon_entropy(&tokens)
        })
        .collect();

    let n = sentence_entropies.len() as f64;
    let mean_entropy = sentence_entropies.iter().sum::<f64>() / n;

    let variance = if sentence_entropies.len() > 1 {
        sentence_entropies
            .iter()
            .map(|h| (h - mean_entropy).powi(2))
            .sum::<f64>()
            / (n - 1.0)
    } else {
        0.0
    };

    PerplexityResult {
        mean_entropy,
        variance,
        std_dev: variance.sqrt(),
        sentence_count: sentence_entropies.len(),
        sentence_entropies,
    }
}
