//! Configuration for the Natural Language Interface pipeline.

use serde::{Deserialize, Serialize};

/// Top-level NLI configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NliConfig {
    /// VAD configuration.
    pub vad: VadConfig,
    /// ASR configuration.
    pub asr: AsrConfig,
    /// Semantic layer configuration.
    pub semantic: SemanticConfig,
    /// Pragmatic layer configuration.
    pub pragmatic: PragmaticConfig,
    /// Generation layer configuration.
    pub generation: GenerationConfig,
}

impl Default for NliConfig {
    fn default() -> Self {
        Self {
            vad: VadConfig::default(),
            asr: AsrConfig::default(),
            semantic: SemanticConfig::default(),
            pragmatic: PragmaticConfig::default(),
            generation: GenerationConfig::default(),
        }
    }
}

/// Voice Activity Detection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    /// RMS energy threshold multiplier above ambient noise floor.
    pub speech_threshold_multiplier: f64,
    /// Sliding window size in samples (default: 50ms at 16kHz = 800 samples).
    pub window_size_samples: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            speech_threshold_multiplier: 1.5,
            window_size_samples: 800,
        }
    }
}

/// ASR configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrConfig {
    /// Minimum confidence threshold to accept a transcript.
    pub min_confidence: f64,
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
        }
    }
}

/// Semantic engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticConfig {
    /// Path to domain vocabulary YAML file.
    pub vocabulary_path: Option<String>,
    /// Minimum intent classification confidence.
    pub min_intent_confidence: f64,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            vocabulary_path: None,
            min_intent_confidence: 0.4,
        }
    }
}

/// Pragmatic / context management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PragmaticConfig {
    /// Maximum number of turns to retain in history.
    pub max_history_turns: usize,
    /// S = U × R × T signal threshold for proactive surfacing.
    pub proactive_signal_threshold: f64,
}

impl Default for PragmaticConfig {
    fn default() -> Self {
        Self {
            max_history_turns: 20,
            proactive_signal_threshold: 0.6,
        }
    }
}

/// Generation layer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Character length threshold for selecting Both modality.
    pub long_content_threshold: usize,
    /// Maximum words in a voice summary.
    pub voice_summary_max_words: usize,
    /// Maximum words per sentence in crisis tone mode.
    pub crisis_max_words_per_sentence: usize,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            long_content_threshold: 400,
            voice_summary_max_words: 100,
            crisis_max_words_per_sentence: 15,
        }
    }
}
