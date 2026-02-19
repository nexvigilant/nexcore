// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Linux power subsystem via sysfs battery interface.
//!
//! Tier: T3 (N Quantity + ∃ Existence + ς State — Linux-specific)
//!
//! Reads from `/sys/class/power_supply/*/` for battery info.

use nexcore_pal::Power;
use nexcore_pal::types::PowerState;
use std::fs;
use std::path::{Path, PathBuf};

/// Linux power subsystem reading from sysfs.
///
/// Tier: T3 (Linux-specific power implementation)
pub struct LinuxPower {
    /// Path to the power supply sysfs directory.
    supply_path: Option<PathBuf>,
    /// Cached state for when sysfs is unavailable (e.g., desktop AC).
    fallback_state: PowerState,
}

impl LinuxPower {
    /// Create a new power subsystem.
    pub fn new() -> Self {
        Self {
            supply_path: None,
            fallback_state: PowerState::Unknown,
        }
    }

    /// Probe sysfs for battery/power supply.
    pub fn probe() -> Self {
        let mut power = Self::new();

        let supply_dir = PathBuf::from("/sys/class/power_supply");
        if !supply_dir.exists() {
            power.fallback_state = PowerState::AcPower; // Desktop without battery
            return power;
        }

        if let Ok(entries) = fs::read_dir(&supply_dir) {
            for entry in entries.flatten() {
                let type_path = entry.path().join("type");
                if let Ok(ptype) = fs::read_to_string(&type_path) {
                    if ptype.trim() == "Battery" {
                        power.supply_path = Some(entry.path());
                        return power;
                    }
                }
            }
        }

        // No battery found — AC desktop
        power.fallback_state = PowerState::AcPower;
        power
    }

    /// Create a virtual power subsystem for testing.
    pub fn virtual_battery(percent: u8, charging: bool) -> Self {
        Self {
            supply_path: None,
            fallback_state: if charging {
                PowerState::Charging { percent }
            } else {
                PowerState::Battery { percent }
            },
        }
    }

    /// Read a sysfs file and parse as the given type.
    fn read_sysfs_value<T: core::str::FromStr>(path: &Path, attr: &str) -> Option<T> {
        let full_path = path.join(attr);
        let content = fs::read_to_string(&full_path).ok()?;
        content.trim().parse().ok()
    }

    /// Read battery status from sysfs.
    fn read_state_from_sysfs(path: &Path) -> PowerState {
        let capacity: u8 = Self::read_sysfs_value(path, "capacity").unwrap_or(0);

        let status = fs::read_to_string(path.join("status")).unwrap_or_default();

        match status.trim() {
            "Charging" => PowerState::Charging { percent: capacity },
            "Full" => PowerState::Full,
            "Discharging" | "Not charging" => PowerState::Battery { percent: capacity },
            _ => PowerState::Unknown,
        }
    }
}

impl Default for LinuxPower {
    fn default() -> Self {
        Self::new()
    }
}

impl Power for LinuxPower {
    fn state(&self) -> PowerState {
        self.supply_path
            .as_ref()
            .map_or(self.fallback_state, |path| {
                Self::read_state_from_sysfs(path)
            })
    }

    fn estimated_runtime_minutes(&self) -> Option<u32> {
        let path = self.supply_path.as_ref()?;

        // Try energy-based calculation first (µWh / µW = hours)
        let energy_now: u64 = Self::read_sysfs_value(path, "energy_now")?;
        let power_now: u64 = Self::read_sysfs_value(path, "power_now")?;

        if power_now == 0 {
            return None;
        }

        // energy_now is in µWh, power_now is in µW
        // hours = µWh / µW, minutes = hours * 60
        let minutes = (energy_now * 60) / power_now;
        Some(minutes as u32)
    }

    fn temperature_decicelsius(&self) -> Option<i32> {
        let path = self.supply_path.as_ref()?;
        // sysfs reports in tenths of degree Celsius
        Self::read_sysfs_value(path, "temp")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_battery_discharging() {
        let power = LinuxPower::virtual_battery(75, false);
        let state = power.state();
        assert_eq!(state, PowerState::Battery { percent: 75 });
        assert_eq!(power.battery_pct(), Some(75));
        assert!(!power.is_charging());
    }

    #[test]
    fn virtual_battery_charging() {
        let power = LinuxPower::virtual_battery(50, true);
        let state = power.state();
        assert_eq!(state, PowerState::Charging { percent: 50 });
        assert!(power.is_charging());
    }

    #[test]
    fn no_battery_desktop() {
        let mut power = LinuxPower::new();
        power.fallback_state = PowerState::AcPower;
        assert!(!power.is_charging());
        assert_eq!(power.battery_pct(), None);
    }

    #[test]
    fn estimated_runtime_no_battery() {
        let power = LinuxPower::new();
        assert!(power.estimated_runtime_minutes().is_none());
    }

    #[test]
    fn temperature_no_battery() {
        let power = LinuxPower::new();
        assert!(power.temperature_decicelsius().is_none());
    }
}
