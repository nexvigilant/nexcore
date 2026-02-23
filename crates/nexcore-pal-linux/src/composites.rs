// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Composite Types
//!
//! Compound types composed from `nexcore-pal-linux` subsystems.
//!
//! ## Linux Platform Composition
//!
//! `LinuxPlatform` is the primary composite: it binds all six Linux PAL
//! subsystem implementations together into a single concrete `Platform`
//! implementor.
//!
//! | Subsystem | Type | Backend |
//! |-----------|------|---------|
//! | Display   | `LinuxDisplay`  | DRM/KMS framebuffer |
//! | Input     | `LinuxInput`    | evdev |
//! | Network   | `LinuxNetwork`  | sockets + sysfs |
//! | Storage   | `LinuxStorage`  | std::fs |
//! | Haptics   | `LinuxHaptics`  | sysfs vibrator |
//! | Power     | `LinuxPower`    | sysfs battery |
//!
//! Additional composite types will be added as cross-subsystem aggregates
//! are identified (e.g., a `LinuxDeviceInfo` struct combining display +
//! power readings).

// Currently empty — composites will be added as the crate evolves.
