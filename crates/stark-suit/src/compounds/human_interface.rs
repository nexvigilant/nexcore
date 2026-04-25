//! # Compound: HUMAN-INTERFACE
//!
//! Pilot interaction + safety net. The suit's "talk to the human" layer.
//!
//! ## Components
//! - `suit_hud`    — helmet AR overlay, voice agent integration, haptics.
//! - `suit_voice`  — ASR / TTS / LLM routing.
//! - `suit_comms`  — link selection across BLE / Wi-Fi / 5G / LoRa / Satcom / VHF.
//! - `suit_safety` — ballistic recovery, fire suppression, e-stop, redundancy.
//!
//! Safety lives here because the human is the ultimate stakeholder of the
//! safety chain — every safety event eventually surfaces in the HUD or
//! voice channel.

/// Compound identifier for telemetry and registry.
pub const HUMAN_INTERFACE_COMPOUND_NAME: &str = "human_interface";

/// Re-export the entire public surface of `suit_hud`.
pub use suit_hud as hud;

/// Re-export the entire public surface of `suit_voice`.
pub use suit_voice as voice;

/// Re-export the entire public surface of `suit_comms`.
pub use suit_comms as comms;

/// Re-export the entire public surface of `suit_safety`.
pub use suit_safety as safety;

/// Convenience: the e-stop controller (parameterized over a watchdog).
pub use suit_safety::e_stop::EStopController;

/// Convenience: the ballistic recovery system.
pub use suit_safety::recovery::BallisticSystem;
