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

pub mod buffer;
pub mod codec;
pub mod composites;
pub mod device;
pub mod error;
pub mod mixer;
pub mod prelude;
pub mod sample;
pub mod stream;

// Re-export main types
pub use buffer::AudioBuffer;
pub use codec::{CodecId, ConversionSpec, ResampleQuality};
pub use device::{AudioDevice, DeviceCapabilities, DeviceId, DeviceState, DeviceType};
pub use error::AudioError;
pub use mixer::{Mixer, MixerSource};
pub use sample::{AudioSpec, ChannelLayout, SampleFormat, SampleRate};
pub use stream::{AudioStream, StreamDirection, StreamId, StreamState};
