// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! OS-level audio manager — orchestrates all audio subsystems.
//!
//! Tier: T3 (Σ + σ + ν + ς + ∂ + N)
//!
//! The `AudioManager` composes all nexcore-audio primitives into a single
//! OS-level subsystem that integrates with the boot sequence, security monitor,
//! and IPC event bus. It owns:
//!
//! - Device inventory (discovered audio hardware)
//! - Active streams (playback and capture)
//! - System mixer (combine all output sources)
//! - Volume control (master + per-device)

use nexcore_audio::{
    AudioDevice, AudioSpec, AudioStream, DeviceId, Mixer, MixerSource, StreamDirection, StreamId,
};
use serde::{Deserialize, Serialize};

/// Audio subsystem state.
///
/// Tier: T2-P (ς State — audio lifecycle)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioState {
    /// Not initialized yet.
    Uninitialized,
    /// Devices discovered, ready for streaming.
    Ready,
    /// Actively playing or capturing audio.
    Active,
    /// Audio subsystem disabled (e.g., silent mode).
    Disabled,
}

/// OS-level audio manager.
///
/// Tier: T3 (Σ Sum — composition of all audio subsystems)
pub struct AudioManager {
    /// Known audio devices.
    devices: Vec<AudioDevice>,
    /// Active audio streams.
    streams: Vec<AudioStream>,
    /// System output mixer.
    mixer: Mixer,
    /// Audio subsystem state.
    state: AudioState,
    /// Master system volume [0.0, 1.0].
    master_volume: f32,
    /// Whether the system is muted.
    muted: bool,
    /// Next stream ID counter.
    next_stream_id: u64,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioManager {
    /// Create a new audio manager (uninitialized).
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            streams: Vec::new(),
            mixer: Mixer::new(AudioSpec::float_stereo()),
            state: AudioState::Uninitialized,
            master_volume: 0.75,
            muted: false,
            next_stream_id: 1,
        }
    }

    /// Initialize the audio subsystem.
    ///
    /// Called during boot. Sets up default mixer configuration.
    pub fn initialize(&mut self) {
        self.mixer.set_master_volume(self.master_volume);
        self.state = AudioState::Ready;
    }

    /// Register an audio device from hardware discovery.
    pub fn register_device(&mut self, device: AudioDevice) {
        // Add to mixer as a source if it's an output device
        if device.device_type.can_output() {
            self.mixer.add_source(MixerSource::new(&device.name));
        }
        self.devices.push(device);
    }

    /// Get the current audio state.
    pub fn state(&self) -> AudioState {
        self.state
    }

    /// Whether the audio subsystem has active streams.
    pub fn is_active(&self) -> bool {
        self.state == AudioState::Active
    }

    /// Refresh the audio state based on stream activity.
    pub fn refresh_state(&mut self) {
        if self.state == AudioState::Disabled || self.state == AudioState::Uninitialized {
            return;
        }

        let any_active = self
            .streams
            .iter()
            .any(|s| s.state() == nexcore_audio::StreamState::Running);

        self.state = if any_active {
            AudioState::Active
        } else {
            AudioState::Ready
        };
    }

    /// Create a new playback stream on a device.
    pub fn create_playback_stream(
        &mut self,
        device_id: &DeviceId,
        spec: AudioSpec,
        buffer_frames: usize,
    ) -> Option<StreamId> {
        // Verify device exists and can output
        let device = self.devices.iter().find(|d| &d.id == device_id)?;
        if !device.device_type.can_output() {
            return None;
        }

        let stream_id = format!("playback-{}", self.next_stream_id);
        self.next_stream_id += 1;

        let stream = AudioStream::new(
            &stream_id,
            device_id.clone(),
            StreamDirection::Playback,
            spec,
            buffer_frames,
        );
        let id = stream.id.clone();
        self.streams.push(stream);
        Some(id)
    }

    /// Create a new capture stream on a device.
    pub fn create_capture_stream(
        &mut self,
        device_id: &DeviceId,
        spec: AudioSpec,
        buffer_frames: usize,
    ) -> Option<StreamId> {
        // Verify device exists and can input
        let device = self.devices.iter().find(|d| &d.id == device_id)?;
        if !device.device_type.can_input() {
            return None;
        }

        let stream_id = format!("capture-{}", self.next_stream_id);
        self.next_stream_id += 1;

        let stream = AudioStream::new(
            &stream_id,
            device_id.clone(),
            StreamDirection::Capture,
            spec,
            buffer_frames,
        );
        let id = stream.id.clone();
        self.streams.push(stream);
        Some(id)
    }

    /// Get a stream by ID.
    pub fn get_stream(&self, id: &StreamId) -> Option<&AudioStream> {
        self.streams.iter().find(|s| &s.id == id)
    }

    /// Get a mutable stream by ID.
    pub fn get_stream_mut(&mut self, id: &StreamId) -> Option<&mut AudioStream> {
        self.streams.iter_mut().find(|s| &s.id == id)
    }

    /// Get all registered devices.
    pub fn devices(&self) -> &[AudioDevice] {
        &self.devices
    }

    /// Get a device by ID.
    pub fn get_device(&self, id: &DeviceId) -> Option<&AudioDevice> {
        self.devices.iter().find(|d| &d.id == id)
    }

    /// Get a mutable device by ID.
    pub fn get_device_mut(&mut self, id: &DeviceId) -> Option<&mut AudioDevice> {
        self.devices.iter_mut().find(|d| &d.id == id)
    }

    /// Get the default output device.
    pub fn default_output(&self) -> Option<&AudioDevice> {
        self.devices
            .iter()
            .find(|d| d.is_default && d.device_type.can_output())
    }

    /// Get the default input device.
    pub fn default_input(&self) -> Option<&AudioDevice> {
        self.devices
            .iter()
            .find(|d| d.is_default && d.device_type.can_input())
    }

    /// Get the mixer.
    pub fn mixer(&self) -> &Mixer {
        &self.mixer
    }

    /// Get a mutable mixer.
    pub fn mixer_mut(&mut self) -> &mut Mixer {
        &mut self.mixer
    }

    /// Set master volume [0.0, 1.0].
    pub fn set_master_volume(&mut self, vol: f32) {
        self.master_volume = vol.clamp(0.0, 1.0);
        self.mixer.set_master_volume(self.master_volume);
    }

    /// Get master volume.
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Toggle system mute.
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        self.mixer.set_master_muted(self.muted);
    }

    /// Set system mute.
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        self.mixer.set_master_muted(muted);
    }

    /// Whether the system is muted.
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Enable or disable the audio subsystem.
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled && self.state == AudioState::Disabled {
            self.state = AudioState::Ready;
            self.refresh_state();
        } else if !enabled {
            self.state = AudioState::Disabled;
        }
    }

    /// Count of registered devices.
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Count of output devices.
    pub fn output_device_count(&self) -> usize {
        self.devices
            .iter()
            .filter(|d| d.device_type.can_output())
            .count()
    }

    /// Count of input devices.
    pub fn input_device_count(&self) -> usize {
        self.devices
            .iter()
            .filter(|d| d.device_type.can_input())
            .count()
    }

    /// Count of active streams.
    pub fn active_stream_count(&self) -> usize {
        self.streams
            .iter()
            .filter(|s| s.state() == nexcore_audio::StreamState::Running)
            .count()
    }

    /// Total stream count.
    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        format!(
            "Audio: {:?} ({} devices, {} streams, master={:.0}%{})",
            self.state,
            self.device_count(),
            self.active_stream_count(),
            self.master_volume * 100.0,
            if self.muted { " MUTED" } else { "" },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_audio::{AudioDevice, DeviceType};

    fn make_speakers() -> AudioDevice {
        AudioDevice::new("hw:0", "Built-in Speakers", DeviceType::Output).as_default()
    }

    fn make_mic() -> AudioDevice {
        AudioDevice::new("hw:1", "Built-in Microphone", DeviceType::Input).as_default()
    }

    fn make_headset() -> AudioDevice {
        AudioDevice::new("hw:2", "USB Headset", DeviceType::Duplex)
    }

    #[test]
    fn new_manager_uninitialized() {
        let am = AudioManager::new();
        assert_eq!(am.state(), AudioState::Uninitialized);
        assert!(!am.is_active());
        assert_eq!(am.device_count(), 0);
    }

    #[test]
    fn initialize_sets_ready() {
        let mut am = AudioManager::new();
        am.initialize();
        assert_eq!(am.state(), AudioState::Ready);
    }

    #[test]
    fn register_device() {
        let mut am = AudioManager::new();
        am.initialize();
        am.register_device(make_speakers());
        assert_eq!(am.device_count(), 1);
        assert_eq!(am.output_device_count(), 1);
        assert_eq!(am.input_device_count(), 0);
    }

    #[test]
    fn register_input_device() {
        let mut am = AudioManager::new();
        am.initialize();
        am.register_device(make_mic());
        assert_eq!(am.device_count(), 1);
        assert_eq!(am.output_device_count(), 0);
        assert_eq!(am.input_device_count(), 1);
    }

    #[test]
    fn register_duplex_device() {
        let mut am = AudioManager::new();
        am.initialize();
        am.register_device(make_headset());
        assert_eq!(am.output_device_count(), 1);
        assert_eq!(am.input_device_count(), 1);
    }

    #[test]
    fn default_devices() {
        let mut am = AudioManager::new();
        am.initialize();
        am.register_device(make_speakers());
        am.register_device(make_mic());

        assert!(am.default_output().is_some());
        assert!(am.default_input().is_some());
    }

    #[test]
    fn create_playback_stream() {
        let mut am = AudioManager::new();
        am.initialize();
        am.register_device(make_speakers());

        let stream_id =
            am.create_playback_stream(&DeviceId::new("hw:0"), AudioSpec::cd_quality(), 1024);
        assert!(stream_id.is_some());
        assert_eq!(am.stream_count(), 1);
    }

    #[test]
    fn create_capture_stream() {
        let mut am = AudioManager::new();
        am.initialize();
        am.register_device(make_mic());

        let stream_id =
            am.create_capture_stream(&DeviceId::new("hw:1"), AudioSpec::voice_quality(), 2048);
        assert!(stream_id.is_some());
        assert_eq!(am.stream_count(), 1);
    }

    #[test]
    fn cannot_playback_on_input_only() {
        let mut am = AudioManager::new();
        am.initialize();
        am.register_device(make_mic());

        let stream_id =
            am.create_playback_stream(&DeviceId::new("hw:1"), AudioSpec::cd_quality(), 1024);
        assert!(stream_id.is_none());
    }

    #[test]
    fn cannot_capture_on_output_only() {
        let mut am = AudioManager::new();
        am.initialize();
        am.register_device(make_speakers());

        let stream_id =
            am.create_capture_stream(&DeviceId::new("hw:0"), AudioSpec::voice_quality(), 2048);
        assert!(stream_id.is_none());
    }

    #[test]
    fn volume_control() {
        let mut am = AudioManager::new();
        am.initialize();
        assert_eq!(am.master_volume(), 0.75);

        am.set_master_volume(0.5);
        assert_eq!(am.master_volume(), 0.5);
        assert_eq!(am.mixer().master_volume(), 0.5);
    }

    #[test]
    fn mute_control() {
        let mut am = AudioManager::new();
        am.initialize();
        assert!(!am.is_muted());

        am.toggle_mute();
        assert!(am.is_muted());
        assert!(am.mixer().is_master_muted());

        am.toggle_mute();
        assert!(!am.is_muted());
    }

    #[test]
    fn disable_and_enable() {
        let mut am = AudioManager::new();
        am.initialize();
        am.register_device(make_speakers());

        am.set_enabled(false);
        assert_eq!(am.state(), AudioState::Disabled);

        am.set_enabled(true);
        assert_eq!(am.state(), AudioState::Ready);
    }

    #[test]
    fn get_device() {
        let mut am = AudioManager::new();
        am.initialize();
        am.register_device(make_speakers());
        assert!(am.get_device(&DeviceId::new("hw:0")).is_some());
        assert!(am.get_device(&DeviceId::new("nonexistent")).is_none());
    }

    #[test]
    fn mixer_accessible() {
        let mut am = AudioManager::new();
        am.initialize();
        am.register_device(make_speakers());
        // Speaker was added as a mixer source
        assert!(am.mixer().source_count() >= 1);
    }

    #[test]
    fn summary_format() {
        let mut am = AudioManager::new();
        am.initialize();
        let s = am.summary();
        assert!(s.contains("Audio"));
        assert!(s.contains("Ready"));
        assert!(s.contains("0 devices"));
    }

    #[test]
    fn multiple_devices() {
        let mut am = AudioManager::new();
        am.initialize();
        am.register_device(make_speakers());
        am.register_device(make_mic());
        am.register_device(make_headset());
        assert_eq!(am.device_count(), 3);
        // speakers + headset = 2 output
        assert_eq!(am.output_device_count(), 2);
        // mic + headset = 2 input
        assert_eq!(am.input_device_count(), 2);
    }

    #[test]
    fn stream_on_nonexistent_device() {
        let mut am = AudioManager::new();
        am.initialize();
        let result =
            am.create_playback_stream(&DeviceId::new("nonexistent"), AudioSpec::cd_quality(), 1024);
        assert!(result.is_none());
    }

    #[test]
    fn active_stream_count() {
        let am = AudioManager::new();
        assert_eq!(am.active_stream_count(), 0);
    }
}
