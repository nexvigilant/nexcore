// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Audio sample types — formats, rates, and channel layouts.
//!
//! Tier: T2-P (N Quantity — numeric sample representation)
//!
//! ## Primitive Grounding
//!
//! | Type | Primitives | Role |
//! |------|-----------|------|
//! | SampleFormat | Σ + N | Numeric encoding variants |
//! | SampleRate | N + ν | Samples per second |
//! | ChannelLayout | Σ + N | Speaker arrangement |
//! | AudioSpec | × (Product) | Complete format specification |

use serde::{Deserialize, Serialize};

/// PCM sample format (bit depth + encoding).
///
/// Tier: T2-P (Σ Sum — format variant union)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SampleFormat {
    /// Signed 16-bit integer (CD quality).
    S16,
    /// Signed 24-bit integer (packed, studio quality).
    S24,
    /// Signed 32-bit integer.
    S32,
    /// 32-bit IEEE float [-1.0, 1.0].
    F32,
    /// 8-bit unsigned (legacy).
    U8,
}

impl SampleFormat {
    /// Bytes per sample for this format.
    pub const fn bytes_per_sample(self) -> usize {
        match self {
            Self::U8 => 1,
            Self::S16 => 2,
            Self::S24 => 3,
            Self::S32 | Self::F32 => 4,
        }
    }

    /// Bits per sample.
    pub const fn bits_per_sample(self) -> u32 {
        (self.bytes_per_sample() * 8) as u32
    }

    /// Whether this is a floating-point format.
    pub const fn is_float(self) -> bool {
        matches!(self, Self::F32)
    }

    /// Whether this is an integer format.
    pub const fn is_integer(self) -> bool {
        !self.is_float()
    }

    /// Human-readable name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::U8 => "U8",
            Self::S16 => "S16",
            Self::S24 => "S24",
            Self::S32 => "S32",
            Self::F32 => "F32",
        }
    }
}

impl std::fmt::Display for SampleFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Standard sample rates.
///
/// Tier: T2-P (N + ν — quantified frequency)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SampleRate {
    /// 8 kHz (telephony).
    Hz8000,
    /// 11.025 kHz (low quality).
    Hz11025,
    /// 16 kHz (wideband voice).
    Hz16000,
    /// 22.05 kHz (AM radio quality).
    Hz22050,
    /// 44.1 kHz (CD quality).
    Hz44100,
    /// 48 kHz (DVD/Blu-ray, default for pro audio).
    Hz48000,
    /// 88.2 kHz (high-res audio).
    Hz88200,
    /// 96 kHz (high-res audio).
    Hz96000,
    /// 176.4 kHz (ultra high-res).
    Hz176400,
    /// 192 kHz (ultra high-res).
    Hz192000,
    /// Custom sample rate.
    Custom(u32),
}

impl SampleRate {
    /// Get the rate in Hz.
    pub const fn hz(self) -> u32 {
        match self {
            Self::Hz8000 => 8000,
            Self::Hz11025 => 11025,
            Self::Hz16000 => 16000,
            Self::Hz22050 => 22050,
            Self::Hz44100 => 44100,
            Self::Hz48000 => 48000,
            Self::Hz88200 => 88200,
            Self::Hz96000 => 96000,
            Self::Hz176400 => 176_400,
            Self::Hz192000 => 192_000,
            Self::Custom(hz) => hz,
        }
    }

    /// Create from raw Hz value, mapping to known rates when possible.
    pub const fn from_hz(hz: u32) -> Self {
        match hz {
            8000 => Self::Hz8000,
            11025 => Self::Hz11025,
            16000 => Self::Hz16000,
            22050 => Self::Hz22050,
            44100 => Self::Hz44100,
            48000 => Self::Hz48000,
            88200 => Self::Hz88200,
            96000 => Self::Hz96000,
            176_400 => Self::Hz176400,
            192_000 => Self::Hz192000,
            other => Self::Custom(other),
        }
    }

    /// Whether this is a standard (non-custom) rate.
    pub const fn is_standard(self) -> bool {
        !matches!(self, Self::Custom(_))
    }

    /// Period in microseconds between samples.
    pub fn period_us(self) -> f64 {
        1_000_000.0 / f64::from(self.hz())
    }
}

impl std::fmt::Display for SampleRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hz = self.hz();
        if hz >= 1000 {
            write!(f, "{}.{}kHz", hz / 1000, (hz % 1000) / 100)
        } else {
            write!(f, "{hz}Hz")
        }
    }
}

/// Channel layout — speaker arrangement.
///
/// Tier: T2-P (Σ + N — variant sum of channel counts)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelLayout {
    /// Single channel.
    Mono,
    /// Left + Right.
    Stereo,
    /// 2.1 — Left, Right, LFE.
    Surround21,
    /// 5.1 — FL, FR, C, LFE, SL, SR.
    Surround51,
    /// 7.1 — FL, FR, C, LFE, SL, SR, BL, BR.
    Surround71,
    /// Custom channel count.
    Custom(u16),
}

impl ChannelLayout {
    /// Number of channels.
    pub const fn channels(self) -> u16 {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
            Self::Surround21 => 3,
            Self::Surround51 => 6,
            Self::Surround71 => 8,
            Self::Custom(n) => n,
        }
    }

    /// Create from channel count.
    pub const fn from_channels(n: u16) -> Self {
        match n {
            1 => Self::Mono,
            2 => Self::Stereo,
            3 => Self::Surround21,
            6 => Self::Surround51,
            8 => Self::Surround71,
            other => Self::Custom(other),
        }
    }

    /// Whether this layout is standard.
    pub const fn is_standard(self) -> bool {
        !matches!(self, Self::Custom(_))
    }
}

impl std::fmt::Display for ChannelLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mono => write!(f, "Mono"),
            Self::Stereo => write!(f, "Stereo"),
            Self::Surround21 => write!(f, "2.1"),
            Self::Surround51 => write!(f, "5.1"),
            Self::Surround71 => write!(f, "7.1"),
            Self::Custom(n) => write!(f, "{n}ch"),
        }
    }
}

/// Complete audio specification — format + rate + layout.
///
/// Tier: T2-C (× Product — composite specification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioSpec {
    /// Sample format.
    pub format: SampleFormat,
    /// Sample rate.
    pub rate: SampleRate,
    /// Channel layout.
    pub layout: ChannelLayout,
}

impl AudioSpec {
    /// Create a new audio specification.
    pub const fn new(format: SampleFormat, rate: SampleRate, layout: ChannelLayout) -> Self {
        Self {
            format,
            rate,
            layout,
        }
    }

    /// Standard CD quality: S16 / 44.1kHz / Stereo.
    pub const fn cd_quality() -> Self {
        Self::new(
            SampleFormat::S16,
            SampleRate::Hz44100,
            ChannelLayout::Stereo,
        )
    }

    /// Standard DVD quality: S24 / 48kHz / 5.1.
    pub const fn dvd_quality() -> Self {
        Self::new(
            SampleFormat::S24,
            SampleRate::Hz48000,
            ChannelLayout::Surround51,
        )
    }

    /// Voice quality: S16 / 16kHz / Mono.
    pub const fn voice_quality() -> Self {
        Self::new(SampleFormat::S16, SampleRate::Hz16000, ChannelLayout::Mono)
    }

    /// Float processing: F32 / 48kHz / Stereo.
    pub const fn float_stereo() -> Self {
        Self::new(
            SampleFormat::F32,
            SampleRate::Hz48000,
            ChannelLayout::Stereo,
        )
    }

    /// Bytes per frame (all channels, one sample instant).
    pub const fn bytes_per_frame(self) -> usize {
        self.format.bytes_per_sample() * self.layout.channels() as usize
    }

    /// Bytes per second at this spec.
    pub const fn bytes_per_second(self) -> usize {
        self.bytes_per_frame() * self.rate.hz() as usize
    }

    /// Duration in seconds for a given byte count.
    pub fn duration_secs(self, bytes: usize) -> f64 {
        let bps = self.bytes_per_second();
        if bps == 0 {
            return 0.0;
        }
        bytes as f64 / bps as f64
    }

    /// Byte count for a given duration in seconds.
    pub fn bytes_for_duration(self, seconds: f64) -> usize {
        (self.bytes_per_second() as f64 * seconds) as usize
    }
}

impl std::fmt::Display for AudioSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.format, self.rate, self.layout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_format_bytes() {
        assert_eq!(SampleFormat::U8.bytes_per_sample(), 1);
        assert_eq!(SampleFormat::S16.bytes_per_sample(), 2);
        assert_eq!(SampleFormat::S24.bytes_per_sample(), 3);
        assert_eq!(SampleFormat::S32.bytes_per_sample(), 4);
        assert_eq!(SampleFormat::F32.bytes_per_sample(), 4);
    }

    #[test]
    fn sample_format_bits() {
        assert_eq!(SampleFormat::S16.bits_per_sample(), 16);
        assert_eq!(SampleFormat::S24.bits_per_sample(), 24);
        assert_eq!(SampleFormat::F32.bits_per_sample(), 32);
    }

    #[test]
    fn sample_format_float_vs_int() {
        assert!(SampleFormat::F32.is_float());
        assert!(!SampleFormat::S16.is_float());
        assert!(SampleFormat::S16.is_integer());
    }

    #[test]
    fn sample_format_display() {
        assert_eq!(SampleFormat::S16.to_string(), "S16");
        assert_eq!(SampleFormat::F32.to_string(), "F32");
    }

    #[test]
    fn sample_rate_hz() {
        assert_eq!(SampleRate::Hz44100.hz(), 44100);
        assert_eq!(SampleRate::Hz48000.hz(), 48000);
        assert_eq!(SampleRate::Custom(32000).hz(), 32000);
    }

    #[test]
    fn sample_rate_from_hz() {
        assert_eq!(SampleRate::from_hz(44100), SampleRate::Hz44100);
        assert_eq!(SampleRate::from_hz(12345), SampleRate::Custom(12345));
    }

    #[test]
    fn sample_rate_standard() {
        assert!(SampleRate::Hz44100.is_standard());
        assert!(!SampleRate::Custom(32000).is_standard());
    }

    #[test]
    fn sample_rate_period() {
        let period = SampleRate::Hz48000.period_us();
        // 1_000_000 / 48000 ≈ 20.833...
        assert!((period - 20.833).abs() < 0.1);
    }

    #[test]
    fn sample_rate_display() {
        assert_eq!(SampleRate::Hz44100.to_string(), "44.1kHz");
        assert_eq!(SampleRate::Hz48000.to_string(), "48.0kHz");
    }

    #[test]
    fn channel_layout_count() {
        assert_eq!(ChannelLayout::Mono.channels(), 1);
        assert_eq!(ChannelLayout::Stereo.channels(), 2);
        assert_eq!(ChannelLayout::Surround51.channels(), 6);
        assert_eq!(ChannelLayout::Surround71.channels(), 8);
        assert_eq!(ChannelLayout::Custom(4).channels(), 4);
    }

    #[test]
    fn channel_layout_from_channels() {
        assert_eq!(ChannelLayout::from_channels(1), ChannelLayout::Mono);
        assert_eq!(ChannelLayout::from_channels(2), ChannelLayout::Stereo);
        assert_eq!(ChannelLayout::from_channels(4), ChannelLayout::Custom(4));
    }

    #[test]
    fn channel_layout_display() {
        assert_eq!(ChannelLayout::Stereo.to_string(), "Stereo");
        assert_eq!(ChannelLayout::Surround51.to_string(), "5.1");
        assert_eq!(ChannelLayout::Custom(4).to_string(), "4ch");
    }

    #[test]
    fn audio_spec_cd_quality() {
        let spec = AudioSpec::cd_quality();
        assert_eq!(spec.format, SampleFormat::S16);
        assert_eq!(spec.rate, SampleRate::Hz44100);
        assert_eq!(spec.layout, ChannelLayout::Stereo);
    }

    #[test]
    fn audio_spec_bytes_per_frame() {
        let spec = AudioSpec::cd_quality();
        // S16 = 2 bytes * 2 channels = 4 bytes per frame
        assert_eq!(spec.bytes_per_frame(), 4);
    }

    #[test]
    fn audio_spec_bytes_per_second() {
        let spec = AudioSpec::cd_quality();
        // 4 bytes/frame * 44100 frames/sec = 176400 bytes/sec
        assert_eq!(spec.bytes_per_second(), 176_400);
    }

    #[test]
    fn audio_spec_duration() {
        let spec = AudioSpec::cd_quality();
        let one_sec_bytes = spec.bytes_per_second();
        let dur = spec.duration_secs(one_sec_bytes);
        assert!((dur - 1.0).abs() < 0.001);
    }

    #[test]
    fn audio_spec_bytes_for_duration() {
        let spec = AudioSpec::cd_quality();
        let bytes = spec.bytes_for_duration(1.0);
        assert_eq!(bytes, 176_400);
    }

    #[test]
    fn audio_spec_display() {
        let spec = AudioSpec::cd_quality();
        let s = spec.to_string();
        assert!(s.contains("S16"));
        assert!(s.contains("44.1kHz"));
        assert!(s.contains("Stereo"));
    }

    #[test]
    fn voice_quality_spec() {
        let spec = AudioSpec::voice_quality();
        assert_eq!(spec.format, SampleFormat::S16);
        assert_eq!(spec.rate, SampleRate::Hz16000);
        assert_eq!(spec.layout, ChannelLayout::Mono);
        // 2 bytes/sample * 1 channel * 16000 = 32000 bytes/sec
        assert_eq!(spec.bytes_per_second(), 32000);
    }

    #[test]
    fn dvd_quality_spec() {
        let spec = AudioSpec::dvd_quality();
        assert_eq!(spec.layout, ChannelLayout::Surround51);
        // S24 = 3 bytes * 6 channels = 18 bytes/frame * 48000 = 864000
        assert_eq!(spec.bytes_per_second(), 864_000);
    }

    #[test]
    fn float_stereo_spec() {
        let spec = AudioSpec::float_stereo();
        assert!(spec.format.is_float());
        // F32 = 4 bytes * 2 channels = 8 bytes/frame * 48000 = 384000
        assert_eq!(spec.bytes_per_second(), 384_000);
    }
}
