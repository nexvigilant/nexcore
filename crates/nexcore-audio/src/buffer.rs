// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Audio ring buffer — lock-free-style circular buffer for streaming.
//!
//! Tier: T2-C (σ + ∂ + N — sequenced, bounded, quantified)
//!
//! ## Design
//!
//! The ring buffer uses a single-producer single-consumer model:
//! - `write()` appends frames from the producer side
//! - `read()` extracts frames from the consumer side
//! - Fixed capacity prevents unbounded growth (∂ Boundary)
//! - Wrapping indices maintain O(1) operations (σ Sequence)

use crate::error::AudioError;
use crate::sample::AudioSpec;

/// Audio ring buffer for streaming data.
///
/// Tier: T2-C (σ Sequence + ∂ Boundary + N Quantity)
pub struct AudioBuffer {
    /// Raw sample data storage.
    data: Vec<u8>,
    /// Capacity in bytes.
    capacity: usize,
    /// Read position (wrapping).
    read_pos: usize,
    /// Write position (wrapping).
    write_pos: usize,
    /// Number of valid bytes in buffer.
    len: usize,
    /// Audio specification for this buffer.
    spec: AudioSpec,
    /// Overflow counter.
    overflows: u64,
    /// Underrun counter.
    underruns: u64,
}

impl AudioBuffer {
    /// Create a new audio buffer with capacity in frames.
    ///
    /// `frame_capacity` is the number of complete frames the buffer can hold.
    pub fn new(spec: AudioSpec, frame_capacity: usize) -> Self {
        // bytes_per_frame() is bounded; frame_capacity is caller-supplied.
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "capacity = bytes_per_frame * frame_capacity; caller is responsible for passing a valid capacity that does not overflow"
        )]
        let capacity = spec.bytes_per_frame() * frame_capacity;
        Self {
            data: vec![0u8; capacity],
            capacity,
            read_pos: 0,
            write_pos: 0,
            len: 0,
            spec,
            overflows: 0,
            underruns: 0,
        }
    }

    /// Audio specification.
    pub fn spec(&self) -> AudioSpec {
        self.spec
    }

    /// Capacity in bytes.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Capacity in frames.
    pub fn frame_capacity(&self) -> usize {
        let bpf = self.spec.bytes_per_frame();
        if bpf == 0 {
            return 0;
        }
        // Integer division — no overflow possible.
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "division by bpf which is guarded non-zero above"
        )]
        {
            self.capacity / bpf
        }
    }

    /// Current fill level in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether the buffer is full.
    pub fn is_full(&self) -> bool {
        self.len >= self.capacity
    }

    /// Available space in bytes.
    pub fn available(&self) -> usize {
        self.capacity.saturating_sub(self.len)
    }

    /// Current fill in frames.
    pub fn frames(&self) -> usize {
        let bpf = self.spec.bytes_per_frame();
        if bpf == 0 {
            return 0;
        }
        // Integer division — no overflow possible.
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "division by bpf which is guarded non-zero above"
        )]
        {
            self.len / bpf
        }
    }

    /// Available frames for writing.
    pub fn available_frames(&self) -> usize {
        let bpf = self.spec.bytes_per_frame();
        if bpf == 0 {
            return 0;
        }
        // Integer division — no overflow possible.
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "division by bpf which is guarded non-zero above"
        )]
        {
            self.available() / bpf
        }
    }

    /// Fill level as percentage [0.0, 1.0].
    pub fn fill_level(&self) -> f64 {
        if self.capacity == 0 {
            return 0.0;
        }
        // usize → f64: sufficient precision for fill-level percentage.
        #[allow(
            clippy::as_conversions,
            reason = "usize values cast to f64 for ratio computation; precision adequate for fill-level display"
        )]
        {
            self.len as f64 / self.capacity as f64
        }
    }

    /// Write audio data into the buffer.
    ///
    /// Returns the number of bytes actually written. If the buffer is full,
    /// records an overflow and returns 0.
    #[allow(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        reason = "ring buffer invariants guarantee all index arithmetic stays within [0, capacity): \
                  write_pos < capacity, first_chunk <= capacity - write_pos, \
                  second_chunk <= to_write <= space <= capacity, \
                  len <= capacity throughout"
    )]
    pub fn write(&mut self, data: &[u8]) -> usize {
        let space = self.available();
        if space == 0 {
            if !data.is_empty() {
                self.overflows += 1;
            }
            return 0;
        }

        let to_write = data.len().min(space);

        // Write in up to two segments (wrap-around)
        let first_chunk = (self.capacity - self.write_pos).min(to_write);
        self.data[self.write_pos..self.write_pos + first_chunk]
            .copy_from_slice(&data[..first_chunk]);

        let second_chunk = to_write - first_chunk;
        if second_chunk > 0 {
            self.data[..second_chunk].copy_from_slice(&data[first_chunk..to_write]);
        }

        self.write_pos = (self.write_pos + to_write) % self.capacity;
        self.len += to_write;

        to_write
    }

    /// Read audio data from the buffer.
    ///
    /// Returns the number of bytes actually read. If the buffer is empty,
    /// records an underrun and returns 0.
    #[allow(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        reason = "ring buffer invariants guarantee all index arithmetic stays within [0, capacity): \
                  read_pos < capacity, first_chunk <= capacity - read_pos, \
                  second_chunk <= to_read <= len <= capacity, \
                  len >= to_read so subtraction cannot underflow"
    )]
    pub fn read(&mut self, output: &mut [u8]) -> usize {
        if self.len == 0 {
            if !output.is_empty() {
                self.underruns += 1;
            }
            return 0;
        }

        let to_read = output.len().min(self.len);

        // Read in up to two segments (wrap-around)
        let first_chunk = (self.capacity - self.read_pos).min(to_read);
        output[..first_chunk]
            .copy_from_slice(&self.data[self.read_pos..self.read_pos + first_chunk]);

        let second_chunk = to_read - first_chunk;
        if second_chunk > 0 {
            output[first_chunk..to_read].copy_from_slice(&self.data[..second_chunk]);
        }

        self.read_pos = (self.read_pos + to_read) % self.capacity;
        self.len -= to_read;

        to_read
    }

    /// Discard all data in the buffer.
    pub fn clear(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
        self.len = 0;
    }

    /// Total overflow events.
    pub fn overflows(&self) -> u64 {
        self.overflows
    }

    /// Total underrun events.
    pub fn underruns(&self) -> u64 {
        self.underruns
    }

    /// Reset overflow/underrun counters.
    pub fn reset_counters(&mut self) {
        self.overflows = 0;
        self.underruns = 0;
    }

    /// Duration of audio currently in buffer (seconds).
    pub fn buffered_duration_secs(&self) -> f64 {
        self.spec.duration_secs(self.len)
    }

    /// Health check — returns error if overflow/underrun rate is high.
    pub fn health_check(&self) -> Result<(), AudioError> {
        if self.overflows > 100 {
            // overflows is u64; usize is at least 32 bits. For error reporting,
            // saturating cast preserves the intent (count that is "large enough to matter").
            #[allow(
                clippy::as_conversions,
                reason = "u64 → usize for error diagnostics; saturating_cast not available in const position; value is bounded by usage (>100 threshold already checked)"
            )]
            return Err(AudioError::BufferOverflow {
                samples_lost: self.overflows as usize,
            });
        }
        if self.underruns > 100 {
            #[allow(
                clippy::as_conversions,
                reason = "u64 → usize for error diagnostics; value is bounded by usage (>100 threshold already checked)"
            )]
            return Err(AudioError::BufferUnderrun {
                silence_inserted: self.underruns as usize,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::{ChannelLayout, SampleFormat, SampleRate};

    fn test_spec() -> AudioSpec {
        AudioSpec::new(
            SampleFormat::S16,
            SampleRate::Hz48000,
            ChannelLayout::Stereo,
        )
    }

    #[test]
    fn new_buffer_empty() {
        let buf = AudioBuffer::new(test_spec(), 1024);
        assert!(buf.is_empty());
        assert!(!buf.is_full());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.frames(), 0);
        assert_eq!(buf.frame_capacity(), 1024);
    }

    #[test]
    fn capacity_bytes() {
        let spec = test_spec(); // S16 stereo = 4 bytes/frame
        let buf = AudioBuffer::new(spec, 100);
        assert_eq!(buf.capacity(), 400); // 100 frames * 4 bytes
    }

    #[test]
    fn write_and_read() {
        let mut buf = AudioBuffer::new(test_spec(), 256);
        let data = vec![1u8; 100];
        let written = buf.write(&data);
        assert_eq!(written, 100);
        assert_eq!(buf.len(), 100);

        let mut output = vec![0u8; 50];
        let read = buf.read(&mut output);
        assert_eq!(read, 50);
        assert_eq!(buf.len(), 50);
        assert_eq!(output, vec![1u8; 50]);
    }

    #[test]
    fn write_wraps_around() {
        let mut buf = AudioBuffer::new(test_spec(), 64); // 256 bytes
        let data = vec![0xAA; 200];
        buf.write(&data);
        assert_eq!(buf.len(), 200);

        // Read 150, freeing space
        let mut out = vec![0u8; 150];
        buf.read(&mut out);
        assert_eq!(buf.len(), 50);

        // Write 100 more — should wrap
        let data2 = vec![0xBB; 100];
        let written = buf.write(&data2);
        assert_eq!(written, 100);
        assert_eq!(buf.len(), 150);
    }

    #[test]
    fn overflow_when_full() {
        let mut buf = AudioBuffer::new(test_spec(), 16); // 64 bytes
        let data = vec![0u8; 64];
        buf.write(&data);
        assert!(buf.is_full());

        // Try to write more
        let overflow_data = vec![0u8; 10];
        let written = buf.write(&overflow_data);
        assert_eq!(written, 0);
        assert_eq!(buf.overflows(), 1);
    }

    #[test]
    fn underrun_when_empty() {
        let mut buf = AudioBuffer::new(test_spec(), 16);
        let mut out = vec![0u8; 10];
        let read = buf.read(&mut out);
        assert_eq!(read, 0);
        assert_eq!(buf.underruns(), 1);
    }

    #[test]
    fn fill_level() {
        let mut buf = AudioBuffer::new(test_spec(), 100); // 400 bytes
        assert!((buf.fill_level() - 0.0).abs() < f64::EPSILON);

        buf.write(&vec![0u8; 200]);
        assert!((buf.fill_level() - 0.5).abs() < 0.01);

        buf.write(&vec![0u8; 200]);
        assert!((buf.fill_level() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clear_resets() {
        let mut buf = AudioBuffer::new(test_spec(), 100);
        buf.write(&vec![0u8; 200]);
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn available_space() {
        let mut buf = AudioBuffer::new(test_spec(), 100); // 400 bytes
        assert_eq!(buf.available(), 400);

        buf.write(&vec![0u8; 100]);
        assert_eq!(buf.available(), 300);
    }

    #[test]
    fn available_frames() {
        let mut buf = AudioBuffer::new(test_spec(), 100);
        assert_eq!(buf.available_frames(), 100);

        // Write 10 frames = 40 bytes
        buf.write(&vec![0u8; 40]);
        assert_eq!(buf.available_frames(), 90);
    }

    #[test]
    fn buffered_duration() {
        let spec = AudioSpec::cd_quality(); // 176400 bytes/sec
        let mut buf = AudioBuffer::new(spec, 44100); // 1 second of frames
        let one_sec = spec.bytes_per_second();
        buf.write(&vec![0u8; one_sec]);
        let dur = buf.buffered_duration_secs();
        assert!((dur - 1.0).abs() < 0.01);
    }

    #[test]
    fn reset_counters() {
        let mut buf = AudioBuffer::new(test_spec(), 4); // 16 bytes
        buf.write(&vec![0u8; 16]);
        buf.write(&vec![0u8; 1]); // overflow
        assert_eq!(buf.overflows(), 1);

        buf.reset_counters();
        assert_eq!(buf.overflows(), 0);
        assert_eq!(buf.underruns(), 0);
    }

    #[test]
    fn health_check_passes() {
        let buf = AudioBuffer::new(test_spec(), 100);
        assert!(buf.health_check().is_ok());
    }

    #[test]
    fn spec_preserved() {
        let spec = AudioSpec::dvd_quality();
        let buf = AudioBuffer::new(spec, 100);
        assert_eq!(buf.spec(), spec);
    }

    #[test]
    fn partial_read() {
        let mut buf = AudioBuffer::new(test_spec(), 100);
        buf.write(&vec![42u8; 10]);

        let mut out = vec![0u8; 20];
        let read = buf.read(&mut out);
        assert_eq!(read, 10);
        assert_eq!(&out[..10], &[42u8; 10]);
    }

    #[test]
    fn partial_write() {
        let mut buf = AudioBuffer::new(test_spec(), 4); // 16 bytes
        buf.write(&vec![0u8; 10]);

        // Only 6 bytes remain
        let written = buf.write(&vec![0u8; 20]);
        assert_eq!(written, 6);
    }

    #[test]
    fn empty_write_no_overflow() {
        let mut buf = AudioBuffer::new(test_spec(), 4);
        buf.write(&vec![0u8; 16]); // fill
        let written = buf.write(&[]); // empty write to full buffer
        assert_eq!(written, 0);
        assert_eq!(buf.overflows(), 0); // empty write shouldn't count
    }

    #[test]
    fn empty_read_no_underrun() {
        let mut buf = AudioBuffer::new(test_spec(), 4);
        let mut out = vec![];
        let read = buf.read(&mut out);
        assert_eq!(read, 0);
        assert_eq!(buf.underruns(), 0); // empty read shouldn't count
    }
}
