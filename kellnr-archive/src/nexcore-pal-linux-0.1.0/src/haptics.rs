// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Linux haptics implementation via sysfs vibrator or force feedback.
//!
//! Tier: T3 (ν Frequency + N Quantity — Linux-specific)

use nexcore_pal::Haptics;
use nexcore_pal::error::HapticsError;
use nexcore_pal::types::HapticPulse;
use std::fs;
use std::path::PathBuf;

/// Linux haptics backed by sysfs vibrator or input force feedback.
///
/// Tier: T3 (Linux-specific haptics implementation)
pub struct LinuxHaptics {
    /// Path to vibrator control (e.g., `/sys/class/leds/vibrator/`).
    vibrator_path: Option<PathBuf>,
    /// Whether haptics hardware is present.
    available: bool,
}

impl LinuxHaptics {
    /// Create haptics subsystem.
    pub fn new() -> Self {
        Self {
            vibrator_path: None,
            available: false,
        }
    }

    /// Probe for haptics hardware.
    pub fn probe() -> Self {
        let mut haptics = Self::new();

        // Try sysfs vibrator (common on Android/embedded Linux)
        let vibrator_paths = [
            PathBuf::from("/sys/class/leds/vibrator"),
            PathBuf::from("/sys/class/timed_output/vibrator"),
            PathBuf::from("/sys/devices/virtual/timed_output/vibrator"),
        ];

        for path in &vibrator_paths {
            if path.exists() {
                haptics.vibrator_path = Some(path.clone());
                haptics.available = true;
                return haptics;
            }
        }

        haptics
    }

    /// Create a virtual haptics for testing.
    pub fn virtual_haptics() -> Self {
        Self {
            vibrator_path: None,
            available: true,
        }
    }
}

impl Default for LinuxHaptics {
    fn default() -> Self {
        Self::new()
    }
}

impl Haptics for LinuxHaptics {
    fn vibrate(&mut self, pattern: &[HapticPulse]) -> Result<(), HapticsError> {
        if !self.available {
            return Err(HapticsError::NotAvailable);
        }

        if pattern.is_empty() {
            return Err(HapticsError::PatternInvalid);
        }

        // Write to vibrator sysfs if available
        if let Some(ref path) = self.vibrator_path {
            for pulse in pattern {
                let brightness = path.join("brightness");
                let _ = fs::write(&brightness, pulse.intensity.to_string());

                // Duration would normally use a timer; on real hardware,
                // the kernel driver handles timing
                let duration_path = path.join("duration");
                if duration_path.exists() {
                    let _ = fs::write(&duration_path, pulse.duration_ms.to_string());
                    let activate = path.join("activate");
                    if activate.exists() {
                        let _ = fs::write(&activate, "1");
                    }
                }
            }
        }

        // Virtual mode: accept the pattern without hardware write
        Ok(())
    }

    fn stop(&mut self) -> Result<(), HapticsError> {
        if let Some(ref path) = self.vibrator_path {
            let brightness = path.join("brightness");
            let _ = fs::write(&brightness, "0");
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        self.available
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_haptics_available() {
        let haptics = LinuxHaptics::virtual_haptics();
        assert!(haptics.is_available());
    }

    #[test]
    fn no_haptics_not_available() {
        let haptics = LinuxHaptics::new();
        assert!(!haptics.is_available());
    }

    #[test]
    fn vibrate_without_hardware() {
        let mut haptics = LinuxHaptics::new();
        let result = haptics.vibrate(&[HapticPulse::tap()]);
        assert!(result.is_err());
    }

    #[test]
    fn vibrate_virtual() {
        let mut haptics = LinuxHaptics::virtual_haptics();
        let result = haptics.vibrate(&[HapticPulse::tap(), HapticPulse::notification()]);
        assert!(result.is_ok());
    }

    #[test]
    fn vibrate_empty_pattern() {
        let mut haptics = LinuxHaptics::virtual_haptics();
        let result = haptics.vibrate(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn stop_virtual() {
        let mut haptics = LinuxHaptics::virtual_haptics();
        assert!(haptics.stop().is_ok());
    }
}
