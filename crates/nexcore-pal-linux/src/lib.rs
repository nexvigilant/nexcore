// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # NexCore PAL Linux — Linux Platform Implementation
//!
//! Concrete implementations of `nexcore-pal` traits for Linux.
//!
//! ## Subsystems
//!
//! | Module   | Backend             | Sysfs Path                    |
//! |----------|---------------------|-------------------------------|
//! | Display  | DRM/KMS framebuffer | `/sys/class/drm/`, `/dev/fb0` |
//! | Input    | evdev               | `/dev/input/event*`           |
//! | Network  | sockets + sysfs     | `/sys/class/net/`             |
//! | Storage  | std::fs             | Rooted filesystem             |
//! | Haptics  | sysfs vibrator      | `/sys/class/leds/vibrator/`   |
//! | Power    | sysfs battery       | `/sys/class/power_supply/`    |
//!
//! ## Usage
//!
//! ```rust,no_run
//! use nexcore_pal::FormFactor;
//! use nexcore_pal_linux::LinuxPlatform;
//!
//! // Auto-probe hardware
//! let platform = LinuxPlatform::new(FormFactor::Desktop, "/var/nexcore");
//!
//! // Or use virtual platform for testing
//! let virtual_platform = LinuxPlatform::virtual_platform(FormFactor::Watch);
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod composites;
pub mod display;
pub mod haptics;
pub mod input;
pub mod network;
pub mod platform;
pub mod power;
pub mod prelude;
pub mod storage;

// Re-export the main type
pub use platform::LinuxPlatform;
