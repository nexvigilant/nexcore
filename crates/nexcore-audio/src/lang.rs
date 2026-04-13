// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Language Detection — identify spoken language from audio features.
//!
//! Uses a combination of phonetic distribution analysis and n-gram
//! frequency matching on transcribed text. Designed for English/Spanish
//! bilingual detection (Matthew's use case) but extensible to any pair.
//!
//! ## Approach
//!
//! Two-stage detection:
//! 1. **Pre-ASR hint** — ZCR/energy distribution patterns differ between
//!    languages (Spanish has more vowel energy, English more fricatives).
//!    This provides a prior probability that biases Whisper's `language` param.
//! 2. **Post-ASR confirmation** — n-gram analysis on the transcribed text
//!    confirms or overrides the audio-based hint.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// English.
    English,
    /// Spanish.
    Spanish,
    /// Unknown / not enough signal.
    Unknown,
}

impl Language {
    /// Whisper language code.
    #[must_use]
    pub fn whisper_code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Spanish => "es",
            Self::Unknown => "en", // default to English
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::English => write!(f, "en"),
            Self::Spanish => write!(f, "es"),
            Self::Unknown => write!(f, "??"),
        }
    }
}

/// Language detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangResult {
    /// Detected language.
    pub language: Language,
    /// Confidence (0.0..1.0).
    pub confidence: f32,
    /// English probability.
    pub p_english: f32,
    /// Spanish probability.
    pub p_spanish: f32,
    /// Detection method used.
    pub method: DetectionMethod,
}

/// How the language was detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionMethod {
    /// From audio features (ZCR/energy distribution).
    AudioHint,
    /// From transcribed text n-grams.
    TextNgram,
    /// Combined audio + text.
    Combined,
    /// Default fallback.
    Default,
}

// Common English trigrams (top 20 by frequency)
const EN_TRIGRAMS: &[&str] = &[
    "the", "and", "ing", "tion", "her", "for", "tha", "ent", "ion", "ter", "was", "you", "ith",
    "ver", "all", "wit", "thi", "hat", "tha", "ere",
];

// Common Spanish trigrams
const ES_TRIGRAMS: &[&str] = &[
    "que", "ión", "ent", "nte", "ado", "los", "las", "aci", "ión", "est", "con", "del", "mos",
    "por", "una", "ien", "cia", "ara", "sta", "ero",
];

/// Detect language from transcribed text using trigram frequency.
///
/// Returns a `LangResult` with probabilities for each language.
pub fn detect_from_text(text: &str) -> LangResult {
    let lower = text.to_lowercase();
    if lower.len() < 10 {
        return LangResult {
            language: Language::Unknown,
            confidence: 0.0,
            p_english: 0.5,
            p_spanish: 0.5,
            method: DetectionMethod::Default,
        };
    }

    let en_score = trigram_score(&lower, EN_TRIGRAMS);
    let es_score = trigram_score(&lower, ES_TRIGRAMS);
    let total = en_score + es_score;

    if total < 0.001 {
        return LangResult {
            language: Language::Unknown,
            confidence: 0.0,
            p_english: 0.5,
            p_spanish: 0.5,
            method: DetectionMethod::TextNgram,
        };
    }

    let p_en = en_score / total;
    let p_es = es_score / total;
    let confidence = (p_en - p_es).abs();

    let language = if p_en > p_es + 0.1 {
        Language::English
    } else if p_es > p_en + 0.1 {
        Language::Spanish
    } else {
        Language::Unknown
    };

    LangResult {
        language,
        confidence,
        p_english: p_en,
        p_spanish: p_es,
        method: DetectionMethod::TextNgram,
    }
}

/// Detect language from audio features (pre-ASR hint).
///
/// Spanish has higher vowel energy ratios and lower ZCR variance
/// compared to English (more fricatives, consonant clusters).
pub fn detect_from_audio(avg_energy: f32, avg_zcr: f32, energy_variance: f32) -> LangResult {
    // Heuristic: Spanish speech tends to have:
    // - Higher sustained energy (more vowels)
    // - Lower ZCR (fewer fricatives)
    // - Lower energy variance (more even syllable timing)
    //
    // These are probabilistic signals, not deterministic.
    let mut en_score: f32 = 0.0;
    let mut es_score: f32 = 0.0;

    // High ZCR favors English (fricatives: s, f, th, sh)
    if avg_zcr > 0.15 {
        en_score += 0.3;
    } else {
        es_score += 0.2;
    }

    // High energy variance favors English (stress-timed)
    if energy_variance > 0.02 {
        en_score += 0.2;
    } else {
        es_score += 0.3; // Spanish is syllable-timed (more even)
    }

    // High sustained energy favors Spanish (more open vowels)
    if avg_energy > 0.15 {
        es_score += 0.2;
    } else {
        en_score += 0.1;
    }

    let total = en_score + es_score;
    let p_en = if total > 0.0 { en_score / total } else { 0.5 };
    let p_es = if total > 0.0 { es_score / total } else { 0.5 };
    let confidence = (p_en - p_es).abs();

    let language = if confidence < 0.15 {
        Language::Unknown
    } else if p_en > p_es {
        Language::English
    } else {
        Language::Spanish
    };

    LangResult {
        language,
        confidence,
        p_english: p_en,
        p_spanish: p_es,
        method: DetectionMethod::AudioHint,
    }
}

/// Combine audio hint and text detection results.
pub fn combine(audio: &LangResult, text: &LangResult) -> LangResult {
    // Text detection is more reliable — weight it 70%
    let p_en = audio.p_english * 0.3 + text.p_english * 0.7;
    let p_es = audio.p_spanish * 0.3 + text.p_spanish * 0.7;
    let confidence = (p_en - p_es).abs();

    let language = if confidence < 0.1 {
        Language::Unknown
    } else if p_en > p_es {
        Language::English
    } else {
        Language::Spanish
    };

    LangResult {
        language,
        confidence,
        p_english: p_en,
        p_spanish: p_es,
        method: DetectionMethod::Combined,
    }
}

/// Count trigram matches normalized by text length.
fn trigram_score(text: &str, trigrams: &[&str]) -> f32 {
    if text.len() < 3 {
        return 0.0;
    }
    let hits: usize = trigrams.iter().filter(|&&tri| text.contains(tri)).count();
    #[allow(
        clippy::as_conversions,
        reason = "hits and trigrams.len() are small usize"
    )]
    {
        hits as f32 / trigrams.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_text_detected() {
        let result = detect_from_text(
            "The quick brown fox jumps over the lazy dog and runs through the forest",
        );
        assert_eq!(result.language, Language::English);
        assert!(result.p_english > result.p_spanish);
        assert!(result.confidence > 0.1);
    }

    #[test]
    fn spanish_text_detected() {
        let result =
            detect_from_text("Los pacientes con reacciones adversas deben consultar con el médico");
        assert_eq!(result.language, Language::Spanish);
        assert!(result.p_spanish > result.p_english);
    }

    #[test]
    fn short_text_returns_unknown() {
        let result = detect_from_text("hi");
        assert_eq!(result.language, Language::Unknown);
        assert_eq!(result.method, DetectionMethod::Default);
    }

    #[test]
    fn audio_hint_high_zcr_favors_english() {
        let result = detect_from_audio(0.10, 0.25, 0.03);
        // High ZCR + high variance → English
        assert_eq!(result.language, Language::English);
    }

    #[test]
    fn audio_hint_low_zcr_favors_spanish() {
        let result = detect_from_audio(0.20, 0.08, 0.01);
        // Low ZCR + low variance + high energy → Spanish
        assert_eq!(result.language, Language::Spanish);
    }

    #[test]
    fn combine_results() {
        let audio = detect_from_audio(0.10, 0.25, 0.03); // English hint
        let text =
            detect_from_text("The patient reported adverse reactions after taking the medication");
        let combined = combine(&audio, &text);
        assert_eq!(combined.language, Language::English);
        assert_eq!(combined.method, DetectionMethod::Combined);
        // Combined should agree on English; confidence may vary based on weighting
        assert!(combined.p_english > combined.p_spanish);
    }

    #[test]
    fn whisper_codes() {
        assert_eq!(Language::English.whisper_code(), "en");
        assert_eq!(Language::Spanish.whisper_code(), "es");
        assert_eq!(Language::Unknown.whisper_code(), "en"); // defaults to English
    }

    #[test]
    fn language_display() {
        assert_eq!(format!("{}", Language::English), "en");
        assert_eq!(format!("{}", Language::Spanish), "es");
    }
}
