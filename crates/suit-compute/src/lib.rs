//! # suit-compute — Iron Vigil Compute Domain
//!
//! Typed abstractions and orchestration logic for the Iron Vigil suit's distributed processing backbone.
//!
//! ## Modules
//! - `flight`: Interfaces for the STM32H7 hard-real-time flight MCU.
//! - `exo`: Interfaces for the Zynq UltraScale+ exoskeleton vision/actuation MCU.
//! - `soc`: Interfaces for the high-level AI companion SoC (NVIDIA Orin NX).
//! - `safety`: Triple-Modular Redundancy (TMR) voting and watchdog logic.
//! - `storage`: Telemetry and black-box data logging interfaces.

pub mod flight {
    //! Flight MCU (STM32H7) — hard-real-time flight control and stabilization.
    /// Represents the high-level state of the flight control loop.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FlightState {
        /// MCU initializing, performing POST and sensor calibration.
        Boot,
        /// Full flight capability, all sensors and actuators nominal.
        Nominal,
        /// Non-critical subsystem failure, flight envelope restricted.
        Degraded,
        /// Critical failure, entering fail-safe or auto-rotation mode.
        Critical,
    }
}

pub mod exo {
    //! Exo MCU (Zynq UltraScale+) — vision pre-processing and exoskeleton joint coordination.
}

pub mod soc {
    //! Companion SoC (NVIDIA Orin NX) — high-level AI, HUD rendering, and sensor fusion.
}

pub mod safety {
    //! Redundant Safety MCU (Watchdog + Voter) — fail-safe monitoring and TMR voting.

    /// Result of a Triple-Modular Redundancy (TMR) vote.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum VoteResult<T> {
        /// All three sensors agree.
        Unanimous(T),
        /// Two sensors agree, one disagrees.
        Majority(T),
        /// No two sensors agree (critical failure).
        Divergent,
    }
}

pub mod storage {
    //! Storage — NVMe for black-box log, SD for config.
}
