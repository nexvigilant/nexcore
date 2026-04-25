//! # Iron Vigil Stark Suit
//!
//! Unified umbrella crate for the wearable suit. Condenses 12 component
//! crates into 4 compounds, exposed as one importable surface.
//!
//! ## Topology
//!
//! ```text
//!  COMPONENTS (12)            COMPOUNDS (4)              PRODUCT (1)
//!  ───────────────            ─────────────              ────────────
//!  suit-primitives ┐
//!  suit-sensors    ├─────►  perception     ┐
//!  suit-perception ┘                        │
//!                                           │
//!  suit-actuator   ┐                        │
//!  suit-flight     ├─────►  control         ├─►  stark_suit
//!  suit-compute    ┘                        │
//!                                           │
//!  suit-power      ┐                        │
//!  suit-power-mcp  ├─────►  power           │
//!  suit-thermal    ┘                        │
//!                                           │
//!  suit-hud        ┐                        │
//!  suit-voice      ├─────►  human_interface ┘
//!  suit-comms      │
//!  suit-safety     ┘
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use stark_suit::{perception, control, power, human_interface};
//!
//! let world  = perception::PerceptionEngine::new("model/intent.bin".into());
//! let engine = power::PowerEngine::new();
//! let stop   = human_interface::EStopController { watchdog: my_wd };
//! ```
//!
//! Importing a compound brings its component re-exports into a single
//! flat namespace — the consumer never has to know which underlying
//! crate provides which type.
//!
//! ## Compound boundaries
//!
//! Each compound is a **Rust module** here, not a crate. The components
//! stay as separate crates (independent build, test, and audit boundaries).
//! `stark-suit` is purely re-export. No new logic. No new types.
//!
//! Future evolution: when a compound matures past re-export, individual
//! `compounds::*` modules MAY add cross-component glue (e.g. a unified
//! `power_thermal::ThermalDeratedSoc` that fuses thermal derating into
//! SOC estimation). Such glue is allowed only here at the compound layer
//! to preserve component independence.

#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod compounds;

// Top-level re-exports of the four compound modules under shorter names.
pub use compounds::control;
pub use compounds::human_interface;
pub use compounds::perception;
pub use compounds::power;

/// Compile-time list of all component crates this umbrella unifies.
/// Used by `nexcore_health_probe` and the bay-wide reconciler to
/// confirm coverage.
pub const COMPONENT_CRATES: &[&str] = &[
    "suit-primitives",
    "suit-sensors",
    "suit-perception",
    "suit-actuator",
    "suit-flight",
    "suit-compute",
    "suit-power",
    "suit-power-mcp",
    "suit-thermal",
    "suit-hud",
    "suit-voice",
    "suit-comms",
    "suit-safety",
];

/// Compile-time list of compound module names exposed by this umbrella.
pub const COMPOUNDS: &[&str] = &["perception", "control", "power", "human_interface"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_count_matches_compound_topology() {
        // 13 components condense into 4 compounds.
        assert_eq!(COMPONENT_CRATES.len(), 13);
        assert_eq!(COMPOUNDS.len(), 4);
    }

    #[test]
    fn compound_modules_are_reachable() {
        // Smoke: each compound module exists and is importable.
        let _ = perception::PERCEPTION_COMPOUND_NAME;
        let _ = control::CONTROL_COMPOUND_NAME;
        let _ = power::POWER_COMPOUND_NAME;
        let _ = human_interface::HUMAN_INTERFACE_COMPOUND_NAME;
    }
}
