//! Guardian tools: Homeostasis control loop, threat sensing, and response execution
//!
//! Biological-inspired security control system with:
//! - Sensing: PAMPs (external threats) / DAMPs (internal damage)
//! - Decision: Risk evaluation with amplification/ceiling
//! - Response: Actuators for alerts, blocks, escalations

use crate::params::{
    FdaBridgeBatchParams, FdaBridgeEvaluateParams, GuardianActuatorsListParams,
    GuardianCeilingForOriginatorParams, GuardianEvaluatePvParams, GuardianHistoryParams,
    GuardianInjectSignalParams, GuardianOriginatorClassifyParams, GuardianSensorsListParams,
    GuardianSpace3DComputeParams, GuardianStatusParams, GuardianSubscribeParams,
    GuardianTickParams, PvControlLoopTickParams,
};
use nexcore_vigilance::control::{FdaDataBridge, LoopMetrics, PvControlLoop, PvSafetyState};
use nexcore_vigilance::guardian::event_bus::{EventBus, GuardianEvent};
use nexcore_vigilance::guardian::homeostasis::{
    DecisionEngine, HomeostasisLoop, LoopIterationResult, evaluate_pv_risk,
};
use nexcore_vigilance::guardian::response::ResponseCeiling;
use nexcore_vigilance::guardian::response::{AlertActuator, AuditLogActuator, CytokineActuator};
use nexcore_vigilance::guardian::sensing::biological::BiologicalVitalSignsSensor;
use nexcore_vigilance::guardian::sensing::{
    ExternalSensor, InternalSensor, SignalSource, ThreatLevel, ThreatSignal,
};
use nexcore_vigilance::guardian::space3d::{SafetySpace3DInput, compute_safety_point};
use nexcore_vigilance::guardian::{OriginatorType, RiskContext};
use nexcore_vigilance::tov::HarmType;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::Mutex;

/// Global homeostasis loop instance (lazily initialized)
static HOMEOSTASIS_LOOP: OnceLock<Mutex<HomeostasisLoop>> = OnceLock::new();

/// Global cytokine bus as Arc (for CytokineActuator)
static CYTOKINE_BUS_ARC: OnceLock<Arc<nexcore_cytokine::CytokineBus>> = OnceLock::new();

/// Event history buffer for Guardian (circular buffer of last N events)
static EVENT_HISTORY: OnceLock<parking_lot::Mutex<Vec<Value>>> = OnceLock::new();

/// Guardian EventBus (local to MCP process)
static EVENT_BUS: OnceLock<EventBus> = OnceLock::new();

const MAX_HISTORY_SIZE: usize = 1000;
const DEFAULT_SUBSCRIBE_LIMIT: usize = 50;

fn get_history() -> &'static parking_lot::Mutex<Vec<Value>> {
    EVENT_HISTORY.get_or_init(|| parking_lot::Mutex::new(Vec::with_capacity(MAX_HISTORY_SIZE)))
}

fn get_event_bus() -> &'static EventBus {
    EVENT_BUS.get_or_init(EventBus::default)
}

fn get_cytokine_bus_arc() -> &'static Arc<nexcore_cytokine::CytokineBus> {
    CYTOKINE_BUS_ARC.get_or_init(|| {
        // Create a new bus instance that's independent of global_bus()
        // This ensures Guardian has a dedicated bus for actuator emissions
        Arc::new(nexcore_cytokine::CytokineBus::new("guardian-mcp"))
    })
}

fn push_event(event: Value) {
    let mut history = get_history().lock();
    if history.len() >= MAX_HISTORY_SIZE {
        history.remove(0);
    }
    history.push(event);
}

fn publish_event(event: GuardianEvent) {
    let value = serde_json::to_value(&event)
        .unwrap_or_else(|_| json!({ "error": "guardian_event_serialize_failed" }));
    push_event(value);
    let _ = get_event_bus().publish(event);
}

fn threat_level_from_severity(severity: f64) -> ThreatLevel {
    if severity < 0.2 {
        ThreatLevel::Info
    } else if severity < 0.4 {
        ThreatLevel::Low
    } else if severity < 0.6 {
        ThreatLevel::Medium
    } else if severity < 0.8 {
        ThreatLevel::High
    } else {
        ThreatLevel::Critical
    }
}

fn signal_source_from_input(source: &str, pattern: &str) -> SignalSource {
    match source.to_lowercase().as_str() {
        "external" | "pamp" => SignalSource::Pamp {
            source_id: "mcp".to_string(),
            vector: pattern.to_string(),
        },
        "internal" | "damp" => SignalSource::Damp {
            subsystem: "guardian".to_string(),
            damage_type: pattern.to_string(),
        },
        "pv" => SignalSource::Hybrid {
            external: "pv".to_string(),
            internal: pattern.to_string(),
        },
        _ => SignalSource::Hybrid {
            external: "unknown".to_string(),
            internal: pattern.to_string(),
        },
    }
}

fn event_variant(value: &Value) -> Option<&str> {
    value
        .as_object()
        .and_then(|o| o.keys().next().map(|k| k.as_str()))
}

fn event_matches_filter(value: &Value, filter: &str) -> bool {
    let filter = filter.to_lowercase();
    if filter == "all" {
        return true;
    }
    match event_variant(value) {
        Some("SignalDetected") => filter == "signal" || filter == "signals",
        Some("ActionTaken") => filter == "action" || filter == "actions",
        Some("LoopTick") => filter == "tick" || filter == "ticks",
        Some("ThresholdBreached") => filter == "threshold" || filter == "thresholds",
        _ => false,
    }
}

/// Get or initialize the global homeostasis loop
pub(crate) fn get_loop() -> &'static Mutex<HomeostasisLoop> {
    HOMEOSTASIS_LOOP.get_or_init(|| {
        let mut control_loop = nexcore_guardian_engine::create_monitoring_loop();
        // Wire actuators
        control_loop.add_actuator(AlertActuator::new());
        control_loop.add_actuator(AuditLogActuator::new());
        // Wire cytokine actuator for Guardian→cytokine bus propagation
        control_loop.add_actuator(CytokineActuator::new(get_cytokine_bus_arc().clone()));
        Mutex::new(control_loop)
    })
}

/// Threat level numeric weight for comparison (only-escalate logic)
fn threat_level_weight(level: &str) -> u8 {
    match level {
        "Critical" => 4,
        "High" => 3,
        "Medium" => 2,
        "Low" => 1,
        _ => 0,
    }
}

/// Persist tick result to guardian-state.json so hooks see fresh data.
/// Best-effort: errors are silently ignored to not affect MCP response.
fn persist_tick_to_state(result: &LoopIterationResult) {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    let state_path = format!("{home}/.claude/hooks/state/guardian-state.json");

    // Read existing state (or start fresh)
    let mut state: serde_json::Map<String, Value> = std::fs::read_to_string(&state_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    // Increment iteration
    let iteration = state.get("iteration").and_then(|v| v.as_u64()).unwrap_or(0) + 1;
    state.insert("iteration".to_string(), json!(iteration));

    // Accumulate signals and actions (session-wide totals)
    let prev_signals = state
        .get("signals_detected")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.insert(
        "signals_detected".to_string(),
        json!(prev_signals + result.signals_detected as u64),
    );

    let prev_actions = state
        .get("actions_taken")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    state.insert(
        "actions_taken".to_string(),
        json!(prev_actions + result.actions_taken as u64),
    );

    // Tick threat level: homeostasis loop signals include routine sensor pings
    // (audit log, cytokine, vital signs). Only high action counts indicate real threat.
    // Threshold: >3 actions with results containing non-success = elevated
    let failed_actions = result.results.iter().filter(|r| !r.success).count();
    let tick_threat = if failed_actions > 3 {
        "High"
    } else if failed_actions > 0 {
        "Medium"
    } else {
        "Low"
    };

    // Only escalate max_threat_level, never de-escalate within session
    let current_max = state
        .get("max_threat_level")
        .and_then(|v| v.as_str())
        .unwrap_or("Low")
        .to_string();
    if threat_level_weight(tick_threat) > threat_level_weight(&current_max) {
        state.insert("max_threat_level".to_string(), json!(tick_threat));
    }

    // Write last_tick — the data guardian-gate checks for staleness
    state.insert(
        "last_tick".to_string(),
        json!({
            "iteration_id": result.iteration_id,
            "timestamp": result.timestamp.to_rfc3339(),
            "signals_detected": result.signals_detected,
            "actions_taken": result.actions_taken,
            "max_threat_level": tick_threat,
            "results": result.results.iter().map(|r| json!({
                "actuator": r.actuator,
                "success": r.success,
                "message": r.message,
            })).collect::<Vec<Value>>(),
            "duration_ms": result.duration_ms,
        }),
    );

    // Best-effort write — state persistence is non-critical for MCP response
    if let Ok(json_str) = serde_json::to_string_pretty(&state) {
        std::fs::write(&state_path, json_str).ok();
    }
}

/// Run one iteration of the homeostasis control loop
pub async fn homeostasis_tick(_params: GuardianTickParams) -> Result<CallToolResult, McpError> {
    let mut control_loop = get_loop().lock().await;
    let result: LoopIterationResult = control_loop.tick().await;

    publish_event(GuardianEvent::LoopTick(result.clone()));

    // Persist to guardian-state.json for hook consumption (best-effort)
    persist_tick_to_state(&result);

    let json = json!({
        "iteration_id": result.iteration_id,
        "timestamp": result.timestamp.to_rfc3339(),
        "signals_detected": result.signals_detected,
        "actions_taken": result.actions_taken,
        "results": result.results.iter().map(|r| json!({
            "actuator": r.actuator,
            "success": r.success,
            "message": r.message,
        })).collect::<Vec<_>>(),
        "duration_ms": result.duration_ms,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Evaluate PV risk context and get recommended responses
pub fn evaluate_pv(params: GuardianEvaluatePvParams) -> Result<CallToolResult, McpError> {
    let context = RiskContext {
        drug: params.drug.clone(),
        event: params.event.clone(),
        prr: params.prr,
        ror_lower: params.ror_lower,
        ic025: params.ic025,
        eb05: params.eb05,
        n: params.n,
        originator: OriginatorType::default(),
    };

    let (score, actions) = evaluate_pv_risk(&context);

    let action_summaries: Vec<_> = actions
        .iter()
        .map(|a| {
            json!({
                "type": format!("{:?}", a).split('{').next().unwrap_or("Unknown").trim(),
                "details": format!("{:?}", a),
            })
        })
        .collect();

    let json = json!({
        "drug": params.drug,
        "event": params.event,
        "risk_score": { "score": score.score, "level": score.level, "factors": score.factors },
        "recommended_actions": action_summaries,
        "inputs": { "prr": params.prr, "ror_lower": params.ror_lower, "ic025": params.ic025, "eb05": params.eb05, "n": params.n },
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get homeostasis loop status
pub async fn status(_params: GuardianStatusParams) -> Result<CallToolResult, McpError> {
    let control_loop = get_loop().lock().await;
    let json = json!({
        "iteration_count": control_loop.iteration_count(),
        "sensor_count": control_loop.sensor_count(),
        "actuator_count": control_loop.actuator_count(),
        "status": "healthy",
    });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Reset the homeostasis loop state
pub async fn reset() -> Result<CallToolResult, McpError> {
    let mut control_loop = get_loop().lock().await;
    control_loop.reset();
    let json = json!({ "status": "reset", "iteration_count": control_loop.iteration_count() });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Inject a test signal into the Guardian system
pub fn inject_signal(params: GuardianInjectSignalParams) -> Result<CallToolResult, McpError> {
    let signal = ThreatSignal::new(
        params.pattern.clone(),
        threat_level_from_severity(params.severity),
        signal_source_from_input(&params.source, &params.pattern),
    );
    let signal = if let Some(context) = &params.context {
        signal.with_metadata("context", context)
    } else {
        signal
    };
    publish_event(GuardianEvent::SignalDetected(signal.clone()));
    let event = json!({
        "type": "signal_injected",
        "source": params.source,
        "pattern": params.pattern,
        "severity": params.severity,
        "signal_id": signal.id,
    });
    Ok(CallToolResult::success(vec![Content::text(
        event.to_string(),
    )]))
}

/// List all registered sensors
pub async fn sensors_list(_params: GuardianSensorsListParams) -> Result<CallToolResult, McpError> {
    let control_loop = get_loop().lock().await;
    let json = json!({ "count": control_loop.sensor_count(), "status": "active" });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// List all registered actuators
pub async fn actuators_list(
    _params: GuardianActuatorsListParams,
) -> Result<CallToolResult, McpError> {
    let control_loop = get_loop().lock().await;
    let json = json!({ "count": control_loop.actuator_count(), "status": "active" });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get Guardian event history
pub fn history(params: GuardianHistoryParams) -> Result<CallToolResult, McpError> {
    let history = get_history().lock();
    let events: Vec<_> = history
        .iter()
        .rev()
        .filter(|e| event_matches_filter(e, &params.filter))
        .take(params.limit)
        .cloned()
        .collect();
    let response = json!({
        "total": history.len(),
        "events": events,
        "filter": params.filter,
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Subscribe to Guardian events (polling snapshot)
pub fn subscribe(params: GuardianSubscribeParams) -> Result<CallToolResult, McpError> {
    let history = get_history().lock();
    let events: Vec<_> = history
        .iter()
        .rev()
        .filter(|e| event_matches_filter(e, &params.events))
        .take(DEFAULT_SUBSCRIBE_LIMIT)
        .cloned()
        .collect();
    let response = json!({
        "events": events,
        "total": history.len(),
        "filter": params.events,
        "receiver_count": get_event_bus().receiver_count(),
        "note": "Polling snapshot; MCP does not stream events. Use guardian_history with a limit for paging.",
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// PV Control Loop
// ============================================================================

/// Build a response for the PV control loop tick
fn build_pv_loop_response(state: &PvSafetyState, action: &str, metrics: &LoopMetrics) -> Value {
    json!({
        "safety_state": { "case_count": state.case_count, "prr": state.prr, "ror": state.ror, "signal_detected": state.signal_detected },
        "recommended_action": action,
        "metrics": { "iterations": metrics.iterations, "actions_taken": metrics.actions_taken },
    })
}

/// Run one iteration of the PV control loop
pub fn pv_control_loop_tick(params: PvControlLoopTickParams) -> Result<CallToolResult, McpError> {
    let mut pv_loop = PvControlLoop::new();
    match pv_loop.tick_with_data(params.a, params.b, params.c, params.d) {
        Ok(state) => {
            let action = pv_loop
                .last_action()
                .map_or("None".to_string(), |a| format!("{:?}", a));
            let response = build_pv_loop_response(&state, &action, pv_loop.metrics());
            Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(
            json!({"error": e.message}).to_string(),
        )])),
    }
}

// ============================================================================
// FDA Data Bridge Tools
// ============================================================================

/// Evaluate a drug-event pair through the FDA Data Bridge
pub fn fda_bridge_evaluate(params: FdaBridgeEvaluateParams) -> Result<CallToolResult, McpError> {
    let mut bridge = FdaDataBridge::new();
    match bridge.evaluate(params.a, params.b, params.c, params.d) {
        Ok(result) => {
            let response = json!({ "severity": format!("{:?}", result.severity), "action": format!("{:?}", result.action), "summary": result.summary });
            Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(
            json!({"error": e.message}).to_string(),
        )])),
    }
}

/// Batch evaluate multiple drug-event pairs through the FDA Data Bridge
pub fn fda_bridge_batch(params: FdaBridgeBatchParams) -> Result<CallToolResult, McpError> {
    let mut bridge = FdaDataBridge::new();
    let pairs: Vec<_> = params
        .tables
        .iter()
        .map(|t| (t[0], t[1], t[2], t[3]))
        .collect();
    match bridge.evaluate_batch(pairs) {
        Ok(results) => {
            let count = results.len();
            let signals = results
                .iter()
                .filter(|r| r.safety_state.signal_detected)
                .count();
            Ok(CallToolResult::success(vec![Content::text(
                json!({"count": count, "signals": signals}).to_string(),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(
            json!({"error": e.message}).to_string(),
        )])),
    }
}

// ============================================================================
// {G, V, R} Framework Tools
// ============================================================================

/// Classify an entity by its {G, V, R} capabilities
pub fn originator_classify(
    params: GuardianOriginatorClassifyParams,
) -> Result<CallToolResult, McpError> {
    let originator = match (
        params.has_goal_selection,
        params.has_value_evaluation,
        params.has_refusal_capacity,
    ) {
        (false, false, false) => OriginatorType::Tool,
        (false, false, true) => OriginatorType::AgentWithR,
        (false, true, true) => OriginatorType::AgentWithVR,
        (true, false, true) => OriginatorType::AgentWithGR,
        (true, true, true) => OriginatorType::AgentWithGVR,
        _ => OriginatorType::Tool,
    };
    let response = json!({ "type": format!("{:?}", originator), "multiplier": originator.ceiling_multiplier() });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Get autonomy-aware ceiling limits for an originator type
pub fn ceiling_for_originator(
    params: GuardianCeilingForOriginatorParams,
) -> Result<CallToolResult, McpError> {
    let originator = match params.originator_type.to_lowercase().as_str() {
        "tool" => OriginatorType::Tool,
        "vr" => OriginatorType::AgentWithVR,
        "gvr" => OriginatorType::AgentWithGVR,
        _ => OriginatorType::Tool,
    };
    let multiplier = originator.ceiling_multiplier();
    let response = json!({ "type": format!("{:?}", originator), "multiplier": multiplier });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// 3D Safety Space Visualization
// ============================================================================

/// Compute a point in 3D safety space
pub fn space3d_compute(params: GuardianSpace3DComputeParams) -> Result<CallToolResult, McpError> {
    let input = SafetySpace3DInput {
        prr: params.prr,
        ror_lower: params.ror_lower,
        ic025: params.ic025,
        eb05: params.eb05,
        n: params.n,
        originator: OriginatorType::Tool,
        harm_type: None,
        hierarchy_level: params.hierarchy_level,
        signal_metrics_present: params.signal_metrics_present,
    };
    let point = compute_safety_point(&input);
    let response = json!({ "severity": point.severity.value, "likelihood": point.likelihood.value, "zone": format!("{:?}", point.zone) });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_homeostasis_tick() {
        assert!(homeostasis_tick(GuardianTickParams {}).await.is_ok());
    }
    #[test]
    fn test_evaluate_pv() {
        let p = GuardianEvaluatePvParams {
            drug: "D".into(),
            event: "E".into(),
            prr: 3.5,
            ror_lower: 2.0,
            ic025: 0.5,
            eb05: 2.5,
            n: 10,
        };
        assert!(evaluate_pv(p).is_ok());
    }
    #[tokio::test]
    async fn test_status() {
        assert!(status(GuardianStatusParams {}).await.is_ok());
    }
}
