//! # RTK GNSS and NTRIP Client
//!
//! Handles high-precision GNSS localization using RTK corrections from an NTRIP caster.

use nexcore_error::NexError as Error;
use serde::{Deserialize, Serialize};

/// Represents the high-precision state of the RTK GNSS module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtkGnssState {
    /// Precise geodetic coordinates (lat, lon, alt)
    pub position: (f64, f64, f64),
    /// RTK Solution status (e.g., Float, Fixed)
    pub status: RtkStatus,
    /// Number of satellites used in the solution
    pub sats: u8,
}

/// RTK Solution Status indicating accuracy level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RtkStatus {
    /// No RTK solution (unreliable)
    None,
    /// RTK Float solution (decimeter accuracy)
    Float,
    /// RTK Fixed solution (centimeter accuracy)
    Fixed,
}

/// NTRIP Client to handle correction stream retrieval.
pub struct NtripClient {
    /// URL of the NTRIP caster (e.g., "ntrip.rtk2go.com:2101")
    pub caster_url: String,
    /// Authentication credentials (user:password base64 encoded)
    pub auth: String,
    /// Mountpoint identifying the specific base station
    pub mountpoint: String,
}

impl NtripClient {
    /// Initializes a connection to the NTRIP caster to receive RTCM corrections.
    pub fn connect(&mut self) -> Result<(), Error> {
        // TODO: Implement TCP socket handling for the NTRIP stream.
        Ok(())
    }

    /// Fetches the latest RTCM correction data to feed into the GNSS receiver.
    pub fn fetch_corrections(&self) -> Result<Vec<u8>, Error> {
        // TODO: Read correction frame from the stream.
        Ok(vec![])
    }
}
