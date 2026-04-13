//! Suit HUD MCP tools — helmet state, HUD frame, haptics, voice agent.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::suit_hud::{
    HapticCueParams, HelmetStateParams, HudFrameComposeParams, VoiceAgentStateParams,
};

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

/// Get current helmet state snapshot.
pub fn helmet_state(_p: HelmetStateParams) -> Result<CallToolResult, McpError> {
    let state = suit_hud::HelmetState::default();
    let value =
        serde_json::to_value(&state).unwrap_or_else(|_| json!({"error": "serialization failed"}));
    ok_json(value)
}

/// Compose a HUD frame from provided data sources.
pub fn hud_frame_compose(p: HudFrameComposeParams) -> Result<CallToolResult, McpError> {
    use suit_hud::hud::*;

    let mut frame = HudFrame::new(0, 0);

    // Attitude layer
    frame.add_layer(HudLayer::Attitude(AttitudeIndicator {
        pitch_deg: p.pitch_deg.unwrap_or(0.0),
        roll_deg: p.roll_deg.unwrap_or(0.0),
        heading_deg: p.heading_deg.unwrap_or(0.0),
        altitude_m: p.altitude_m.unwrap_or(0.0),
        speed_mps: p.speed_mps.unwrap_or(0.0),
        ..AttitudeIndicator::default()
    }));

    // Systems layer
    let thermal = match p.thermal.as_deref() {
        Some("elevated") => ThermalStatus::Elevated,
        Some("warning") => ThermalStatus::Warning,
        Some("critical") => ThermalStatus::Critical,
        _ => ThermalStatus::Nominal,
    };
    frame.add_layer(HudLayer::Systems(SystemsGlanceable {
        power: p.power.unwrap_or(1.0),
        integrity: p.integrity.unwrap_or(1.0),
        thermal,
        ..SystemsGlanceable::default()
    }));

    // Threat layer (parse from JSON string if provided)
    if let Some(threats_json) = &p.threats {
        if let Ok(markers) = serde_json::from_str::<Vec<ThreatMarker>>(threats_json) {
            frame.add_layer(HudLayer::Threat(ThreatOverlay {
                threats: markers,
                obstacles: vec![],
            }));
        }
    }

    // Voice panel (empty but present)
    frame.add_layer(HudLayer::Voice(VoicePanel::default()));

    let value =
        serde_json::to_value(&frame).unwrap_or_else(|_| json!({"error": "serialization failed"}));
    ok_json(json!({
        "frame": value,
        "layers": frame.layers.len(),
        "threats": frame.threat_count(),
        "systems_ok": frame.systems_health().map(|s| s.thermal == ThermalStatus::Nominal),
    }))
}

/// Send a haptic cue.
pub fn haptic_cue(p: HapticCueParams) -> Result<CallToolResult, McpError> {
    use suit_hud::haptics::*;

    let zone = match p.zone.as_str() {
        "wrist_left" => HapticZone::WristLeft,
        "wrist_right" => HapticZone::WristRight,
        "palm_left" => HapticZone::PalmLeft,
        "palm_right" => HapticZone::PalmRight,
        "torso_front" => HapticZone::TorsoFront,
        "torso_back" => HapticZone::TorsoBack,
        "torso_left" => HapticZone::TorsoLeft,
        "torso_right" => HapticZone::TorsoRight,
        "shoulder_left" => HapticZone::ShoulderLeft,
        "shoulder_right" => HapticZone::ShoulderRight,
        "elbow_left" => HapticZone::ElbowLeft,
        "elbow_right" => HapticZone::ElbowRight,
        "knee_left" => HapticZone::KneeLeft,
        "knee_right" => HapticZone::KneeRight,
        other => return err_result(&format!("unknown zone: {other}")),
    };

    let intensity = p.intensity.unwrap_or(0.5);
    let duration_ms = p.duration_ms.unwrap_or(200);

    let cue = match p.pattern.as_str() {
        "pulse" => HapticCue::pulse(zone, intensity, duration_ms, "manual_pulse"),
        "threat_alert" => HapticCue::threat_alert(zone, intensity),
        "nav_left" => HapticCue::nav_cue(false),
        "nav_right" => HapticCue::nav_cue(true),
        "buzz" => HapticCue {
            zones: vec![zone],
            pattern: TactilePattern::Buzz {
                intensity,
                on_ms: 100,
                off_ms: 100,
                cycles: 3,
            },
            priority: 2,
            label: "manual_buzz".to_string(),
        },
        "ramp" => HapticCue {
            zones: vec![zone],
            pattern: TactilePattern::Ramp {
                start: 0.1,
                end: intensity,
                duration_ms,
            },
            priority: 2,
            label: "manual_ramp".to_string(),
        },
        other => return err_result(&format!("unknown pattern: {other}")),
    };

    let value =
        serde_json::to_value(&cue).unwrap_or_else(|_| json!({"error": "serialization failed"}));
    ok_json(json!({
        "cue": value,
        "zone_type": if zone.is_erm() { "ERM motor" } else if zone.is_belt() { "vibrotactile belt" } else { "force feedback" },
    }))
}

/// Get voice agent state.
pub fn voice_agent_state(_p: VoiceAgentStateParams) -> Result<CallToolResult, McpError> {
    let state = suit_hud::AgentState::default();
    let value =
        serde_json::to_value(&state).unwrap_or_else(|_| json!({"error": "serialization failed"}));
    ok_json(value)
}
