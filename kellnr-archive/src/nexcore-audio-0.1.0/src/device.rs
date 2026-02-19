// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Audio device abstraction — hardware and virtual audio endpoints.
//!
//! Tier: T2-C (∃ + ς + Σ — existence of devices with state and variants)
//!
//! Devices are discovered by the platform (PAL) and registered with the
//! audio subsystem. Each device has capabilities (supported formats, rates)
//! and a lifecycle state.

use crate::sample::{AudioSpec, ChannelLayout, SampleFormat, SampleRate};
use serde::{Deserialize, Serialize};

/// Unique device identifier.
///
/// Tier: T1 (∃ Existence — identity)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(String);

impl DeviceId {
    /// Create a new device ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Device type (direction of audio flow).
///
/// Tier: T2-P (Σ Sum — direction variants)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    /// Audio output (speakers, headphones).
    Output,
    /// Audio input (microphone).
    Input,
    /// Both input and output (headset, USB audio interface).
    Duplex,
}

impl DeviceType {
    /// Whether this device can output audio.
    pub const fn can_output(self) -> bool {
        matches!(self, Self::Output | Self::Duplex)
    }

    /// Whether this device can capture audio.
    pub const fn can_input(self) -> bool {
        matches!(self, Self::Input | Self::Duplex)
    }
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Output => write!(f, "Output"),
            Self::Input => write!(f, "Input"),
            Self::Duplex => write!(f, "Duplex"),
        }
    }
}

/// Device state (lifecycle).
///
/// Tier: T2-P (ς State — device lifecycle)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceState {
    /// Device detected but not opened.
    Available,
    /// Device is open and streaming.
    Active,
    /// Device is open but paused.
    Paused,
    /// Device encountered an error.
    Error,
    /// Device was disconnected (hot-unplug).
    Disconnected,
}

impl DeviceState {
    /// Whether the device is usable.
    pub const fn is_usable(self) -> bool {
        matches!(self, Self::Available | Self::Active | Self::Paused)
    }

    /// Whether the device is streaming.
    pub const fn is_streaming(self) -> bool {
        matches!(self, Self::Active)
    }
}

impl std::fmt::Display for DeviceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Available => write!(f, "Available"),
            Self::Active => write!(f, "Active"),
            Self::Paused => write!(f, "Paused"),
            Self::Error => write!(f, "Error"),
            Self::Disconnected => write!(f, "Disconnected"),
        }
    }
}

/// Device capabilities — what formats/rates/layouts are supported.
///
/// Tier: T2-C (Σ + N + ∂ — bounded set of supported configurations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Supported sample formats.
    pub formats: Vec<SampleFormat>,
    /// Supported sample rates.
    pub rates: Vec<SampleRate>,
    /// Supported channel layouts.
    pub layouts: Vec<ChannelLayout>,
    /// Minimum buffer size in frames.
    pub min_buffer_frames: usize,
    /// Maximum buffer size in frames.
    pub max_buffer_frames: usize,
}

impl DeviceCapabilities {
    /// Check if a spec is supported by this device.
    pub fn supports(&self, spec: &AudioSpec) -> bool {
        self.formats.contains(&spec.format)
            && self.rates.contains(&spec.rate)
            && self.layouts.contains(&spec.layout)
    }

    /// Get the best matching spec for this device, preferring higher quality.
    pub fn preferred_spec(&self) -> Option<AudioSpec> {
        let format = self.formats.first().copied()?;
        let rate = self.rates.last().copied()?; // Prefer highest rate
        let layout = self.layouts.last().copied()?; // Prefer most channels
        Some(AudioSpec::new(format, rate, layout))
    }
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            formats: vec![SampleFormat::S16, SampleFormat::F32],
            rates: vec![SampleRate::Hz44100, SampleRate::Hz48000],
            layouts: vec![ChannelLayout::Mono, ChannelLayout::Stereo],
            min_buffer_frames: 64,
            max_buffer_frames: 8192,
        }
    }
}

/// An audio device (hardware or virtual).
///
/// Tier: T3 (∃ + ς + Σ + ∂ + N — full device model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Unique device identifier.
    pub id: DeviceId,
    /// Human-readable name.
    pub name: String,
    /// Device type (input/output/duplex).
    pub device_type: DeviceType,
    /// Current state.
    pub state: DeviceState,
    /// Device capabilities.
    pub capabilities: DeviceCapabilities,
    /// Whether this is the system default device.
    pub is_default: bool,
    /// Current volume level [0.0, 1.0] (None if not applicable).
    volume: Option<f32>,
    /// Whether the device is muted.
    muted: bool,
}

impl AudioDevice {
    /// Create a new audio device.
    pub fn new(id: impl Into<String>, name: impl Into<String>, device_type: DeviceType) -> Self {
        Self {
            id: DeviceId::new(id),
            name: name.into(),
            device_type,
            state: DeviceState::Available,
            capabilities: DeviceCapabilities::default(),
            is_default: false,
            volume: Some(0.75),
            muted: false,
        }
    }

    /// Mark as default device.
    #[must_use]
    pub fn as_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    /// Set custom capabilities.
    #[must_use]
    pub fn with_capabilities(mut self, caps: DeviceCapabilities) -> Self {
        self.capabilities = caps;
        self
    }

    /// Get current volume [0.0, 1.0].
    pub fn volume(&self) -> Option<f32> {
        if self.muted { Some(0.0) } else { self.volume }
    }

    /// Set volume, clamping to [0.0, 1.0].
    pub fn set_volume(&mut self, vol: f32) {
        self.volume = Some(vol.clamp(0.0, 1.0));
    }

    /// Get raw volume (ignoring mute state).
    pub fn raw_volume(&self) -> Option<f32> {
        self.volume
    }

    /// Whether the device is muted.
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Toggle mute state.
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    /// Set mute state.
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Effective volume (0 if muted, otherwise volume level).
    pub fn effective_volume(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.volume.unwrap_or(0.0)
        }
    }

    /// Check if this device supports a given audio spec.
    pub fn supports(&self, spec: &AudioSpec) -> bool {
        self.capabilities.supports(spec)
    }

    /// Get a summary string.
    pub fn summary(&self) -> String {
        format!(
            "{} ({}) [{}] {:?} vol={:.0}%{}",
            self.name,
            self.id,
            self.device_type,
            self.state,
            self.effective_volume() * 100.0,
            if self.is_default { " (default)" } else { "" },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_output() -> AudioDevice {
        AudioDevice::new("hw:0", "Built-in Speakers", DeviceType::Output)
    }

    fn test_input() -> AudioDevice {
        AudioDevice::new("hw:1", "Built-in Microphone", DeviceType::Input)
    }

    #[test]
    fn device_creation() {
        let dev = test_output();
        assert_eq!(dev.id.as_str(), "hw:0");
        assert_eq!(dev.name, "Built-in Speakers");
        assert_eq!(dev.device_type, DeviceType::Output);
        assert_eq!(dev.state, DeviceState::Available);
    }

    #[test]
    fn device_type_capabilities() {
        assert!(DeviceType::Output.can_output());
        assert!(!DeviceType::Output.can_input());
        assert!(DeviceType::Input.can_input());
        assert!(!DeviceType::Input.can_output());
        assert!(DeviceType::Duplex.can_output());
        assert!(DeviceType::Duplex.can_input());
    }

    #[test]
    fn device_state_usable() {
        assert!(DeviceState::Available.is_usable());
        assert!(DeviceState::Active.is_usable());
        assert!(DeviceState::Paused.is_usable());
        assert!(!DeviceState::Error.is_usable());
        assert!(!DeviceState::Disconnected.is_usable());
    }

    #[test]
    fn device_state_streaming() {
        assert!(DeviceState::Active.is_streaming());
        assert!(!DeviceState::Available.is_streaming());
        assert!(!DeviceState::Paused.is_streaming());
    }

    #[test]
    fn volume_control() {
        let mut dev = test_output();
        assert_eq!(dev.volume(), Some(0.75));

        dev.set_volume(0.5);
        assert_eq!(dev.volume(), Some(0.5));

        // Clamping
        dev.set_volume(1.5);
        assert_eq!(dev.volume(), Some(1.0));

        dev.set_volume(-0.5);
        assert_eq!(dev.volume(), Some(0.0));
    }

    #[test]
    fn mute_control() {
        let mut dev = test_output();
        assert!(!dev.is_muted());

        dev.toggle_mute();
        assert!(dev.is_muted());
        assert_eq!(dev.volume(), Some(0.0));
        assert_eq!(dev.effective_volume(), 0.0);

        // Raw volume preserved
        assert_eq!(dev.raw_volume(), Some(0.75));

        dev.toggle_mute();
        assert!(!dev.is_muted());
        assert_eq!(dev.volume(), Some(0.75));
    }

    #[test]
    fn default_device() {
        let dev = AudioDevice::new("hw:0", "Speakers", DeviceType::Output).as_default();
        assert!(dev.is_default);
    }

    #[test]
    fn capabilities_support() {
        let caps = DeviceCapabilities::default();
        let cd = AudioSpec::cd_quality();
        assert!(caps.supports(&cd));

        let weird = AudioSpec::new(
            SampleFormat::S24,
            SampleRate::Hz96000,
            ChannelLayout::Surround71,
        );
        assert!(!caps.supports(&weird));
    }

    #[test]
    fn capabilities_preferred_spec() {
        let caps = DeviceCapabilities::default();
        let spec = caps.preferred_spec();
        assert!(spec.is_some());
    }

    #[test]
    fn device_supports() {
        let dev = test_output();
        assert!(dev.supports(&AudioSpec::cd_quality()));
    }

    #[test]
    fn device_summary() {
        let dev = test_output().as_default();
        let s = dev.summary();
        assert!(s.contains("Built-in Speakers"));
        assert!(s.contains("hw:0"));
        assert!(s.contains("default"));
    }

    #[test]
    fn device_id_display() {
        let id = DeviceId::new("hw:0");
        assert_eq!(id.to_string(), "hw:0");
    }

    #[test]
    fn device_type_display() {
        assert_eq!(DeviceType::Output.to_string(), "Output");
        assert_eq!(DeviceType::Input.to_string(), "Input");
        assert_eq!(DeviceType::Duplex.to_string(), "Duplex");
    }

    #[test]
    fn device_state_display() {
        assert_eq!(DeviceState::Available.to_string(), "Available");
        assert_eq!(DeviceState::Active.to_string(), "Active");
        assert_eq!(DeviceState::Disconnected.to_string(), "Disconnected");
    }

    #[test]
    fn input_device() {
        let dev = test_input();
        assert_eq!(dev.device_type, DeviceType::Input);
        assert!(dev.device_type.can_input());
        assert!(!dev.device_type.can_output());
    }

    #[test]
    fn set_muted_explicit() {
        let mut dev = test_output();
        dev.set_muted(true);
        assert!(dev.is_muted());
        dev.set_muted(false);
        assert!(!dev.is_muted());
    }
}
