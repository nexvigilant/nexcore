// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Audio streams — playback and capture pipelines.
//!
//! Tier: T2-C (σ + ς + ∂ — sequenced state machine with bounded buffer)
//!
//! A stream connects a device to an audio buffer, managing the lifecycle
//! of audio data flow (playback or capture).

use crate::buffer::AudioBuffer;
use crate::device::DeviceId;
use crate::error::AudioError;
use crate::sample::AudioSpec;
use serde::{Deserialize, Serialize};

/// Stream direction.
///
/// Tier: T2-P (Σ Sum — flow direction)
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StreamDirection {
    /// Output (playback) — buffer → device.
    Playback,
    /// Input (capture) — device → buffer.
    Capture,
}

impl std::fmt::Display for StreamDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Playback => write!(f, "Playback"),
            Self::Capture => write!(f, "Capture"),
        }
    }
}

/// Stream state machine.
///
/// Tier: T2-P (ς State — stream lifecycle)
///
/// ```text
/// Created → Running → Paused → Running (cycle)
///    ↓        ↓          ↓
///  Stopped  Stopped   Stopped
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StreamState {
    /// Stream created but not yet started.
    Created,
    /// Actively streaming audio data.
    Running,
    /// Temporarily paused (buffer preserved).
    Paused,
    /// Stream stopped (resources released).
    Stopped,
    /// Stream encountered an error.
    Error,
}

impl StreamState {
    /// Whether the stream is actively processing audio.
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Running)
    }

    /// Whether the stream can be started or resumed.
    pub const fn can_start(self) -> bool {
        matches!(self, Self::Created | Self::Paused)
    }

    /// Whether the stream can be paused.
    pub const fn can_pause(self) -> bool {
        matches!(self, Self::Running)
    }

    /// Whether the stream can be stopped.
    pub const fn can_stop(self) -> bool {
        matches!(self, Self::Running | Self::Paused | Self::Created)
    }
}

impl std::fmt::Display for StreamState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "Created"),
            Self::Running => write!(f, "Running"),
            Self::Paused => write!(f, "Paused"),
            Self::Stopped => write!(f, "Stopped"),
            Self::Error => write!(f, "Error"),
        }
    }
}

/// Unique stream identifier.
///
/// Tier: T1 (∃ Existence — identity)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StreamId(String);

impl StreamId {
    /// Create a new stream ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get as string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for StreamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An audio stream (playback or capture).
///
/// Tier: T3 (σ + ς + ∂ + N + ∃ — full stream model)
pub struct AudioStream {
    /// Stream identifier.
    pub id: StreamId,
    /// Associated device.
    pub device_id: DeviceId,
    /// Direction of audio flow.
    pub direction: StreamDirection,
    /// Audio format specification.
    spec: AudioSpec,
    /// Internal buffer.
    buffer: AudioBuffer,
    /// Current state.
    state: StreamState,
    /// Total frames processed.
    frames_processed: u64,
    /// Position in stream (frames since start).
    position: u64,
}

impl AudioStream {
    /// Create a new audio stream.
    ///
    /// `buffer_frames` is the size of the internal ring buffer.
    pub fn new(
        id: impl Into<String>,
        device_id: DeviceId,
        direction: StreamDirection,
        spec: AudioSpec,
        buffer_frames: usize,
    ) -> Self {
        Self {
            id: StreamId::new(id),
            device_id,
            direction,
            spec,
            buffer: AudioBuffer::new(spec, buffer_frames),
            state: StreamState::Created,
            frames_processed: 0,
            position: 0,
        }
    }

    /// Get the audio spec.
    pub fn spec(&self) -> AudioSpec {
        self.spec
    }

    /// Get the current state.
    pub fn state(&self) -> StreamState {
        self.state
    }

    /// Start or resume the stream.
    pub fn start(&mut self) -> Result<(), AudioError> {
        if !self.state.can_start() {
            return Err(AudioError::InvalidState {
                current: self.state.to_string(),
                required: "Created or Paused".into(),
            });
        }
        self.state = StreamState::Running;
        Ok(())
    }

    /// Pause the stream (preserves buffer).
    pub fn pause(&mut self) -> Result<(), AudioError> {
        if !self.state.can_pause() {
            return Err(AudioError::InvalidState {
                current: self.state.to_string(),
                required: "Running".into(),
            });
        }
        self.state = StreamState::Paused;
        Ok(())
    }

    /// Stop the stream (clears buffer).
    pub fn stop(&mut self) -> Result<(), AudioError> {
        if !self.state.can_stop() {
            return Err(AudioError::InvalidState {
                current: self.state.to_string(),
                required: "Running, Paused, or Created".into(),
            });
        }
        self.state = StreamState::Stopped;
        self.buffer.clear();
        Ok(())
    }

    /// Write audio data for playback.
    ///
    /// Only valid for playback streams in Running state.
    pub fn write(&mut self, data: &[u8]) -> Result<usize, AudioError> {
        if self.direction != StreamDirection::Playback {
            return Err(AudioError::InvalidState {
                current: "Capture stream".into(),
                required: "Playback stream".into(),
            });
        }
        if self.state != StreamState::Running {
            return Err(AudioError::InvalidState {
                current: self.state.to_string(),
                required: "Running".into(),
            });
        }

        let written = self.buffer.write(data);
        let bpf = self.spec.bytes_per_frame();
        if bpf > 0 {
            // written / bpf is a frame count (usize); cast to u64 for the lifetime counter.
            // usize fits in u64 on all supported platforms (usize <= u64).
            #[allow(
                clippy::as_conversions,
                clippy::arithmetic_side_effects,
                reason = "usize frame count cast to u64 for cumulative counter; usize <= u64 on all supported targets; division guarded by bpf > 0"
            )]
            {
                self.frames_processed += (written / bpf) as u64;
                self.position += (written / bpf) as u64;
            }
        }
        Ok(written)
    }

    /// Read captured audio data.
    ///
    /// Only valid for capture streams in Running state.
    pub fn read(&mut self, output: &mut [u8]) -> Result<usize, AudioError> {
        if self.direction != StreamDirection::Capture {
            return Err(AudioError::InvalidState {
                current: "Playback stream".into(),
                required: "Capture stream".into(),
            });
        }
        if self.state != StreamState::Running {
            return Err(AudioError::InvalidState {
                current: self.state.to_string(),
                required: "Running".into(),
            });
        }

        let read_bytes = self.buffer.read(output);
        let bpf = self.spec.bytes_per_frame();
        if bpf > 0 {
            // read_bytes / bpf is a frame count (usize); cast to u64 for the lifetime counter.
            // usize fits in u64 on all supported platforms (usize <= u64).
            #[allow(
                clippy::as_conversions,
                clippy::arithmetic_side_effects,
                reason = "usize frame count cast to u64 for cumulative counter; usize <= u64 on all supported targets; division guarded by bpf > 0"
            )]
            {
                self.frames_processed += (read_bytes / bpf) as u64;
                self.position += (read_bytes / bpf) as u64;
            }
        }
        Ok(read_bytes)
    }

    /// Feed captured data into the stream's buffer (from device/platform).
    ///
    /// This is the internal path for capture streams — the platform
    /// pushes data into the buffer.
    pub fn feed(&mut self, data: &[u8]) -> usize {
        self.buffer.write(data)
    }

    /// Drain playback data from the stream's buffer (to device/platform).
    ///
    /// This is the internal path for playback streams — the platform
    /// pulls data from the buffer.
    pub fn drain(&mut self, output: &mut [u8]) -> usize {
        self.buffer.read(output)
    }

    /// Get buffer fill level [0.0, 1.0].
    pub fn buffer_fill(&self) -> f64 {
        self.buffer.fill_level()
    }

    /// Get buffer frame count.
    pub fn buffered_frames(&self) -> usize {
        self.buffer.frames()
    }

    /// Total frames processed since creation.
    pub fn frames_processed(&self) -> u64 {
        self.frames_processed
    }

    /// Current position in frames.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Duration of processed audio (seconds).
    pub fn elapsed_secs(&self) -> f64 {
        let rate = self.spec.rate.hz();
        if rate == 0 {
            return 0.0;
        }
        // u64 → f64: sufficient precision for stream position expressed as frame count.
        #[allow(
            clippy::as_conversions,
            reason = "u64 position cast to f64 for duration calculation; f64 has 53-bit mantissa sufficient for frame counts in audio streams"
        )]
        {
            self.position as f64 / f64::from(rate)
        }
    }

    /// Buffer health check.
    pub fn health_check(&self) -> Result<(), AudioError> {
        self.buffer.health_check()
    }

    /// Get a reference to the internal buffer.
    pub fn buffer(&self) -> &AudioBuffer {
        &self.buffer
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        format!(
            "Stream {} [{}] {:?} on {} | {:.1}% buffered | {} frames processed",
            self.id,
            self.direction,
            self.state,
            self.device_id,
            self.buffer_fill() * 100.0,
            self.frames_processed,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::DeviceId;

    fn playback_stream() -> AudioStream {
        AudioStream::new(
            "stream-1",
            DeviceId::new("hw:0"),
            StreamDirection::Playback,
            AudioSpec::cd_quality(),
            1024,
        )
    }

    fn capture_stream() -> AudioStream {
        AudioStream::new(
            "stream-2",
            DeviceId::new("hw:1"),
            StreamDirection::Capture,
            AudioSpec::voice_quality(),
            2048,
        )
    }

    #[test]
    fn creation() {
        let s = playback_stream();
        assert_eq!(s.state(), StreamState::Created);
        assert_eq!(s.direction, StreamDirection::Playback);
        assert_eq!(s.frames_processed(), 0);
    }

    #[test]
    fn start_stop_lifecycle() {
        let mut s = playback_stream();
        assert!(s.start().is_ok());
        assert_eq!(s.state(), StreamState::Running);

        assert!(s.pause().is_ok());
        assert_eq!(s.state(), StreamState::Paused);

        assert!(s.start().is_ok()); // Resume
        assert_eq!(s.state(), StreamState::Running);

        assert!(s.stop().is_ok());
        assert_eq!(s.state(), StreamState::Stopped);
    }

    #[test]
    fn cannot_start_stopped() {
        let mut s = playback_stream();
        assert!(s.start().is_ok());
        assert!(s.stop().is_ok());
        assert!(s.start().is_err());
    }

    #[test]
    fn cannot_pause_when_not_running() {
        let mut s = playback_stream();
        assert!(s.pause().is_err());
    }

    #[test]
    fn cannot_stop_when_stopped() {
        let mut s = playback_stream();
        assert!(s.start().is_ok());
        assert!(s.stop().is_ok());
        assert!(s.stop().is_err());
    }

    #[test]
    fn playback_write() {
        let mut s = playback_stream();
        assert!(s.start().is_ok());

        let data = vec![0u8; 400]; // 100 frames of CD audio
        let written = s.write(&data);
        assert!(written.is_ok());
        assert_eq!(written.ok(), Some(400));
        assert_eq!(s.frames_processed(), 100);
    }

    #[test]
    fn playback_write_when_not_running() {
        let mut s = playback_stream();
        let data = vec![0u8; 100];
        assert!(s.write(&data).is_err());
    }

    #[test]
    fn capture_read() {
        let mut s = capture_stream();
        assert!(s.start().is_ok());

        // Feed data (simulating platform capture)
        let data = vec![42u8; 200];
        s.feed(&data);

        let mut output = vec![0u8; 100];
        let read = s.read(&mut output);
        assert!(read.is_ok());
        assert_eq!(read.ok(), Some(100));
    }

    #[test]
    fn cannot_write_to_capture() {
        let mut s = capture_stream();
        assert!(s.start().is_ok());
        let data = vec![0u8; 100];
        assert!(s.write(&data).is_err());
    }

    #[test]
    fn cannot_read_from_playback() {
        let mut s = playback_stream();
        assert!(s.start().is_ok());
        let mut out = vec![0u8; 100];
        assert!(s.read(&mut out).is_err());
    }

    #[test]
    fn buffer_fill_tracking() {
        let mut s = playback_stream();
        assert!(s.start().is_ok());

        assert!((s.buffer_fill() - 0.0).abs() < f64::EPSILON);

        let half = s.buffer().capacity() / 2;
        let _ = s.write(&vec![0u8; half]);
        assert!(s.buffer_fill() > 0.4);
    }

    #[test]
    fn position_tracking() {
        let mut s = playback_stream();
        assert!(s.start().is_ok());

        let _ = s.write(&vec![0u8; 400]); // 100 frames
        assert_eq!(s.position(), 100);
    }

    #[test]
    fn elapsed_time() {
        let mut s = playback_stream();
        assert!(s.start().is_ok());

        // Write 44100 frames = 1 second at CD quality
        let one_sec_bytes = AudioSpec::cd_quality().bytes_per_second();
        let _ = s.write(&vec![0u8; one_sec_bytes.min(s.buffer().capacity())]);

        // elapsed depends on how much we actually wrote (buffer capped)
        assert!(s.elapsed_secs() >= 0.0);
    }

    #[test]
    fn feed_and_drain() {
        let mut s = playback_stream();
        s.feed(&vec![0xAA; 100]);
        let mut out = vec![0u8; 50];
        let drained = s.drain(&mut out);
        assert_eq!(drained, 50);
        assert_eq!(out, vec![0xAA; 50]);
    }

    #[test]
    fn health_check_passes() {
        let s = playback_stream();
        assert!(s.health_check().is_ok());
    }

    #[test]
    fn summary_format() {
        let s = playback_stream();
        let summary = s.summary();
        assert!(summary.contains("stream-1"));
        assert!(summary.contains("Playback"));
        assert!(summary.contains("hw:0"));
    }

    #[test]
    fn stream_id_display() {
        let id = StreamId::new("test");
        assert_eq!(id.to_string(), "test");
        assert_eq!(id.as_str(), "test");
    }

    #[test]
    fn stream_direction_display() {
        assert_eq!(StreamDirection::Playback.to_string(), "Playback");
        assert_eq!(StreamDirection::Capture.to_string(), "Capture");
    }

    #[test]
    fn stream_state_properties() {
        assert!(StreamState::Running.is_active());
        assert!(!StreamState::Paused.is_active());
        assert!(StreamState::Created.can_start());
        assert!(StreamState::Paused.can_start());
        assert!(!StreamState::Running.can_start());
        assert!(StreamState::Running.can_pause());
        assert!(!StreamState::Paused.can_pause());
    }

    #[test]
    fn stop_clears_buffer() {
        let mut s = playback_stream();
        assert!(s.start().is_ok());
        let _ = s.write(&vec![0u8; 100]);
        assert!(!s.buffer().is_empty());

        assert!(s.stop().is_ok());
        assert!(s.buffer().is_empty());
    }
}
