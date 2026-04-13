// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Echo Cancellation — reference signal subtraction.
//!
//! When Vigil speaks (TTS output), the microphone picks up that audio.
//! Without echo cancellation, Vigil transcribes its own voice in a
//! feedback loop. The current hack (mute file) creates dead zones.
//!
//! This module provides two approaches:
//! 1. **Reference subtraction** — subtract the known TTS waveform from mic input
//! 2. **Correlation-based gating** — detect when mic signal correlates with reference
//!
//! Both are pure Rust, zero external dependencies.
//!
//! ## JARVIS Layer 0
//!
//! Echo cancellation is the membrane (∂) between Vigil's output and input.
//! Without it, the system's output becomes its input — a conservation violation.

use serde::Serialize;

/// Configuration for the echo canceller.
#[derive(Debug, Clone)]
pub struct AecConfig {
    /// Maximum delay (in samples) between reference and mic pickup.
    /// Accounts for speaker→mic propagation. Default: 4800 (300ms at 16kHz).
    pub max_delay_samples: usize,
    /// Correlation threshold to detect echo presence.
    /// Above this → echo detected, subtract. Default: 0.3.
    pub correlation_threshold: f32,
    /// Subtraction gain (0.0..1.0) when mic energy ≈ reference energy (pure echo).
    /// 1.0 = full subtraction, 0.8 = conservative. Default: 0.85.
    pub subtraction_gain: f32,
    /// Reduced subtraction gain for double-talk.
    /// Applied when mic_energy > `double_talk_ratio` × ref_energy.
    /// Lower = preserve more real speech. Default: 0.3.
    pub double_talk_gain: f32,
    /// Energy ratio threshold for double-talk detection.
    /// When mic_energy > ratio × ref_energy, real speech is present.
    /// Default: 1.5.
    pub double_talk_ratio: f32,
    /// Residual suppression floor. After subtraction, if residual is
    /// below this, zero it out. Prevents musical noise. Default: 0.005.
    pub residual_floor: f32,
}

impl Default for AecConfig {
    fn default() -> Self {
        Self {
            max_delay_samples: 4800, // 300ms at 16kHz
            correlation_threshold: 0.3,
            subtraction_gain: 0.85,
            double_talk_gain: 0.3,
            double_talk_ratio: 1.5,
            residual_floor: 0.005,
        }
    }
}

/// Result of echo cancellation on a frame.
#[derive(Debug, Clone, Serialize)]
pub struct AecResult {
    /// Whether echo was detected in this frame.
    pub echo_detected: bool,
    /// Correlation coefficient between mic and reference.
    pub correlation: f32,
    /// Estimated delay in samples (if echo detected).
    pub delay_samples: usize,
    /// RMS of the output (after cancellation).
    pub output_rms: f32,
    /// Whether double-talk was detected (real speech mixed with echo).
    pub double_talk: bool,
    /// Actual subtraction gain used (lower during double-talk).
    pub gain_used: f32,
}

/// Echo canceller using reference signal correlation and subtraction.
///
/// Feed it the reference signal (what Vigil is saying) and the mic signal
/// (what the mic picks up). It outputs the mic signal with the echo removed.
pub struct EchoCanceller {
    config: AecConfig,
    /// Circular buffer of recent reference samples.
    reference_buffer: Vec<f32>,
    /// Write position in reference buffer.
    ref_write_pos: usize,
    /// Whether the reference is currently active (TTS playing).
    ref_active: bool,
    /// Frames since reference went inactive.
    frames_since_ref: u32,
    /// Tail length — how long echo persists after reference stops.
    /// Default: 30 frames (~900ms at 30ms/frame).
    tail_frames: u32,
}

impl EchoCanceller {
    /// Create a new echo canceller.
    #[must_use]
    pub fn new(config: AecConfig) -> Self {
        let buf_size = config.max_delay_samples * 2; // extra room for correlation search
        Self {
            reference_buffer: vec![0.0; buf_size],
            ref_write_pos: 0,
            ref_active: false,
            frames_since_ref: 100, // start as inactive
            tail_frames: 30,
            config,
        }
    }

    /// Feed reference audio (TTS output). Call this as TTS produces samples.
    pub fn feed_reference(&mut self, samples: &[f32]) {
        self.ref_active = true;
        self.frames_since_ref = 0;
        for &s in samples {
            if self.ref_write_pos >= self.reference_buffer.len() {
                self.ref_write_pos = 0;
            }
            self.reference_buffer[self.ref_write_pos] = s;
            self.ref_write_pos = self.ref_write_pos.saturating_add(1);
        }
    }

    /// Signal that reference playback has stopped.
    pub fn reference_stopped(&mut self) {
        self.ref_active = false;
    }

    /// Process a mic frame, removing echo from reference.
    ///
    /// Returns the cleaned frame and metadata.
    pub fn process(&mut self, mic_frame: &[f32]) -> (Vec<f32>, AecResult) {
        if !self.ref_active {
            self.frames_since_ref = self.frames_since_ref.saturating_add(1);
        }

        // If no reference has been active recently, pass through
        if self.frames_since_ref > self.tail_frames {
            let rms = crate::vad::rms_energy(mic_frame);
            return (
                mic_frame.to_vec(),
                AecResult {
                    echo_detected: false,
                    correlation: 0.0,
                    delay_samples: 0,
                    output_rms: rms,
                    double_talk: false,
                    gain_used: 0.0,
                },
            );
        }

        // Find best correlation between mic and reference at different delays
        let (best_corr, best_delay) = self.find_best_correlation(mic_frame);

        if best_corr < self.config.correlation_threshold {
            // Low correlation — likely genuine speech, not echo
            let rms = crate::vad::rms_energy(mic_frame);
            return (
                mic_frame.to_vec(),
                AecResult {
                    echo_detected: false,
                    correlation: best_corr,
                    delay_samples: best_delay,
                    output_rms: rms,
                    double_talk: false,
                    gain_used: 0.0,
                },
            );
        }

        // Echo detected — subtract reference at estimated delay
        // Adaptive gain: when mic is significantly louder than reference,
        // real speech is mixed in (double-talk). Reduce subtraction to preserve it.
        let mic_rms = crate::vad::rms_energy(mic_frame);
        let ref_rms = self.reference_rms_at_delay(mic_frame.len(), best_delay);
        let gain = if ref_rms > 0.001 && mic_rms > ref_rms * self.config.double_talk_ratio {
            // Double-talk detected: mic energy exceeds reference × ratio
            self.config.double_talk_gain
        } else {
            self.config.subtraction_gain
        };

        let mut output = Vec::with_capacity(mic_frame.len());
        let n = mic_frame.len();

        for (i, &mic_sample) in mic_frame.iter().enumerate() {
            let offset = best_delay + (n - 1 - i);
            let ref_sample = self.ref_sample_at(offset);

            let mut cleaned = mic_sample - ref_sample * gain;

            // Residual suppression
            if cleaned.abs() < self.config.residual_floor {
                cleaned = 0.0;
            }

            output.push(cleaned.clamp(-1.0, 1.0));
        }

        let is_double_talk = gain < self.config.subtraction_gain;
        let rms = crate::vad::rms_energy(&output);

        (
            output,
            AecResult {
                echo_detected: true,
                correlation: best_corr,
                delay_samples: best_delay,
                output_rms: rms,
                double_talk: is_double_talk,
                gain_used: gain,
            },
        )
    }

    /// Compute RMS of reference signal at a given delay (for double-talk detection).
    fn reference_rms_at_delay(&self, frame_len: usize, delay: usize) -> f32 {
        let mut sum_sq: f64 = 0.0;
        for i in 0..frame_len {
            let offset = delay + (frame_len - 1 - i);
            let s = self.ref_sample_at(offset) as f64;
            sum_sq += s * s;
        }
        if frame_len == 0 {
            return 0.0;
        }
        #[allow(clippy::as_conversions, reason = "frame_len bounded by mic frame size")]
        let mean = sum_sq / frame_len as f64;
        mean.sqrt() as f32
    }

    /// Find the delay that maximizes normalized cross-correlation.
    fn find_best_correlation(&self, mic_frame: &[f32]) -> (f32, usize) {
        let mut best_corr: f32 = 0.0;
        let mut best_delay: usize = 0;
        let buf_len = self.reference_buffer.len();
        let frame_len = mic_frame.len();

        // Search delays in steps (full sample search is expensive)
        let step = 16; // ~1ms steps at 16kHz
        let max_delay = self.config.max_delay_samples.min(buf_len / 2);

        let mut delay = 0;
        while delay < max_delay {
            let corr = self.correlation_at_delay(mic_frame, frame_len, buf_len, delay);
            if corr > best_corr {
                best_corr = corr;
                best_delay = delay;
            }
            delay = delay.saturating_add(step);
        }

        // Refine around best: search ±step at sample resolution
        let refine_start = best_delay.saturating_sub(step);
        let refine_end = (best_delay + step).min(max_delay);
        let mut d = refine_start;
        while d <= refine_end {
            let corr = self.correlation_at_delay(mic_frame, frame_len, buf_len, d);
            if corr > best_corr {
                best_corr = corr;
                best_delay = d;
            }
            d = d.saturating_add(1);
        }

        (best_corr, best_delay)
    }

    /// Safe circular index: look back `offset` samples from `ref_write_pos`.
    fn ref_sample_at(&self, offset: usize) -> f32 {
        let buf_len = self.reference_buffer.len();
        if buf_len == 0 || offset >= buf_len {
            return 0.0;
        }
        let idx = if self.ref_write_pos >= offset {
            self.ref_write_pos - offset
        } else {
            buf_len - (offset - self.ref_write_pos)
        };
        self.reference_buffer[idx]
    }

    /// Normalized cross-correlation at a specific delay.
    fn correlation_at_delay(
        &self,
        mic_frame: &[f32],
        _frame_len: usize,
        _buf_len: usize,
        delay: usize,
    ) -> f32 {
        let mut sum_xy: f64 = 0.0;
        let mut sum_xx: f64 = 0.0;
        let mut sum_yy: f64 = 0.0;
        let n = mic_frame.len();

        for (i, &mic_sample) in mic_frame.iter().enumerate() {
            // How far back from write_pos: delay + (n - 1 - i)
            // i=0 is the oldest sample in the mic frame, i=n-1 is the newest.
            // The newest mic sample corresponds to the reference written `delay` samples ago.
            let offset = delay + (n - 1 - i);
            let ref_sample = self.ref_sample_at(offset) as f64;

            let m = mic_sample as f64;
            sum_xy += m * ref_sample;
            sum_xx += m * m;
            sum_yy += ref_sample * ref_sample;
        }

        let denom = (sum_xx * sum_yy).sqrt();
        if denom < 1e-10 {
            return 0.0;
        }

        (sum_xy / denom) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_when_no_reference() {
        let mut aec = EchoCanceller::new(AecConfig::default());
        let mic = vec![0.1f32; 480];
        let (output, result) = aec.process(&mic);
        assert!(!result.echo_detected);
        assert_eq!(output.len(), mic.len());
        // Should be unchanged
        assert!((output[0] - 0.1).abs() < 0.001);
    }

    #[test]
    fn detects_echo_from_reference() {
        let mut aec = EchoCanceller::new(AecConfig {
            correlation_threshold: 0.2,
            max_delay_samples: 480,
            ..AecConfig::default()
        });

        // Generate a reference signal
        let reference: Vec<f32> = (0..480)
            .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16000.0).sin())
            .collect();

        // Feed reference
        aec.feed_reference(&reference);

        // Mic picks up the same signal (no delay for simplicity)
        let mic = reference.clone();
        let (_output, result) = aec.process(&mic);

        // Should detect high correlation
        assert!(
            result.correlation > 0.2,
            "correlation too low: {}",
            result.correlation
        );
    }

    #[test]
    fn reference_stop_clears_after_tail() {
        let mut aec = EchoCanceller::new(AecConfig::default());
        aec.tail_frames = 2; // short tail for test

        let reference: Vec<f32> = (0..480)
            .map(|i| 0.3 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16000.0).sin())
            .collect();

        aec.feed_reference(&reference);
        aec.reference_stopped();

        // Process frames until tail expires
        let mic = vec![0.1f32; 480];
        aec.process(&mic); // frame 1
        aec.process(&mic); // frame 2
        let (_, result) = aec.process(&mic); // frame 3 — tail expired

        assert!(
            !result.echo_detected,
            "should passthrough after tail expires"
        );
    }

    #[test]
    fn output_clamped_to_valid_range() {
        let mut aec = EchoCanceller::new(AecConfig {
            subtraction_gain: 1.0,
            correlation_threshold: 0.0, // always subtract
            max_delay_samples: 480,
            ..AecConfig::default()
        });

        let reference: Vec<f32> = (0..480)
            .map(|i| 0.9 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16000.0).sin())
            .collect();

        aec.feed_reference(&reference);

        let mic: Vec<f32> = (0..480).map(|_| 0.95).collect();
        let (output, _) = aec.process(&mic);

        for s in &output {
            assert!(*s >= -1.0 && *s <= 1.0, "output sample out of range: {}", s);
        }
    }
}
