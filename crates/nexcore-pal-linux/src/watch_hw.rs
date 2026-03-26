// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Galaxy Watch 7 (SM-L315U) Hardware Configuration
//!
//! Discovered via ADB + device tree extraction on 2026-03-22.
//! SoC: Samsung Exynos W1000 (s5e5535), 5x Cortex-A55, 1.76GB RAM.
//!
//! This module provides compile-time hardware constants and runtime
//! device path configuration for the NexWatch OS PAL implementation.

/// Display hardware — Samsung AMOLED via DECON DRM.
///
/// Two panel variants: LX83805 (BOE) and LX83806 (SDC).
/// Both support 480x480@60Hz (HS) and 480x480@30Hz (NS).
pub mod display {
    /// DRM card device path.
    pub const DRM_CARD: &str = "/dev/dri/card0";
    /// DRM render node for GPU compute.
    pub const DRM_RENDER: &str = "/dev/dri/renderD128";
    /// Display controller (DECON) MMIO base.
    pub const DECON_BASE: u64 = 0x138C_0000;
    /// Display serial interface (DSI) MMIO base.
    pub const DSI_BASE: u64 = 0x138D_0000;
    /// D-PHY for DSI.
    pub const DPHY_BASE: u64 = 0x138E_0000;
    /// Mali GPU MMIO base.
    pub const GPU_BASE: u64 = 0x1410_0000;
    /// Panel resolution.
    pub const WIDTH: u32 = 480;
    pub const HEIGHT: u32 = 480;
    /// Native refresh rate (Hz).
    pub const REFRESH_HZ: u32 = 60;
    /// Display density (DPI).
    pub const DENSITY_DPI: u32 = 340;
    /// Panel is circular with full-radius corners.
    pub const CORNER_RADIUS: u32 = 240;
}

/// Touch input — Zinitix ZTM730 on I2C bus 4.
pub mod touch {
    /// I2C bus path for the touch controller.
    pub const I2C_BUS: &str = "/sys/devices/platform/10330000.hsi2c/i2c-4/4-0020";
    /// evdev device for touch events.
    pub const EVENT_TOUCH_PRIMARY: &str = "/dev/input/event3";
    /// Secondary touch event device (multitouch).
    pub const EVENT_TOUCH_SECONDARY: &str = "/dev/input/event4";
    /// I2C address.
    pub const I2C_ADDR: u8 = 0x20;
    /// Driver name as reported by the kernel.
    pub const DRIVER_NAME: &str = "sec_touchscreen";
    /// Compatible string from device tree.
    pub const DT_COMPATIBLE: &str = "zinitix,ztm730_ts";
}

/// Physical button input.
pub mod buttons {
    /// Power key (S2MPW05 PMIC).
    pub const EVENT_POWER: &str = "/dev/input/event2";
    /// GPIO keys (hardware buttons).
    pub const EVENT_GPIO: &str = "/dev/input/event0";
    /// Virtual keyboard + dpad.
    pub const EVENT_VIRTUAL: &str = "/dev/input/event1";
    /// Power key driver.
    pub const POWER_KEY_DRIVER: &str = "s2mpw05-power-keys";
}

/// Audio — Samsung AUD9002X codec via ABOX.
pub mod audio {
    /// ALSA control device.
    pub const CONTROL: &str = "/dev/snd/controlC0";
    /// Primary playback PCM.
    pub const PCM_PLAYBACK: &str = "/dev/snd/pcmC0D0p";
    /// ABOX audio subsystem MMIO base.
    pub const ABOX_BASE: u64 = 0x14E5_0000;
    /// ABOX GIC MMIO base.
    pub const ABOX_GIC_BASE: u64 = 0x14EF_0000;
    /// Audio codec compatible string.
    pub const CODEC_COMPATIBLE: &str = "codec,aud9002x";
    /// Platform-specific audio HAL library.
    pub const HAL_LIB: &str = "audio.primary.s5e5535.so";
    /// Number of playback PCM devices available.
    pub const PLAYBACK_PCMS: usize = 20;
    /// Number of capture PCM devices available.
    pub const CAPTURE_PCMS: usize = 48;
}

/// Haptic feedback — Samsung sec_vibrator.
pub mod haptics {
    /// Vibrator compatible string.
    pub const DT_COMPATIBLE: &str = "sec_vibrator";
    /// Normal haptic intensity ratio (0-100).
    pub const NORMAL_RATIO: u32 = 100;
    /// High temperature reference threshold (degrees C).
    pub const HIGH_TEMP_REF: u32 = 48;
    /// High temperature reduced ratio.
    pub const HIGH_TEMP_RATIO: u32 = 65;
    /// Supported haptic engine modes.
    pub const ENGINE_MODES: &[&str] = &[
        "INTENSITY",
        "HAPTIC_ENGINE",
        "FIFO_HAPTIC_ENGINE",
        "HYBRID_HAPTIC_ENGINE",
    ];
}

/// Power management — Samsung S2MPW05 PMIC.
pub mod power {
    /// Battery sysfs path.
    pub const BATTERY_PATH: &str = "/sys/class/power_supply/battery";
    /// USB power sysfs path.
    pub const USB_PATH: &str = "/sys/class/power_supply/usb";
    /// AC power sysfs path.
    pub const AC_PATH: &str = "/sys/class/power_supply/ac";
    /// Wireless charging sysfs path.
    pub const WIRELESS_PATH: &str = "/sys/class/power_supply/wireless";
    /// Charger driver.
    pub const CHARGER: &str = "s2mpw05-charger";
    /// Fuel gauge driver.
    pub const FUEL_GAUGE: &str = "s2mpw05-fuelgauge";
    /// RTC driver.
    pub const RTC: &str = "s2mpw05-rtc";
}

/// Network — Samsung SCSC WiFi/BLE combo + LTE modem.
pub mod network {
    /// WiFi/BLE combo chip MMIO base.
    pub const SCSC_BASE: u64 = 0x12AC_0000;
    /// SCSC driver compatible string.
    pub const DT_COMPATIBLE: &str = "samsung,scsc_wifibt";
    /// LTE modem radio partition.
    pub const RADIO_PARTITION: &str = "/dev/block/by-name/radio";
    /// Samsung RIL version.
    pub const RIL_VERSION: &str = "Samsung RIL v5.0";
    /// GNSS mailbox MMIO base.
    pub const GNSS_MAILBOX_BASE: u64 = 0x12AA_0000;
}

/// I2C sensor bus mapping (from device tree + ADB probing).
pub mod sensors {
    /// I2C bus 0: PMIC and core sensors.
    pub const BUS0_PATH: &str = "/sys/bus/i2c/devices/i2c-0";
    /// I2C bus 1: Magnetometer at 0x0c.
    pub const BUS1_PATH: &str = "/sys/bus/i2c/devices/i2c-1";
    /// I2C bus 2: Heart rate (0x30) + SpO2 (0x3e).
    pub const BUS2_PATH: &str = "/sys/bus/i2c/devices/i2c-2";
    /// I2C bus 3: Barometer/temperature at 0x28.
    pub const BUS3_PATH: &str = "/sys/bus/i2c/devices/i2c-3";
    /// I2C bus 4: Touch controller at 0x20.
    pub const BUS4_PATH: &str = "/sys/bus/i2c/devices/i2c-4";

    /// Heart rate monitor I2C address.
    pub const HRM_ADDR: u8 = 0x30;
    /// SpO2 sensor I2C address.
    pub const SPO2_ADDR: u8 = 0x3e;
    /// Barometer I2C address.
    pub const BARO_ADDR: u8 = 0x28;
    /// Magnetometer I2C address.
    pub const MAG_ADDR: u8 = 0x0c;
}

/// Storage layout (from /dev/block/by-name/).
pub mod storage {
    /// Boot partition (kernel + ramdisk).
    pub const BOOT: &str = "/dev/block/by-name/boot";
    /// Init boot (GKI ramdisk).
    pub const INIT_BOOT: &str = "/dev/block/by-name/init_boot";
    /// System (super partition, dm-verity).
    pub const SUPER: &str = "/dev/block/by-name/super";
    /// User data (22GB f2fs).
    pub const USERDATA: &str = "/dev/block/by-name/userdata";
    /// Radio (LTE modem firmware).
    pub const RADIO: &str = "/dev/block/by-name/radio";
    /// EFS (IMEI — NEVER MODIFY).
    pub const EFS: &str = "/dev/block/by-name/efs";
    /// Verified boot metadata.
    pub const VBMETA: &str = "/dev/block/by-name/vbmeta";
    /// Recovery.
    pub const RECOVERY: &str = "/dev/block/by-name/recovery";
}

/// SoC identification.
pub mod soc {
    /// Board name from getprop.
    pub const BOARD: &str = "s5e5535";
    /// Marketing name.
    pub const NAME: &str = "Samsung Exynos W1000";
    /// CPU implementation (Cortex-A55).
    pub const CPU_PART: u16 = 0xd05;
    /// Number of CPU cores.
    pub const CPU_CORES: usize = 5;
    /// Total RAM in KB.
    pub const RAM_KB: usize = 1_804_316;
    /// eMMC total in KB.
    pub const EMMC_KB: usize = 30_535_680;
    /// Kernel version on stock firmware.
    pub const KERNEL_VERSION: &str = "6.1.43-android14-11";
    /// USB controller MMIO base.
    pub const USB_BASE: u64 = 0x10B0_0000;
    /// eMMC controller MMIO base.
    pub const EMMC_BASE: u64 = 0x10A6_0000;
    /// Watchdog timer MMIO base.
    pub const WDT_BASE: u64 = 0x1005_0000;
}
