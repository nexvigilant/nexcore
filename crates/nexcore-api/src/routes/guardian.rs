//! Guardian Homeostasis Control Loop endpoints
//!
//! Biological-inspired security control system:
//! - Sensing: PAMPs (external threats) / DAMPs (internal damage)
//! - Decision: Risk evaluation with amplification/ceiling
//! - Response: Actuators for alerts, blocks, escalations
//! - WebSocket: Real-time state updates to connected clients

use axum::{
    Json, Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
};
use nexcore_vigilance::guardian::event_bus::GuardianEvent as DomainGuardianEvent;
use nexcore_vigilance::guardian::homeostasis::{
    DecisionEngine, HomeostasisLoop, LoopIterationResult, evaluate_pv_risk,
};
use nexcore_vigilance::guardian::response::{AlertActuator, AuditLogActuator};
use nexcore_vigilance::guardian::sensing::biological::BiologicalVitalSignsSensor;
use nexcore_vigilance::guardian::sensing::{ExternalSensor, InternalSensor};
use nexcore_vigilance::guardian::{OriginatorType, RiskContext};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::sync::{Mutex, broadcast};
use utoipa::ToSchema;

use super::common::ApiResult;

/// Global homeostasis loop instance
static HOMEOSTASIS_LOOP: OnceLock<Mutex<HomeostasisLoop>> = OnceLock::new();

/// Broadcast channel for WebSocket updates (capacity: 100 messages)
static BROADCAST_TX: OnceLock<broadcast::Sender<GuardianEvent>> = OnceLock::new();

/// Get or initialize the broadcast channel
fn get_broadcast() -> broadcast::Sender<GuardianEvent> {
    BROADCAST_TX
        .get_or_init(|| {
            let (tx, _) = broadcast::channel(100);
            tx
        })
        .clone()
}

// ============================================================================
// WebSocket Event Types
// ============================================================================

/// Events broadcast to WebSocket clients
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
#[allow(dead_code)]
pub enum GuardianEvent {
    /// Homeostasis tick completed
    Tick(TickResponse),
    /// Risk evaluation completed
    Evaluation(EvaluatePvResponse),
    /// System status update
    Status(StatusResponse),
    /// System reset
    Reset(ResetResponse),
    /// Heartbeat to keep connection alive
    Heartbeat { timestamp: String },
}

/// Get or initialize the global homeostasis loop
pub(crate) fn get_loop() -> &'static Mutex<HomeostasisLoop> {
    HOMEOSTASIS_LOOP.get_or_init(|| {
        let mut control_loop = HomeostasisLoop::new(DecisionEngine::new());
        control_loop.add_sensor(ExternalSensor::new());
        control_loop.add_sensor(InternalSensor::new());
        control_loop.add_sensor(BiologicalVitalSignsSensor::new());
        control_loop.add_actuator(AlertActuator::new());
        control_loop.add_actuator(AuditLogActuator::new());
        Mutex::new(control_loop)
    })
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Homeostasis tick response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TickResponse {
    /// Iteration identifier
    pub iteration_id: String,
    /// Timestamp of iteration (ISO8601)
    pub timestamp: String,
    /// Number of signals detected
    pub signals_detected: usize,
    /// Number of actions taken
    pub actions_taken: usize,
    /// Actuator results
    pub results: Vec<ActuatorResultSummary>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Actuator result summary
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ActuatorResultSummary {
    /// Actuator name
    pub actuator: String,
    /// Success status
    pub success: bool,
    /// Result message
    pub message: String,
}

/// PV risk evaluation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct EvaluatePvRequest {
    /// Drug name
    pub drug: String,
    /// Adverse event name
    pub event: String,
    /// PRR (Proportional Reporting Ratio) value
    pub prr: f64,
    /// ROR lower confidence interval
    pub ror_lower: f64,
    /// IC 2.5th percentile
    pub ic025: f64,
    /// EB05 (EBGM 5th percentile)
    pub eb05: f64,
    /// Number of cases
    pub n: u64,
}

/// PV risk evaluation response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EvaluatePvResponse {
    /// Drug name
    pub drug: String,
    /// Event name
    pub event: String,
    /// Risk score details
    pub risk_score: RiskScoreDetails,
    /// Recommended response actions
    pub recommended_actions: Vec<ActionSummary>,
}

/// Risk score details
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RiskScoreDetails {
    /// Overall score (0-100)
    pub score: f64,
    /// Risk level (Critical/High/Medium/Low)
    pub level: String,
    /// Contributing factors
    pub factors: Vec<String>,
}

/// Action summary
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ActionSummary {
    /// Action type
    pub action_type: String,
    /// Action description
    pub description: String,
}

/// Guardian status response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StatusResponse {
    /// Number of iterations run
    pub iteration_count: u64,
    /// Number of registered sensors
    pub sensor_count: usize,
    /// Number of registered actuators
    pub actuator_count: usize,
    /// Registered sensors info
    pub sensors: Vec<SensorInfo>,
    /// Registered actuators info
    pub actuators: Vec<ActuatorInfo>,
    /// System status
    pub status: String,
    /// Whether the loop is currently paused
    pub paused: bool,
    /// Current risk threshold (0-100)
    pub risk_threshold: f64,
}

/// Sensor information
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SensorInfo {
    /// Sensor name
    pub name: String,
    /// Sensor type (PAMPs/DAMPs/Hybrid)
    pub sensor_type: String,
    /// Description
    pub description: String,
}

/// Actuator information
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ActuatorInfo {
    /// Actuator name
    pub name: String,
    /// Priority (higher = executes first)
    pub priority: u8,
    /// Description
    pub description: String,
}

/// Reset response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ResetResponse {
    /// Status message
    pub status: String,
    /// Reset message
    pub message: String,
    /// New iteration count (should be 0)
    pub iteration_count: u64,
}

/// Request to update the decision engine risk threshold
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateThresholdRequest {
    /// Metric name (currently only "risk_threshold" is supported)
    pub metric: String,
    /// New threshold value (clamped to 0-100)
    pub value: f64,
}

/// Response from a threshold update
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ThresholdResponse {
    /// Metric that was updated
    pub metric: String,
    /// Previous threshold value
    pub old_value: f64,
    /// New threshold value (after clamping)
    pub new_value: f64,
}

/// Response from a pause or resume command
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PauseResumeResponse {
    /// Whether the loop is currently paused
    pub paused: bool,
    /// Human-readable status message
    pub message: String,
    /// Current iteration count (unchanged by pause/resume)
    pub iteration_count: u64,
}

// ============================================================================
// Router
// ============================================================================

/// Guardian router
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/tick", post(tick))
        .route("/evaluate", post(evaluate_pv))
        .route("/status", get(status))
        .route("/reset", post(reset))
        .route("/threshold", post(update_threshold))
        .route("/pause", post(pause))
        .route("/resume", post(resume))
        .route("/ws", get(ws_handler))
}

// ============================================================================
// WebSocket Handler
// ============================================================================

/// WebSocket connection handler for real-time Guardian updates
pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handle a WebSocket connection
async fn handle_socket(mut socket: WebSocket) {
    // Subscribe to the broadcast channel
    let mut rx = get_broadcast().subscribe();

    // Send initial status (best-effort, client just connected)
    let event = GuardianEvent::Status(get_current_status().await);
    if let Ok(json) = serde_json::to_string(&event) {
        // Best-effort initial send - if it fails, client will get updates anyway
        if socket.send(Message::Text(json.into())).await.is_err() {
            return; // Client disconnected immediately
        }
    }

    // Stream events to the client
    loop {
        tokio::select! {
            // Receive from broadcast channel and forward to WebSocket
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        if let Ok(json) = serde_json::to_string(&event) {
                            if socket.send(Message::Text(json.into())).await.is_err() {
                                break; // Client disconnected
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Missed some messages, continue
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break; // Channel closed
                    }
                }
            }
            // Handle incoming WebSocket messages (ping/pong, close)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        // Best-effort pong - if send fails, connection is dead anyway
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    _ => {} // Ignore other messages
                }
            }
        }
    }
}

/// Build sensor/actuator info lists (avoids duplication across handlers)
fn build_sensor_info() -> Vec<SensorInfo> {
    vec![
        SensorInfo {
            name: "external-sensor".to_string(),
            sensor_type: "PAMPs".to_string(),
            description: "External threat detection (API attacks, injection attempts)".to_string(),
        },
        SensorInfo {
            name: "internal-sensor".to_string(),
            sensor_type: "DAMPs".to_string(),
            description: "Internal damage detection (memory leaks, service failures)".to_string(),
        },
    ]
}

fn build_actuator_info() -> Vec<ActuatorInfo> {
    vec![
        ActuatorInfo {
            name: "alert-actuator".to_string(),
            priority: 80,
            description: "Send alert notifications".to_string(),
        },
        ActuatorInfo {
            name: "audit-log-actuator".to_string(),
            priority: 100,
            description: "Record audit log entries".to_string(),
        },
    ]
}

/// Get current status without going through the API result wrapper
async fn get_current_status() -> StatusResponse {
    let control_loop = get_loop().lock().await;

    let status_label = if control_loop.is_paused() {
        "paused"
    } else {
        "healthy"
    };

    StatusResponse {
        iteration_count: control_loop.iteration_count(),
        sensor_count: control_loop.sensor_count(),
        actuator_count: control_loop.actuator_count(),
        sensors: build_sensor_info(),
        actuators: build_actuator_info(),
        status: status_label.to_string(),
        paused: control_loop.is_paused(),
        risk_threshold: control_loop.get_threshold(),
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// Run one iteration of the homeostasis control loop
#[utoipa::path(
    post,
    path = "/api/v1/guardian/tick",
    tag = "guardian",
    responses(
        (status = 200, description = "Tick completed successfully", body = TickResponse)
    )
)]
pub async fn tick() -> ApiResult<TickResponse> {
    let mut control_loop = get_loop().lock().await;

    let result: LoopIterationResult = control_loop.tick().await;

    // Clone for the EventBus bridge before fields are moved into the API response
    let result_for_bus = result.clone();

    let results: Vec<ActuatorResultSummary> = result
        .results
        .iter()
        .map(|r| ActuatorResultSummary {
            actuator: r.actuator.clone(),
            success: r.success,
            message: r.message.clone(),
        })
        .collect();

    let response = TickResponse {
        iteration_id: result.iteration_id,
        timestamp: result.timestamp.to_rfc3339(),
        signals_detected: result.signals_detected,
        actions_taken: result.actions_taken,
        results,
        duration_ms: result.duration_ms,
    };

    // Broadcast to local WebSocket clients - ok if no subscribers connected
    #[allow(unused_results)] // Reason: best-effort broadcast, no subscribers is valid state
    let _ = get_broadcast().send(GuardianEvent::Tick(response.clone()));

    // Publish to EventBus (CAP-P2-001 bridge) - domain-typed event for WS bridge clients
    let _ =
        super::guardian_ws::get_event_bus().publish(DomainGuardianEvent::LoopTick(result_for_bus));

    Ok(Json(response))
}

/// Evaluate PV risk and get recommended responses
#[utoipa::path(
    post,
    path = "/api/v1/guardian/evaluate",
    tag = "guardian",
    request_body = EvaluatePvRequest,
    responses(
        (status = 200, description = "Risk evaluation complete", body = EvaluatePvResponse)
    )
)]
pub async fn evaluate_pv(Json(req): Json<EvaluatePvRequest>) -> ApiResult<EvaluatePvResponse> {
    let context = RiskContext {
        drug: req.drug.clone(),
        event: req.event.clone(),
        prr: req.prr,
        ror_lower: req.ror_lower,
        ic025: req.ic025,
        eb05: req.eb05,
        n: req.n,
        originator: OriginatorType::default(),
    };

    let (score, actions) = evaluate_pv_risk(&context);

    let action_summaries: Vec<ActionSummary> = actions
        .iter()
        .map(|a| {
            let debug_str = format!("{:?}", a);
            let action_type = debug_str
                .split('{')
                .next()
                .unwrap_or("Unknown")
                .trim()
                .to_string();
            ActionSummary {
                action_type,
                description: debug_str,
            }
        })
        .collect();

    let response = EvaluatePvResponse {
        drug: req.drug,
        event: req.event,
        risk_score: RiskScoreDetails {
            score: score.score.value,
            level: score.level,
            factors: score.factors,
        },
        recommended_actions: action_summaries,
    };

    // Broadcast to WebSocket clients - ok if no subscribers connected
    #[allow(unused_results)] // Reason: best-effort broadcast, no subscribers is valid state
    let _ = get_broadcast().send(GuardianEvent::Evaluation(response.clone()));

    Ok(Json(response))
}

/// Get homeostasis loop status
#[utoipa::path(
    get,
    path = "/api/v1/guardian/status",
    tag = "guardian",
    responses(
        (status = 200, description = "Status retrieved", body = StatusResponse)
    )
)]
pub async fn status() -> ApiResult<StatusResponse> {
    let control_loop = get_loop().lock().await;

    let status_label = if control_loop.is_paused() {
        "paused"
    } else {
        "healthy"
    };

    Ok(Json(StatusResponse {
        iteration_count: control_loop.iteration_count(),
        sensor_count: control_loop.sensor_count(),
        actuator_count: control_loop.actuator_count(),
        sensors: build_sensor_info(),
        actuators: build_actuator_info(),
        status: status_label.to_string(),
        paused: control_loop.is_paused(),
        risk_threshold: control_loop.get_threshold(),
    }))
}

/// Reset the homeostasis loop state
#[utoipa::path(
    post,
    path = "/api/v1/guardian/reset",
    tag = "guardian",
    responses(
        (status = 200, description = "Loop reset successfully", body = ResetResponse)
    )
)]
pub async fn reset() -> ApiResult<ResetResponse> {
    let mut control_loop = get_loop().lock().await;

    control_loop.reset();

    let response = ResetResponse {
        status: "reset".to_string(),
        message: "Homeostasis loop state reset successfully".to_string(),
        iteration_count: control_loop.iteration_count(),
    };

    // Broadcast to WebSocket clients - ok if no subscribers connected
    #[allow(unused_results)] // Reason: best-effort broadcast, no subscribers is valid state
    let _ = get_broadcast().send(GuardianEvent::Reset(response.clone()));

    Ok(Json(response))
}

/// Update a Guardian decision engine threshold
///
/// Currently supports the `risk_threshold` metric. The value is clamped
/// to the 0-100 range by the decision engine.
#[utoipa::path(
    post,
    path = "/api/v1/guardian/threshold",
    tag = "guardian",
    request_body = UpdateThresholdRequest,
    responses(
        (status = 200, description = "Threshold updated successfully", body = ThresholdResponse),
        (status = 400, description = "Unknown metric name", body = super::common::ApiError)
    )
)]
pub async fn update_threshold(
    Json(req): Json<UpdateThresholdRequest>,
) -> ApiResult<ThresholdResponse> {
    if req.metric != "risk_threshold" {
        return Err(super::common::ApiError::new(
            "VALIDATION_ERROR",
            format!("Unknown metric '{}'. Supported: risk_threshold", req.metric),
        ));
    }

    let mut control_loop = get_loop().lock().await;
    let old_value = control_loop.set_threshold(req.value);
    let new_value = control_loop.get_threshold();

    Ok(Json(ThresholdResponse {
        metric: req.metric,
        old_value,
        new_value,
    }))
}

/// Pause the Guardian homeostasis loop
///
/// While paused, `tick` returns empty results without running sensing,
/// decision, or response phases.
#[utoipa::path(
    post,
    path = "/api/v1/guardian/pause",
    tag = "guardian",
    responses(
        (status = 200, description = "Loop paused", body = PauseResumeResponse)
    )
)]
pub async fn pause() -> ApiResult<PauseResumeResponse> {
    let mut control_loop = get_loop().lock().await;
    control_loop.pause();

    Ok(Json(PauseResumeResponse {
        paused: true,
        message: "Guardian homeostasis loop paused".to_string(),
        iteration_count: control_loop.iteration_count(),
    }))
}

/// Resume the Guardian homeostasis loop
///
/// Re-enables sensing, decision, and response phases on subsequent ticks.
#[utoipa::path(
    post,
    path = "/api/v1/guardian/resume",
    tag = "guardian",
    responses(
        (status = 200, description = "Loop resumed", body = PauseResumeResponse)
    )
)]
pub async fn resume() -> ApiResult<PauseResumeResponse> {
    let mut control_loop = get_loop().lock().await;
    control_loop.resume();

    Ok(Json(PauseResumeResponse {
        paused: false,
        message: "Guardian homeostasis loop resumed".to_string(),
        iteration_count: control_loop.iteration_count(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::common::ApiError;
    use std::sync::OnceLock;
    use tokio::sync::Mutex;

    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    async fn test_lock() -> tokio::sync::MutexGuard<'static, ()> {
        TEST_MUTEX.get_or_init(|| Mutex::new(())).lock().await
    }

    #[tokio::test]
    async fn test_status() -> Result<(), ApiError> {
        let _guard = test_lock().await;
        let response = status().await?.0;
        assert_eq!(response.status, "healthy");
        Ok(())
    }

    #[tokio::test]
    async fn test_tick() -> Result<(), ApiError> {
        let _guard = test_lock().await;
        tick().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_evaluate_pv() -> Result<(), ApiError> {
        let _guard = test_lock().await;
        let req = EvaluatePvRequest {
            drug: "TestDrug".to_string(),
            event: "TestEvent".to_string(),
            prr: 3.5,
            ror_lower: 2.0,
            ic025: 0.5,
            eb05: 2.5,
            n: 10,
        };
        let response = evaluate_pv(Json(req)).await?.0;
        assert_eq!(response.risk_score.level, "Critical");
        Ok(())
    }

    #[tokio::test]
    async fn test_reset() -> Result<(), ApiError> {
        let _guard = test_lock().await;
        let response = reset().await?.0;
        assert_eq!(response.status, "reset");
        Ok(())
    }

    #[tokio::test]
    async fn test_update_threshold() -> Result<(), ApiError> {
        let _guard = test_lock().await;
        // First reset to known state
        reset().await?;

        let req = UpdateThresholdRequest {
            metric: "risk_threshold".to_string(),
            value: 75.0,
        };
        let response = update_threshold(Json(req)).await?.0;
        assert_eq!(response.metric, "risk_threshold");
        assert!((response.new_value - 75.0).abs() < f64::EPSILON);

        // Verify unknown metric returns error
        let bad_req = UpdateThresholdRequest {
            metric: "unknown_metric".to_string(),
            value: 50.0,
        };
        let result = update_threshold(Json(bad_req)).await;
        assert!(result.is_err());

        // Reset back to default
        reset().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_pause_resume_loop() -> Result<(), ApiError> {
        let _guard = test_lock().await;
        // Reset to known state
        reset().await?;

        // Pause the loop
        let pause_resp = pause().await?.0;
        assert!(pause_resp.paused);

        // Status should reflect paused
        let status_resp = status().await?.0;
        assert!(status_resp.paused);
        assert_eq!(status_resp.status, "paused");

        // Tick while paused should return zero signals/actions
        let tick_resp = tick().await?.0;
        assert_eq!(tick_resp.signals_detected, 0);
        assert_eq!(tick_resp.actions_taken, 0);
        assert!(tick_resp.iteration_id.contains("paused"));

        // Resume
        let resume_resp = resume().await?.0;
        assert!(!resume_resp.paused);

        // Status should reflect healthy again
        let status_resp = status().await?.0;
        assert!(!status_resp.paused);
        assert_eq!(status_resp.status, "healthy");

        // Reset
        reset().await?;

        Ok(())
    }
}
