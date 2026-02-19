//! # AVC Engine
//!
//! The Autonomous Vigilance Company core engine.
//! Implements the SENSE → COMPARE → DECIDE → ACT → LEARN → HOMEOSTASIS loop.
//!
//! ## Grounding
//!
//! AVC is a T3 system grounded to 9 T1 primitives:
//! κ(Comparison) + σ(Sequence) + ∂(Boundary) + ρ(Recursion) +
//! ς(State) + μ(Mapping) + π(Persistence) + →(Causality) + N(Quantity)
//!
//! ## Flow
//!
//! ```text
//! σ(sense) → κ(compare) → →(decide) → π(audit) → ρ(learn)
//!     ↑                                              │
//!     └──────────────── ς(homeostasis) ←─────────────┘
//! ```

use std::collections::HashMap;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::lex_primitiva::{GroundsTo, LexPrimitiva, PrimitiveComposition};

use super::types::{
    ADJUSTMENT_FACTOR, AuditRecord, AvcError, Baseline, CRITICAL_BOUNDARY, ClientConfig, ClientId,
    Comparison, Decision, Detection, Domain, Event, Feedback, Frequency, HEALTH_ALERT,
    HEALTH_DEGRADED, HEALTH_NOMINAL, HumanCommand, LEARNING_BATCH_SIZE, MAX_FALSE_NEGATIVE_RATE,
    MAX_FALSE_POSITIVE_RATE, OperationalState, Outcome, PvAction, SignalId, Source, Threshold,
    TimeSeries,
};

/// Autonomous Vigilance Company — the T3 system.
///
/// Grounding: κ + σ + ∂ + ρ + ς + μ + π + → + N (9 T1 primitives)
///
/// Implements a self-improving detection engine that:
/// 1. **Senses** incoming events from client streams (σ + μ)
/// 2. **Compares** metrics against baselines and thresholds (κ + ∂)
/// 3. **Decides** whether to alert, escalate, or log (→ + ∂ + ς)
/// 4. **Acts** on the decision with audit trail (→ + π)
/// 5. **Learns** from feedback to improve detection (ρ on κ and ∂)
/// 6. **Self-manages** via homeostasis loop (ς + ∂ + ρ)
///
/// Tier: T3 (9 unique T1 primitives)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Avc {
    // ── ς: Operational state ──
    state: OperationalState,

    // ── μ + σ: Client event streams ──
    streams: HashMap<String, TimeSeries<f64>>,

    // ── μ + σ: Domain baselines ──
    baselines: HashMap<String, Baseline>,

    // ── μ + ∂: Domain thresholds ──
    thresholds: HashMap<String, Threshold>,

    // ── μ + ∂ + λ: Client configurations ──
    clients: HashMap<u64, ClientConfig>,

    // ── σ + ρ: Feedback buffer for learning ──
    feedback_buffer: Vec<Feedback>,

    // ── π + σ: Decision audit ledger ──
    ledger: Vec<AuditRecord<String>>,

    // ── N: Counters ──
    total_events: u64,
    total_detections: u64,
    total_alerts: u64,
    total_escalations: u64,
    true_positives: u64,
    false_positives: u64,
    true_negatives: u64,
    false_negatives: u64,

    // ── N: Signal ID counter ──
    next_signal_id: u64,

    // ── bool: Halted flag ──
    halted: bool,
}

impl Default for Avc {
    fn default() -> Self {
        Self::new()
    }
}

impl Avc {
    /// Creates a new AVC with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: OperationalState::Nominal,
            streams: HashMap::new(),
            baselines: HashMap::new(),
            thresholds: HashMap::new(),
            clients: HashMap::new(),
            feedback_buffer: Vec::new(),
            ledger: Vec::new(),
            total_events: 0,
            total_detections: 0,
            total_alerts: 0,
            total_escalations: 0,
            true_positives: 0,
            false_positives: 0,
            true_negatives: 0,
            false_negatives: 0,
            next_signal_id: 1,
            halted: false,
        }
    }

    // ═══════════════════════════════════════════════════════
    // CONFIGURATION
    // ═══════════════════════════════════════════════════════

    /// Registers a client with its configuration.
    pub fn register_client(&mut self, config: ClientConfig) {
        // Merge thresholds into domain thresholds
        for (domain, thresh) in &config.thresholds {
            self.thresholds.entry(domain.clone()).or_insert(*thresh);
        }
        self.clients.insert(config.client_id.0, config);
    }

    /// Sets the baseline for a domain.
    pub fn set_baseline(&mut self, domain: &str, baseline: Baseline) {
        self.baselines.insert(domain.to_string(), baseline);
    }

    /// Sets the threshold for a domain.
    pub fn set_threshold(&mut self, domain: &str, threshold: Threshold) {
        self.thresholds.insert(domain.to_string(), threshold);
    }

    /// Returns current operational state.
    #[must_use]
    pub fn state(&self) -> OperationalState {
        self.state
    }

    /// Returns whether the system is halted.
    #[must_use]
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    // ═══════════════════════════════════════════════════════
    // SENSE: σ + μ
    // ═══════════════════════════════════════════════════════

    /// Ingests an event into the system.
    ///
    /// # Errors
    /// Returns `Err` if the system is halted.
    pub fn ingest(&mut self, event: Event) -> Result<(), AvcError> {
        if self.halted {
            return Err(AvcError::Halted);
        }

        self.total_events += 1;

        // Store in stream
        let stream = self
            .streams
            .entry(event.domain.clone())
            .or_insert_with(TimeSeries::new);
        stream.push(event.timestamp, event.metric);

        Ok(())
    }

    // ═══════════════════════════════════════════════════════
    // COMPARE: κ + ∂
    // ═══════════════════════════════════════════════════════

    /// Compares an event's metric against domain baseline and threshold.
    ///
    /// # Errors
    /// Returns `Err` if domain has no baseline or threshold configured.
    pub fn compare(&self, event: &Event) -> Result<Detection, AvcError> {
        let baseline = self
            .baselines
            .get(&event.domain)
            .ok_or_else(|| AvcError::UnknownDomain(event.domain.clone()))?;

        let threshold = self
            .thresholds
            .get(&event.domain)
            .ok_or_else(|| AvcError::UnknownDomain(event.domain.clone()))?;

        let comparison = Comparison::new(event.metric, baseline.expected());
        Ok(Detection::evaluate(comparison, threshold))
    }

    // ═══════════════════════════════════════════════════════
    // DECIDE: → + ∂ + ς
    // ═══════════════════════════════════════════════════════

    /// Decides what action to take based on detection and system state.
    #[must_use]
    pub fn decide(&mut self, detection: Detection, domain: &str) -> PvAction {
        let signal_id = SignalId(self.next_signal_id);
        self.next_signal_id += 1;

        if detection.signal_present {
            self.total_detections += 1;

            if detection.severity >= CRITICAL_BOUNDARY || self.state == OperationalState::Critical {
                // Critical zone: route to human
                self.total_escalations += 1;
                PvAction::Escalate {
                    signal_id,
                    detection,
                    reason: format!(
                        "severity={:.4} >= critical_boundary={CRITICAL_BOUNDARY} (domain={domain}, delta={:.4})",
                        detection.severity, detection.comparison.delta
                    ),
                }
            } else {
                // Autonomous zone: alert without human
                self.total_alerts += 1;
                PvAction::AutoAlert {
                    signal_id,
                    detection,
                    domain: domain.to_string(),
                }
            }
        } else {
            // No signal detected
            PvAction::Log {
                detection,
                domain: domain.to_string(),
            }
        }
    }

    // ═══════════════════════════════════════════════════════
    // ACT: → + π
    // ═══════════════════════════════════════════════════════

    /// Executes an action and records an audit entry.
    ///
    /// # Errors
    /// Returns `Err` on serialization failure.
    pub fn act(&mut self, action: PvAction) -> Result<ActionResult, AvcError> {
        let decision = Decision::new(action.clone(), self.state);

        // Audit (π)
        let description = match &action {
            PvAction::AutoAlert {
                signal_id,
                domain,
                detection,
            } => {
                format!(
                    "AUTO_ALERT signal={} domain={domain} delta={:.4} conf={:.4}",
                    signal_id.0, detection.comparison.delta, detection.confidence
                )
            }
            PvAction::Escalate {
                signal_id, reason, ..
            } => {
                format!("ESCALATE signal={} reason={reason}", signal_id.0)
            }
            PvAction::Log { domain, detection } => {
                format!(
                    "LOG domain={domain} delta={:.4}",
                    detection.comparison.delta
                )
            }
        };

        let audit = AuditRecord::stamp(description)?;
        self.ledger.push(audit);

        Ok(ActionResult {
            action: decision.action,
            state_at_decision: decision.system_state,
            timestamp: decision.timestamp,
        })
    }

    // ═══════════════════════════════════════════════════════
    // LEARN: ρ(κ) + ρ(∂)
    // ═══════════════════════════════════════════════════════

    /// Records feedback for learning.
    pub fn record_feedback(&mut self, feedback: Feedback) {
        // Update confusion matrix
        match feedback.actual_outcome {
            Outcome::TruePositive => self.true_positives += 1,
            Outcome::TrueNegative => self.true_negatives += 1,
            Outcome::FalsePositive => self.false_positives += 1,
            Outcome::FalseNegative => self.false_negatives += 1,
        }

        self.feedback_buffer.push(feedback);

        // Batch learning (ρ on κ)
        if self.feedback_buffer.len() >= LEARNING_BATCH_SIZE {
            self.calibrate_thresholds();
            self.feedback_buffer.clear();
        }
    }

    /// Calibrates thresholds based on accumulated feedback (ρ on ∂).
    fn calibrate_thresholds(&mut self) {
        let fpr = self.false_positive_rate();
        let fnr = self.false_negative_rate();

        if fpr > MAX_FALSE_POSITIVE_RATE {
            // Too many false positives → tighten thresholds (fewer alerts)
            let adjusted: Vec<(String, Threshold)> = self
                .thresholds
                .iter()
                .map(|(k, t)| (k.clone(), Threshold(t.0.adjust_up(ADJUSTMENT_FACTOR))))
                .collect();
            for (k, t) in adjusted {
                self.thresholds.insert(k, t);
            }
        }

        if fnr > MAX_FALSE_NEGATIVE_RATE {
            // Too many false negatives → loosen thresholds (more alerts)
            let adjusted: Vec<(String, Threshold)> = self
                .thresholds
                .iter()
                .map(|(k, t)| (k.clone(), Threshold(t.0.adjust_down(ADJUSTMENT_FACTOR))))
                .collect();
            for (k, t) in adjusted {
                self.thresholds.insert(k, t);
            }
        }
    }

    /// False positive rate: FP / (FP + TN).
    #[must_use]
    pub fn false_positive_rate(&self) -> f64 {
        let total = self.false_positives + self.true_negatives;
        if total == 0 {
            0.0
        } else {
            self.false_positives as f64 / total as f64
        }
    }

    /// False negative rate: FN / (FN + TP).
    #[must_use]
    pub fn false_negative_rate(&self) -> f64 {
        let total = self.false_negatives + self.true_positives;
        if total == 0 {
            0.0
        } else {
            self.false_negatives as f64 / total as f64
        }
    }

    /// Precision: TP / (TP + FP).
    #[must_use]
    pub fn precision(&self) -> f64 {
        let total = self.true_positives + self.false_positives;
        if total == 0 {
            1.0
        } else {
            self.true_positives as f64 / total as f64
        }
    }

    /// Recall: TP / (TP + FN).
    #[must_use]
    pub fn recall(&self) -> f64 {
        let total = self.true_positives + self.false_negatives;
        if total == 0 {
            1.0
        } else {
            self.true_positives as f64 / total as f64
        }
    }

    // ═══════════════════════════════════════════════════════
    // HOMEOSTASIS: ς + ∂ + ρ
    // ═══════════════════════════════════════════════════════

    /// Self-management: evaluates health and adjusts operational state.
    pub fn homeostasis(&mut self) {
        let health = self.health_score();

        self.state = if health > HEALTH_NOMINAL {
            OperationalState::Nominal
        } else if health > HEALTH_ALERT {
            OperationalState::Alert
        } else if health > HEALTH_DEGRADED {
            OperationalState::Degraded
        } else {
            OperationalState::Critical
        };

        // Self-healing when degraded
        if self.state == OperationalState::Degraded {
            self.shed_load();
        }
    }

    /// Computes overall health score (0.0-1.0).
    #[must_use]
    pub fn health_score(&self) -> f64 {
        let precision = self.precision();
        let recall = self.recall();

        // F1 score as primary health metric
        if precision + recall < f64::EPSILON {
            1.0 // No data yet — assume healthy
        } else {
            2.0 * precision * recall / (precision + recall)
        }
    }

    /// Load shedding — trim old data to reduce memory pressure.
    fn shed_load(&mut self) {
        // Keep only recent events in streams
        for stream in self.streams.values_mut() {
            if stream.len() > 10_000 {
                let drained = stream.drain();
                let keep = drained.len().saturating_sub(5_000);
                let mut new_ts = TimeSeries::new();
                for (ts, v) in drained.into_iter().skip(keep) {
                    new_ts.push(ts, v);
                }
                *stream = new_ts;
            }
        }

        // Trim audit ledger
        if self.ledger.len() > 50_000 {
            let keep = self.ledger.len().saturating_sub(25_000);
            self.ledger = self.ledger.split_off(keep);
        }
    }

    // ═══════════════════════════════════════════════════════
    // PROCESS: The Full Cycle
    // ═══════════════════════════════════════════════════════

    /// Processes a single event through the full SENSE→COMPARE→DECIDE→ACT cycle.
    ///
    /// # Errors
    /// Returns `Err` if the system is halted or the domain is unconfigured.
    pub fn process(&mut self, event: Event) -> Result<ActionResult, AvcError> {
        if self.halted {
            return Err(AvcError::Halted);
        }

        // SENSE (σ)
        let domain = event.domain.clone();
        self.ingest(event.clone())?;

        // COMPARE (κ)
        let detection = self.compare(&event)?;

        // DECIDE (→)
        let action = self.decide(detection, &domain);

        // ACT (→ + π)
        self.act(action)
    }

    // ═══════════════════════════════════════════════════════
    // HUMAN OVERRIDE
    // ═══════════════════════════════════════════════════════

    /// Processes a human override command.
    ///
    /// # Errors
    /// Returns `Err` on audit failure.
    pub fn human_override(&mut self, cmd: HumanCommand) -> Result<(), AvcError> {
        let description = match &cmd {
            HumanCommand::ForceState(s) => format!("HUMAN_OVERRIDE: force state to {s:?}"),
            HumanCommand::AdjustThreshold { domain, .. } => {
                format!("HUMAN_OVERRIDE: adjust threshold for {domain}")
            }
            HumanCommand::Halt => "HUMAN_OVERRIDE: system halt".to_string(),
        };

        let audit = AuditRecord::stamp(description)?;
        self.ledger.push(audit);

        match cmd {
            HumanCommand::ForceState(s) => self.state = s,
            HumanCommand::AdjustThreshold { domain, threshold } => {
                self.thresholds.insert(domain, threshold);
            }
            HumanCommand::Halt => self.halted = true,
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════
    // METRICS
    // ═══════════════════════════════════════════════════════

    /// Returns a snapshot of system metrics.
    #[must_use]
    pub fn metrics(&self) -> AvcMetrics {
        AvcMetrics {
            state: self.state,
            total_events: self.total_events,
            total_detections: self.total_detections,
            total_alerts: self.total_alerts,
            total_escalations: self.total_escalations,
            true_positives: self.true_positives,
            false_positives: self.false_positives,
            true_negatives: self.true_negatives,
            false_negatives: self.false_negatives,
            precision: self.precision(),
            recall: self.recall(),
            false_positive_rate: self.false_positive_rate(),
            false_negative_rate: self.false_negative_rate(),
            health_score: self.health_score(),
            domains: self.baselines.len(),
            clients: self.clients.len(),
            ledger_size: self.ledger.len(),
            feedback_buffer_size: self.feedback_buffer.len(),
        }
    }

    /// Returns the audit ledger.
    #[must_use]
    pub fn ledger(&self) -> &[AuditRecord<String>] {
        &self.ledger
    }
}

/// Result of executing an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// The action that was taken.
    pub action: PvAction,
    /// System state at the time of decision.
    pub state_at_decision: OperationalState,
    /// Timestamp.
    pub timestamp: SystemTime,
}

/// System metrics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvcMetrics {
    pub state: OperationalState,
    pub total_events: u64,
    pub total_detections: u64,
    pub total_alerts: u64,
    pub total_escalations: u64,
    pub true_positives: u64,
    pub false_positives: u64,
    pub true_negatives: u64,
    pub false_negatives: u64,
    pub precision: f64,
    pub recall: f64,
    pub false_positive_rate: f64,
    pub false_negative_rate: f64,
    pub health_score: f64,
    pub domains: usize,
    pub clients: usize,
    pub ledger_size: usize,
    pub feedback_buffer_size: usize,
}

impl GroundsTo for Avc {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,  // κ - detection core
            LexPrimitiva::Sequence,    // σ - event streams
            LexPrimitiva::Boundary,    // ∂ - thresholds
            LexPrimitiva::Recursion,   // ρ - learning loop
            LexPrimitiva::State,       // ς - operational state
            LexPrimitiva::Mapping,     // μ - domain/client routing
            LexPrimitiva::Persistence, // π - audit ledger
            LexPrimitiva::Causality,   // → - action chain
            LexPrimitiva::Quantity,    // N - metrics
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex_primitiva::GroundingTier;

    fn setup_avc() -> Avc {
        let mut avc = Avc::new();

        // Configure a domain
        let baseline = Baseline::from_values(&[10.0, 11.0, 9.0, 10.5, 10.0]);
        avc.set_baseline("pharma", baseline);

        if let Ok(thresh) = Threshold::new(0.0, 5.0) {
            avc.set_threshold("pharma", thresh);
        }

        avc
    }

    #[test]
    fn test_avc_t3_grounding() {
        let comp = Avc::primitive_composition();
        assert_eq!(
            GroundingTier::classify(&comp),
            GroundingTier::T3DomainSpecific
        );
        assert_eq!(comp.unique().len(), 9);
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn test_avc_default_state() {
        let avc = Avc::new();
        assert_eq!(avc.state(), OperationalState::Nominal);
        assert!(!avc.is_halted());
        assert!((avc.health_score() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_avc_process_no_signal() {
        let mut avc = setup_avc();

        // Event within baseline range — should NOT trigger
        let event = Event::new("pharma", 10.5, Source("test".into()));
        let result = avc.process(event);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert!(matches!(r.action, PvAction::Log { .. }));
        }

        let metrics = avc.metrics();
        assert_eq!(metrics.total_events, 1);
        assert_eq!(metrics.total_detections, 0);
    }

    #[test]
    fn test_avc_process_alert() {
        let mut avc = setup_avc();

        // Event far from baseline — delta > 5.0 threshold
        let event = Event::new("pharma", 20.0, Source("test".into()));
        let result = avc.process(event);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert!(matches!(r.action, PvAction::AutoAlert { .. }));
        }

        let metrics = avc.metrics();
        assert_eq!(metrics.total_detections, 1);
        assert_eq!(metrics.total_alerts, 1);
    }

    #[test]
    fn test_avc_process_escalation() {
        let mut avc = setup_avc();

        // Set a tight threshold so delta exceeds CRITICAL_BOUNDARY
        if let Ok(thresh) = Threshold::new(0.0, 0.01) {
            avc.set_threshold("pharma", thresh);
        }

        // Event that produces delta >= 0.95 (CRITICAL_BOUNDARY)
        let event = Event::new("pharma", 11.0, Source("test".into()));
        let result = avc.process(event);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert!(matches!(r.action, PvAction::Escalate { .. }));
        }

        let metrics = avc.metrics();
        assert_eq!(metrics.total_escalations, 1);
    }

    #[test]
    fn test_avc_feedback_learning() {
        let mut avc = setup_avc();

        let det = Detection {
            comparison: Comparison::new(1.0, 0.0),
            signal_present: true,
            confidence: 0.9,
            severity: 0.5,
        };

        // Record true positives
        for _ in 0..50 {
            avc.record_feedback(Feedback::new(det, Outcome::TruePositive));
        }
        // Record false positives
        for _ in 0..50 {
            avc.record_feedback(Feedback::new(det, Outcome::FalsePositive));
        }

        // After 100 feedbacks, calibration should have triggered
        assert_eq!(avc.feedback_buffer.len(), 0); // Cleared after batch

        let metrics = avc.metrics();
        assert_eq!(metrics.true_positives, 50);
        assert_eq!(metrics.false_positives, 50);
        assert!(metrics.false_positive_rate > 0.0);
    }

    #[test]
    fn test_avc_homeostasis() {
        let mut avc = Avc::new();

        // Simulate poor detection performance
        let det = Detection {
            comparison: Comparison::new(1.0, 0.0),
            signal_present: true,
            confidence: 0.9,
            severity: 0.5,
        };

        // All false positives → health degrades
        for _ in 0..10 {
            avc.record_feedback(Feedback::new(det, Outcome::FalsePositive));
        }

        avc.homeostasis();
        // Health = F1(precision=0, recall=1) = 0 → Critical
        assert_eq!(avc.state(), OperationalState::Critical);
    }

    #[test]
    fn test_avc_human_override() {
        let mut avc = Avc::new();

        let result = avc.human_override(HumanCommand::ForceState(OperationalState::Alert));
        assert!(result.is_ok());
        assert_eq!(avc.state(), OperationalState::Alert);

        let result = avc.human_override(HumanCommand::Halt);
        assert!(result.is_ok());
        assert!(avc.is_halted());

        // Verify events are rejected when halted
        let event = Event::new("test", 1.0, Source("test".into()));
        assert!(avc.process(event).is_err());
    }

    #[test]
    fn test_avc_audit_trail() {
        let mut avc = setup_avc();

        let event = Event::new("pharma", 10.5, Source("test".into()));
        let _result = avc.process(event);

        assert!(!avc.ledger().is_empty());

        // Verify audit integrity
        for record in avc.ledger() {
            let verified = record.verify();
            assert!(verified.is_ok());
            if let Ok(v) = verified {
                assert!(v);
            }
        }
    }

    #[test]
    fn test_avc_unknown_domain() {
        let mut avc = Avc::new();

        let event = Event::new("unknown", 1.0, Source("test".into()));
        let ingest = avc.ingest(event.clone());
        assert!(ingest.is_ok()); // Ingest always works

        let compare = avc.compare(&event);
        assert!(compare.is_err()); // Compare fails without baseline
    }

    #[test]
    fn test_avc_metrics_snapshot() {
        let avc = Avc::new();
        let metrics = avc.metrics();

        assert_eq!(metrics.total_events, 0);
        assert_eq!(metrics.total_detections, 0);
        assert!((metrics.precision - 1.0).abs() < f64::EPSILON);
        assert!((metrics.recall - 1.0).abs() < f64::EPSILON);
        assert!((metrics.health_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_nine_primitives_coverage() {
        let comp = Avc::primitive_composition();
        let unique = comp.unique();

        // Verify all 9 primitives from the pseudocode
        assert!(unique.contains(&LexPrimitiva::Comparison)); // κ
        assert!(unique.contains(&LexPrimitiva::Sequence)); // σ
        assert!(unique.contains(&LexPrimitiva::Boundary)); // ∂
        assert!(unique.contains(&LexPrimitiva::Recursion)); // ρ
        assert!(unique.contains(&LexPrimitiva::State)); // ς
        assert!(unique.contains(&LexPrimitiva::Mapping)); // μ
        assert!(unique.contains(&LexPrimitiva::Persistence)); // π
        assert!(unique.contains(&LexPrimitiva::Causality)); // →
        assert!(unique.contains(&LexPrimitiva::Quantity)); // N

        assert_eq!(unique.len(), 9);
    }
}
