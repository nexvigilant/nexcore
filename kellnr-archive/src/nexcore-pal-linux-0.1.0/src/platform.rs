// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Linux platform — composite implementation of all PAL traits.
//!
//! Tier: T3 (Σ Sum — composition of all Linux subsystems)

use nexcore_pal::Platform;
use nexcore_pal::types::FormFactor;

use crate::display::LinuxDisplay;
use crate::haptics::LinuxHaptics;
use crate::input::LinuxInput;
use crate::network::LinuxNetwork;
use crate::power::LinuxPower;
use crate::storage::LinuxStorage;

/// Complete Linux platform implementation.
///
/// Tier: T3 (Linux OS platform — full subsystem composition)
///
/// This is the concrete type passed to `NexCoreOs<LinuxPlatform>`.
pub struct LinuxPlatform {
    /// Form factor of this device.
    form_factor: FormFactor,
    /// Display subsystem.
    display: LinuxDisplay,
    /// Input subsystem.
    input: LinuxInput,
    /// Network subsystem.
    network: LinuxNetwork,
    /// Storage subsystem.
    storage: LinuxStorage,
    /// Haptics subsystem.
    haptics: LinuxHaptics,
    /// Power subsystem.
    power: LinuxPower,
    /// Platform name string.
    name: String,
}

impl LinuxPlatform {
    /// Build a Linux platform for a specific form factor.
    pub fn new(form_factor: FormFactor, storage_root: &str) -> Self {
        let name = match form_factor {
            FormFactor::Watch => format!("linux-{}-watch", std::env::consts::ARCH),
            FormFactor::Phone => format!("linux-{}-phone", std::env::consts::ARCH),
            FormFactor::Desktop => format!("linux-{}-desktop", std::env::consts::ARCH),
        };

        let display = match form_factor {
            FormFactor::Watch => LinuxDisplay::new(
                nexcore_pal::Resolution::new(450, 450),
                nexcore_pal::DisplayShape::Circle,
            ),
            FormFactor::Phone => LinuxDisplay::new(
                nexcore_pal::Resolution::new(1080, 2400),
                nexcore_pal::DisplayShape::RoundedRect { corner_radius: 40 },
            ),
            FormFactor::Desktop => LinuxDisplay::probe().unwrap_or_else(|_| {
                LinuxDisplay::virtual_display(nexcore_pal::Resolution::new(1920, 1080))
            }),
        };

        Self {
            form_factor,
            display,
            input: LinuxInput::probe().unwrap_or_else(|_| LinuxInput::new()),
            network: LinuxNetwork::probe(),
            storage: LinuxStorage::new(storage_root),
            haptics: LinuxHaptics::probe(),
            power: LinuxPower::probe(),
            name,
        }
    }

    /// Create a virtual platform for testing at a custom storage root.
    ///
    /// Use this when tests need isolated storage (e.g., vault tests).
    pub fn virtual_platform_at(form_factor: FormFactor, storage_root: &str) -> Self {
        let resolution = form_factor.min_resolution();
        let shape = match form_factor {
            FormFactor::Watch => nexcore_pal::DisplayShape::Circle,
            FormFactor::Phone => nexcore_pal::DisplayShape::RoundedRect { corner_radius: 40 },
            FormFactor::Desktop => nexcore_pal::DisplayShape::Rectangle,
        };

        Self {
            form_factor,
            display: LinuxDisplay::new(resolution, shape),
            input: LinuxInput::new(),
            network: LinuxNetwork::virtual_network(true),
            storage: LinuxStorage::new(storage_root),
            haptics: LinuxHaptics::virtual_haptics(),
            power: LinuxPower::virtual_battery(100, false),
            name: format!("linux-{}-virtual", std::env::consts::ARCH),
        }
    }

    /// Create a virtual platform for testing (no hardware required).
    pub fn virtual_platform(form_factor: FormFactor) -> Self {
        let resolution = form_factor.min_resolution();
        let shape = match form_factor {
            FormFactor::Watch => nexcore_pal::DisplayShape::Circle,
            FormFactor::Phone => nexcore_pal::DisplayShape::RoundedRect { corner_radius: 40 },
            FormFactor::Desktop => nexcore_pal::DisplayShape::Rectangle,
        };

        Self {
            form_factor,
            display: LinuxDisplay::new(resolution, shape),
            input: LinuxInput::new(),
            network: LinuxNetwork::virtual_network(true),
            storage: LinuxStorage::new("/tmp/nexcore-os"),
            haptics: LinuxHaptics::virtual_haptics(),
            power: LinuxPower::virtual_battery(100, false),
            name: format!("linux-{}-virtual", std::env::consts::ARCH),
        }
    }
}

impl Platform for LinuxPlatform {
    type Display = LinuxDisplay;
    type Input = LinuxInput;
    type Network = LinuxNetwork;
    type Storage = LinuxStorage;
    type Haptics = LinuxHaptics;
    type Power = LinuxPower;

    fn form_factor(&self) -> FormFactor {
        self.form_factor
    }

    fn display(&self) -> &Self::Display {
        &self.display
    }

    fn display_mut(&mut self) -> &mut Self::Display {
        &mut self.display
    }

    fn input(&self) -> &Self::Input {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Self::Input {
        &mut self.input
    }

    fn network(&self) -> &Self::Network {
        &self.network
    }

    fn network_mut(&mut self) -> &mut Self::Network {
        &mut self.network
    }

    fn storage(&self) -> &Self::Storage {
        &self.storage
    }

    fn storage_mut(&mut self) -> &mut Self::Storage {
        &mut self.storage
    }

    fn haptics(&self) -> &Self::Haptics {
        &self.haptics
    }

    fn haptics_mut(&mut self) -> &mut Self::Haptics {
        &mut self.haptics
    }

    fn power(&self) -> &Self::Power {
        &self.power
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn data_dir(&self) -> &str {
        self.storage.root()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_pal::{Display, Power};

    #[test]
    fn virtual_watch_platform() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Watch);
        assert_eq!(platform.form_factor(), FormFactor::Watch);
        assert_eq!(platform.display().resolution().width, 360);
        assert_eq!(platform.display().resolution().height, 360);
        assert!(platform.name().contains("virtual"));
    }

    #[test]
    fn virtual_phone_platform() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Phone);
        assert_eq!(platform.form_factor(), FormFactor::Phone);
        assert!(platform.display().resolution().height > platform.display().resolution().width);
    }

    #[test]
    fn virtual_desktop_platform() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        assert_eq!(platform.form_factor(), FormFactor::Desktop);
        assert!(!platform.form_factor().touch_primary());
    }

    #[test]
    fn platform_name_contains_arch() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let arch = std::env::consts::ARCH;
        assert!(platform.name().contains(arch));
    }

    #[test]
    fn power_on_virtual() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Watch);
        assert_eq!(platform.power().battery_pct(), Some(100));
    }
}
