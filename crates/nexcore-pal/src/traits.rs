// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Platform Abstraction Layer trait definitions.
//!
//! These traits define the hardware abstraction boundary between NexCore OS
//! and the underlying platform. All traits are `no_std` compatible at the
//! definition level — implementations may use `std` (via `nexcore-pal-linux`).
//!
//! ## Grammar Analysis
//!
//! The PAL is a **Type-1 grammar** (context-sensitive):
//! - σ (Sequence): Event streams, framebuffer writes
//! - Σ (Sum): Error variants, event types
//! - ρ (Recursion): Nested event handling
//! - κ (Comparison): Power state thresholds, resolution checks
//!
//! Four generators = Chomsky Level 1. No ∃ (Existence) needed at trait level,
//! so we don't need a Turing-complete abstraction — correct.

use crate::error::{DisplayError, HapticsError, InputError, NetworkError, StorageError};
use crate::types::{
    DisplayShape, FormFactor, HapticPulse, InputEvent, PixelFormat, PowerState, Resolution,
};

/// Display subsystem abstraction.
///
/// Tier: T2-C (μ Mapping + ∂ Boundary + ∃ Existence)
///
/// Maps framebuffer data to the physical display. The boundary is defined
/// by resolution and pixel format. Existence = display availability.
pub trait Display {
    /// Get the current display resolution.
    fn resolution(&self) -> Resolution;

    /// Get the display shape.
    fn shape(&self) -> DisplayShape;

    /// Get the pixel format.
    fn pixel_format(&self) -> PixelFormat;

    /// Get the display refresh rate in Hz.
    fn refresh_rate(&self) -> u32;

    /// Present a framebuffer to the display.
    ///
    /// The framebuffer must match `resolution() * pixel_format().bytes_per_pixel()` bytes.
    fn present(&mut self, framebuffer: &[u8]) -> Result<(), DisplayError>;

    /// Set the display brightness (0-255).
    fn set_brightness(&mut self, level: u8) -> Result<(), DisplayError>;

    /// Check if the display is currently on.
    fn is_on(&self) -> bool;

    /// Turn display on or off.
    fn set_power(&mut self, on: bool) -> Result<(), DisplayError>;
}

/// Input subsystem abstraction.
///
/// Tier: T2-C (σ Sequence + ∃ Existence + μ Mapping)
///
/// Sequences of input events mapped from hardware to platform-agnostic types.
pub trait Input {
    /// Poll for pending input events.
    ///
    /// Returns events that have occurred since the last poll.
    /// Requires `std` or `alloc` feature for `Vec`.
    #[cfg(feature = "std")]
    fn poll_events(&mut self) -> Result<Vec<InputEvent>, InputError>;

    /// Poll for a single event (non-allocating, suitable for `no_std`).
    fn poll_event(&mut self) -> Result<Option<InputEvent>, InputError>;

    /// Check if there are events pending.
    fn has_events(&self) -> bool;
}

/// Network subsystem abstraction.
///
/// Tier: T2-C (μ Mapping + ∂ Boundary + σ Sequence)
///
/// Maps data to network destinations with boundary checks (connectivity).
pub trait Network {
    /// Check if network is connected.
    fn is_connected(&self) -> bool;

    /// Send data to a destination address.
    ///
    /// Returns the number of bytes sent.
    fn send(&mut self, data: &[u8], dest: &str) -> Result<usize, NetworkError>;

    /// Receive data into buffer.
    ///
    /// Returns the number of bytes received.
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize, NetworkError>;

    /// Get the current IP address (as bytes, IPv4 = 4 bytes, IPv6 = 16 bytes).
    fn local_addr(&self) -> Option<[u8; 16]>;

    /// Get signal strength (0-100) if applicable (WiFi/cellular).
    fn signal_strength(&self) -> Option<u8>;
}

/// Storage subsystem abstraction.
///
/// Tier: T2-C (π Persistence + μ Mapping + ∂ Boundary)
///
/// Persistent mapping from paths to data with boundary checks (permissions, capacity).
pub trait Storage {
    /// Read file contents.
    #[cfg(feature = "std")]
    fn read(&self, path: &str) -> Result<Vec<u8>, StorageError>;

    /// Write data to a file.
    fn write(&mut self, path: &str, data: &[u8]) -> Result<(), StorageError>;

    /// Delete a file.
    fn delete(&mut self, path: &str) -> Result<(), StorageError>;

    /// Check if a path exists.
    fn exists(&self, path: &str) -> bool;

    /// Get available storage in bytes.
    fn available_bytes(&self) -> Result<u64, StorageError>;

    /// Get total storage in bytes.
    fn total_bytes(&self) -> Result<u64, StorageError>;
}

/// Haptics subsystem abstraction.
///
/// Tier: T2-C (ν Frequency + N Quantity + σ Sequence)
///
/// Sequences of haptic pulses with frequency/intensity control.
pub trait Haptics {
    /// Play a haptic pattern.
    fn vibrate(&mut self, pattern: &[HapticPulse]) -> Result<(), HapticsError>;

    /// Stop any current vibration.
    fn stop(&mut self) -> Result<(), HapticsError>;

    /// Check if haptics hardware is available.
    fn is_available(&self) -> bool;
}

/// Power subsystem abstraction.
///
/// Tier: T2-C (N Quantity + ∃ Existence + ς State)
///
/// Battery level (quantity), charging state (state), sensor existence.
pub trait Power {
    /// Get current power state.
    fn state(&self) -> PowerState;

    /// Get battery percentage (0-100), None if no battery.
    fn battery_pct(&self) -> Option<u8> {
        self.state().battery_pct()
    }

    /// Check if device is charging.
    fn is_charging(&self) -> bool {
        self.state().is_charging()
    }

    /// Get estimated runtime remaining in minutes.
    fn estimated_runtime_minutes(&self) -> Option<u32>;

    /// Get battery temperature in celsius * 10 (e.g., 350 = 35.0°C).
    fn temperature_decicelsius(&self) -> Option<i32>;
}

/// Composite platform trait — the full hardware abstraction.
///
/// Tier: T3 (Σ Sum — composition of all subsystem traits)
///
/// A `Platform` implementor provides access to all hardware subsystems.
/// This is the generic bound for `NexCoreOs<P: Platform>`.
pub trait Platform {
    /// The display implementation type.
    type Display: Display;
    /// The input implementation type.
    type Input: Input;
    /// The network implementation type.
    type Network: Network;
    /// The storage implementation type.
    type Storage: Storage;
    /// The haptics implementation type.
    type Haptics: Haptics;
    /// The power implementation type.
    type Power: Power;

    /// Get the form factor of this device.
    fn form_factor(&self) -> FormFactor;

    /// Get a reference to the display subsystem.
    fn display(&self) -> &Self::Display;
    /// Get a mutable reference to the display subsystem.
    fn display_mut(&mut self) -> &mut Self::Display;

    /// Get a reference to the input subsystem.
    fn input(&self) -> &Self::Input;
    /// Get a mutable reference to the input subsystem.
    fn input_mut(&mut self) -> &mut Self::Input;

    /// Get a reference to the network subsystem.
    fn network(&self) -> &Self::Network;
    /// Get a mutable reference to the network subsystem.
    fn network_mut(&mut self) -> &mut Self::Network;

    /// Get a reference to the storage subsystem.
    fn storage(&self) -> &Self::Storage;
    /// Get a mutable reference to the storage subsystem.
    fn storage_mut(&mut self) -> &mut Self::Storage;

    /// Get a reference to the haptics subsystem.
    fn haptics(&self) -> &Self::Haptics;
    /// Get a mutable reference to the haptics subsystem.
    fn haptics_mut(&mut self) -> &mut Self::Haptics;

    /// Get a reference to the power subsystem.
    fn power(&self) -> &Self::Power;

    /// Platform name (e.g., "linux-x86_64", "linux-aarch64-watch").
    fn name(&self) -> &str;

    /// Get the platform's persistent data directory path.
    ///
    /// All OS-level persistent data (vault, state, logs) lives under this root.
    /// Returns a string path suitable for `Path::new()` or `PathBuf::from()`.
    fn data_dir(&self) -> &str;
}
