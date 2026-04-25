//! # stark-suit-station (library)
//!
//! Public library surface for the Iron Vigil Stark Suit station daemon.
//! The binary at `src/main.rs` consumes this lib; downstream integration
//! tests import the BMS surface directly via `stark_suit_station::bms::*`
//! rather than the v0.3 `#[path]` hack.
//!
//! Stable public surface (v0.5):
//!
//! - [`bms`] — BMS telemetry source trait + 3 backends
//!   (`MockBmsSource`, `ReplayBmsSource`, `SerialBmsSource`)
//! - [`loops`] — 4 control loops (perception/power/control/human_interface)
//! - [`state`] — `StationState` + per-compound snapshot types
//! - [`mcp`] — `StarkSuitMcpServer` (rmcp `ServerHandler`)

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod bms;
pub mod loops;
pub mod mcp;
pub mod perception;
pub mod state;
