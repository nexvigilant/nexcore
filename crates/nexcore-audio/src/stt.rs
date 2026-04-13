// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Speech-to-Text — typed wrapper around faster-whisper.
//!
//! Provides a safe Rust interface to STT transcription by invoking
//! a Python subprocess with faster-whisper. No unsafe code, no FFI.
//!
//! ## JARVIS Layer 1
//!
//! Layer 0 (VAD + AEC) gates audio. Layer 1 converts speech to text.
//! This module takes a WAV file or raw audio buffer and returns a
//! typed `Transcript` with text, confidence, language, and segments.
//!
//! ## Design Decision
//!
//! Why subprocess instead of whisper.cpp FFI?
//! 1. `#![forbid(unsafe_code)]` — FFI bindings require unsafe
//! 2. faster-whisper (CTranslate2) is 3-4x faster than whisper.cpp on CPU
//! 3. The Python model is already loaded in vigil-listen's process
//! 4. For standalone use, subprocess latency (~200ms) is acceptable
//!
//! When whisper-rs matures with safe abstractions, swap the backend.

use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

/// Configuration for the STT engine.
#[derive(Debug, Clone)]
pub struct SttConfig {
    /// Whisper model name. Default: "medium.en".
    pub model: String,
    /// Language hint. Default: "en".
    pub language: String,
    /// Beam size for decoder. Default: 3.
    pub beam_size: u8,
    /// Initial prompt for vocabulary priming.
    pub initial_prompt: String,
    /// Minimum average log probability to accept. Default: -1.0.
    pub min_avg_logprob: f32,
    /// Maximum no-speech probability to accept. Default: 0.6.
    pub max_no_speech_prob: f32,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            model: "medium.en".to_string(),
            language: "en".to_string(),
            beam_size: 3,
            initial_prompt: String::new(),
            min_avg_logprob: -1.0,
            max_no_speech_prob: 0.6,
        }
    }
}

/// A single transcription segment with timing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    /// Segment text.
    pub text: String,
    /// Start time in seconds.
    pub start: f64,
    /// End time in seconds.
    pub end: f64,
    /// Average log probability (confidence).
    pub avg_logprob: f64,
    /// No-speech probability.
    pub no_speech_prob: f64,
}

/// Complete transcription result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    /// Full transcribed text.
    pub text: String,
    /// Individual segments with timing.
    pub segments: Vec<Segment>,
    /// Detected language.
    pub language: String,
    /// Overall confidence (average of segment logprobs).
    pub confidence: f64,
    /// Duration of input audio in seconds.
    pub duration_secs: f64,
    /// Whether the transcription was accepted (above confidence thresholds).
    pub accepted: bool,
}

/// STT engine errors.
#[derive(Debug)]
pub enum SttError {
    /// Input file not found.
    FileNotFound(String),
    /// Python/faster-whisper subprocess failed.
    SubprocessFailed(String),
    /// Failed to parse output.
    ParseError(String),
    /// Transcription below confidence threshold.
    LowConfidence {
        /// The text that was transcribed.
        text: String,
        /// The confidence score.
        confidence: f64,
    },
}

impl std::fmt::Display for SttError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(p) => write!(f, "audio file not found: {p}"),
            Self::SubprocessFailed(msg) => write!(f, "stt subprocess failed: {msg}"),
            Self::ParseError(msg) => write!(f, "stt parse error: {msg}"),
            Self::LowConfidence { text, confidence } => {
                write!(f, "low confidence ({confidence:.2}): {text}")
            }
        }
    }
}

/// Transcribe a WAV file using faster-whisper via subprocess.
///
/// Returns a typed `Transcript` or an error.
pub fn transcribe_file(path: &Path, config: &SttConfig) -> Result<Transcript, SttError> {
    if !path.exists() {
        return Err(SttError::FileNotFound(path.display().to_string()));
    }

    let path_str = path.display().to_string();

    // Build the Python one-liner that runs faster-whisper
    let py_script = format!(
        r#"
import json, sys
from faster_whisper import WhisperModel
model = WhisperModel("{model}", device="cpu", compute_type="int8")
segments, info = model.transcribe(
    "{path}",
    beam_size={beam},
    language="{lang}",
    initial_prompt="""{prompt}""",
    vad_filter=False,
)
seg_list = list(segments)
result = {{
    "text": " ".join(s.text.strip() for s in seg_list).strip(),
    "language": info.language,
    "duration": info.duration,
    "segments": [
        {{
            "text": s.text.strip(),
            "start": s.start,
            "end": s.end,
            "avg_logprob": s.avg_logprob,
            "no_speech_prob": s.no_speech_prob,
        }}
        for s in seg_list
    ],
}}
print(json.dumps(result))
"#,
        model = config.model,
        path = path_str.replace('"', r#"\""#),
        beam = config.beam_size,
        lang = config.language,
        prompt = config.initial_prompt.replace('"', r#"\""#),
    );

    let output = Command::new("python3")
        .arg("-c")
        .arg(&py_script)
        .output()
        .map_err(|e| SttError::SubprocessFailed(format!("spawn: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SttError::SubprocessFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| SttError::ParseError(format!("{e}: {stdout}")))?;

    let text = raw["text"].as_str().unwrap_or("").to_string();
    let language = raw["language"].as_str().unwrap_or("en").to_string();
    let duration_secs = raw["duration"].as_f64().unwrap_or(0.0);

    let segments: Vec<Segment> = raw["segments"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|s| Segment {
                    text: s["text"].as_str().unwrap_or("").to_string(),
                    start: s["start"].as_f64().unwrap_or(0.0),
                    end: s["end"].as_f64().unwrap_or(0.0),
                    avg_logprob: s["avg_logprob"].as_f64().unwrap_or(-1.0),
                    no_speech_prob: s["no_speech_prob"].as_f64().unwrap_or(1.0),
                })
                .collect()
        })
        .unwrap_or_default();

    let confidence = if segments.is_empty() {
        -1.0
    } else {
        segments.iter().map(|s| s.avg_logprob).sum::<f64>() / segments.len() as f64
    };

    let max_no_speech = segments
        .iter()
        .map(|s| s.no_speech_prob)
        .fold(0.0f64, f64::max);

    let accepted = confidence >= config.min_avg_logprob as f64
        && max_no_speech <= config.max_no_speech_prob as f64;

    Ok(Transcript {
        text,
        segments,
        language,
        confidence,
        duration_secs,
        accepted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_not_found_returns_error() {
        let config = SttConfig::default();
        let result = transcribe_file(Path::new("/nonexistent/audio.wav"), &config);
        assert!(result.is_err());
        match result {
            Err(SttError::FileNotFound(p)) => assert!(p.contains("nonexistent")),
            other => panic!("expected FileNotFound, got {other:?}"),
        }
    }

    #[test]
    fn default_config_values() {
        let config = SttConfig::default();
        assert_eq!(config.model, "medium.en");
        assert_eq!(config.language, "en");
        assert_eq!(config.beam_size, 3);
    }

    #[test]
    fn segment_serialization() {
        let seg = Segment {
            text: "hello world".to_string(),
            start: 0.0,
            end: 1.5,
            avg_logprob: -0.3,
            no_speech_prob: 0.05,
        };
        let json = serde_json::to_string(&seg);
        assert!(json.is_ok());
        let json = json.unwrap_or_default();
        assert!(json.contains("hello world"));
    }

    #[test]
    fn transcript_serialization() {
        let transcript = Transcript {
            text: "test transcript".to_string(),
            segments: vec![],
            language: "en".to_string(),
            confidence: -0.5,
            duration_secs: 2.0,
            accepted: true,
        };
        let json = serde_json::to_string_pretty(&transcript);
        assert!(json.is_ok());
    }
}
