//! Parameter types for audio MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Compute audio spec properties (bytes/frame, bytes/sec, duration).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioSpecComputeParams {
    /// Sample format: "s16", "s24", "s32", "f32", or "u8".
    pub format: String,
    /// Sample rate in Hz (e.g., 44100, 48000, 96000) or preset name.
    pub rate: u32,
    /// Channel layout: "mono", "stereo", "surround_51", "surround_71", or a channel count.
    pub layout: String,
    /// Optional: byte count to compute duration for.
    pub bytes: Option<usize>,
    /// Optional: duration in seconds to compute byte count for.
    pub duration_secs: Option<f64>,
}

/// List standard audio spec presets (CD, DVD, voice, float_stereo).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioSpecPresetsParams {}

/// Get properties of a sample format.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioFormatInfoParams {
    /// Sample format: "s16", "s24", "s32", "f32", or "u8".
    pub format: String,
}

/// Get properties of a sample rate.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioRateInfoParams {
    /// Sample rate in Hz.
    pub rate: u32,
}

/// Convert a single audio sample between formats.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioConvertSampleParams {
    /// The sample value to convert.
    pub value: f64,
    /// Source format: "s16", "f32", or "u8".
    pub from: String,
    /// Target format: "s16", "f32", or "u8".
    pub to: String,
}

/// Resample an F32 audio buffer between sample rates.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioResampleParams {
    /// F32 samples to resample.
    pub samples: Vec<f32>,
    /// Source sample rate in Hz.
    pub from_rate: u32,
    /// Target sample rate in Hz.
    pub to_rate: u32,
    /// Resample quality: "nearest" or "linear".
    pub quality: Option<String>,
}

/// List all codec types.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioCodecCatalogParams {}

/// Check device capabilities against a spec.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioDeviceCapabilitiesParams {
    /// Supported sample formats: ["s16", "f32", ...].
    pub formats: Vec<String>,
    /// Supported sample rates in Hz.
    pub rates: Vec<u32>,
    /// Supported channel layouts: ["mono", "stereo", ...].
    pub layouts: Vec<String>,
    /// Minimum buffer frames.
    pub min_buffer_frames: Option<usize>,
    /// Maximum buffer frames.
    pub max_buffer_frames: Option<usize>,
}

/// Compute stereo pan gains using constant-power pan law.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioMixerPanParams {
    /// Pan position: -1.0 (full left) to 1.0 (full right), 0.0 = center.
    pub pan: f32,
    /// Source volume (0.0–1.0).
    pub volume: Option<f32>,
    /// Whether source is muted.
    pub muted: Option<bool>,
}

/// Get available state transitions for an audio stream state.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioStreamTransitionsParams {
    /// Stream state: "created", "running", "paused", "stopped", or "error".
    pub state: String,
}

// ========================================================================
// Layer 0: Audio I/O Primitives (VAD, AEC, Noise Gate)
// ========================================================================

/// Process an audio frame through the Voice Activity Detector.
/// Returns speech/silence classification with energy, ZCR, and state.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioVadProcessParams {
    /// F32 audio samples (mono, [-1.0, 1.0]).
    pub samples: Vec<f32>,
    /// Energy threshold for speech detection. Default: 0.02.
    pub energy_threshold: Option<f32>,
    /// Zero-crossing rate ceiling. Frames above this are noise. Default: 0.4.
    pub zcr_ceiling: Option<f32>,
}

/// Process an audio frame through the Noise Gate.
/// Attenuates frames below the adaptive noise floor.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioNoiseGateParams {
    /// F32 audio samples (mono, [-1.0, 1.0]).
    pub samples: Vec<f32>,
    /// Threshold multiplier over noise floor. Default: 2.0.
    pub threshold_multiplier: Option<f32>,
    /// Minimum gain when gate is closed (0.0=silence, 0.1=-20dB). Default: 0.0.
    pub floor_gain: Option<f32>,
}

/// Transcribe a WAV file using faster-whisper STT.
/// Returns text, segments with timing, confidence, and language.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AudioTranscribeParams {
    /// Path to WAV file to transcribe.
    pub path: String,
    /// Whisper model name. Default: "medium.en".
    pub model: Option<String>,
    /// Language hint. Default: "en".
    pub language: Option<String>,
    /// Initial prompt for vocabulary priming.
    pub initial_prompt: Option<String>,
}
