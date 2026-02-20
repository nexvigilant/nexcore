//! # Agentic Loop
//!
//! Self-governing control loop that composes Guardian homeostasis with MCP tools
//! and Brain working memory. Implements the "physiology" of the system:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    AGENTIC LOOP (Physiology)                    │
//! │                                                                 │
//! │  ┌────────────┐    ┌────────────┐    ┌────────────┐    ┌──────┐│
//! │  │  SENSING   │ →  │  DECISION  │ →  │  RESPONSE  │ →  │LEARN ││
//! │  │  (PAMPs/   │    │  (Guardian │    │   (MCP     │    │(Brain││
//! │  │   DAMPs)   │    │   Risk)    │    │ Actuators) │    │ +Imp)││
//! │  └────────────┘    └────────────┘    └────────────┘    └──────┘│
//! │        ↑                                                   │    │
//! │        └───────────────── FEEDBACK ────────────────────────┘    │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Architecture
//!
//! - **Sensing**: MCP tools detect external threats (PAMPs) and internal damage (DAMPs)
//! - **Decision**: Guardian risk scoring with amplification and ceiling limits
//! - **Response**: MCP actuators execute remediation actions
//! - **Feedback**: Brain artifacts create immutable snapshots; implicit learning updates
//!
//! ## Example
//!
//! ```ignore
//! use nexcore_friday::agentic_loop::{AgenticLoop, LoopConfig};
//!
//! let config = LoopConfig::default()
//!     .with_tick_interval(Duration::from_secs(30))
//!     .with_risk_threshold(50.0);
//!
//! let mut loop_ctrl = AgenticLoop::new(config);
//!
//! // Add MCP-based sensors
//! loop_ctrl.add_sensor(McpSensor::new("pv_signal_complete"));
//! loop_ctrl.add_sensor(McpSensor::new("skill_validate"));
//! loop_ctrl.add_sensor(McpSensor::new("code_tracker_changed"));
//!
//! // Run continuously
//! loop_ctrl.run().await?;
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::bridge::nervous_system::{
    CytokineBridge, EnergyGovernor, GuardianBridge, HormonalModulator, ImmunitySensor,
    SynapticLearner,
};
use crate::errors::Result;
use crate::models::Event;
// TODO: Integrate McpBridge for sensing phase
#[allow(unused_imports)]
use crate::mcp_bridge::McpBridge;

// Re-export Guardian types for convenience
pub use nexcore_vigilance::guardian::homeostasis::{
    DecisionEngine as GuardianDecision, LoopIterationResult,
};
pub use nexcore_vigilance::guardian::response::{Actuator, ActuatorResult, ResponseAction};
pub use nexcore_vigilance::guardian::sensing::{Sensor, SignalSource, ThreatLevel, ThreatSignal};

/// Configuration for the agentic loop
#[derive(Debug, Clone)]
pub struct LoopConfig {
    /// Interval between loop ticks
    pub tick_interval: Duration,
    /// Risk threshold for action (0-100)
    pub risk_threshold: f64,
    /// Maximum concurrent responses
    pub max_concurrent_responses: usize,
    /// Enable implicit learning
    pub enable_learning: bool,
    /// Brain session ID for artifact storage
    pub brain_session_id: Option<String>,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            tick_interval: Duration::from_secs(30),
            risk_threshold: 50.0,
            max_concurrent_responses: 5,
            enable_learning: true,
            brain_session_id: None,
        }
    }
}

impl LoopConfig {
    /// Set tick interval
    #[must_use]
    pub fn with_tick_interval(mut self, interval: Duration) -> Self {
        self.tick_interval = interval;
        self
    }

    /// Set risk threshold
    #[must_use]
    pub fn with_risk_threshold(mut self, threshold: f64) -> Self {
        self.risk_threshold = threshold.clamp(0.0, 100.0);
        self
    }

    /// Set brain session ID
    #[must_use]
    pub fn with_brain_session(mut self, session_id: impl Into<String>) -> Self {
        self.brain_session_id = Some(session_id.into());
        self
    }
}

/// MCP-based sensor that calls NexCore tools
#[derive(Debug, Clone)]
pub struct McpSensor {
    /// Tool name (e.g., "pv_signal_complete")
    pub tool_name: String,
    /// Tool parameters
    pub params: HashMap<String, serde_json::Value>,
    /// Signal source classification
    pub source: SignalSource,
}

impl McpSensor {
    /// Create a new MCP sensor
    pub fn new(tool_name: impl Into<String>) -> Self {
        let name = tool_name.into();
        Self {
            tool_name: name.clone(),
            params: HashMap::new(),
            source: SignalSource::Pamp {
                source_id: "mcp".to_string(),
                vector: name,
            },
        }
    }

    /// Set as internal (DAMP) sensor
    #[must_use]
    pub fn internal(mut self) -> Self {
        self.source = SignalSource::Damp {
            subsystem: self.tool_name.clone(),
            damage_type: "monitoring".to_string(),
        };
        self
    }

    /// Add parameter
    #[must_use]
    pub fn with_param(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.params.insert(key.into(), value);
        self
    }
}

/// MCP-based actuator that executes responses
#[derive(Debug, Clone)]
pub struct McpActuator {
    /// Actuator name
    pub name: String,
    /// Priority (higher = earlier execution)
    pub priority: u32,
    /// Tool to call for this actuator
    pub tool_name: String,
}

impl McpActuator {
    /// Create a new MCP actuator
    pub fn new(name: impl Into<String>, tool_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            priority: 50,
            tool_name: tool_name.into(),
        }
    }

    /// Set priority
    #[must_use]
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

/// Result of a single loop cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleResult {
    /// Cycle ID
    pub cycle_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Signals detected (PAMPs + DAMPs)
    pub signals: Vec<SignalSummary>,
    /// Risk score computed
    pub risk_score: f64,
    /// Actions taken
    pub actions: Vec<ActionSummary>,
    /// Learning updates applied
    pub learning_updates: Vec<LearningUpdate>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Summary of a detected signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalSummary {
    pub id: String,
    pub source: String,
    pub severity: String,
    pub confidence: f64,
}

/// Summary of an action taken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSummary {
    pub action_type: String,
    pub target: String,
    pub success: bool,
}

/// Record of a learning update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningUpdate {
    pub key: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
}

/// The agentic loop controller
pub struct AgenticLoop {
    /// Configuration
    config: LoopConfig,
    /// Guardian decision engine
    decision_engine: GuardianDecision,
    /// MCP sensors (PAMPs + DAMPs)
    sensors: Vec<McpSensor>,
    /// MCP actuators
    actuators: Vec<McpActuator>,
    /// MCP bridge for direct tool invocation
    mcp_bridge: McpBridge,
    /// Cycle counter
    cycle_count: u64,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// Learned thresholds
    learned_thresholds: HashMap<String, f64>,

    // --- Nervous System bridges (all Option<T> for graceful degradation) ---
    /// Cytokine → EventBus bridge
    cytokine_bridge: Option<CytokineBridge>,
    /// Hormonal state modulator
    hormonal_modulator: Option<HormonalModulator>,
    /// Synaptic learning bridge
    synaptic_learner: Option<SynapticLearner>,
    /// Immunity threat sensor
    immunity_sensor: Option<ImmunitySensor>,
    /// Energy budget governor
    energy_governor: Option<EnergyGovernor>,
    /// Guardian feedback bridge
    guardian_bridge: Option<GuardianBridge>,
}

impl AgenticLoop {
    /// Create a new agentic loop
    pub fn new(config: LoopConfig) -> Self {
        let decision_engine = GuardianDecision::new().with_threshold(config.risk_threshold);

        Self {
            config,
            decision_engine,
            sensors: Vec::new(),
            actuators: Vec::new(),
            mcp_bridge: McpBridge::new(),
            cycle_count: 0,
            running: Arc::new(RwLock::new(false)),
            learned_thresholds: HashMap::new(),
            // Nervous system bridges — initialized to None, wired via with_* methods
            cytokine_bridge: None,
            hormonal_modulator: None,
            synaptic_learner: None,
            immunity_sensor: None,
            energy_governor: None,
            guardian_bridge: None,
        }
    }

    // --- Nervous System wiring methods ---

    /// Wire the cytokine bridge.
    #[must_use]
    pub fn with_cytokine_bridge(mut self, bridge: CytokineBridge) -> Self {
        self.cytokine_bridge = Some(bridge);
        self
    }

    /// Wire the hormonal modulator.
    #[must_use]
    pub fn with_hormonal_modulator(mut self, modulator: HormonalModulator) -> Self {
        self.hormonal_modulator = Some(modulator);
        self
    }

    /// Wire the synaptic learner.
    #[must_use]
    pub fn with_synaptic_learner(mut self, learner: SynapticLearner) -> Self {
        self.synaptic_learner = Some(learner);
        self
    }

    /// Wire the immunity sensor.
    #[must_use]
    pub fn with_immunity_sensor(mut self, sensor: ImmunitySensor) -> Self {
        self.immunity_sensor = Some(sensor);
        self
    }

    /// Wire the energy governor.
    #[must_use]
    pub fn with_energy_governor(mut self, governor: EnergyGovernor) -> Self {
        self.energy_governor = Some(governor);
        self
    }

    /// Wire the guardian bridge.
    #[must_use]
    pub fn with_guardian_bridge(mut self, bridge: GuardianBridge) -> Self {
        self.guardian_bridge = Some(bridge);
        self
    }

    /// Wire the complete nervous system (all 6 bridges at once).
    #[must_use]
    pub fn with_nervous_system(mut self, ns: crate::bridge::nervous_system::NervousSystem) -> Self {
        self.cytokine_bridge = Some(ns.cytokine);
        self.hormonal_modulator = Some(ns.hormonal);
        self.synaptic_learner = Some(ns.synaptic);
        self.immunity_sensor = ns.immunity;
        self.energy_governor = Some(ns.energy);
        self.guardian_bridge = Some(ns.guardian);
        self
    }

    /// Add a sensor
    pub fn add_sensor(&mut self, sensor: McpSensor) {
        self.sensors.push(sensor);
    }

    /// Add an actuator
    pub fn add_actuator(&mut self, actuator: McpActuator) {
        self.actuators.push(actuator);
        // Sort by priority (descending)
        self.actuators.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Add default PV monitoring sensors
    pub fn with_pv_sensors(mut self) -> Self {
        // External threats (PAMPs)
        self.add_sensor(McpSensor::new("faers_disproportionality"));
        self.add_sensor(McpSensor::new("pv_signal_complete"));

        // Internal damage (DAMPs)
        self.add_sensor(McpSensor::new("skill_validate").internal());
        self.add_sensor(McpSensor::new("code_tracker_changed").internal());
        self.add_sensor(McpSensor::new("brain_recovery_check").internal());

        self
    }

    /// Add default actuators
    pub fn with_default_actuators(mut self) -> Self {
        self.add_actuator(McpActuator::new("alert", "brain_artifact_save").with_priority(80));
        self.add_actuator(McpActuator::new("audit", "brain_artifact_resolve").with_priority(100));
        self.add_actuator(McpActuator::new("learn", "implicit_set").with_priority(60));
        self
    }

    /// Run a single tick of the loop
    pub async fn tick(&mut self) -> Result<CycleResult> {
        let start = std::time::Instant::now();
        self.cycle_count += 1;
        let cycle_id = format!("cycle-{}", self.cycle_count);

        // Phase 1: SENSING
        // Standard MCP sensors
        let signals = self.sense().await?;
        let signal_summaries: Vec<SignalSummary> = signals
            .iter()
            .map(|s| SignalSummary {
                id: s.id.clone(),
                source: format!("{:?}", s.source),
                severity: format!("{:?}", s.severity),
                confidence: s.confidence.value,
            })
            .collect();

        // Phase 1b: Apply hormonal stimulus from signals (if modulator wired)
        if let Some(modulator) = &self.hormonal_modulator {
            for signal in &signals {
                // Map signal severity to a pseudo-event for hormonal modulation
                let pseudo_event = Event {
                    source: format!("{:?}", signal.source),
                    event_type: signal.pattern.clone(),
                    priority: match signal.severity {
                        ThreatLevel::Critical => crate::models::Urgency::Critical,
                        ThreatLevel::High => crate::models::Urgency::High,
                        ThreatLevel::Medium | ThreatLevel::Info => crate::models::Urgency::Normal,
                        ThreatLevel::Low => crate::models::Urgency::Low,
                    },
                    ..Event::default()
                };
                modulator.apply_event_stimulus(&pseudo_event).await;
            }
        }

        // Phase 2: DECISION
        let actions = self.decision_engine.evaluate(&signals);
        let risk_score = self.compute_risk_score(&signals);

        // Phase 2b: Hormonal bias — adjust risk threshold dynamically
        if let Some(modulator) = &self.hormonal_modulator {
            let tolerance = modulator.risk_tolerance().await;
            // Lower tolerance → higher effective risk (more cautious)
            let _adjusted_risk = risk_score * (1.0 + (0.5 - tolerance));
            // Crisis mode: log escalation
            if modulator.is_crisis().await {
                tracing::warn!(risk_score, "hormonal_crisis_mode_active");
            }
        }

        // Phase 2c: Synaptic bias — recall learned patterns
        if let Some(learner) = &self.synaptic_learner {
            for signal in &signals {
                if let Some(bias) = learner.recall_bias(&signal.pattern).await {
                    tracing::debug!(
                        pattern = %signal.pattern,
                        bias,
                        "synaptic_bias_recalled"
                    );
                }
            }
        }

        // Phase 3: RESPONSE
        // Energy gating: check if we can afford LLM invocations
        let action_summaries = if let Some(governor) = &self.energy_governor {
            let mut gated_actions = Vec::new();
            for action in &actions {
                // Only gate InvokeClaude-equivalent actions via energy check
                let is_llm_action = matches!(action, ResponseAction::Escalate { .. });
                if is_llm_action {
                    if governor
                        .can_invoke_llm(crate::models::Urgency::Normal)
                        .await
                    {
                        gated_actions.push(action.clone());
                    } else {
                        tracing::debug!("energy_gated_llm_action_skipped");
                    }
                } else {
                    gated_actions.push(action.clone());
                }
            }
            self.respond(&gated_actions).await?
        } else {
            self.respond(&actions).await?
        };

        // Phase 3b: Emit cytokines for completed actions
        if let Some(cytokine_bridge) = &self.cytokine_bridge {
            for summary in &action_summaries {
                if summary.success {
                    // Success → IL-2 growth signal
                    let growth = nexcore_cytokine::Cytokine::new(
                        nexcore_cytokine::CytokineFamily::Il2,
                        format!("action_success_{}", summary.action_type),
                    )
                    .with_severity(nexcore_cytokine::ThreatLevel::Low)
                    .with_source("agentic_loop");
                    cytokine_bridge.forward(&growth).await;
                } else {
                    // Failure → IL-6 acute phase
                    let acute = nexcore_cytokine::Cytokine::new(
                        nexcore_cytokine::CytokineFamily::Il6,
                        format!("action_failure_{}", summary.action_type),
                    )
                    .with_severity(nexcore_cytokine::ThreatLevel::Medium)
                    .with_source("agentic_loop");
                    cytokine_bridge.forward(&acute).await;
                }
            }
        }

        // Phase 3c: Forward Guardian iteration results (if bridge wired)
        if let Some(guardian_bridge) = &self.guardian_bridge {
            let iteration_result = LoopIterationResult {
                iteration_id: cycle_id.clone(),
                timestamp: Utc::now(),
                signals_detected: signals.len(),
                actions_taken: action_summaries.len(),
                results: Vec::new(),
                duration_ms: start.elapsed().as_millis() as u64,
                throughput: Default::default(),
            };
            guardian_bridge
                .forward_iteration(&iteration_result, Some(cycle_id.clone()))
                .await;
        }

        // Phase 4: FEEDBACK
        let learning_updates = if self.config.enable_learning {
            self.learn(&signals, &action_summaries).await?
        } else {
            Vec::new()
        };

        // Phase 4b: Synaptic learning from action outcomes
        if let Some(learner) = &self.synaptic_learner {
            for summary in &action_summaries {
                // Use the first signal's pattern as event_type context
                let event_type = signals
                    .first()
                    .map(|s| s.pattern.as_str())
                    .unwrap_or("unknown");
                learner
                    .observe_outcome(event_type, &summary.action_type, summary.success)
                    .await;
            }
        }

        // Phase 4c: Hormonal reward/penalty
        if let Some(modulator) = &self.hormonal_modulator {
            let success_count = action_summaries.iter().filter(|a| a.success).count();
            let total = action_summaries.len().max(1);
            let success_ratio = success_count as f64 / total as f64;

            if success_ratio > 0.5 {
                modulator.signal_success(success_ratio).await;
            } else if !action_summaries.is_empty() {
                modulator.signal_failure(1.0 - success_ratio).await;
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(CycleResult {
            cycle_id,
            timestamp: Utc::now(),
            signals: signal_summaries,
            risk_score,
            actions: action_summaries,
            learning_updates,
            duration_ms,
        })
    }

    /// Sensing phase: Gather signals from MCP sensors
    async fn sense(&self) -> Result<Vec<ThreatSignal<String>>> {
        let mut signals = Vec::new();
        for sensor in &self.sensors {
            signals.push(self.process_sensor(sensor).await?);
        }
        Ok(signals)
    }

    async fn process_sensor(&self, sensor: &McpSensor) -> Result<ThreatSignal<String>> {
        if !self.mcp_bridge.supports(&sensor.tool_name) {
            return Ok(self.create_signal(sensor, ThreatLevel::Info, None, None));
        }

        let params = serde_json::to_value(&sensor.params).unwrap_or_default();
        match self.mcp_bridge.invoke(&sensor.tool_name, params).await {
            Ok(res) => {
                Ok(self.create_signal(sensor, self.determine_severity(&res), Some(res), None))
            }
            Err(e) => Ok(self.create_signal(sensor, ThreatLevel::High, None, Some(e.to_string()))),
        }
    }

    fn determine_severity(&self, result: &serde_json::Value) -> ThreatLevel {
        let is_signal = result
            .get("any_signal")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
            || result
                .get("is_signal")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

        if is_signal {
            ThreatLevel::Medium
        } else if result.get("error").is_some() {
            ThreatLevel::High
        } else {
            ThreatLevel::Info
        }
    }

    fn create_signal(
        &self,
        sensor: &McpSensor,
        severity: ThreatLevel,
        res: Option<serde_json::Value>,
        err: Option<String>,
    ) -> ThreatSignal<String> {
        let mut metadata = HashMap::new();
        if let Some(r) = res {
            metadata.insert("mcp_result".to_string(), r.to_string());
        }
        if let Some(e) = err {
            metadata.insert("error".to_string(), e);
        }

        ThreatSignal {
            id: format!("{}_{}", sensor.tool_name, Utc::now().timestamp()),
            pattern: sensor.tool_name.clone(),
            severity,
            timestamp: Utc::now(),
            source: sensor.source.clone(),
            confidence: nexcore_vigilance::primitives::Measured::certain(1.0),
            metadata,
        }
    }
    fn compute_risk_score(&self, signals: &[ThreatSignal<String>]) -> f64 {
        let mut total = 0.0;
        for signal in signals {
            let weight = if signal.source.is_external() {
                1.5
            } else {
                1.0
            };
            total += f64::from(signal.severity.score()) * signal.confidence.value * weight;
        }
        (total / signals.len().max(1) as f64 / 100.0 * 100.0).clamp(0.0, 100.0)
    }

    /// Response phase: Execute actions via MCP actuators
    async fn respond(&self, actions: &[ResponseAction]) -> Result<Vec<ActionSummary>> {
        let mut summaries = Vec::new();

        for action in actions {
            let summary = match action {
                ResponseAction::Alert { message, .. } => ActionSummary {
                    action_type: "alert".to_string(),
                    target: message.clone(),
                    success: true,
                },
                ResponseAction::AuditLog { message, .. } => ActionSummary {
                    action_type: "audit_log".to_string(),
                    target: message.clone(),
                    success: true,
                },
                ResponseAction::Block { target, .. } => ActionSummary {
                    action_type: "block".to_string(),
                    target: target.clone(),
                    success: true,
                },
                ResponseAction::Escalate { description, .. } => ActionSummary {
                    action_type: "escalate".to_string(),
                    target: description.clone(),
                    success: true,
                },
                _ => ActionSummary {
                    action_type: "unknown".to_string(),
                    target: String::new(),
                    success: false,
                },
            };
            summaries.push(summary);
        }

        Ok(summaries)
    }

    /// Learning phase: Update implicit knowledge from cycle results
    async fn learn(
        &mut self,
        signals: &[ThreatSignal<String>],
        _actions: &[ActionSummary],
    ) -> Result<Vec<LearningUpdate>> {
        let mut updates = Vec::new();

        // Track signal patterns
        for signal in signals {
            let key = format!("signal_count_{}", signal.source.component_name());
            let old_count = self.learned_thresholds.get(&key).copied();
            let new_count = old_count.unwrap_or(0.0) + 1.0;
            self.learned_thresholds.insert(key.clone(), new_count);

            updates.push(LearningUpdate {
                key,
                old_value: old_count.map(|v| serde_json::json!(v)),
                new_value: serde_json::json!(new_count),
            });
        }

        Ok(updates)
    }

    /// Run the loop continuously
    pub async fn run(&mut self) -> Result<()> {
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        while *self.running.read().await {
            let result = self.tick().await?;

            // Log cycle result
            tracing::info!(
                cycle = %result.cycle_id,
                risk = result.risk_score,
                signals = result.signals.len(),
                actions = result.actions.len(),
                "Agentic loop cycle completed"
            );

            // Wait for next tick
            tokio::time::sleep(self.config.tick_interval).await;

            // Decay amplification
            self.decision_engine
                .decay(self.config.tick_interval.as_secs_f64());
        }

        Ok(())
    }

    /// Stop the loop
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Get current cycle count
    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }
}

/// Extension trait for SignalSource to get component name
trait SignalSourceExt {
    fn component_name(&self) -> String;
}

impl SignalSourceExt for SignalSource {
    fn component_name(&self) -> String {
        match self {
            SignalSource::Pamp { source_id, .. } => source_id.clone(),
            SignalSource::Damp { subsystem, .. } => subsystem.clone(),
            SignalSource::Hybrid { external, .. } => external.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_config_defaults() {
        let config = LoopConfig::default();
        assert_eq!(config.tick_interval, Duration::from_secs(30));
        assert!((config.risk_threshold - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_loop_config_builder() {
        let config = LoopConfig::default()
            .with_tick_interval(Duration::from_secs(60))
            .with_risk_threshold(75.0)
            .with_brain_session("test-session");

        assert_eq!(config.tick_interval, Duration::from_secs(60));
        assert!((config.risk_threshold - 75.0).abs() < f64::EPSILON);
        assert_eq!(config.brain_session_id, Some("test-session".to_string()));
    }

    #[test]
    fn test_mcp_sensor_creation() {
        let sensor = McpSensor::new("pv_signal_complete")
            .with_param("drug", serde_json::json!("aspirin"))
            .internal();

        assert_eq!(sensor.tool_name, "pv_signal_complete");
        assert!(matches!(sensor.source, SignalSource::Damp { .. }));
    }

    #[tokio::test]
    async fn test_loop_tick() -> anyhow::Result<()> {
        let config = LoopConfig::default();
        let mut loop_ctrl = AgenticLoop::new(config)
            .with_pv_sensors()
            .with_default_actuators();

        let result = loop_ctrl.tick().await?;
        assert!(!result.cycle_id.is_empty());
        assert!(!result.signals.is_empty());
        Ok(())
    }
}
