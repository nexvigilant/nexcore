//! Parameter types for suit-hud MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Get current helmet state (visor, optics, eye tracker, mic, ventilation).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HelmetStateParams {}

/// Compose a HUD frame from data sources.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HudFrameComposeParams {
    /// Pitch in degrees.
    pub pitch_deg: Option<f32>,
    /// Roll in degrees.
    pub roll_deg: Option<f32>,
    /// Heading in degrees.
    pub heading_deg: Option<f32>,
    /// Altitude in meters.
    pub altitude_m: Option<f32>,
    /// Speed in m/s.
    pub speed_mps: Option<f32>,
    /// Power level (0.0..1.0).
    pub power: Option<f32>,
    /// Suit integrity (0.0..1.0).
    pub integrity: Option<f32>,
    /// Thermal status: "nominal", "elevated", "warning", "critical".
    pub thermal: Option<String>,
    /// Threat markers as JSON array.
    pub threats: Option<String>,
}

/// Send a haptic cue to a body zone.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HapticCueParams {
    /// Target zone: "wrist_left", "wrist_right", "palm_left", "palm_right",
    /// "torso_front", "torso_back", "torso_left", "torso_right",
    /// "shoulder_left", "shoulder_right", "elbow_left", "elbow_right",
    /// "knee_left", "knee_right".
    pub zone: String,
    /// Pattern: "pulse", "buzz", "ramp", "threat_alert", "nav_left", "nav_right".
    pub pattern: String,
    /// Intensity (0.0..1.0). Default: 0.5.
    pub intensity: Option<f32>,
    /// Duration in milliseconds. Default: 200.
    pub duration_ms: Option<u32>,
}

/// Get voice agent state (wake word, ASR, LLM, TTS, MCP bridge).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VoiceAgentStateParams {}
