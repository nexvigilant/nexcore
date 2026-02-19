//! Compendious scoring — Cs = (I / E) x C x R
//!
//! Ported from `nexcore-mcp/src/tools/compendious.rs:352-419`.
//! Scores text for information density.

use std::collections::HashSet;

/// Stopwords for information content filtering.
const STOPWORDS: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
    "from", "as", "is", "was", "are", "were", "been", "be", "have", "has", "had", "do", "does",
    "did", "will", "would", "could", "should", "may", "might", "must", "shall", "can", "this",
    "that", "these", "those", "it", "its", "they", "them", "their",
];

/// Result of scoring a text passage.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScoreResult {
    pub compendious_score: f64,
    pub information_content: f64,
    pub expression_cost: usize,
    pub completeness: f64,
    pub readability: f64,
    pub interpretation: String,
    pub limiting_factor: String,
}

/// Scorer for text information density.
pub struct CompendiousScorer;

impl CompendiousScorer {
    /// Score text with optional required elements.
    pub fn score(text: &str, required: &[String]) -> ScoreResult {
        let i = Self::information_content(text);
        let e = Self::expression_cost(text);
        let c = Self::completeness(text, required);
        let r = Self::readability(text);
        let density = if e > 0 { i / e as f64 } else { 0.0 };
        let score = density * c * r;

        ScoreResult {
            compendious_score: score,
            information_content: i,
            expression_cost: e,
            completeness: c,
            readability: r,
            interpretation: Self::interpret(score).to_string(),
            limiting_factor: Self::limiting_factor(density, c, r),
        }
    }

    /// Count unique non-stopword tokens, weighted.
    pub fn information_content(text: &str) -> f64 {
        let lowercased = text.to_lowercase();
        let stopword_set: HashSet<&str> = STOPWORDS.iter().copied().collect();
        let words: HashSet<&str> = lowercased
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .filter(|w| !stopword_set.contains(w))
            .collect();
        words.len() as f64 * 4.0
    }

    /// Word count as expression cost.
    pub fn expression_cost(text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// Fraction of required elements present.
    pub fn completeness(text: &str, required: &[String]) -> f64 {
        if required.is_empty() {
            return 1.0;
        }
        let text_lower = text.to_lowercase();
        let present = required
            .iter()
            .filter(|req| text_lower.contains(&req.to_lowercase()))
            .count();
        present as f64 / required.len() as f64
    }

    /// Flesch-based readability mapped to (0.1, 1.0) via sigmoid.
    pub fn readability(text: &str) -> f64 {
        let words: Vec<&str> = text.split_whitespace().collect();
        let sentences = text
            .matches(|c| c == '.' || c == '!' || c == '?')
            .count()
            .max(1);
        let avg_words_per_sentence = words.len() as f64 / sentences as f64;
        let avg_syllables = Self::estimate_avg_syllables(&words);
        let raw = 206.835 - (1.015 * avg_words_per_sentence) - (84.6 * avg_syllables);
        let sigmoid = 1.0 / (1.0 + (-0.02 * raw).exp());
        0.1 + 0.9 * sigmoid
    }

    fn estimate_avg_syllables(words: &[&str]) -> f64 {
        if words.is_empty() {
            return 1.0;
        }
        let total: usize = words.iter().map(|w| Self::count_syllables(w)).sum();
        total as f64 / words.len() as f64
    }

    fn count_syllables(word: &str) -> usize {
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
        let chars: Vec<char> = word.to_lowercase().chars().collect();
        let mut count = 0;
        let mut prev_vowel = false;
        for c in &chars {
            let is_vowel = vowels.contains(c);
            if is_vowel && !prev_vowel {
                count += 1;
            }
            prev_vowel = is_vowel;
        }
        if chars.last() == Some(&'e') && count > 1 {
            count -= 1;
        }
        count.max(1)
    }

    fn interpret(score: f64) -> &'static str {
        match score {
            s if s < 0.5 => "Verbose - Aggressive compression needed",
            s if s < 1.0 => "Adequate - Minor optimization possible",
            s if s < 2.0 => "Efficient - Good compendious quality",
            s if s < 5.0 => "Excellent - Publishable density",
            _ => "Exceptional - Reference-grade compression",
        }
    }

    fn limiting_factor(density: f64, c: f64, r: f64) -> String {
        let factors = [
            (density, "Information Density"),
            (c, "Completeness"),
            (r, "Readability"),
        ];
        factors
            .iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(val, name)| format!("{name} ({val:.2})"))
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_empty() {
        let r = CompendiousScorer::score("", &[]);
        assert_eq!(r.expression_cost, 0);
        assert_eq!(r.compendious_score, 0.0);
    }

    #[test]
    fn score_dense_text() {
        let text = "Arrhenius threshold gates signal detection via exponential activation energy.";
        let r = CompendiousScorer::score(text, &[]);
        assert!(r.compendious_score > 0.0);
        assert!(r.readability > 0.1);
    }

    #[test]
    fn completeness_tracks_elements() {
        let text = "The cat sat on the mat.";
        let required = vec!["cat".to_string(), "dog".to_string()];
        let c = CompendiousScorer::completeness(text, &required);
        assert!((c - 0.5).abs() < f64::EPSILON);
    }
}
