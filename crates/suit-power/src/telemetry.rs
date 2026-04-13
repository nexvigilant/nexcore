//! # Power Telemetry Bridge
//!
//! Bridges suit-power state transitions to the nexcore telemetry stream.

use wksp_types::power::PowerStatusMessage;

/// Telemetry client for power system logging.
pub struct PowerTelemetryBridge;

impl PowerTelemetryBridge {
    /// Dispatches a power status message to the system telemetry stream.
    pub fn dispatch(msg: &PowerStatusMessage) {
        // Here we would interface with nexcore-telemetry-core to push the message.
        tracing::info!(target: "telemetry", "Dispatching power status: {:?}", msg);
    }
}
