//! # Suit Perception
//!
//! This crate implements the PERCEPTION_DOMAIN for the suit system.
//!
//! ## Modules
//!
//! - **vestibular**: Inertial + Position (IMU, GPS, Magnetometer, Barometer)
//! - **exteroceptive**: External sensing (Cameras, LiDAR, Thermal, Proximity)
//! - **proprioceptive**: Internal body state (Joint encoders, EMG, Insoles, Health)
//! - **fusion**: Multi-sensor fusion and state estimation
//! - **intent**: Machine learning models for user intent classification

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Exteroceptive sense: External sensing (Cameras, LiDAR, Thermal, Proximity)
pub mod exteroceptive;
/// Multi-sensor fusion and state estimation
pub mod fusion;
/// RTK GNSS and NTRIP Client
pub mod gnss;
/// Hardware interface for IMU array
pub mod imu_interface;
/// Machine learning models for user intent classification
pub mod intent;
/// High-level perception engine
pub mod perception_engine;
/// Proprioceptive sense: Internal body state (Joint encoders, EMG, Insoles, Health)
pub mod proprioceptive;
/// Vestibular sense: Inertial + Position (IMU, GPS, Magnetometer, Barometer)
pub mod vestibular;

#[cfg(test)]
pub mod vestibular_tests;

/// Re-export of common types
pub mod prelude {
    pub use crate::intent::Intent;
    pub use crate::proprioceptive::BodyState;
    pub use crate::vestibular::InertialState;
}

/// Re-export suit-sensors types for consumers
pub use suit_sensors;
