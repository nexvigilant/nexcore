// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Prelude — nexcore-audio
//!
//! Convenience re-exports for the most commonly used audio types.
//!
//! Import everything from this module with:
//!
//! ```rust
//! use nexcore_audio::prelude::*;
//! ```
//!
//! ## Included Types
//!
//! | Category | Types |
//! |----------|-------|
//! | Error | [`AudioError`] |
//! | Sample | [`AudioSpec`], [`ChannelLayout`], [`SampleFormat`], [`SampleRate`] |
//! | Buffer | [`AudioBuffer`] |
//! | Codec | [`CodecId`], [`ConversionSpec`], [`ResampleQuality`] |
//! | Device | [`AudioDevice`], [`DeviceCapabilities`], [`DeviceId`], [`DeviceState`], [`DeviceType`] |
//! | Stream | [`AudioStream`], [`StreamDirection`], [`StreamId`], [`StreamState`] |
//! | Mixer | [`Mixer`], [`MixerSource`] |

// ── Error ────────────────────────────────────────────────────────────────────

pub use crate::error::AudioError;

// ── Sample ───────────────────────────────────────────────────────────────────

pub use crate::sample::{AudioSpec, ChannelLayout, SampleFormat, SampleRate};

// ── Buffer ───────────────────────────────────────────────────────────────────

pub use crate::buffer::AudioBuffer;

// ── Codec ────────────────────────────────────────────────────────────────────

pub use crate::codec::{CodecId, ConversionSpec, ResampleQuality};

// ── Device ───────────────────────────────────────────────────────────────────

pub use crate::device::{AudioDevice, DeviceCapabilities, DeviceId, DeviceState, DeviceType};

// ── Stream ───────────────────────────────────────────────────────────────────

pub use crate::stream::{AudioStream, StreamDirection, StreamId, StreamState};

// ── Mixer ────────────────────────────────────────────────────────────────────

pub use crate::mixer::{Mixer, MixerSource};
