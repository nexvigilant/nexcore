// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # PAL Prelude
//!
//! Convenience re-exports of the most-used types from `nexcore-pal`.
//!
//! ## Usage
//!
//! ```rust
//! use nexcore_pal::prelude::*;
//! ```
//!
//! This brings into scope all traits, common types, and the top-level
//! error type needed when writing code against the PAL.

// Traits
pub use crate::traits::{Display, Haptics, Input, Network, Platform, Power, Storage};

// Top-level error
pub use crate::error::PalError;

// Most-used types
pub use crate::types::{
    CrownEvent, DisplayShape, FormFactor, HapticPulse, InputEvent, KeyCode, KeyEvent, KeyState,
    Modifiers, PixelFormat, PointerButton, PointerEvent, PowerState, Resolution, TouchEvent,
    TouchPhase,
};

// Grounding prelude (feature-gated)
#[cfg(feature = "grounding")]
pub use crate::primitives::{GroundsTo, LexPrimitiva, PrimitiveComposition, Tier};
