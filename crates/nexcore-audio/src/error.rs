// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Audio subsystem error types.
//!
//! Tier: T2-P (Σ Sum — error variant union)

use serde::{Deserialize, Serialize};

/// Audio subsystem errors.
///
/// Tier: T2-P (Σ Sum — all audio failure modes)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioError {
    /// Device not found by ID.
    DeviceNotFound(String),
    /// Device is already in use.
    DeviceInUse(String),
    /// Unsupported sample format.
    UnsupportedFormat {
        /// What was requested.
        requested: String,
        /// What the device supports.
        supported: Vec<String>,
    },
    /// Unsupported sample rate.
    UnsupportedSampleRate {
        /// Requested rate in Hz.
        requested: u32,
        /// Supported rates.
        supported: Vec<u32>,
    },
    /// Buffer overflow — producer is faster than consumer.
    BufferOverflow {
        /// Samples lost.
        samples_lost: usize,
    },
    /// Buffer underrun — consumer is faster than producer.
    BufferUnderrun {
        /// Samples of silence inserted.
        silence_inserted: usize,
    },
    /// Stream is not in correct state for this operation.
    InvalidState {
        /// Current state.
        current: String,
        /// Required state.
        required: String,
    },
    /// Volume out of range [0.0, 1.0].
    VolumeOutOfRange(String),
    /// Codec error.
    CodecError(String),
    /// Platform/driver error.
    PlatformError(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceNotFound(id) => write!(f, "audio device not found: {id}"),
            Self::DeviceInUse(id) => write!(f, "audio device in use: {id}"),
            Self::UnsupportedFormat {
                requested,
                supported,
            } => {
                write!(
                    f,
                    "unsupported format {requested}, supported: {supported:?}"
                )
            }
            Self::UnsupportedSampleRate {
                requested,
                supported,
            } => {
                write!(
                    f,
                    "unsupported sample rate {requested}Hz, supported: {supported:?}"
                )
            }
            Self::BufferOverflow { samples_lost } => {
                write!(f, "buffer overflow: {samples_lost} samples lost")
            }
            Self::BufferUnderrun { silence_inserted } => {
                write!(f, "buffer underrun: {silence_inserted} samples of silence")
            }
            Self::InvalidState { current, required } => {
                write!(f, "invalid state: {current}, required: {required}")
            }
            Self::VolumeOutOfRange(v) => write!(f, "volume out of range: {v}"),
            Self::CodecError(msg) => write!(f, "codec error: {msg}"),
            Self::PlatformError(msg) => write!(f, "platform error: {msg}"),
        }
    }
}

impl std::error::Error for AudioError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_not_found_display() {
        let e = AudioError::DeviceNotFound("hw:0".into());
        assert!(e.to_string().contains("hw:0"));
    }

    #[test]
    fn buffer_overflow_display() {
        let e = AudioError::BufferOverflow { samples_lost: 42 };
        assert!(e.to_string().contains("42"));
    }

    #[test]
    fn unsupported_format_display() {
        let e = AudioError::UnsupportedFormat {
            requested: "F64".into(),
            supported: vec!["S16".into(), "F32".into()],
        };
        let s = e.to_string();
        assert!(s.contains("F64"));
        assert!(s.contains("S16"));
    }

    #[test]
    fn all_variants_display() {
        let variants: Vec<AudioError> = vec![
            AudioError::DeviceNotFound("x".into()),
            AudioError::DeviceInUse("x".into()),
            AudioError::UnsupportedFormat {
                requested: "x".into(),
                supported: vec![],
            },
            AudioError::UnsupportedSampleRate {
                requested: 96000,
                supported: vec![44100, 48000],
            },
            AudioError::BufferOverflow { samples_lost: 0 },
            AudioError::BufferUnderrun {
                silence_inserted: 0,
            },
            AudioError::InvalidState {
                current: "a".into(),
                required: "b".into(),
            },
            AudioError::VolumeOutOfRange("1.5".into()),
            AudioError::CodecError("bad".into()),
            AudioError::PlatformError("driver".into()),
        ];
        for v in &variants {
            assert!(!v.to_string().is_empty());
        }
        assert_eq!(variants.len(), 10);
    }

    #[test]
    fn error_trait_impl() {
        let e = AudioError::CodecError("test".into());
        let _: &dyn std::error::Error = &e;
    }

    #[test]
    fn clone_and_eq() {
        let a = AudioError::DeviceNotFound("hw:0".into());
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_format() {
        let e = AudioError::BufferUnderrun {
            silence_inserted: 10,
        };
        let debug = format!("{e:?}");
        assert!(debug.contains("BufferUnderrun"));
    }
}
