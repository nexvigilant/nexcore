//! Maestro → Vigil bridge.
//!
//! Receives HTTP status updates from maestro-core's StatusServer
//! and converts them into Vigil Events emitted to the EventBus.
//! This enables bidirectional communication:
//!   Vigil → Maestro: MaestroExecutor dispatches via REST
//!   Maestro → Vigil: StatusServer POSTs here via webhook callback

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tracing::info;

use crate::events::EventBus;
use crate::models::{Event, Urgency};

/// Payload matching maestro-core's SessionStatusPayload.
/// Duplicated here to avoid cross-workspace dependency.
#[derive(Debug, Clone, Deserialize)]
pub struct MaestroStatusPayload {
    pub session_id: u32,
    pub project_path: String,
    pub status: String,
    pub message: String,
    pub needs_input_prompt: Option<String>,
}

/// Shared state for the maestro bridge handler.
#[derive(Clone)]
pub struct MaestroBridgeState {
    pub bus: EventBus,
}

/// HTTP handler: receives maestro-core status updates, emits Vigil Events.
///
/// Mount on Vigil's webhook router:
/// ```ignore
/// .route("/maestro-status", post(maestro_status_handler))
/// ```
pub async fn maestro_status_handler(
    State(state): State<MaestroBridgeState>,
    Json(payload): Json<MaestroStatusPayload>,
) -> Result<&'static str, StatusCode> {
    let event_type = map_status_to_event_type(&payload.status);
    let priority = map_status_to_priority(&payload.status);

    let event = Event {
        source: "maestro".to_string(),
        event_type,
        priority,
        payload: serde_json::json!({
            "session_id": payload.session_id,
            "project_path": payload.project_path,
            "status": payload.status,
            "message": payload.message,
            "needs_input_prompt": payload.needs_input_prompt,
        }),
        ..Event::default()
    };

    info!(
        session_id = payload.session_id,
        status = %payload.status,
        "maestro_status_received"
    );

    state.bus.emit(event).await;
    Ok("OK")
}

/// Map maestro session status strings to Vigil event types.
fn map_status_to_event_type(status: &str) -> String {
    match status {
        "idle" | "Idle" => "session_idle".to_string(),
        "working" | "Working" => "session_working".to_string(),
        "waiting_input" | "WaitingInput" => "session_needs_input".to_string(),
        "error" | "Error" => "session_error".to_string(),
        "stopped" | "Stopped" => "session_stopped".to_string(),
        other => format!("session_{other}"),
    }
}

/// Map maestro session status to Vigil priority.
fn map_status_to_priority(status: &str) -> Urgency {
    match status {
        "error" | "Error" => Urgency::High,
        "waiting_input" | "WaitingInput" => Urgency::High,
        _ => Urgency::Normal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_mapping() {
        assert_eq!(map_status_to_event_type("idle"), "session_idle");
        assert_eq!(map_status_to_event_type("Working"), "session_working");
        assert_eq!(map_status_to_event_type("custom"), "session_custom");
    }

    #[test]
    fn test_priority_mapping() {
        assert_eq!(map_status_to_priority("error"), Urgency::High);
        assert_eq!(map_status_to_priority("idle"), Urgency::Normal);
        assert_eq!(map_status_to_priority("WaitingInput"), Urgency::High);
    }
}
