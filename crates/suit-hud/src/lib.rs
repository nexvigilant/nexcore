//! # suit-hud — Iron Vigil Human Interface Domain
//!
//! Typed abstractions for the JARVIS suit's human-facing systems:
//!
//! - **Helmet** (7.1) — shell, visor, AR optics, eye tracker, audio, mic array, ventilation
//! - **HUD** (7.2) — attitude indicator, threat overlay, systems health, nav waypoints, voice panel
//! - **Voice Agent** (7.3) — wake word, ASR, LLM router, TTS, MCP bridge
//! - **Haptics** (7.4) — wrist ERM, torso belt, force feedback
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                  HUMAN INTERFACE                     │
//! │                                                      │
//! │  Helmet ──> HUD ──> AR Overlay ──> Pilot's eyes     │
//! │    │                    ▲                             │
//! │    │         Threat data│  Systems data               │
//! │    │                    │                             │
//! │  Mic Array ──> Voice Agent ──> LLM ──> TTS ──> Bone │
//! │                    │                     Conduction   │
//! │                    ▼                                  │
//! │              MCP Bridge ──> Suit Telemetry           │
//! │                                                      │
//! │  Haptics ◄── Threat alerts, nav cues, force feedback │
//! └─────────────────────────────────────────────────────┘
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod bridge;
pub mod haptics;
pub mod helmet;
pub mod hud;
pub mod voice;

// Re-export key types
pub use haptics::{HapticCue, HapticZone, TactilePattern};
pub use helmet::{ArOptics, EyeTracker, GazePoint, HelmetState, MicArray, VisorState};
pub use hud::{AttitudeIndicator, HudFrame, HudLayer, SystemsGlanceable, ThreatOverlay, Waypoint};
pub use voice::{AgentState, LlmRoute, VoiceCommand, WakeWordState};
