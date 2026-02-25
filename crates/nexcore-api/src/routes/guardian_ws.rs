//! Guardian WebSocket Real-Time Bridge (CAP-P2-001)
//!
//! Bridges the Guardian `EventBus` (CAP-P2-003) to WebSocket clients,
//! providing real-time streaming of homeostasis loop events.
//!
//! ## T1 Primitive Grounding
//!
//! - **Sequence** (sigma): Event stream from `EventBus` -> WS frames
//! - **Mapping** (mu): `event_bus::GuardianEvent` -> `WsMessage` JSON envelope
//! - **State** (varsigma): Per-connection lifecycle (connect -> stream -> disconnect)
//!
//! ## Protocol
//!
//! **Server -> Client:** JSON `WsMessage` envelopes wrapping Guardian events
//!
//! ```json
//! {"msg_type": "loop_tick", "payload": {...}, "timestamp": "2026-02-05T..."}
//! {"msg_type": "signal_detected", "payload": {...}, "timestamp": "..."}
//! {"msg_type": "action_taken", "payload": {...}, "timestamp": "..."}
//! {"msg_type": "threshold_breached", "payload": {...}, "timestamp": "..."}
//! {"msg_type": "status", "payload": {...}, "timestamp": "..."}
//! ```
//!
//! **Client -> Server:** JSON commands (optional)
//!
//! ```json
//! {"command": "pause"}
//! {"command": "resume"}
//! {"command": "status"}
//! ```

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use nexcore_chrono::DateTime;
use nexcore_vigilance::guardian::event_bus::{EventBus, GuardianEvent};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use tokio::sync::broadcast;

// =============================================================================
// Global EventBus (shared with REST handlers via get_event_bus())
// =============================================================================

/// Global EventBus instance, lazily initialized.
///
/// REST handlers in `guardian.rs` call `get_event_bus().publish(event)` after
/// each operation. WebSocket clients subscribe via `get_event_bus().subscribe()`.
static EVENT_BUS: OnceLock<Arc<EventBus>> = OnceLock::new();

/// Get or initialize the global EventBus.
///
/// The bus is created with the default capacity (1024 events). Subscribers
/// that fall behind will receive a `Lagged` error rather than blocking
/// the publisher.
pub fn get_event_bus() -> Arc<EventBus> {
    EVENT_BUS
        .get_or_init(|| Arc::new(EventBus::default()))
        .clone()
}

// =============================================================================
// WsMessage Envelope
// =============================================================================

/// Wire-format envelope for WebSocket messages.
///
/// Every event sent to a client is wrapped in this envelope, providing
/// a stable schema regardless of the inner `GuardianEvent` variant.
#[derive(Debug, Clone, Serialize)]
pub struct WsMessage {
    /// Event type discriminator (e.g. "loop_tick", "signal_detected")
    pub msg_type: String,
    /// Event payload as unstructured JSON
    pub payload: serde_json::Value,
    /// ISO-8601 timestamp of when the message was serialized for transmission
    pub timestamp: String,
}

/// Client-to-server command sent over WebSocket.
#[derive(Debug, Deserialize)]
struct WsCommand {
    /// Command name: "pause", "resume", or "status"
    command: String,
}

// =============================================================================
// Event Mapping: GuardianEvent -> WsMessage
// =============================================================================

/// Map a domain `GuardianEvent` to a wire-format `WsMessage`.
///
/// Each variant gets a stable `msg_type` string and its payload is
/// serialized to `serde_json::Value`. If serialization fails (should
/// never happen for these types), the payload falls back to a string
/// representation.
fn map_event_to_ws_message(event: &GuardianEvent) -> WsMessage {
    let (msg_type, payload) = match event {
        GuardianEvent::LoopTick(result) => (
            "loop_tick",
            serde_json::to_value(result)
                .unwrap_or_else(|_| serde_json::Value::String(format!("{result:?}"))),
        ),
        GuardianEvent::SignalDetected(signal) => (
            "signal_detected",
            serde_json::to_value(signal)
                .unwrap_or_else(|_| serde_json::Value::String(format!("{signal:?}"))),
        ),
        GuardianEvent::ActionTaken { action, result } => (
            "action_taken",
            serde_json::json!({
                "action": serde_json::to_value(action).unwrap_or_else(|_| {
                    serde_json::Value::String(format!("{action:?}"))
                }),
                "result": serde_json::to_value(result).unwrap_or_else(|_| {
                    serde_json::Value::String(format!("{result:?}"))
                }),
            }),
        ),
        GuardianEvent::ThresholdBreached {
            metric,
            value,
            threshold,
        } => (
            "threshold_breached",
            serde_json::json!({
                "metric": metric,
                "value": value,
                "threshold": threshold,
            }),
        ),
    };

    WsMessage {
        msg_type: msg_type.to_string(),
        payload,
        timestamp: DateTime::now().to_rfc3339(),
    }
}

// =============================================================================
// WebSocket Handler
// =============================================================================

/// WebSocket upgrade handler for the Guardian EventBus bridge.
///
/// Accepts a WebSocket upgrade request and spawns the bidirectional
/// event streaming loop. Mounted at `/api/v1/guardian/ws/bridge`.
pub async fn ws_bridge_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    tracing::info!("Guardian WS bridge: new connection request");
    ws.on_upgrade(handle_bridge_socket)
}

/// Core WebSocket loop: bridges EventBus events to the client and
/// accepts optional commands from the client.
async fn handle_bridge_socket(mut socket: WebSocket) {
    let bus = get_event_bus();
    let mut rx = bus.subscribe();

    tracing::info!(
        "Guardian WS bridge: client connected (active receivers: {})",
        bus.receiver_count()
    );

    // Send a welcome/status message so the client knows the connection is live
    let welcome = WsMessage {
        msg_type: "connected".to_string(),
        payload: serde_json::json!({
            "bus_capacity": bus.capacity(),
            "active_receivers": bus.receiver_count(),
        }),
        timestamp: DateTime::now().to_rfc3339(),
    };
    if let Ok(json) = serde_json::to_string(&welcome) {
        if socket.send(Message::Text(json.into())).await.is_err() {
            tracing::warn!("Guardian WS bridge: client disconnected during welcome");
            return;
        }
    }

    // Bidirectional event loop
    loop {
        tokio::select! {
            // EventBus -> WebSocket (server push)
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        let ws_msg = map_event_to_ws_message(&event);
                        if let Ok(json) = serde_json::to_string(&ws_msg) {
                            if socket.send(Message::Text(json.into())).await.is_err() {
                                tracing::info!("Guardian WS bridge: client disconnected (send failed)");
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(
                            "Guardian WS bridge: subscriber lagged by {n} events, continuing"
                        );
                        // Notify client about the lag
                        let lag_msg = WsMessage {
                            msg_type: "lag_warning".to_string(),
                            payload: serde_json::json!({
                                "missed_events": n,
                                "message": "Subscriber fell behind; some events were dropped",
                            }),
                            timestamp: DateTime::now().to_rfc3339(),
                        };
                        if let Ok(json) = serde_json::to_string(&lag_msg) {
                            // Best-effort lag notification
                            let _ = socket.send(Message::Text(json.into())).await;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        tracing::info!("Guardian WS bridge: event bus closed, terminating");
                        break;
                    }
                }
            }

            // WebSocket -> Server (client commands)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_client_command(&text, &mut socket).await;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            tracing::info!("Guardian WS bridge: client disconnected (pong failed)");
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!("Guardian WS bridge: client disconnected (close/eof)");
                        break;
                    }
                    _ => {
                        // Binary, Pong, or other frame types -- ignore
                    }
                }
            }
        }
    }

    tracing::info!(
        "Guardian WS bridge: connection closed (remaining receivers: {})",
        bus.receiver_count().saturating_sub(1)
    );
    // rx is dropped here, automatically unsubscribing from the broadcast
}

/// Handle a text command from a WebSocket client.
///
/// Supported commands:
/// - `{"command": "pause"}` - Pause the homeostasis loop
/// - `{"command": "resume"}` - Resume the homeostasis loop
/// - `{"command": "status"}` - Request current loop status
///
/// Responses are sent as `WsMessage` with `msg_type: "command_response"`.
async fn handle_client_command(text: &str, socket: &mut WebSocket) {
    let cmd: WsCommand = match serde_json::from_str(text) {
        Ok(c) => c,
        Err(_) => {
            let err_msg = WsMessage {
                msg_type: "error".to_string(),
                payload: serde_json::json!({
                    "message": "Invalid command format. Expected: {\"command\": \"pause|resume|status\"}",
                    "received": text,
                }),
                timestamp: DateTime::now().to_rfc3339(),
            };
            if let Ok(json) = serde_json::to_string(&err_msg) {
                let _ = socket.send(Message::Text(json.into())).await;
            }
            return;
        }
    };

    let response_payload = match cmd.command.as_str() {
        "pause" => {
            // Delegate to the homeostasis loop via the guardian module's get_loop()
            // We access it through the same static as the REST handlers
            let control_loop = super::guardian::get_loop().lock().await;
            // pause() requires &mut, but we only have shared access via the
            // module-level function. The REST pause/resume handlers hold
            // the mutex. For WS commands, we publish through the event bus
            // and let the REST handler do the actual mutation.
            //
            // Direct pause: requires re-exporting a pause helper. For now,
            // we document that WS pause/resume are advisory and the client
            // should use the REST endpoints for authoritative control.
            drop(control_loop);
            serde_json::json!({
                "command": "pause",
                "status": "advisory",
                "message": "Use POST /api/v1/guardian/pause for authoritative control. \
                            WS commands are read-only in this version.",
            })
        }
        "resume" => {
            serde_json::json!({
                "command": "resume",
                "status": "advisory",
                "message": "Use POST /api/v1/guardian/resume for authoritative control. \
                            WS commands are read-only in this version.",
            })
        }
        "status" => {
            let bus = get_event_bus();
            serde_json::json!({
                "command": "status",
                "bus_capacity": bus.capacity(),
                "active_receivers": bus.receiver_count(),
            })
        }
        other => {
            serde_json::json!({
                "command": other,
                "status": "unknown",
                "message": format!("Unknown command '{}'. Supported: pause, resume, status", other),
            })
        }
    };

    let response = WsMessage {
        msg_type: "command_response".to_string(),
        payload: response_payload,
        timestamp: DateTime::now().to_rfc3339(),
    };
    if let Ok(json) = serde_json::to_string(&response) {
        let _ = socket.send(Message::Text(json.into())).await;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage {
            msg_type: "loop_tick".to_string(),
            payload: serde_json::json!({"iteration_id": "iter-1"}),
            timestamp: "2026-02-05T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&msg);
        assert!(json.is_ok());
        let json_str = json.unwrap_or_default();
        assert!(json_str.contains("loop_tick"));
        assert!(json_str.contains("iter-1"));
    }

    #[test]
    fn test_event_bus_initialization() {
        let bus1 = get_event_bus();
        let bus2 = get_event_bus();
        // Same Arc (same underlying EventBus)
        assert_eq!(bus1.capacity(), bus2.capacity());
        assert_eq!(bus1.capacity(), 1024);
    }

    #[test]
    fn test_map_threshold_breached() {
        let event = GuardianEvent::ThresholdBreached {
            metric: "prr".to_string(),
            value: 3.5,
            threshold: 2.0,
        };
        let ws_msg = map_event_to_ws_message(&event);
        assert_eq!(ws_msg.msg_type, "threshold_breached");
        assert_eq!(ws_msg.payload["metric"], "prr");
    }

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        use nexcore_vigilance::guardian::event_bus::EventBus;
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        let event = GuardianEvent::ThresholdBreached {
            metric: "ic025".to_string(),
            value: 1.5,
            threshold: 0.0,
        };

        // Publish
        let result = bus.publish(event);
        assert!(result.is_ok());

        // Receive
        let received = rx.recv().await;
        assert!(received.is_ok());
        if let Ok(GuardianEvent::ThresholdBreached { metric, .. }) = received {
            assert_eq!(metric, "ic025");
        } else {
            panic!("Expected ThresholdBreached");
        }
    }

    #[test]
    fn test_ws_command_deserialization() {
        let json = r#"{"command": "pause"}"#;
        let cmd: Result<WsCommand, _> = serde_json::from_str(json);
        assert!(cmd.is_ok());
        let cmd = cmd.unwrap_or_else(|_| WsCommand {
            command: String::new(),
        });
        assert_eq!(cmd.command, "pause");
    }

    #[test]
    fn test_ws_command_invalid() {
        let json = r#"{"not_a_command": true}"#;
        let cmd: Result<WsCommand, _> = serde_json::from_str(json);
        // serde will fail because `command` is required
        assert!(cmd.is_err());
    }

    #[test]
    fn test_map_all_event_variants() {
        // Verify all GuardianEvent variants produce valid WsMessages
        use nexcore_chrono::DateTime;
        use nexcore_primitives::measurement::Measured;
        use nexcore_vigilance::guardian::homeostasis::{
            ActuatorResultSummary as DomainActuatorSummary, LoopIterationResult, ThroughputMonitor,
        };
        use nexcore_vigilance::guardian::response::{ActuatorResult, ResponseAction};
        use nexcore_vigilance::guardian::sensing::{SignalSource, ThreatLevel, ThreatSignal};
        use std::collections::HashMap;

        let events = vec![
            GuardianEvent::LoopTick(LoopIterationResult {
                iteration_id: "iter-1".to_string(),
                timestamp: DateTime::now(),
                signals_detected: 2,
                actions_taken: 1,
                results: vec![DomainActuatorSummary {
                    actuator: "alert".to_string(),
                    success: true,
                    message: "ok".to_string(),
                }],
                duration_ms: 5,
                throughput: ThroughputMonitor::default(),
            }),
            GuardianEvent::SignalDetected(ThreatSignal {
                id: "sig-1".to_string(),
                pattern: "test-pattern".to_string(),
                severity: ThreatLevel::High,
                timestamp: DateTime::now(),
                source: SignalSource::Damp {
                    subsystem: "test".to_string(),
                    damage_type: "test".to_string(),
                },
                confidence: Measured::certain(0.9),
                metadata: HashMap::new(),
            }),
            GuardianEvent::ActionTaken {
                action: ResponseAction::Alert {
                    severity: ThreatLevel::Medium,
                    message: "test alert".to_string(),
                    recipients: vec!["ops@test.com".to_string()],
                },
                result: ActuatorResult::success("delivered"),
            },
            GuardianEvent::ThresholdBreached {
                metric: "eb05".to_string(),
                value: 2.5,
                threshold: 2.0,
            },
        ];

        let expected_types = [
            "loop_tick",
            "signal_detected",
            "action_taken",
            "threshold_breached",
        ];

        for (event, expected_type) in events.iter().zip(expected_types.iter()) {
            let ws_msg = map_event_to_ws_message(event);
            assert_eq!(ws_msg.msg_type, *expected_type);
            // Verify the whole WsMessage serializes cleanly
            let json = serde_json::to_string(&ws_msg);
            assert!(
                json.is_ok(),
                "Failed to serialize WsMessage for {expected_type}"
            );
        }
    }
}
