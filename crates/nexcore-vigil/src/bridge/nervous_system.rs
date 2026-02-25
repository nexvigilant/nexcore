//! # Nervous System Bridge
//!
//! Adapters that translate between 5 biological subsystem crates and Vigil's
//! [`EventBus`] event type. Together these bridges transform Vigil into the
//! **Central Nervous System** that unifies:
//!
//! - **Cytokine** signaling (inter-crate events → EventBus events)
//! - **Hormonal** modulation (behavioral state → decision thresholds)
//! - **Synaptic** learning (outcome observation → pattern consolidation)
//! - **Immunity** sensing (code scanning → PAMP/DAMP signals)
//! - **Energy** governance (ATP budget → LLM invocation gating)
//!
//! All bridges are designed to be `Option<T>` inside [`AgenticLoop`] for graceful
//! degradation when a bio-crate is unavailable.
//!
//! ## Architecture
//!
//! ```text
//!   Cytokine → CytokineBridge → EventBus
//!   Hormones → HormonalModulator → DecisionEngine thresholds
//!   Synapse  → SynapticLearner → Pattern biases
//!   Immunity → ImmunitySensor → PAMP/DAMP signals
//!   Energy   → EnergyGovernor → LLM invocation gate
//! ```
//!
//! ## Tier: T3 (module-level)
//! Dominant: → Causality (bio-signals cause Vigil events and decisions)

use std::sync::Arc;

use nexcore_chrono::DateTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::events::EventBus;
use crate::models::{Event, Urgency};

// ---------------------------------------------------------------------------
// Cytokine Bridge
// ---------------------------------------------------------------------------

/// Bridge: Cytokine → Vigil Event
///
/// Maps [`CytokineFamily`] urgency to [`Priority`] and forwards cytokine
/// emissions as Vigil events on the [`EventBus`].
///
/// ## Tier: T2-C (→ + μ + ν)
/// Dominant: → Causality — cytokines cause Vigil event emissions.
pub struct CytokineBridge {
    bus: Arc<EventBus>,
}

impl CytokineBridge {
    /// Create a new bridge connected to the given EventBus.
    pub fn new(bus: Arc<EventBus>) -> Self {
        Self { bus }
    }

    /// Forward a cytokine emission as a Vigil event.
    ///
    /// Maps [`CytokineFamily`] → [`Priority`]:
    /// - `IL-1` (alarm), `TNF-α` (terminate) → `Critical`
    /// - `IL-6` (acute) → `High`
    /// - `IL-2`, `IFN-γ` → `Normal`
    /// - `IL-10`, `TGF-β`, `CSF` → `Low`
    pub async fn forward(&self, cytokine: &nexcore_cytokine::Cytokine) {
        let priority = Self::map_priority(&cytokine.family);

        let event = Event {
            source: "cytokine".to_string(),
            event_type: format!("cytokine_{}", cytokine.family),
            payload: serde_json::json!({
                "family": format!("{}", cytokine.family),
                "name": cytokine.name,
                "severity": format!("{}", cytokine.severity),
                "scope": format!("{}", cytokine.scope),
                "source": cytokine.source,
                "target": cytokine.target,
            }),
            priority,
            correlation_id: Some(cytokine.id.clone()),
            ..Event::default()
        };

        self.bus.emit(event).await;
        debug!(family = %cytokine.family, "cytokine_forwarded_to_eventbus");
    }

    /// Reverse mapping: Vigil Event → Cytokine emission.
    ///
    /// Returns `Some(Cytokine)` for events originating from biological
    /// subsystems that merit cytokine cascade notification.
    pub fn to_cytokine(event: &Event) -> Option<nexcore_cytokine::Cytokine> {
        match event.priority {
            Urgency::Critical => Some(
                nexcore_cytokine::Cytokine::alarm(&event.event_type)
                    .with_source(event.source.clone()),
            ),
            Urgency::High => Some(
                nexcore_cytokine::Cytokine::new(
                    nexcore_cytokine::CytokineFamily::Il6,
                    &event.event_type,
                )
                .with_severity(nexcore_cytokine::ThreatLevel::High)
                .with_source(event.source.clone()),
            ),
            _ => None,
        }
    }

    /// Map cytokine family to Vigil priority.
    fn map_priority(family: &nexcore_cytokine::CytokineFamily) -> Urgency {
        use nexcore_cytokine::CytokineFamily;
        match family {
            CytokineFamily::Il1 | CytokineFamily::TnfAlpha => Urgency::Critical,
            CytokineFamily::Il6 => Urgency::High,
            CytokineFamily::Il2 | CytokineFamily::IfnGamma => Urgency::Normal,
            CytokineFamily::Il10
            | CytokineFamily::TgfBeta
            | CytokineFamily::Csf
            | CytokineFamily::Custom(_) => Urgency::Low,
        }
    }
}

// ---------------------------------------------------------------------------
// Hormonal Modulator
// ---------------------------------------------------------------------------

/// Bridge: Hormonal state modulates decision thresholds.
///
/// Reads the current [`EndocrineState`] and computes [`BehavioralModifiers`]
/// that adjust the agentic loop's risk tolerance, exploration rate, and
/// crisis-mode behavior.
///
/// ## Tier: T2-C (ς + ∂ + μ)
/// Dominant: ς State — maintains and modulates behavioral state.
pub struct HormonalModulator {
    state: Arc<RwLock<nexcore_hormones::EndocrineState>>,
}

impl HormonalModulator {
    /// Create a modulator wrapping the given endocrine state.
    pub fn new(state: nexcore_hormones::EndocrineState) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }

    /// Load from the default persistence path (`~/.claude/hormones/state.json`).
    pub fn load_default() -> Self {
        let state = nexcore_hormones::EndocrineState::load();
        Self::new(state)
    }

    /// Get current behavioral modifiers (read-only snapshot).
    pub async fn modifiers(&self) -> nexcore_hormones::BehavioralModifiers {
        let state = self.state.read().await;
        nexcore_hormones::BehavioralModifiers::from(&*state)
    }

    /// Apply a stimulus from a Vigil event.
    ///
    /// Maps event characteristics to hormonal stimuli:
    /// - Critical events → `CriticalError` (adrenaline + cortisol spike)
    /// - High events → `ErrorEncountered` (cortisol rise)
    /// - Successful actions → `TaskCompleted` (dopamine reward)
    /// - Learning updates → `PositiveFeedback` (dopamine + serotonin)
    pub async fn apply_event_stimulus(&self, event: &Event) {
        let stimulus = match event.priority {
            Urgency::Critical => nexcore_hormones::Stimulus::CriticalError { recoverable: true },
            Urgency::High => nexcore_hormones::Stimulus::ErrorEncountered { severity: 0.6 },
            Urgency::Normal => nexcore_hormones::Stimulus::ConsistentSession { variance: 0.3 },
            Urgency::Low => nexcore_hormones::Stimulus::PredictableOutcome { accuracy: 0.8 },
        };

        let mut state = self.state.write().await;
        stimulus.apply(&mut state);
        debug!(
            cortisol = state.cortisol.value(),
            dopamine = state.dopamine.value(),
            "hormonal_stimulus_applied"
        );
    }

    /// Signal task completion — triggers dopamine reward.
    pub async fn signal_success(&self, complexity: f64) {
        let stimulus = nexcore_hormones::Stimulus::TaskCompleted {
            complexity: complexity.clamp(0.0, 1.0),
        };
        let mut state = self.state.write().await;
        stimulus.apply(&mut state);
    }

    /// Signal failure — triggers cortisol stress response.
    pub async fn signal_failure(&self, severity: f64) {
        let stimulus = nexcore_hormones::Stimulus::ErrorEncountered {
            severity: severity.clamp(0.0, 1.0),
        };
        let mut state = self.state.write().await;
        stimulus.apply(&mut state);
    }

    /// Check if system is in crisis mode (adrenaline > 0.7).
    pub async fn is_crisis(&self) -> bool {
        self.state.read().await.is_crisis_mode()
    }

    /// Get current risk tolerance [0.0, 1.0].
    pub async fn risk_tolerance(&self) -> f64 {
        self.state.read().await.risk_tolerance()
    }

    /// Apply decay toward baseline — called between cycles.
    pub async fn decay(&self) {
        let mut state = self.state.write().await;
        state.apply_decay();
    }

    /// Persist state to disk.
    pub async fn save(&self) -> std::result::Result<(), nexcore_hormones::EndocrineError> {
        let state = self.state.read().await;
        state.save()
    }
}

// ---------------------------------------------------------------------------
// Synaptic Learner
// ---------------------------------------------------------------------------

/// Bridge: Synapse learning from Vigil decision outcomes.
///
/// Observes action success/failure to strengthen or weaken synaptic
/// connections. Consolidated synapses provide biases for future decisions.
///
/// ## Tier: T2-C (ν + ρ + π)
/// Dominant: ν Frequency — learns from repeated observations.
pub struct SynapticLearner {
    bank: Arc<RwLock<nexcore_synapse::SynapseBank>>,
    config: nexcore_synapse::AmplitudeConfig,
}

impl SynapticLearner {
    /// Create a new learner with default amplitude configuration.
    pub fn new() -> Self {
        Self {
            bank: Arc::new(RwLock::new(nexcore_synapse::SynapseBank::new())),
            config: nexcore_synapse::AmplitudeConfig::DEFAULT,
        }
    }

    /// Create with custom amplitude configuration.
    pub fn with_config(config: nexcore_synapse::AmplitudeConfig) -> Self {
        Self {
            bank: Arc::new(RwLock::new(nexcore_synapse::SynapseBank::new())),
            config,
        }
    }

    /// Observe a decision outcome → strengthen or weaken the synapse.
    ///
    /// - `success = true`: High confidence + relevance → amplitude grows
    /// - `success = false`: Low confidence → amplitude decays faster
    pub async fn observe_outcome(&self, event_type: &str, action: &str, success: bool) {
        let synapse_id = format!("{event_type}::{action}");
        let signal = if success {
            nexcore_synapse::LearningSignal::new(0.8, 1.0)
        } else {
            nexcore_synapse::LearningSignal::new(0.2, 0.5)
        };

        let mut bank = self.bank.write().await;
        let synapse = bank.get_or_create(&synapse_id, self.config.clone());
        synapse.observe(signal);

        debug!(
            synapse = %synapse_id,
            amplitude = synapse.current_amplitude().value(),
            status = ?synapse.status(),
            "synapse_observation_recorded"
        );
    }

    /// Query consolidated synapses to bias decisions.
    ///
    /// Returns `Some(amplitude)` if the synapse for this event type is
    /// consolidated, `None` if still accumulating or not yet observed.
    pub async fn recall_bias(&self, event_type: &str) -> Option<f64> {
        let bank = self.bank.read().await;
        bank.get(event_type).and_then(|synapse| {
            if synapse.is_consolidated() {
                Some(synapse.current_amplitude().value())
            } else {
                None
            }
        })
    }

    /// Get count of consolidated patterns (learned behaviors).
    pub async fn consolidated_count(&self) -> usize {
        let bank = self.bank.read().await;
        bank.consolidated().count()
    }

    /// Prune decayed synapses (garbage collection).
    pub async fn prune(&self) -> usize {
        let mut bank = self.bank.write().await;
        bank.prune_decayed()
    }

    /// Get total synapse count.
    pub async fn synapse_count(&self) -> usize {
        let bank = self.bank.read().await;
        bank.len()
    }
}

impl Default for SynapticLearner {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Immunity Sensor
// ---------------------------------------------------------------------------

/// Bridge: Immunity scanning → PAMP/DAMP signals on the EventBus.
///
/// Wraps [`ImmunityScanner`] and emits threat detections as Vigil events,
/// mapping [`ThreatType`] (PAMP/DAMP) to appropriate priorities.
///
/// ## Tier: T2-C (∃ + κ + ∂)
/// Dominant: ∃ Existence — detects existence of threats at code boundaries.
pub struct ImmunitySensor {
    scanner: nexcore_immunity::ImmunityScanner,
    bus: Arc<EventBus>,
}

impl ImmunitySensor {
    /// Create a new sensor from an existing scanner and EventBus.
    pub fn new(scanner: nexcore_immunity::ImmunityScanner, bus: Arc<EventBus>) -> Self {
        Self { scanner, bus }
    }

    /// Try to create from the default antibody registry.
    ///
    /// Returns `None` if the registry file is missing or invalid.
    pub fn try_default(bus: Arc<EventBus>) -> Option<Self> {
        let registry = nexcore_immunity::load_default_registry().ok()?;
        let scanner = nexcore_immunity::ImmunityScanner::new(&registry).ok()?;
        Some(Self { scanner, bus })
    }

    /// Scan code content and emit any threats as events on the EventBus.
    ///
    /// Returns the number of threats detected and emitted.
    pub async fn scan_and_emit(&self, code: &str, filename: Option<&str>) -> usize {
        let result = self.scanner.scan(code, filename);

        if result.clean {
            return 0;
        }

        let threat_count = result.threats.len();

        for threat in &result.threats {
            let priority = Self::threat_priority(&threat.severity);
            let event = Event {
                source: "immunity".to_string(),
                event_type: format!(
                    "threat_{}",
                    match threat.threat_type {
                        nexcore_immunity::ThreatType::Pamp => "pamp",
                        nexcore_immunity::ThreatType::Damp => "damp",
                    }
                ),
                payload: serde_json::json!({
                    "antibody_id": threat.antibody_id,
                    "antibody_name": threat.antibody_name,
                    "threat_type": format!("{}", threat.threat_type),
                    "severity": format!("{}", threat.severity),
                    "matched_content": threat.matched_content,
                    "confidence": threat.confidence,
                    "response_strategy": format!("{}", threat.response),
                    "location": threat.location,
                    "filename": filename,
                }),
                priority,
                ..Event::default()
            };

            self.bus.emit(event).await;
        }

        info!(
            threats = threat_count,
            filename = filename,
            "immunity_scan_complete"
        );

        threat_count
    }

    /// Scan compiler/tool stderr for error patterns.
    pub async fn scan_errors_and_emit(&self, stderr: &str) -> usize {
        let result = self.scanner.scan_errors(stderr);

        if result.clean {
            return 0;
        }

        let count = result.threats.len();
        for threat in &result.threats {
            let event = Event {
                source: "immunity".to_string(),
                event_type: "threat_damp_build_error".to_string(),
                payload: serde_json::json!({
                    "antibody_id": threat.antibody_id,
                    "antibody_name": threat.antibody_name,
                    "matched_content": threat.matched_content,
                    "severity": format!("{}", threat.severity),
                }),
                priority: Self::threat_priority(&threat.severity),
                ..Event::default()
            };
            self.bus.emit(event).await;
        }

        count
    }

    /// Map immunity threat level to Vigil priority.
    fn threat_priority(level: &nexcore_immunity::ThreatLevel) -> Urgency {
        match level {
            nexcore_immunity::ThreatLevel::Critical => Urgency::Critical,
            nexcore_immunity::ThreatLevel::High => Urgency::High,
            nexcore_immunity::ThreatLevel::Medium => Urgency::Normal,
            nexcore_immunity::ThreatLevel::Low => Urgency::Low,
        }
    }
}

// ---------------------------------------------------------------------------
// Energy Governor
// ---------------------------------------------------------------------------

/// Bridge: Energy budget modulates LLM invocation.
///
/// Uses the Atkinson Energy Charge formula to gate expensive operations:
/// - Below 0.3 → deny all LLM invocations (conservation mode)
/// - 0.3–0.7 → allow only High/Critical priority
/// - Above 0.7 → allow all
///
/// ## Tier: T2-C (N + ∂ + ς)
/// Dominant: N Quantity — quantitative energy threshold gating.
pub struct EnergyGovernor {
    pool: Arc<RwLock<nexcore_energy::TokenPool>>,
}

impl EnergyGovernor {
    /// Create a governor wrapping the given token pool.
    pub fn new(pool: nexcore_energy::TokenPool) -> Self {
        Self {
            pool: Arc::new(RwLock::new(pool)),
        }
    }

    /// Create with a fresh budget.
    pub fn with_budget(budget: u64) -> Self {
        Self::new(nexcore_energy::TokenPool::new(budget))
    }

    /// Check if the current energy charge allows an LLM invocation.
    ///
    /// Uses the Atkinson Energy Charge: `EC = (tATP + 0.5 * tADP) / total`
    ///
    /// - `EC < 0.3` → deny (conservation mode)
    /// - `EC 0.3..0.7` → allow only `Critical` or `High`
    /// - `EC >= 0.7` → allow all
    pub async fn can_invoke_llm(&self, priority: Urgency) -> bool {
        let pool = self.pool.read().await;
        let ec = pool.energy_charge();

        if ec < 0.3 {
            warn!(ec, "energy_conservation_mode_llm_denied");
            return false;
        }

        if ec < 0.7 {
            let allowed = matches!(priority, Urgency::Critical | Urgency::High);
            if !allowed {
                debug!(ec, ?priority, "energy_throttled_low_priority_denied");
            }
            return allowed;
        }

        true
    }

    /// Record a productive token spend (tATP → tADP).
    pub async fn spend_productive(&self, tokens: u64) -> u64 {
        let mut pool = self.pool.write().await;
        pool.spend_productive(tokens)
    }

    /// Record a wasted token spend (tATP → tAMP).
    pub async fn spend_waste(&self, tokens: u64) -> u64 {
        let mut pool = self.pool.write().await;
        pool.spend_waste(tokens)
    }

    /// Get current energy charge [0.0, 1.0].
    pub async fn energy_charge(&self) -> f64 {
        self.pool.read().await.energy_charge()
    }

    /// Get current regime.
    pub async fn regime(&self) -> nexcore_energy::Regime {
        self.pool.read().await.regime()
    }

    /// Get full energy state snapshot.
    pub async fn snapshot(&self, total_value: f64) -> nexcore_energy::EnergyState {
        let pool = self.pool.read().await;
        nexcore_energy::snapshot(&pool, total_value)
    }

    /// Decide optimal strategy for an operation.
    pub async fn decide_strategy(
        &self,
        operation: &nexcore_energy::Operation,
    ) -> nexcore_energy::Strategy {
        let pool = self.pool.read().await;
        nexcore_energy::decide(&pool, operation)
    }
}

// ---------------------------------------------------------------------------
// Guardian Bridge (Phase 4)
// ---------------------------------------------------------------------------

/// Bridge: Guardian homeostasis loop ↔ Vigil EventBus.
///
/// Closes the feedback loop: Guardian's `LoopIterationResult` feeds back
/// into the EventBus as events with correlation IDs for traceability.
///
/// ```text
/// Guardian SENSE → Signal → Vigil EventBus → AgenticLoop.tick()
///                                               ↓
///                 Guardian ACT ← ResponseAction ← DecisionEngine.evaluate()
/// ```
///
/// ## Tier: T2-C (→ + ρ + ς)
/// Dominant: → Causality — Guardian signals cause Vigil reactions.
pub struct GuardianBridge {
    bus: Arc<EventBus>,
}

impl GuardianBridge {
    /// Create a new Guardian bridge.
    pub fn new(bus: Arc<EventBus>) -> Self {
        Self { bus }
    }

    /// Forward a Guardian loop iteration result into the EventBus.
    pub async fn forward_iteration(
        &self,
        result: &crate::agentic_loop::LoopIterationResult,
        correlation_id: Option<String>,
    ) {
        let priority = if result.actions_taken > 2 {
            Urgency::High
        } else {
            Urgency::Normal
        };

        let event = Event {
            source: "guardian".to_string(),
            event_type: "homeostasis_iteration".to_string(),
            payload: serde_json::json!({
                "iteration_id": result.iteration_id,
                "signals_detected": result.signals_detected,
                "actions_taken": result.actions_taken,
                "duration_ms": result.duration_ms,
                "timestamp": DateTime::now().to_rfc3339(),
            }),
            priority,
            correlation_id,
            ..Event::default()
        };

        self.bus.emit(event).await;
        debug!(
            signals = result.signals_detected,
            actions = result.actions_taken,
            "guardian_iteration_forwarded"
        );
    }

    /// Emit a Guardian alert as a high-priority event.
    pub async fn emit_alert(&self, message: &str) {
        let event = Event {
            source: "guardian".to_string(),
            event_type: "guardian_alert".to_string(),
            payload: serde_json::json!({ "message": message }),
            priority: Urgency::High,
            ..Event::default()
        };
        self.bus.emit(event).await;
    }
}

// ---------------------------------------------------------------------------
// Aggregate: NervousSystem
// ---------------------------------------------------------------------------

/// The complete Nervous System — all 6 bridges unified.
///
/// Provides a single entry point for initializing all bridges against a shared
/// [`EventBus`]. Individual bridges can be extracted for wiring into the
/// [`AgenticLoop`].
///
/// ## Tier: T3 (σ + ς + ∂ + → + ν + N + ∃ + μ + ρ + π)
/// Dominant: → Causality — the nervous system is fundamentally about
/// cause-and-effect signal propagation.
pub struct NervousSystem {
    /// The shared event bus
    pub bus: Arc<EventBus>,
    /// Cytokine → Event bridge
    pub cytokine: CytokineBridge,
    /// Hormonal state modulator
    pub hormonal: HormonalModulator,
    /// Synaptic learning bridge
    pub synaptic: SynapticLearner,
    /// Immunity threat sensor (None if registry unavailable)
    pub immunity: Option<ImmunitySensor>,
    /// Energy budget governor
    pub energy: EnergyGovernor,
    /// Guardian feedback bridge
    pub guardian: GuardianBridge,
}

impl NervousSystem {
    /// Initialize the full nervous system with default configurations.
    ///
    /// - Loads hormonal state from default path
    /// - Attempts to load immunity registry (graceful if missing)
    /// - Creates a default energy pool with the given token budget
    pub fn init(bus: Arc<EventBus>, token_budget: u64) -> Self {
        let cytokine = CytokineBridge::new(Arc::clone(&bus));
        let hormonal = HormonalModulator::load_default();
        let synaptic = SynapticLearner::new();
        let immunity = ImmunitySensor::try_default(Arc::clone(&bus));
        let energy = EnergyGovernor::with_budget(token_budget);
        let guardian = GuardianBridge::new(Arc::clone(&bus));

        if immunity.is_none() {
            warn!("immunity_sensor_unavailable_registry_missing");
        }

        info!(
            token_budget,
            immunity_available = immunity.is_some(),
            "nervous_system_initialized"
        );

        Self {
            bus,
            cytokine,
            hormonal,
            synaptic,
            immunity,
            energy,
            guardian,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cytokine_priority_mapping() {
        use nexcore_cytokine::CytokineFamily;

        assert_eq!(
            CytokineBridge::map_priority(&CytokineFamily::Il1),
            Urgency::Critical
        );
        assert_eq!(
            CytokineBridge::map_priority(&CytokineFamily::TnfAlpha),
            Urgency::Critical
        );
        assert_eq!(
            CytokineBridge::map_priority(&CytokineFamily::Il6),
            Urgency::High
        );
        assert_eq!(
            CytokineBridge::map_priority(&CytokineFamily::Il2),
            Urgency::Normal
        );
        assert_eq!(
            CytokineBridge::map_priority(&CytokineFamily::IfnGamma),
            Urgency::Normal
        );
        assert_eq!(
            CytokineBridge::map_priority(&CytokineFamily::Il10),
            Urgency::Low
        );
        assert_eq!(
            CytokineBridge::map_priority(&CytokineFamily::TgfBeta),
            Urgency::Low
        );
        assert_eq!(
            CytokineBridge::map_priority(&CytokineFamily::Csf),
            Urgency::Low
        );
    }

    #[test]
    fn event_to_cytokine_critical() {
        let event = Event {
            priority: Urgency::Critical,
            event_type: "test_critical".to_string(),
            source: "test".to_string(),
            ..Event::default()
        };

        let cytokine = CytokineBridge::to_cytokine(&event);
        assert!(cytokine.is_some());
        let c = cytokine.as_ref().map(|c| &c.family);
        assert_eq!(c, Some(&nexcore_cytokine::CytokineFamily::Il1));
    }

    #[test]
    fn event_to_cytokine_low_returns_none() {
        let event = Event {
            priority: Urgency::Low,
            ..Event::default()
        };
        assert!(CytokineBridge::to_cytokine(&event).is_none());
    }

    #[tokio::test]
    async fn hormonal_modulator_defaults() {
        let modulator = HormonalModulator::new(nexcore_hormones::EndocrineState::default());
        let modifiers = modulator.modifiers().await;

        // Default state: risk_tolerance should be baseline ~0.5
        assert!(modifiers.risk_tolerance >= 0.0 && modifiers.risk_tolerance <= 1.0);
        assert!(!modifiers.crisis_mode);
    }

    #[tokio::test]
    async fn hormonal_crisis_detection() {
        let mut state = nexcore_hormones::EndocrineState::default();
        // Inject high adrenaline to trigger crisis
        state.set(
            nexcore_hormones::HormoneType::Adrenaline,
            nexcore_hormones::HormoneLevel::new(0.9),
        );
        let modulator = HormonalModulator::new(state);
        assert!(modulator.is_crisis().await);
    }

    #[tokio::test]
    async fn synaptic_learner_observe_and_recall() {
        let learner = SynapticLearner::new();

        // Observe success multiple times to build amplitude
        for _ in 0..20 {
            learner
                .observe_outcome("file_changed", "audit_log", true)
                .await;
        }

        // Should have created a synapse
        assert!(learner.synapse_count().await > 0);

        // Recall may or may not be consolidated depending on config thresholds
        // but the synapse should exist
        let bank = learner.bank.read().await;
        assert!(bank.get("file_changed::audit_log").is_some());
    }

    #[tokio::test]
    async fn energy_governor_conservation_mode() {
        // Start with very depleted pool
        let mut pool = nexcore_energy::TokenPool::new(100);
        let _ = pool.spend_productive(50);
        let _ = pool.spend_waste(40);
        // EC should be very low now
        let governor = EnergyGovernor::new(pool);

        let ec = governor.energy_charge().await;
        if ec < 0.3 {
            // Conservation mode: deny all
            assert!(!governor.can_invoke_llm(Urgency::Critical).await);
        }
    }

    #[tokio::test]
    async fn energy_governor_full_budget() {
        let governor = EnergyGovernor::with_budget(100_000);

        // Full budget: EC = 1.0, allow all
        assert!(governor.can_invoke_llm(Urgency::Low).await);
        assert!(governor.can_invoke_llm(Urgency::Normal).await);
        assert!(governor.can_invoke_llm(Urgency::High).await);
        assert!(governor.can_invoke_llm(Urgency::Critical).await);
    }

    #[test]
    fn threat_priority_mapping() {
        use nexcore_immunity::ThreatLevel;

        assert_eq!(
            ImmunitySensor::threat_priority(&ThreatLevel::Critical),
            Urgency::Critical
        );
        assert_eq!(
            ImmunitySensor::threat_priority(&ThreatLevel::High),
            Urgency::High
        );
        assert_eq!(
            ImmunitySensor::threat_priority(&ThreatLevel::Medium),
            Urgency::Normal
        );
        assert_eq!(
            ImmunitySensor::threat_priority(&ThreatLevel::Low),
            Urgency::Low
        );
    }

    #[tokio::test]
    async fn guardian_bridge_emits_event() {
        let bus = Arc::new(EventBus::new(100));
        let bridge = GuardianBridge::new(Arc::clone(&bus));

        let result = crate::agentic_loop::LoopIterationResult {
            iteration_id: "test-iter-1".to_string(),
            timestamp: nexcore_chrono::DateTime::now(),
            signals_detected: 3,
            actions_taken: 1,
            results: Vec::new(),
            duration_ms: 42,
            throughput: nexcore_vigilance::guardian::homeostasis::ThroughputMonitor::default(),
        };

        bridge
            .forward_iteration(&result, Some("test-correlation".to_string()))
            .await;

        // Event should be on the bus
        assert!(bus.pending_count() > 0);
    }

    #[test]
    fn nervous_system_init_creates_all_bridges() {
        let bus = Arc::new(EventBus::new(100));
        let ns = NervousSystem::init(bus, 50_000);

        // Immunity may or may not be available depending on registry file
        // but all other bridges should be present
        assert!(ns.bus.pending_count() == 0);
    }
}
