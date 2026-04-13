// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Voice Activity Detection — pure Rust, zero dependencies.
//!
//! Two-feature detector using RMS energy + zero-crossing rate (ZCR).
//! Energy catches loud speech, ZCR distinguishes voiced from unvoiced
//! noise (keyboard clicks, HVAC have high ZCR but low energy;
//! speech has moderate ZCR with sustained energy).
//!
//! ## JARVIS Layer 0
//!
//! This is the gate between silence and the STT pipeline.
//! False positives waste compute (Whisper runs on noise).
//! False negatives lose speech (user repeats themselves).
//! The threshold is the boundary (∂) that separates signal from void (∅).
//!
//! ## Usage
//!
//! ```
//! use nexcore_audio::vad::{VoiceDetector, VadConfig};
//!
//! let mut vad = VoiceDetector::new(VadConfig::default());
//!
//! // Feed F32 audio frames (mono, [-1.0, 1.0])
//! let frame = vec![0.0f32; 480]; // 30ms at 16kHz
//! let result = vad.process(&frame);
//! assert!(!result.is_speech); // silence
//! ```

use serde::{Deserialize, Serialize};

/// Configuration for the voice activity detector.
#[derive(Debug, Clone)]
pub struct VadConfig {
    /// RMS energy threshold for speech detection.
    /// Frames below this are silence. Default: 0.02 (quiet room).
    pub energy_threshold: f32,
    /// Zero-crossing rate ceiling. Frames above this are likely
    /// non-speech noise (keyboard, clicks). Default: 0.4.
    pub zcr_ceiling: f32,
    /// Number of consecutive speech frames to confirm onset.
    /// Prevents single-frame spikes from triggering. Default: 3.
    pub onset_frames: u32,
    /// Number of consecutive silence frames to confirm offset.
    /// Prevents brief pauses from cutting speech. Default: 15.
    pub offset_frames: u32,
    /// Adaptive threshold decay factor (0.0..1.0).
    /// Higher = slower adaptation to changing noise floor.
    /// Default: 0.995.
    pub adaptation_rate: f32,
    /// Minimum energy to ever consider (hard floor).
    /// Prevents adaptation from dropping threshold to zero. Default: 0.005.
    pub min_energy: f32,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            energy_threshold: 0.02,
            zcr_ceiling: 0.4,
            onset_frames: 3,
            offset_frames: 15,
            adaptation_rate: 0.995,
            min_energy: 0.005,
        }
    }
}

/// Result of processing a single audio frame.
#[derive(Debug, Clone, Serialize)]
pub struct VadResult {
    /// Whether this frame is classified as speech.
    pub is_speech: bool,
    /// RMS energy of the frame.
    pub energy: f32,
    /// Zero-crossing rate of the frame (0.0..1.0).
    pub zcr: f32,
    /// Current adaptive energy threshold.
    pub threshold: f32,
    /// Current detector state.
    pub state: VadState,
}

/// Detector state machine.
///
/// ```text
/// Silence ──[energy > threshold]──> Onset ──[onset_frames reached]──> Speech
///    ^                                 │                                  │
///    │                                 │                                  │
///    └──[onset not sustained]──────────┘                                  │
///    └──[offset_frames reached]──── Offset <──[energy < threshold]────────┘
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VadState {
    /// No speech detected.
    Silence,
    /// Speech may be starting (counting onset frames).
    Onset,
    /// Active speech.
    Speech,
    /// Speech may be ending (counting offset frames).
    Offset,
}

/// Two-feature voice activity detector with adaptive threshold.
pub struct VoiceDetector {
    config: VadConfig,
    state: VadState,
    /// Consecutive frames supporting current transition.
    transition_count: u32,
    /// Adaptive noise floor estimate.
    noise_floor: f32,
    /// Total frames processed.
    frame_count: u64,
}

impl VoiceDetector {
    /// Create a new detector with the given configuration.
    #[must_use]
    pub fn new(config: VadConfig) -> Self {
        let noise_floor = config.energy_threshold;
        Self {
            config,
            state: VadState::Silence,
            transition_count: 0,
            noise_floor,
            frame_count: 0,
        }
    }

    /// Current detector state.
    #[must_use]
    pub fn state(&self) -> VadState {
        self.state
    }

    /// Current adaptive noise floor estimate.
    #[must_use]
    pub fn noise_floor(&self) -> f32 {
        self.noise_floor
    }

    /// Total frames processed.
    #[must_use]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Reset to initial state.
    pub fn reset(&mut self) {
        self.state = VadState::Silence;
        self.transition_count = 0;
        self.noise_floor = self.config.energy_threshold;
        self.frame_count = 0;
    }

    /// Process a mono F32 audio frame. Returns classification result.
    ///
    /// Frame length should be 10-30ms of audio (160-480 samples at 16kHz).
    pub fn process(&mut self, frame: &[f32]) -> VadResult {
        self.frame_count = self.frame_count.saturating_add(1);

        let energy = rms_energy(frame);
        let zcr = zero_crossing_rate(frame);

        // Adapt noise floor during silence (tracks slow environmental changes)
        if self.state == VadState::Silence {
            self.noise_floor = self.noise_floor * self.config.adaptation_rate
                + energy * (1.0 - self.config.adaptation_rate);
            if self.noise_floor < self.config.min_energy {
                self.noise_floor = self.config.min_energy;
            }
        }

        // Effective threshold: 3x noise floor (proven in vigil-mic-calibrate: 2.5x P90)
        let threshold = (self.noise_floor * 3.0).max(self.config.min_energy);

        // Two-feature gate: energy must exceed threshold AND ZCR must be
        // below ceiling (high ZCR + low energy = non-speech noise)
        let is_frame_speech = energy > threshold && zcr < self.config.zcr_ceiling;

        // State machine transitions
        match self.state {
            VadState::Silence => {
                if is_frame_speech {
                    self.state = VadState::Onset;
                    self.transition_count = 1;
                }
            }
            VadState::Onset => {
                if is_frame_speech {
                    self.transition_count = self.transition_count.saturating_add(1);
                    if self.transition_count >= self.config.onset_frames {
                        self.state = VadState::Speech;
                        self.transition_count = 0;
                    }
                } else {
                    // Onset not sustained — back to silence
                    self.state = VadState::Silence;
                    self.transition_count = 0;
                }
            }
            VadState::Speech => {
                if !is_frame_speech {
                    self.state = VadState::Offset;
                    self.transition_count = 1;
                }
            }
            VadState::Offset => {
                if is_frame_speech {
                    // Speech resumed — back to active
                    self.state = VadState::Speech;
                    self.transition_count = 0;
                } else {
                    self.transition_count = self.transition_count.saturating_add(1);
                    if self.transition_count >= self.config.offset_frames {
                        self.state = VadState::Silence;
                        self.transition_count = 0;
                    }
                }
            }
        }

        let is_speech = matches!(
            self.state,
            VadState::Speech | VadState::Onset | VadState::Offset
        );

        VadResult {
            is_speech,
            energy,
            zcr,
            threshold,
            state: self.state,
        }
    }
}

/// Compute RMS (root mean square) energy of an audio frame.
///
/// Returns 0.0 for empty frames.
#[must_use]
pub fn rms_energy(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = frame.iter().map(|&s| (s as f64) * (s as f64)).sum();
    #[allow(
        clippy::as_conversions,
        reason = "frame.len() is usize, safe to cast to f64 for division"
    )]
    let mean = sum_sq / frame.len() as f64;
    mean.sqrt() as f32
}

/// Compute zero-crossing rate of an audio frame.
///
/// Returns fraction of adjacent sample pairs that cross zero (0.0..1.0).
/// Speech: typically 0.05-0.25. Noise/clicks: 0.3-0.5.
#[must_use]
pub fn zero_crossing_rate(frame: &[f32]) -> f32 {
    if frame.len() < 2 {
        return 0.0;
    }
    let crossings = frame
        .windows(2)
        .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
        .count();
    #[allow(
        clippy::as_conversions,
        reason = "crossings and frame.len() are usize, safe for f32 ratio"
    )]
    {
        crossings as f32 / (frame.len() - 1) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_stays_silent() {
        let mut vad = VoiceDetector::new(VadConfig::default());
        let silence = vec![0.0f32; 480];
        let result = vad.process(&silence);
        assert!(!result.is_speech);
        assert_eq!(result.state, VadState::Silence);
    }

    #[test]
    fn loud_signal_detects_speech() {
        let mut vad = VoiceDetector::new(VadConfig {
            onset_frames: 1, // detect immediately for testing
            ..VadConfig::default()
        });

        // Simulate loud speech (sine wave at 0.3 amplitude)
        let frame: Vec<f32> = (0..480)
            .map(|i| 0.3 * (2.0 * std::f32::consts::PI * 300.0 * i as f32 / 16000.0).sin())
            .collect();

        let result = vad.process(&frame);
        assert!(result.is_speech);
        assert!(result.energy > 0.1);
    }

    #[test]
    fn onset_requires_sustained_frames() {
        let mut vad = VoiceDetector::new(VadConfig {
            onset_frames: 3,
            ..VadConfig::default()
        });

        let speech: Vec<f32> = (0..480)
            .map(|i| 0.3 * (2.0 * std::f32::consts::PI * 300.0 * i as f32 / 16000.0).sin())
            .collect();

        // Frame 1: onset started, not yet confirmed
        let r1 = vad.process(&speech);
        assert_eq!(r1.state, VadState::Onset);

        // Frame 2: still onset
        let r2 = vad.process(&speech);
        assert_eq!(r2.state, VadState::Onset);

        // Frame 3: confirmed speech
        let r3 = vad.process(&speech);
        assert_eq!(r3.state, VadState::Speech);
    }

    #[test]
    fn offset_holds_through_brief_pause() {
        let mut vad = VoiceDetector::new(VadConfig {
            onset_frames: 1,
            offset_frames: 5,
            ..VadConfig::default()
        });

        let speech: Vec<f32> = (0..480)
            .map(|i| 0.3 * (2.0 * std::f32::consts::PI * 300.0 * i as f32 / 16000.0).sin())
            .collect();
        let silence = vec![0.0f32; 480];

        // Enter speech (frame 1 → Onset, frame 2 → Speech)
        vad.process(&speech);
        vad.process(&speech);
        assert_eq!(vad.state(), VadState::Speech);

        // Brief silence — should enter offset, not drop to silence
        vad.process(&silence);
        assert_eq!(vad.state(), VadState::Offset);

        // Speech resumes — back to speech
        vad.process(&speech);
        assert_eq!(vad.state(), VadState::Speech);
    }

    #[test]
    fn high_zcr_rejects_noise() {
        let mut vad = VoiceDetector::new(VadConfig {
            onset_frames: 1,
            zcr_ceiling: 0.3,
            ..VadConfig::default()
        });

        // Simulate noise: loud but alternating every sample (ZCR ≈ 1.0)
        let noise: Vec<f32> = (0..480)
            .map(|i| if i % 2 == 0 { 0.3 } else { -0.3 })
            .collect();

        let result = vad.process(&noise);
        // Energy is high but ZCR is ~1.0, exceeding ceiling
        assert!(!result.is_speech, "high-ZCR noise should be rejected");
        assert!(result.zcr > 0.9);
    }

    #[test]
    fn rms_energy_values() {
        assert_eq!(rms_energy(&[]), 0.0);
        assert_eq!(rms_energy(&[0.0, 0.0, 0.0]), 0.0);

        // Constant signal: RMS = absolute value
        let rms = rms_energy(&[0.5, 0.5, 0.5, 0.5]);
        assert!((rms - 0.5).abs() < 0.001);

        // Sine wave: RMS ≈ amplitude / sqrt(2)
        let sine: Vec<f32> = (0..1000)
            .map(|i| (2.0 * std::f32::consts::PI * i as f32 / 100.0).sin())
            .collect();
        let rms = rms_energy(&sine);
        let expected = 1.0 / (2.0f32).sqrt();
        assert!((rms - expected).abs() < 0.01);
    }

    #[test]
    fn zcr_values() {
        assert_eq!(zero_crossing_rate(&[]), 0.0);
        assert_eq!(zero_crossing_rate(&[1.0]), 0.0);

        // All positive: zero crossings
        assert_eq!(zero_crossing_rate(&[0.1, 0.2, 0.3, 0.4]), 0.0);

        // Alternating: maximum crossings
        let zcr = zero_crossing_rate(&[0.1, -0.1, 0.1, -0.1]);
        assert!((zcr - 1.0).abs() < 0.01);

        // Single crossing in 4 samples: 1/3
        let zcr = zero_crossing_rate(&[0.1, 0.2, -0.1, -0.2]);
        assert!((zcr - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn adaptive_threshold_tracks_noise() {
        let mut vad = VoiceDetector::new(VadConfig {
            adaptation_rate: 0.9, // fast adaptation for test
            ..VadConfig::default()
        });

        // Feed low-level ambient noise
        let ambient = vec![0.001f32; 480];
        for _ in 0..50 {
            vad.process(&ambient);
        }

        // Noise floor should have adapted down toward 0.001
        assert!(
            vad.noise_floor() < 0.01,
            "noise floor should adapt down: {}",
            vad.noise_floor()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut vad = VoiceDetector::new(VadConfig {
            onset_frames: 1,
            ..VadConfig::default()
        });

        let speech: Vec<f32> = (0..480)
            .map(|i| 0.3 * (2.0 * std::f32::consts::PI * 300.0 * i as f32 / 16000.0).sin())
            .collect();

        vad.process(&speech);
        assert_ne!(vad.state(), VadState::Silence);

        vad.reset();
        assert_eq!(vad.state(), VadState::Silence);
        assert_eq!(vad.frame_count(), 0);
    }
}
