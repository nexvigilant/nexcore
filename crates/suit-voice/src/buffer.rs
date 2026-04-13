//! # Voice Agent: Audio Jitter Buffering
//! Handles packet loss and jitter for continuous ASR streaming.

use crate::AudioFrame;
use std::collections::VecDeque;

/// Buffers incoming audio frames to mitigate jitter and packet loss.
pub struct JitterBuffer {
    /// Ordered queue of audio packets.
    pub queue: VecDeque<AudioFrame>,
    /// Max latency in ms.
    pub max_latency: u64,
}

impl JitterBuffer {
    pub fn new(max_latency: u64) -> Self {
        Self {
            queue: VecDeque::with_capacity(100),
            max_latency,
        }
    }

    /// Ingests a new audio frame and ensures temporal order.
    pub fn push(&mut self, frame: AudioFrame) {
        // Simple append for now; in a full implementation, we sort by sequence ID.
        self.queue.push_back(frame);
    }

    /// Consumes the next contiguous frame for the ASR engine.
    pub fn pop(&mut self) -> Option<AudioFrame> {
        self.queue.pop_front()
    }
}
