// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # PAL Linux Prelude
//!
//! Convenience re-exports of the most-used types from `nexcore-pal-linux`.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use nexcore_pal_linux::prelude::*;
//! use nexcore_pal::FormFactor;
//!
//! let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
//! ```
//!
//! This brings into scope all concrete Linux subsystem types and the
//! top-level `LinuxPlatform` composite.

// Concrete subsystem implementations
pub use crate::display::LinuxDisplay;
pub use crate::haptics::LinuxHaptics;
pub use crate::input::LinuxInput;
pub use crate::network::LinuxNetwork;
pub use crate::platform::LinuxPlatform;
pub use crate::power::LinuxPower;
pub use crate::storage::LinuxStorage;

// Re-export the PAL trait set for convenience
pub use nexcore_pal::{Display, Haptics, Input, Network, Platform, Power, Storage};

// Re-export common PAL types needed alongside Linux types
pub use nexcore_pal::{
    DisplayShape, FormFactor, HapticPulse, InputEvent, PixelFormat, PowerState, Resolution,
};
