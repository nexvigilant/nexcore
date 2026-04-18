#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # NexCore Audio — OS Audio Subsystem
//!
//! Pure-Rust audio primitives for the NexCore operating system. Provides
//! sample formats, device abstraction, streaming, mixing, and codec support.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │            Application Layer              │
//! │  play_sound() │ record() │ set_volume()  │
//! ├─────────────────────────────────────────┤
//! │              Mixer (Σ)                   │
//! │  Source combination │ Volume │ Pan │ Solo │
//! ├─────────────────────────────────────────┤
//! │           Stream Engine (σ)              │
//! │  Playback/Capture │ Ring Buffer │ State  │
//! ├─────────────────────────────────────────┤
//! │           Codec Layer (μ)                │
//! │  S16↔F32 │ Resample │ Channel Remap     │
//! ├─────────────────────────────────────────┤
//! │          Device Layer (∃)                │
//! │  Discovery │ Capabilities │ Hot-plug     │
//! ├─────────────────────────────────────────┤
//! │          Platform (PAL)                  │
//! │  ALSA │ PulseAudio │ PipeWire │ CoreAudio│
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Primitive Grounding
//!
//! | Component | Primitives | Role |
//! |-----------|-----------|------|
//! | Sample types | N + ν | Numeric audio representation |
//! | Ring buffer | σ + ∂ + N | Bounded sequential data |
//! | Device model | ∃ + ς + Σ | Device existence & state |
//! | Streams | σ + ς + ∂ | Stateful bounded flow |
//! | Mixer | Σ + N + ∂ | Source combination with clipping |
//! | Codecs | μ + Σ + N | Format mapping between variants |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod aec;
pub mod buffer;
pub mod codec;
pub mod composites;
pub mod device;
pub mod disfluency;
pub mod error;
pub mod lang;
pub mod mixer;
pub mod noise;
pub mod pipeline;
pub mod prelude;
pub mod sample;
pub mod stream;
pub mod stt;
pub mod vad;

// Re-export main types
pub use aec::{AecConfig, AecResult, EchoCanceller};
pub use buffer::AudioBuffer;
pub use codec::{CodecId, ConversionSpec, ResampleQuality};
pub use device::{AudioDevice, DeviceCapabilities, DeviceId, DeviceState, DeviceType};
pub use disfluency::{DisfluencyConfig, FilterResult};
pub use error::AudioError;
pub use lang::{LangResult, Language};
pub use mixer::{Mixer, MixerSource};
pub use noise::{GateState, NoiseGate, NoiseGateConfig, NoiseGateResult};
pub use pipeline::{
    AsrPipeline, PipelineConfig, PipelineEvent, PipelineState, PostProcessResult, Utterance,
};
pub use sample::{AudioSpec, ChannelLayout, SampleFormat, SampleRate};
pub use stream::{AudioStream, StreamDirection, StreamId, StreamState};
pub use stt::{Segment, SttConfig, SttError, Transcript};
pub use vad::{VadConfig, VadResult, VadState, VoiceDetector};
