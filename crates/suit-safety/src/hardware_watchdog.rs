//! # Hardware Watchdog Interface
//!
//! Provides the hard-real-time interface for safety-rated watchdog cascades.

/// Trait for the hardware watchdog timer.
pub trait HardwareWatchdog {
    /// Kicks the watchdog within the allowable window.
    fn kick(&mut self);
    /// Forces an immediate E-stop if the watchdog condition is violated.
    fn trigger_emergency_stop(&mut self);
}
