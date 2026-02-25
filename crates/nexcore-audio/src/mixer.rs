// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Audio mixer — combines multiple audio sources.
//!
//! Tier: T2-C (Σ + N + ∂ — sum of sources with quantified levels and clipping boundary)
//!
//! The mixer accepts multiple named sources, each with an independent volume,
//! mute state, and pan position. The `mix()` method combines all active sources
//! into a single output buffer using additive mixing with soft clipping.

use crate::sample::AudioSpec;
use serde::{Deserialize, Serialize};

/// A mixer input source.
///
/// Tier: T2-P (N + ς — quantified volume with mute state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerSource {
    /// Source name/label.
    pub name: String,
    /// Volume level [0.0, 1.0].
    volume: f32,
    /// Whether this source is muted.
    muted: bool,
    /// Pan position [-1.0 = full left, 0.0 = center, 1.0 = full right].
    pan: f32,
    /// Whether this source is solo'd (only solo'd sources are heard).
    solo: bool,
}

impl MixerSource {
    /// Create a new mixer source at default volume (0.75).
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            volume: 0.75,
            muted: false,
            pan: 0.0,
            solo: false,
        }
    }

    /// Set volume, clamping to [0.0, 1.0].
    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
    }

    /// Get volume.
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Get effective volume (0 if muted).
    pub fn effective_volume(&self) -> f32 {
        if self.muted { 0.0 } else { self.volume }
    }

    /// Set mute state.
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Whether muted.
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Toggle mute.
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    /// Set pan, clamping to [-1.0, 1.0].
    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    /// Get pan position.
    pub fn pan(&self) -> f32 {
        self.pan
    }

    /// Left channel gain based on pan law (constant power).
    pub fn left_gain(&self) -> f32 {
        let angle = (self.pan + 1.0) * std::f32::consts::FRAC_PI_4;
        self.effective_volume() * angle.cos()
    }

    /// Right channel gain based on pan law (constant power).
    pub fn right_gain(&self) -> f32 {
        let angle = (self.pan + 1.0) * std::f32::consts::FRAC_PI_4;
        self.effective_volume() * angle.sin()
    }

    /// Set solo state.
    pub fn set_solo(&mut self, solo: bool) {
        self.solo = solo;
    }

    /// Whether solo'd.
    pub fn is_solo(&self) -> bool {
        self.solo
    }
}

/// Audio mixer — combines multiple sources.
///
/// Tier: T3 (Σ + N + ∂ + ς — full mixing engine)
pub struct Mixer {
    /// Named input sources.
    sources: Vec<MixerSource>,
    /// Master output volume [0.0, 1.0].
    master_volume: f32,
    /// Master mute.
    master_muted: bool,
    /// Output specification.
    spec: AudioSpec,
    /// Peak level tracker [0.0, 1.0].
    peak_level: f32,
    /// Whether soft clipping is enabled.
    soft_clip: bool,
}

impl Mixer {
    /// Create a new mixer with given output spec.
    pub fn new(spec: AudioSpec) -> Self {
        Self {
            sources: Vec::new(),
            master_volume: 1.0,
            master_muted: false,
            spec,
            peak_level: 0.0,
            soft_clip: true,
        }
    }

    /// Add a source to the mixer.
    pub fn add_source(&mut self, source: MixerSource) {
        self.sources.push(source);
    }

    /// Remove a source by name.
    pub fn remove_source(&mut self, name: &str) -> bool {
        let before = self.sources.len();
        self.sources.retain(|s| s.name != name);
        self.sources.len() < before
    }

    /// Get a source by name.
    pub fn get_source(&self, name: &str) -> Option<&MixerSource> {
        self.sources.iter().find(|s| s.name == name)
    }

    /// Get a mutable source by name.
    pub fn get_source_mut(&mut self, name: &str) -> Option<&mut MixerSource> {
        self.sources.iter_mut().find(|s| s.name == name)
    }

    /// Number of sources.
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// All sources.
    pub fn sources(&self) -> &[MixerSource] {
        &self.sources
    }

    /// Set master volume.
    pub fn set_master_volume(&mut self, vol: f32) {
        self.master_volume = vol.clamp(0.0, 1.0);
    }

    /// Get master volume.
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Effective master volume (0 if muted).
    pub fn effective_master_volume(&self) -> f32 {
        if self.master_muted {
            0.0
        } else {
            self.master_volume
        }
    }

    /// Set master mute.
    pub fn set_master_muted(&mut self, muted: bool) {
        self.master_muted = muted;
    }

    /// Whether master is muted.
    pub fn is_master_muted(&self) -> bool {
        self.master_muted
    }

    /// Toggle soft clipping.
    pub fn set_soft_clip(&mut self, enabled: bool) {
        self.soft_clip = enabled;
    }

    /// Whether soft clipping is enabled.
    pub fn soft_clip_enabled(&self) -> bool {
        self.soft_clip
    }

    /// Get output audio spec.
    pub fn spec(&self) -> AudioSpec {
        self.spec
    }

    /// Current peak level [0.0, 1.0].
    pub fn peak_level(&self) -> f32 {
        self.peak_level
    }

    /// Reset peak level tracker.
    pub fn reset_peak(&mut self) {
        self.peak_level = 0.0;
    }

    /// Mix F32 sample arrays from all active sources.
    ///
    /// Each source provides its own sample buffer. The mixer combines them
    /// additively, applies master volume, and optionally soft-clips.
    ///
    /// Returns the mixed output buffer.
    pub fn mix_f32(&mut self, source_buffers: &[(&str, &[f32])]) -> Vec<f32> {
        if source_buffers.is_empty() {
            return Vec::new();
        }

        let any_solo = self.sources.iter().any(|s| s.solo);

        // Find max length
        let max_len = source_buffers
            .iter()
            .map(|(_, buf)| buf.len())
            .max()
            .unwrap_or(0);
        let mut output = vec![0.0f32; max_len];

        let master = self.effective_master_volume();

        for (name, samples) in source_buffers {
            if let Some(source) = self.sources.iter().find(|s| s.name == *name) {
                // Skip if solo mode is active and this source isn't solo'd
                if any_solo && !source.solo {
                    continue;
                }

                let vol = source.effective_volume();
                if vol < f32::EPSILON {
                    continue;
                }

                for (i, &sample) in samples.iter().enumerate() {
                    if let Some(out_sample) = output.get_mut(i) {
                        *out_sample += sample * vol;
                    }
                }
            }
        }

        // Apply master volume and track peak
        let mut local_peak: f32 = 0.0;
        for sample in &mut output {
            *sample *= master;

            if self.soft_clip {
                // Soft clip using tanh
                *sample = sample.tanh();
            }

            let abs = sample.abs();
            if abs > local_peak {
                local_peak = abs;
            }
        }

        if local_peak > self.peak_level {
            self.peak_level = local_peak;
        }

        output
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        let active = self.sources.iter().filter(|s| !s.muted).count();
        format!(
            "Mixer: {}/{} sources active, master={:.0}%{}, peak={:.1}%",
            active,
            self.sources.len(),
            self.master_volume * 100.0,
            if self.master_muted { " (MUTED)" } else { "" },
            self.peak_level * 100.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_mixer() -> Mixer {
        Mixer::new(AudioSpec::float_stereo())
    }

    #[test]
    fn mixer_creation() {
        let m = test_mixer();
        assert_eq!(m.source_count(), 0);
        assert_eq!(m.master_volume(), 1.0);
        assert!(!m.is_master_muted());
    }

    #[test]
    fn add_remove_sources() {
        let mut m = test_mixer();
        m.add_source(MixerSource::new("music"));
        m.add_source(MixerSource::new("voice"));
        assert_eq!(m.source_count(), 2);

        assert!(m.remove_source("music"));
        assert_eq!(m.source_count(), 1);

        assert!(!m.remove_source("nonexistent"));
    }

    #[test]
    fn source_volume_control() {
        let mut src = MixerSource::new("test");
        assert_eq!(src.volume(), 0.75);

        src.set_volume(0.5);
        assert_eq!(src.volume(), 0.5);

        // Clamping
        src.set_volume(2.0);
        assert_eq!(src.volume(), 1.0);
        src.set_volume(-1.0);
        assert_eq!(src.volume(), 0.0);
    }

    #[test]
    fn source_mute() {
        let mut src = MixerSource::new("test");
        assert!(!src.is_muted());

        src.toggle_mute();
        assert!(src.is_muted());
        assert_eq!(src.effective_volume(), 0.0);

        src.toggle_mute();
        assert!(!src.is_muted());
        assert!(src.effective_volume() > 0.0);
    }

    #[test]
    fn source_pan() {
        let mut src = MixerSource::new("test");
        assert_eq!(src.pan(), 0.0); // Center

        src.set_pan(-1.0); // Full left
        assert_eq!(src.pan(), -1.0);

        // Clamping
        src.set_pan(5.0);
        assert_eq!(src.pan(), 1.0);
    }

    #[test]
    fn pan_gains() {
        let mut src = MixerSource::new("test");
        src.set_volume(1.0);

        // Center — equal power to both channels
        src.set_pan(0.0);
        let left = src.left_gain();
        let right = src.right_gain();
        assert!((left - right).abs() < 0.01);

        // Full left
        src.set_pan(-1.0);
        assert!(src.left_gain() > src.right_gain());

        // Full right
        src.set_pan(1.0);
        assert!(src.right_gain() > src.left_gain());
    }

    #[test]
    fn master_volume() {
        let mut m = test_mixer();
        m.set_master_volume(0.5);
        assert_eq!(m.master_volume(), 0.5);
        assert_eq!(m.effective_master_volume(), 0.5);

        m.set_master_muted(true);
        assert_eq!(m.effective_master_volume(), 0.0);
    }

    #[test]
    fn mix_empty() {
        let mut m = test_mixer();
        let output = m.mix_f32(&[]);
        assert!(output.is_empty());
    }

    #[test]
    fn mix_single_source() {
        let mut m = test_mixer();
        let mut src = MixerSource::new("music");
        src.set_volume(1.0);
        m.add_source(src);

        let samples = vec![0.5f32; 100];
        let output = m.mix_f32(&[("music", &samples)]);
        assert_eq!(output.len(), 100);

        // With soft clip (tanh), values should be close to input
        for &v in &output {
            assert!(v > 0.4);
        }
    }

    #[test]
    fn mix_two_sources() {
        let mut m = test_mixer();
        let mut music = MixerSource::new("music");
        music.set_volume(0.5);
        let mut voice = MixerSource::new("voice");
        voice.set_volume(0.5);
        m.add_source(music);
        m.add_source(voice);

        let music_buf = vec![0.3f32; 50];
        let voice_buf = vec![0.3f32; 50];
        let output = m.mix_f32(&[("music", &music_buf), ("voice", &voice_buf)]);
        assert_eq!(output.len(), 50);

        // Mixed should be louder than either alone
        // 0.3 * 0.5 + 0.3 * 0.5 = 0.3, then tanh(0.3) ≈ 0.291
        for &v in &output {
            assert!(v > 0.2);
        }
    }

    #[test]
    fn muted_source_excluded() {
        let mut m = test_mixer();
        let mut src = MixerSource::new("music");
        src.set_volume(1.0);
        src.set_muted(true);
        m.add_source(src);

        let samples = vec![0.5f32; 100];
        let output = m.mix_f32(&[("music", &samples)]);

        // Should be all zeros (muted)
        for &v in &output {
            assert!(v.abs() < f32::EPSILON);
        }
    }

    #[test]
    fn solo_mode() {
        let mut m = test_mixer();
        let mut music = MixerSource::new("music");
        music.set_volume(1.0);
        let mut sfx = MixerSource::new("sfx");
        sfx.set_volume(1.0);
        sfx.set_solo(true);
        m.add_source(music);
        m.add_source(sfx);

        let music_buf = vec![0.5f32; 50];
        let sfx_buf = vec![0.3f32; 50];
        let output = m.mix_f32(&[("music", &music_buf), ("sfx", &sfx_buf)]);

        // Only sfx should be heard (solo'd) — tanh(0.3) ≈ 0.291
        for &v in &output {
            assert!(v < 0.35);
            assert!(v > 0.2);
        }
    }

    #[test]
    fn peak_tracking() {
        let mut m = test_mixer();
        let mut src = MixerSource::new("test");
        src.set_volume(1.0);
        m.add_source(src);

        assert_eq!(m.peak_level(), 0.0);

        let samples = vec![0.8f32; 10];
        let _ = m.mix_f32(&[("test", &samples)]);
        assert!(m.peak_level() > 0.5);

        m.reset_peak();
        assert_eq!(m.peak_level(), 0.0);
    }

    #[test]
    fn soft_clip_toggle() {
        let mut m = test_mixer();
        assert!(m.soft_clip_enabled());

        m.set_soft_clip(false);
        assert!(!m.soft_clip_enabled());
    }

    #[test]
    fn get_source() {
        let mut m = test_mixer();
        m.add_source(MixerSource::new("test"));
        assert!(m.get_source("test").is_some());
        assert!(m.get_source("nonexistent").is_none());
    }

    #[test]
    fn get_source_mut() {
        let mut m = test_mixer();
        m.add_source(MixerSource::new("test"));
        if let Some(src) = m.get_source_mut("test") {
            src.set_volume(0.5);
        }
        assert_eq!(m.get_source("test").map(|s| s.volume()), Some(0.5));
    }

    #[test]
    fn summary_format() {
        let mut m = test_mixer();
        m.add_source(MixerSource::new("a"));
        m.add_source(MixerSource::new("b"));
        let s = m.summary();
        assert!(s.contains("Mixer"));
        assert!(s.contains("2/2"));
    }

    #[test]
    fn source_solo_state() {
        let mut src = MixerSource::new("test");
        assert!(!src.is_solo());
        src.set_solo(true);
        assert!(src.is_solo());
    }
}
