//! # Compound: POWER + THERMAL
//!
//! Energy management. The suit's "stay alive" layer.
//!
//! ## Components
//! - `suit_power`     — SOC, load prioritization, BMS, fuel, thermal derating.
//! - `suit_power_mcp` — MCP wrapper exposing power telemetry to AI agents.
//! - `suit_thermal`   — zone monitoring, cooling routing, thermal runaway.

/// Compound identifier for telemetry and registry.
pub const POWER_COMPOUND_NAME: &str = "power";

/// Re-export the entire public surface of `suit_power`.
pub use suit_power as power;

/// Re-export the entire public surface of `suit_power_mcp`.
pub use suit_power_mcp as power_mcp;

/// Re-export the entire public surface of `suit_thermal`.
pub use suit_thermal as thermal;

/// Convenience: the central power management engine.
pub use suit_power::engine::PowerEngine;

/// Convenience: the canonical mission forecast type.
pub use suit_power::mission::MissionForecast;
