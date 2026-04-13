// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Disfluency Filtering — clean transcribed text of speech artifacts.
//!
//! Removes fillers (um, uh, like), repeated words, false starts,
//! and Whisper hallucinations. Produces clean text suitable for
//! command parsing or display.
//!
//! ## Categories
//!
//! 1. **Fillers** — um, uh, er, ah, like, you know, I mean
//! 2. **Repetitions** — "the the", "I I", stuttered words
//! 3. **False starts** — incomplete phrases followed by correction
//! 4. **Hallucinations** — Whisper artifacts (thank you for watching, subscribe)
//! 5. **Noise words** — hmm, huh, oh (when not semantic)

use serde::{Deserialize, Serialize};

/// English filler words/phrases to remove.
const EN_FILLERS: &[&str] = &[
    "um",
    "uh",
    "er",
    "ah",
    "uhm",
    "umm",
    "like",
    "you know",
    "i mean",
    "sort of",
    "kind of",
    "basically",
    "actually",
    "literally",
    "right",
    "so yeah",
    "and stuff",
    "or whatever",
];

/// Spanish filler words.
const ES_FILLERS: &[&str] = &[
    "eh", "este", "pues", "bueno", "o sea", "digamos", "a ver", "mira", "entonces",
];

/// Whisper hallucination patterns (common when transcribing silence/noise).
const HALLUCINATIONS: &[&str] = &[
    "thank you for watching",
    "thanks for watching",
    "please subscribe",
    "like and subscribe",
    "subscribe to",
    "thanks for listening",
    "see you next time",
    "bye bye",
    "the end",
    "music playing",
    "applause",
    "silence",
];

/// Configuration for disfluency filtering.
#[derive(Debug, Clone)]
pub struct DisfluencyConfig {
    /// Remove filler words.
    pub remove_fillers: bool,
    /// Remove repeated adjacent words.
    pub remove_repetitions: bool,
    /// Remove Whisper hallucinations.
    pub remove_hallucinations: bool,
    /// Minimum word count to keep a phrase (shorter = likely noise).
    pub min_words: usize,
    /// Include Spanish fillers.
    pub include_spanish: bool,
}

impl Default for DisfluencyConfig {
    fn default() -> Self {
        Self {
            remove_fillers: true,
            remove_repetitions: true,
            remove_hallucinations: true,
            min_words: 2,
            include_spanish: true,
        }
    }
}

/// Result of disfluency filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterResult {
    /// Cleaned text.
    pub text: String,
    /// Original text (before filtering).
    pub original: String,
    /// Number of fillers removed.
    pub fillers_removed: usize,
    /// Number of repetitions collapsed.
    pub repetitions_removed: usize,
    /// Whether a hallucination was detected and removed.
    pub hallucination_detected: bool,
    /// Whether the text was below minimum word count (empty result).
    pub too_short: bool,
}

/// Filter disfluencies from transcribed text.
pub fn filter(text: &str, config: &DisfluencyConfig) -> FilterResult {
    let original = text.to_string();
    let mut cleaned = text.to_string();
    let mut fillers_removed = 0;
    let mut repetitions_removed = 0;
    let mut hallucination_detected = false;

    // 1. Check for hallucinations (full-text match)
    if config.remove_hallucinations {
        let lower = cleaned.to_lowercase();
        let lower_trimmed = lower.trim().trim_end_matches('.');
        for pattern in HALLUCINATIONS {
            if lower_trimmed.starts_with(pattern) || lower_trimmed == *pattern {
                hallucination_detected = true;
                return FilterResult {
                    text: String::new(),
                    original,
                    fillers_removed: 0,
                    repetitions_removed: 0,
                    hallucination_detected: true,
                    too_short: false,
                };
            }
        }
    }

    // 2. Remove fillers
    if config.remove_fillers {
        let mut fillers: Vec<&str> = EN_FILLERS.to_vec();
        if config.include_spanish {
            fillers.extend_from_slice(ES_FILLERS);
        }

        // Sort by length descending to match longer phrases first
        fillers.sort_by(|a, b| b.len().cmp(&a.len()));

        for filler in &fillers {
            let lower = cleaned.to_lowercase();
            // Match as whole word/phrase (bounded by space or punctuation)
            let pattern_lower = filler.to_lowercase();
            let mut new = String::new();
            let mut remaining = lower.as_str();
            let original_chars: Vec<char> = cleaned.chars().collect();
            let mut char_idx = 0;

            while let Some(pos) = remaining.find(&pattern_lower) {
                // Check word boundary before
                let before_ok = pos == 0
                    || remaining
                        .as_bytes()
                        .get(pos.wrapping_sub(1))
                        .map_or(true, |&b| b == b' ' || b == b',');
                let after_pos = pos + pattern_lower.len();
                let after_ok = after_pos >= remaining.len()
                    || remaining
                        .as_bytes()
                        .get(after_pos)
                        .map_or(true, |&b| b == b' ' || b == b',' || b == b'.');

                if before_ok && after_ok {
                    // Copy original-case chars up to match, skip the filler
                    for c in &original_chars[char_idx..char_idx + pos] {
                        new.push(*c);
                    }
                    char_idx += pos + pattern_lower.len();
                    remaining = &remaining[after_pos..];
                    fillers_removed += 1;
                } else {
                    for c in &original_chars[char_idx..char_idx + pos + 1] {
                        new.push(*c);
                    }
                    char_idx += pos + 1;
                    remaining = &remaining[pos + 1..];
                }
            }
            // Append remainder
            for c in &original_chars[char_idx..] {
                new.push(*c);
            }
            cleaned = new;
        }
    }

    // 3. Remove repeated adjacent words
    if config.remove_repetitions {
        let words: Vec<&str> = cleaned.split_whitespace().collect();
        let mut deduped = Vec::with_capacity(words.len());
        let mut prev: Option<&str> = None;

        for word in &words {
            let w_lower = word.to_lowercase();
            let is_repeat = prev.as_ref().map_or(false, |p| p.to_lowercase() == w_lower);
            if is_repeat {
                repetitions_removed += 1;
            } else {
                deduped.push(*word);
            }
            prev = Some(word);
        }
        cleaned = deduped.join(" ");
    }

    // 4. Clean up whitespace
    cleaned = cleaned
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string();

    // Remove leading/trailing punctuation artifacts
    cleaned = cleaned
        .trim_matches(|c: char| c == ',' || c == ' ')
        .to_string();

    // 5. Check minimum word count
    let word_count = cleaned.split_whitespace().count();
    let too_short = word_count < config.min_words && !cleaned.is_empty();

    if too_short {
        cleaned = String::new();
    }

    FilterResult {
        text: cleaned,
        original,
        fillers_removed,
        repetitions_removed,
        hallucination_detected,
        too_short,
    }
}

/// Quick filter with default config.
pub fn clean(text: &str) -> String {
    filter(text, &DisfluencyConfig::default()).text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_english_fillers() {
        let result = filter(
            "um so I think uh the signal is like really strong",
            &DisfluencyConfig::default(),
        );
        assert!(!result.text.contains("um "));
        assert!(!result.text.contains(" uh "));
        assert!(result.fillers_removed > 0);
        assert!(result.text.contains("signal"));
        assert!(result.text.contains("strong"));
    }

    #[test]
    fn removes_spanish_fillers() {
        let result = filter(
            "este pues los pacientes bueno tienen reacciones",
            &DisfluencyConfig::default(),
        );
        assert!(!result.text.contains("este"));
        assert!(!result.text.contains("pues"));
        assert!(result.text.contains("pacientes"));
    }

    #[test]
    fn collapses_repetitions() {
        let result = filter(
            "the the patient had had a serious reaction",
            &DisfluencyConfig::default(),
        );
        assert_eq!(result.repetitions_removed, 2); // "the the" + "had had"
        assert!(result.text.starts_with("the patient"));
    }

    #[test]
    fn detects_hallucinations() {
        let result = filter("Thank you for watching.", &DisfluencyConfig::default());
        assert!(result.hallucination_detected);
        assert!(result.text.is_empty());
    }

    #[test]
    fn preserves_clean_text() {
        let clean_text = "The adverse event was reported within 24 hours";
        let result = filter(clean_text, &DisfluencyConfig::default());
        assert_eq!(result.text, clean_text);
        assert_eq!(result.fillers_removed, 0);
        assert_eq!(result.repetitions_removed, 0);
    }

    #[test]
    fn minimum_word_count() {
        let result = filter(
            "okay",
            &DisfluencyConfig {
                min_words: 2,
                ..DisfluencyConfig::default()
            },
        );
        assert!(result.too_short);
        assert!(result.text.is_empty());
    }

    #[test]
    fn quick_clean_function() {
        let cleaned = clean("um uh the the signal detection um is complete");
        assert!(cleaned.contains("signal detection"));
        assert!(cleaned.contains("complete"));
        assert!(!cleaned.contains("um"));
    }

    #[test]
    fn empty_input() {
        let result = filter("", &DisfluencyConfig::default());
        assert!(result.text.is_empty());
    }

    #[test]
    fn hallucination_case_insensitive() {
        let result = filter("THANK YOU FOR WATCHING", &DisfluencyConfig::default());
        assert!(result.hallucination_detected);
    }

    #[test]
    fn config_can_disable_features() {
        let config = DisfluencyConfig {
            remove_fillers: false,
            remove_repetitions: false,
            ..DisfluencyConfig::default()
        };
        let result = filter("um the the signal", &config);
        assert!(result.text.contains("um"));
        assert!(result.text.contains("the the"));
    }
}
