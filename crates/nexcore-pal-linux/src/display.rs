// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Linux display implementation via DRM/KMS framebuffer.
//!
//! Tier: T3 (μ Mapping + ∂ Boundary + ∃ Existence — Linux-specific)
//!
//! Phase 1: Framebuffer file interface (`/dev/fb0` or virtual).
//! Phase 2: DRM/KMS mode setting for GPU-accelerated output.

use nexcore_pal::Display;
use nexcore_pal::error::DisplayError;
use nexcore_pal::types::{DisplayShape, PixelFormat, Resolution};
use std::fs;
use std::path::PathBuf;

/// Linux display backed by sysfs framebuffer or DRM.
///
/// Tier: T3 (Linux-specific display implementation)
pub struct LinuxDisplay {
    /// Current resolution.
    resolution: Resolution,
    /// Display shape.
    shape: DisplayShape,
    /// Pixel format.
    pixel_format: PixelFormat,
    /// Refresh rate in Hz.
    refresh_rate: u32,
    /// Whether display is on.
    is_on: bool,
    /// Brightness level (0-255).
    brightness: u8,
    /// Framebuffer device path (e.g., `/dev/fb0`).
    fb_path: Option<PathBuf>,
    /// Backlight sysfs path (e.g., `/sys/class/backlight/...`).
    backlight_path: Option<PathBuf>,
}

impl LinuxDisplay {
    /// Create a new Linux display with given parameters.
    pub fn new(resolution: Resolution, shape: DisplayShape) -> Self {
        Self {
            resolution,
            shape,
            pixel_format: PixelFormat::Rgba8,
            refresh_rate: 60,
            is_on: true,
            brightness: 255,
            fb_path: None,
            backlight_path: None,
        }
    }

    /// Create from DRM/sysfs probing.
    pub fn probe() -> Result<Self, DisplayError> {
        // Probe /sys/class/drm/ for connected displays
        let drm_path = PathBuf::from("/sys/class/drm");
        if !drm_path.exists() {
            return Err(DisplayError::NotFound);
        }

        // Default to a reasonable resolution — actual DRM probing
        // would read EDID and mode info from sysfs
        let resolution =
            Self::read_resolution_from_sysfs().unwrap_or_else(|| Resolution::new(1920, 1080));

        let shape = if resolution.is_square() {
            DisplayShape::Circle // Assume circular for square (watch)
        } else {
            DisplayShape::Rectangle
        };

        let mut display = Self::new(resolution, shape);

        // Try to find framebuffer device
        let fb0 = PathBuf::from("/dev/fb0");
        if fb0.exists() {
            display.fb_path = Some(fb0);
        }

        // Try to find backlight control
        display.backlight_path = Self::find_backlight_path();

        Ok(display)
    }

    /// Read resolution from sysfs DRM modes.
    fn read_resolution_from_sysfs() -> Option<Resolution> {
        // Try reading /sys/class/drm/card0-*/modes
        let drm_dir = fs::read_dir("/sys/class/drm").ok()?;
        for entry in drm_dir.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("card") && name_str.contains('-') {
                let modes_path = entry.path().join("modes");
                if let Ok(modes) = fs::read_to_string(&modes_path) {
                    // First line is the preferred mode: "1920x1080"
                    if let Some(first_mode) = modes.lines().next() {
                        return Self::parse_mode(first_mode);
                    }
                }
            }
        }
        None
    }

    /// Parse a mode string like "1920x1080" into Resolution.
    fn parse_mode(mode: &str) -> Option<Resolution> {
        let parts: Vec<&str> = mode.split('x').collect();
        if parts.len() == 2 {
            let width = parts[0].parse().ok()?;
            let height = parts[1].parse().ok()?;
            return Some(Resolution::new(width, height));
        }
        None
    }

    /// Find the backlight sysfs path.
    fn find_backlight_path() -> Option<PathBuf> {
        let backlight_dir = PathBuf::from("/sys/class/backlight");
        if !backlight_dir.exists() {
            return None;
        }
        let entries = fs::read_dir(&backlight_dir).ok()?;
        for entry in entries.flatten() {
            let brightness_path = entry.path().join("brightness");
            if brightness_path.exists() {
                return Some(entry.path());
            }
        }
        None
    }

    /// Create a virtual display for testing (no hardware required).
    pub fn virtual_display(resolution: Resolution) -> Self {
        Self::new(resolution, DisplayShape::Rectangle)
    }
}

impl Display for LinuxDisplay {
    fn resolution(&self) -> Resolution {
        self.resolution
    }

    fn shape(&self) -> DisplayShape {
        self.shape
    }

    fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }

    fn refresh_rate(&self) -> u32 {
        self.refresh_rate
    }

    fn present(&mut self, framebuffer: &[u8]) -> Result<(), DisplayError> {
        let expected_size =
            self.resolution.pixel_count() as usize * self.pixel_format.bytes_per_pixel() as usize;

        if framebuffer.len() != expected_size {
            return Err(DisplayError::PresentFailed);
        }

        // Write to framebuffer device if available
        if let Some(ref fb_path) = self.fb_path {
            fs::write(fb_path, framebuffer).map_err(|_| DisplayError::PresentFailed)?;
        }

        // Virtual mode: framebuffer is accepted but not written to hardware
        Ok(())
    }

    fn set_brightness(&mut self, level: u8) -> Result<(), DisplayError> {
        self.brightness = level;

        if let Some(ref backlight_path) = self.backlight_path {
            let max_brightness_path = backlight_path.join("max_brightness");
            let brightness_path = backlight_path.join("brightness");

            // Read max brightness, scale our 0-255 to device range
            if let Ok(max_str) = fs::read_to_string(&max_brightness_path) {
                if let Ok(max_val) = max_str.trim().parse::<u64>() {
                    let scaled = (u64::from(level) * max_val) / 255;
                    let _ = fs::write(&brightness_path, scaled.to_string());
                }
            }
        }

        Ok(())
    }

    fn is_on(&self) -> bool {
        self.is_on
    }

    fn set_power(&mut self, on: bool) -> Result<(), DisplayError> {
        self.is_on = on;

        // On real hardware, would write to DRM connector dpms or
        // /sys/class/backlight/*/bl_power
        if let Some(ref backlight_path) = self.backlight_path {
            let power_path = backlight_path.join("bl_power");
            let power_val = if on { "0" } else { "4" }; // FB_BLANK_UNBLANK / FB_BLANK_POWERDOWN
            let _ = fs::write(&power_path, power_val);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_display_creation() {
        let display = LinuxDisplay::virtual_display(Resolution::new(1920, 1080));
        assert_eq!(display.resolution().width, 1920);
        assert_eq!(display.resolution().height, 1080);
        assert!(display.is_on());
    }

    #[test]
    fn virtual_display_present() {
        let mut display = LinuxDisplay::virtual_display(Resolution::new(2, 2));
        // 2x2 RGBA = 16 bytes
        let fb = vec![0u8; 16];
        assert!(display.present(&fb).is_ok());
    }

    #[test]
    fn virtual_display_present_wrong_size() {
        let mut display = LinuxDisplay::virtual_display(Resolution::new(2, 2));
        let fb = vec![0u8; 10]; // Wrong size
        assert!(display.present(&fb).is_err());
    }

    #[test]
    fn display_brightness() {
        let mut display = LinuxDisplay::virtual_display(Resolution::new(100, 100));
        assert!(display.set_brightness(128).is_ok());
    }

    #[test]
    fn display_power_toggle() {
        let mut display = LinuxDisplay::virtual_display(Resolution::new(100, 100));
        assert!(display.is_on());
        assert!(display.set_power(false).is_ok());
        assert!(!display.is_on());
        assert!(display.set_power(true).is_ok());
        assert!(display.is_on());
    }

    #[test]
    fn parse_mode_valid() {
        let r = LinuxDisplay::parse_mode("1920x1080");
        assert!(r.is_some());
        let r = r.unwrap_or(Resolution::new(0, 0));
        assert_eq!(r.width, 1920);
        assert_eq!(r.height, 1080);
    }

    #[test]
    fn parse_mode_invalid() {
        assert!(LinuxDisplay::parse_mode("invalid").is_none());
        assert!(LinuxDisplay::parse_mode("").is_none());
    }

    #[test]
    fn watch_display() {
        let display = LinuxDisplay::new(Resolution::new(450, 450), DisplayShape::Circle);
        assert_eq!(display.shape(), DisplayShape::Circle);
        assert!(display.resolution().is_square());
    }
}
