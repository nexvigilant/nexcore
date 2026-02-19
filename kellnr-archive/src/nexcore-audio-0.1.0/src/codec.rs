// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Audio codec registry — format conversion and encoding.
//!
//! Tier: T2-C (μ + Σ + N — mapping between format variants with numeric conversion)
//!
//! Codecs convert between sample formats (S16 ↔ F32, etc.) and provide
//! sample rate conversion (resampling).

use crate::sample::{AudioSpec, SampleRate};
use serde::{Deserialize, Serialize};

/// Codec identifier.
///
/// Tier: T2-P (Σ Sum — codec variants)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodecId {
    /// Raw PCM (no encoding).
    Pcm,
    /// μ-law companding (telephony).
    MuLaw,
    /// A-law companding (telephony).
    ALaw,
}

impl CodecId {
    /// Human-readable name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pcm => "PCM",
            Self::MuLaw => "μ-law",
            Self::ALaw => "A-law",
        }
    }
}

impl std::fmt::Display for CodecId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Convert a single S16 sample to F32.
///
/// Maps [-32768, 32767] → [-1.0, 1.0].
pub fn s16_to_f32(sample: i16) -> f32 {
    f32::from(sample) / 32768.0
}

/// Convert a single F32 sample to S16.
///
/// Maps [-1.0, 1.0] → [-32768, 32767] with clamping.
pub fn f32_to_s16(sample: f32) -> i16 {
    let clamped = sample.clamp(-1.0, 1.0);
    (clamped * 32767.0) as i16
}

/// Convert a single U8 sample to F32.
///
/// Maps [0, 255] → [-1.0, 1.0].
pub fn u8_to_f32(sample: u8) -> f32 {
    (f32::from(sample) - 128.0) / 128.0
}

/// Convert a single F32 sample to U8.
///
/// Maps [-1.0, 1.0] → [0, 255].
pub fn f32_to_u8(sample: f32) -> u8 {
    let clamped = sample.clamp(-1.0, 1.0);
    ((clamped + 1.0) * 127.5) as u8
}

/// Convert a buffer of S16 samples to F32.
pub fn convert_s16_to_f32(input: &[i16]) -> Vec<f32> {
    input.iter().map(|&s| s16_to_f32(s)).collect()
}

/// Convert a buffer of F32 samples to S16.
pub fn convert_f32_to_s16(input: &[f32]) -> Vec<i16> {
    input.iter().map(|&s| f32_to_s16(s)).collect()
}

/// Simple resampling quality.
///
/// Tier: T2-P (Σ Sum — quality tiers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResampleQuality {
    /// Nearest-neighbor (fastest, worst quality).
    Nearest,
    /// Linear interpolation (good quality, fast).
    Linear,
}

/// Resample F32 audio from one rate to another using linear interpolation.
///
/// Returns resampled buffer. This is a simple synchronous resampler
/// suitable for non-realtime conversion.
pub fn resample_f32(
    input: &[f32],
    from_rate: SampleRate,
    to_rate: SampleRate,
    _quality: ResampleQuality,
) -> Vec<f32> {
    let from_hz = from_rate.hz() as f64;
    let to_hz = to_rate.hz() as f64;

    if (from_hz - to_hz).abs() < f64::EPSILON || input.is_empty() {
        return input.to_vec();
    }

    let ratio = to_hz / from_hz;
    let output_len = (input.len() as f64 * ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 / ratio;
        let idx = src_pos as usize;
        let frac = (src_pos - idx as f64) as f32;

        let sample = if idx + 1 < input.len() {
            input[idx].mul_add(1.0 - frac, input[idx + 1] * frac)
        } else if idx < input.len() {
            input[idx]
        } else {
            0.0
        };

        output.push(sample);
    }

    output
}

/// Conversion specification — describes a format conversion operation.
///
/// Tier: T2-C (μ Mapping — source → destination format mapping)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionSpec {
    /// Source format.
    pub from: AudioSpec,
    /// Destination format.
    pub to: AudioSpec,
}

impl ConversionSpec {
    /// Create a new conversion spec.
    pub const fn new(from: AudioSpec, to: AudioSpec) -> Self {
        Self { from, to }
    }

    /// Whether format conversion is needed (different sample formats).
    pub fn needs_format_conversion(&self) -> bool {
        self.from.format != self.to.format
    }

    /// Whether resampling is needed (different rates).
    pub fn needs_resampling(&self) -> bool {
        self.from.rate != self.to.rate
    }

    /// Whether channel remapping is needed (different layouts).
    pub fn needs_channel_remap(&self) -> bool {
        self.from.layout != self.to.layout
    }

    /// Whether any conversion is needed at all.
    pub fn needs_conversion(&self) -> bool {
        self.needs_format_conversion() || self.needs_resampling() || self.needs_channel_remap()
    }
}

impl std::fmt::Display for ConversionSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} → {}", self.from, self.to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::{ChannelLayout, SampleFormat};

    #[test]
    fn s16_to_f32_conversion() {
        assert!((s16_to_f32(0) - 0.0).abs() < 0.001);
        assert!((s16_to_f32(32767) - 1.0).abs() < 0.001);
        assert!((s16_to_f32(-32768) + 1.0).abs() < 0.001);
    }

    #[test]
    fn f32_to_s16_conversion() {
        assert_eq!(f32_to_s16(0.0), 0);
        assert_eq!(f32_to_s16(1.0), 32767);
        assert_eq!(f32_to_s16(-1.0), -32767);
    }

    #[test]
    fn f32_to_s16_clamping() {
        assert_eq!(f32_to_s16(2.0), 32767);
        assert_eq!(f32_to_s16(-2.0), -32767);
    }

    #[test]
    fn u8_to_f32_conversion() {
        assert!((u8_to_f32(128) - 0.0).abs() < 0.01);
        assert!((u8_to_f32(255) - 1.0).abs() < 0.02);
        assert!((u8_to_f32(0) + 1.0).abs() < 0.01);
    }

    #[test]
    fn f32_to_u8_conversion() {
        assert_eq!(f32_to_u8(0.0), 127); // center
        assert_eq!(f32_to_u8(1.0), 255);
        assert_eq!(f32_to_u8(-1.0), 0);
    }

    #[test]
    fn batch_s16_to_f32() {
        let input = vec![0i16, 16384, -16384];
        let output = convert_s16_to_f32(&input);
        assert_eq!(output.len(), 3);
        assert!((output[0] - 0.0).abs() < 0.01);
        assert!((output[1] - 0.5).abs() < 0.01);
        assert!((output[2] + 0.5).abs() < 0.01);
    }

    #[test]
    fn batch_f32_to_s16() {
        let input = vec![0.0f32, 0.5, -0.5];
        let output = convert_f32_to_s16(&input);
        assert_eq!(output.len(), 3);
        assert_eq!(output[0], 0);
        assert!(output[1] > 16000);
        assert!(output[2] < -16000);
    }

    #[test]
    fn roundtrip_s16_f32() {
        let original = 12345i16;
        let f = s16_to_f32(original);
        let back = f32_to_s16(f);
        assert!((original - back).abs() <= 1);
    }

    #[test]
    fn resample_same_rate() {
        let input = vec![1.0f32; 100];
        let output = resample_f32(
            &input,
            SampleRate::Hz44100,
            SampleRate::Hz44100,
            ResampleQuality::Linear,
        );
        assert_eq!(output.len(), 100);
    }

    #[test]
    fn resample_upsample() {
        let input = vec![0.5f32; 100];
        let output = resample_f32(
            &input,
            SampleRate::Hz44100,
            SampleRate::Hz48000,
            ResampleQuality::Linear,
        );
        // 48000/44100 ≈ 1.088 → ~109 samples
        assert!(output.len() > 100);
        assert!(output.len() <= 110);
    }

    #[test]
    fn resample_downsample() {
        let input = vec![0.5f32; 100];
        let output = resample_f32(
            &input,
            SampleRate::Hz48000,
            SampleRate::Hz44100,
            ResampleQuality::Linear,
        );
        // 44100/48000 ≈ 0.919 → ~92 samples
        assert!(output.len() < 100);
        assert!(output.len() >= 90);
    }

    #[test]
    fn resample_empty() {
        let output = resample_f32(
            &[],
            SampleRate::Hz44100,
            SampleRate::Hz48000,
            ResampleQuality::Linear,
        );
        assert!(output.is_empty());
    }

    #[test]
    fn codec_id_names() {
        assert_eq!(CodecId::Pcm.name(), "PCM");
        assert_eq!(CodecId::MuLaw.to_string(), "μ-law");
        assert_eq!(CodecId::ALaw.to_string(), "A-law");
    }

    #[test]
    fn conversion_spec_identity() {
        let spec = AudioSpec::cd_quality();
        let conv = ConversionSpec::new(spec, spec);
        assert!(!conv.needs_conversion());
        assert!(!conv.needs_format_conversion());
        assert!(!conv.needs_resampling());
        assert!(!conv.needs_channel_remap());
    }

    #[test]
    fn conversion_spec_format_change() {
        let from = AudioSpec::cd_quality();
        let to = AudioSpec::new(
            SampleFormat::F32,
            SampleRate::Hz44100,
            ChannelLayout::Stereo,
        );
        let conv = ConversionSpec::new(from, to);
        assert!(conv.needs_format_conversion());
        assert!(!conv.needs_resampling());
        assert!(conv.needs_conversion());
    }

    #[test]
    fn conversion_spec_rate_change() {
        let from = AudioSpec::cd_quality();
        let to = AudioSpec::new(
            SampleFormat::S16,
            SampleRate::Hz48000,
            ChannelLayout::Stereo,
        );
        let conv = ConversionSpec::new(from, to);
        assert!(!conv.needs_format_conversion());
        assert!(conv.needs_resampling());
        assert!(conv.needs_conversion());
    }

    #[test]
    fn conversion_spec_display() {
        let from = AudioSpec::cd_quality();
        let to = AudioSpec::float_stereo();
        let conv = ConversionSpec::new(from, to);
        let s = conv.to_string();
        assert!(s.contains("S16"));
        assert!(s.contains("F32"));
    }
}
