// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Spectral Noise Gate — adaptive noise floor estimation and suppression.
//!
//! Estimates the noise floor using an exponential moving average during
//! silence, then applies a soft gate that attenuates samples below the
//! threshold. This reduces background hiss, fan noise, and HVAC rumble
//! without clipping speech transients.
//!
//! ## JARVIS Layer 0
//!
//! The noise gate is a filter (μ Mapping) that transforms raw mic input
//! into clean audio before it reaches VAD and STT. It operates on the
//! time-domain signal — no FFT required for the soft gate approach.

use serde::Serialize;

/// Configuration for the noise gate.
#[derive(Debug, Clone)]
pub struct NoiseGateConfig {
    /// Gate threshold as a multiplier of the estimated noise floor.
    /// Samples below `noise_floor * threshold_multiplier` are attenuated.
    /// Default: 2.0 (gate opens at 2x the noise floor).
    pub threshold_multiplier: f32,
    /// Attack time in frames. How quickly the gate opens when signal
    /// exceeds threshold. Default: 1 (immediate).
    pub attack_frames: u32,
    /// Release time in frames. How slowly the gate closes after signal
    /// drops below threshold. Prevents choppy gating. Default: 10.
    pub release_frames: u32,
    /// Noise floor adaptation rate (0.0..1.0). Higher = slower.
    /// Only adapts during silence (when gate is closed). Default: 0.98.
    pub adaptation_rate: f32,
    /// Minimum gain when gate is closed (0.0..1.0).
    /// 0.0 = full silence, 0.1 = -20dB. Default: 0.0.
    pub floor_gain: f32,
    /// Initial noise floor estimate. Default: 0.01.
    pub initial_floor: f32,
}

impl Default for NoiseGateConfig {
    fn default() -> Self {
        Self {
            threshold_multiplier: 2.0,
            attack_frames: 1,
            release_frames: 10,
            adaptation_rate: 0.98,
            floor_gain: 0.0,
            initial_floor: 0.01,
        }
    }
}

/// Current state of the noise gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GateState {
    /// Gate is closed (signal below threshold, output attenuated).
    Closed,
    /// Gate is opening (attack phase).
    Opening,
    /// Gate is fully open (signal above threshold, full passthrough).
    Open,
    /// Gate is closing (release phase).
    Closing,
}

/// Result of processing a frame through the noise gate.
#[derive(Debug, Clone, Serialize)]
pub struct NoiseGateResult {
    /// Current gate state.
    pub state: GateState,
    /// Current gain applied (0.0..1.0).
    pub gain: f32,
    /// Estimated noise floor (RMS).
    pub noise_floor: f32,
    /// Current threshold (noise_floor * multiplier).
    pub threshold: f32,
    /// Input RMS.
    pub input_rms: f32,
    /// Output RMS.
    pub output_rms: f32,
}

/// Adaptive noise gate with smooth attack/release.
pub struct NoiseGate {
    config: NoiseGateConfig,
    state: GateState,
    /// Current gain (0.0..1.0), smoothed by attack/release.
    current_gain: f32,
    /// Estimated noise floor (RMS).
    noise_floor: f32,
    /// Frames in current transition.
    transition_count: u32,
}

impl NoiseGate {
    /// Create a new noise gate.
    #[must_use]
    pub fn new(config: NoiseGateConfig) -> Self {
        let noise_floor = config.initial_floor;
        Self {
            state: GateState::Closed,
            current_gain: config.floor_gain,
            noise_floor,
            transition_count: 0,
            config,
        }
    }

    /// Current gate state.
    #[must_use]
    pub fn state(&self) -> GateState {
        self.state
    }

    /// Current noise floor estimate.
    #[must_use]
    pub fn noise_floor(&self) -> f32 {
        self.noise_floor
    }

    /// Reset to initial state.
    pub fn reset(&mut self) {
        self.state = GateState::Closed;
        self.current_gain = self.config.floor_gain;
        self.noise_floor = self.config.initial_floor;
        self.transition_count = 0;
    }

    /// Process a mono F32 frame. Returns the gated frame and metadata.
    pub fn process(&mut self, frame: &[f32]) -> (Vec<f32>, NoiseGateResult) {
        let input_rms = crate::vad::rms_energy(frame);
        let threshold = self.noise_floor * self.config.threshold_multiplier;

        // State machine: determine target gain
        let _target_gain = if input_rms > threshold {
            1.0
        } else {
            self.config.floor_gain
        };

        // Smooth gain transition
        match self.state {
            GateState::Closed => {
                if input_rms > threshold {
                    self.state = GateState::Opening;
                    self.transition_count = 0;
                } else {
                    // Adapt noise floor during closed state
                    self.noise_floor = self.noise_floor * self.config.adaptation_rate
                        + input_rms * (1.0 - self.config.adaptation_rate);
                }
            }
            GateState::Opening => {
                self.transition_count = self.transition_count.saturating_add(1);
                if self.transition_count >= self.config.attack_frames {
                    self.state = GateState::Open;
                    self.transition_count = 0;
                }
            }
            GateState::Open => {
                if input_rms <= threshold {
                    self.state = GateState::Closing;
                    self.transition_count = 0;
                }
            }
            GateState::Closing => {
                self.transition_count = self.transition_count.saturating_add(1);
                if input_rms > threshold {
                    // Signal returned — reopen
                    self.state = GateState::Open;
                    self.transition_count = 0;
                } else if self.transition_count >= self.config.release_frames {
                    self.state = GateState::Closed;
                    self.transition_count = 0;
                }
            }
        }

        // Compute interpolated gain for this frame
        let frame_gain = match self.state {
            GateState::Closed => self.config.floor_gain,
            GateState::Open => 1.0,
            GateState::Opening => {
                // Linear ramp from current to 1.0
                let progress = if self.config.attack_frames > 0 {
                    self.transition_count as f32 / self.config.attack_frames as f32
                } else {
                    1.0
                };
                self.current_gain + (1.0 - self.current_gain) * progress
            }
            GateState::Closing => {
                // Linear ramp from current to floor
                let progress = if self.config.release_frames > 0 {
                    self.transition_count as f32 / self.config.release_frames as f32
                } else {
                    1.0
                };
                self.current_gain + (self.config.floor_gain - self.current_gain) * progress
            }
        };

        self.current_gain = frame_gain;

        // Apply gain to frame
        let output: Vec<f32> = frame.iter().map(|&s| s * frame_gain).collect();
        let output_rms = crate::vad::rms_energy(&output);

        (
            output,
            NoiseGateResult {
                state: self.state,
                gain: frame_gain,
                noise_floor: self.noise_floor,
                threshold,
                input_rms,
                output_rms,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_stays_gated() {
        let mut gate = NoiseGate::new(NoiseGateConfig::default());
        let silence = vec![0.0f32; 480];
        let (output, result) = gate.process(&silence);
        assert_eq!(result.state, GateState::Closed);
        assert!(result.gain <= 0.01);
        assert!(output.iter().all(|&s| s.abs() < 0.001));
    }

    #[test]
    fn loud_signal_opens_gate() {
        let mut gate = NoiseGate::new(NoiseGateConfig {
            attack_frames: 1,
            ..NoiseGateConfig::default()
        });

        let loud: Vec<f32> = (0..480)
            .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 300.0 * i as f32 / 16000.0).sin())
            .collect();

        let (_, r1) = gate.process(&loud);
        assert!(matches!(r1.state, GateState::Open | GateState::Opening));

        let (_, r2) = gate.process(&loud);
        assert_eq!(r2.state, GateState::Open);
        assert!(r2.gain > 0.9);
    }

    #[test]
    fn release_smooths_gate_close() {
        let mut gate = NoiseGate::new(NoiseGateConfig {
            attack_frames: 1,
            release_frames: 5,
            ..NoiseGateConfig::default()
        });

        let loud: Vec<f32> = (0..480)
            .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 300.0 * i as f32 / 16000.0).sin())
            .collect();
        let silence = vec![0.001f32; 480];

        // Open the gate
        gate.process(&loud);
        gate.process(&loud);
        assert_eq!(gate.state(), GateState::Open);

        // Signal drops — should enter closing, not slam shut
        gate.process(&silence);
        assert_eq!(gate.state(), GateState::Closing);

        // Still closing...
        gate.process(&silence);
        assert_eq!(gate.state(), GateState::Closing);
    }

    #[test]
    fn noise_floor_adapts_during_silence() {
        let mut gate = NoiseGate::new(NoiseGateConfig {
            adaptation_rate: 0.9, // fast for test
            initial_floor: 0.1,
            ..NoiseGateConfig::default()
        });

        // Feed low noise
        let quiet = vec![0.002f32; 480];
        for _ in 0..50 {
            gate.process(&quiet);
        }

        assert!(
            gate.noise_floor() < 0.05,
            "noise floor should adapt down: {}",
            gate.noise_floor()
        );
    }

    #[test]
    fn floor_gain_preserves_ambient() {
        let mut gate = NoiseGate::new(NoiseGateConfig {
            floor_gain: 0.1, // don't fully silence
            ..NoiseGateConfig::default()
        });

        let quiet = vec![0.005f32; 480];
        let (output, result) = gate.process(&quiet);
        assert_eq!(result.state, GateState::Closed);
        // Output should be attenuated but not zero
        assert!(output.iter().any(|&s| s.abs() > 0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut gate = NoiseGate::new(NoiseGateConfig {
            attack_frames: 1,
            ..NoiseGateConfig::default()
        });

        let loud: Vec<f32> = vec![0.5; 480];
        gate.process(&loud);
        gate.process(&loud);

        gate.reset();
        assert_eq!(gate.state(), GateState::Closed);
    }
}
