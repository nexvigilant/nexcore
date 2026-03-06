// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # NexCore PAL — Platform Abstraction Layer
//!
//! Hardware-agnostic trait definitions for NexCore OS.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────┐
//! │            NexCore OS / Apps                  │
//! ├──────────────────────────────────────────────┤
//! │   nexcore-pal (traits — this crate)          │ ← You are here
//! ├──────────────────────────────────────────────┤
//! │   nexcore-pal-linux │ pal-android │ pal-bare │ ← Platform impls
//! ├──────────────────────────────────────────────┤
//! │   Linux Kernel / Android NDK / Bare metal    │
//! └──────────────────────────────────────────────┘
//! ```
//!
//! ## Design Principles
//!
//! 1. **`no_std` core**: Trait definitions compile without `std` or `alloc`
//! 2. **Zero dependencies**: Core traits have no external deps
//! 3. **Static dispatch**: Generic `Platform` trait uses associated types,
//!    enabling `impl Trait` over `dyn Trait` at the OS level
//! 4. **Form factor aware**: `FormFactor` enum drives per-device behavior
//!
//! ## Primitive Grounding
//!
//! | Trait     | Primitives        | Chomsky Level |
//! |-----------|-------------------|---------------|
//! | Display   | μ + ∂ + ∃         | Type-2 (CFG)  |
//! | Input     | σ + ∃ + μ         | Type-2 (CFG)  |
//! | Network   | μ + ∂ + σ         | Type-2 (CFG)  |
//! | Storage   | π + μ + ∂         | Type-2 (CFG)  |
//! | Haptics   | ν + N + σ         | Type-2 (CFG)  |
//! | Power     | N + ∃ + ς         | Type-2 (CFG)  |
//! | Platform  | Σ (all above)     | Type-1 (CSG)  |

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod composites;
pub mod error;
pub mod prelude;
pub mod traits;
pub mod types;

#[cfg(feature = "grounding")]
pub mod grounding;
#[cfg(feature = "grounding")]
pub mod primitives;
#[cfg(feature = "grounding")]
pub mod transfer;

// Re-export commonly used items at crate root
pub use error::PalError;
pub use traits::{Display, Haptics, Input, Network, Platform, Power, Storage};
pub use types::{
    CrownEvent, DisplayShape, FormFactor, HapticPulse, InputEvent, KeyCode, KeyEvent, KeyState,
    Modifiers, PixelFormat, PointerButton, PointerEvent, PowerState, Resolution, TouchEvent,
    TouchPhase,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_basics() {
        let r = Resolution::new(450, 450);
        assert_eq!(r.pixel_count(), 202_500);
        assert!(r.is_square());
        assert!(!r.is_landscape());
    }

    #[test]
    fn resolution_landscape() {
        let r = Resolution::new(1920, 1080);
        assert!(r.is_landscape());
        assert!(!r.is_square());
    }

    #[test]
    fn pixel_format_sizes() {
        assert_eq!(PixelFormat::Rgba8.bytes_per_pixel(), 4);
        assert_eq!(PixelFormat::Bgra8.bytes_per_pixel(), 4);
        assert_eq!(PixelFormat::Rgb8.bytes_per_pixel(), 3);
        assert_eq!(PixelFormat::Rgb565.bytes_per_pixel(), 2);
    }

    #[test]
    fn form_factor_watch() {
        let ff = FormFactor::Watch;
        assert!(ff.touch_primary());
        let min = ff.min_resolution();
        assert_eq!(min.width, 360);
        assert_eq!(min.height, 360);
    }

    #[test]
    fn form_factor_desktop() {
        let ff = FormFactor::Desktop;
        assert!(!ff.touch_primary());
    }

    #[test]
    fn power_state_battery() {
        let ps = PowerState::Battery { percent: 42 };
        assert_eq!(ps.battery_pct(), Some(42));
        assert!(!ps.is_charging());
        assert!(!ps.is_critical());
    }

    #[test]
    fn power_state_critical() {
        let ps = PowerState::Battery { percent: 5 };
        assert!(ps.is_critical());
    }

    #[test]
    fn power_state_charging() {
        let ps = PowerState::Charging { percent: 80 };
        assert!(ps.is_charging());
        assert_eq!(ps.battery_pct(), Some(80));
    }

    #[test]
    fn power_state_full() {
        let ps = PowerState::Full;
        assert!(ps.is_charging());
        assert_eq!(ps.battery_pct(), Some(100));
    }

    #[test]
    fn power_state_ac() {
        let ps = PowerState::AcPower;
        assert!(!ps.is_charging());
        assert_eq!(ps.battery_pct(), None);
    }

    #[test]
    fn haptic_pulse_constructors() {
        let tap = HapticPulse::tap();
        assert_eq!(tap.duration_ms, 50);
        assert_eq!(tap.intensity, 200);

        let notif = HapticPulse::notification();
        assert_eq!(notif.duration_ms, 100);
        assert_eq!(notif.intensity, 255);
    }

    #[test]
    fn modifier_keys() {
        let mods = Modifiers::SHIFT.union(Modifiers::CTRL);
        assert!(mods.contains(Modifiers::SHIFT));
        assert!(mods.contains(Modifiers::CTRL));
        assert!(!mods.contains(Modifiers::ALT));
    }

    #[test]
    fn display_error_formatting() {
        let e = PalError::Display(error::DisplayError::NotFound);
        let s = format!("{e}");
        assert!(s.contains("display not found"));
    }

    #[test]
    fn touch_phases() {
        let _phases = [
            TouchPhase::Started,
            TouchPhase::Moved,
            TouchPhase::Ended,
            TouchPhase::Cancelled,
        ];
        // Verify all variants exist and are distinct
        assert_ne!(TouchPhase::Started, TouchPhase::Ended);
    }
}
