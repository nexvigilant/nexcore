//! # Cross-Domain Transfer Primitives (T2-P)
//!
//! Universal computational patterns that transfer across domains with 2-3 unique
//! Lex Primitiva groundings. These are the "missing middle" between T1 atoms and
//! T2-C composites — extracted from the 2026-02-05 primitive validation audit.
//!
//! Each type is designed to be domain-agnostic while capturing a specific
//! computational pattern found across pharmacovigilance, biology, economics,
//! and systems engineering.

use std::fmt;

use serde::{Deserialize, Serialize};

// ============================================================================
// Error types (SEMVER-01)
// ============================================================================

/// Error from staged validation operations.
#[derive(Debug, Clone, PartialEq)]
pub enum StagedValidationError {
    /// All stages already completed
    AlreadyComplete,
    /// Insufficient evidence to advance
    InsufficientEvidence,
}

impl fmt::Display for StagedValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyComplete => write!(f, "already complete"),
            Self::InsufficientEvidence => write!(f, "insufficient evidence"),
        }
    }
}

/// Error from atomicity operations.
#[derive(Debug, Clone, PartialEq)]
pub enum AtomicityError {
    /// Cannot rollback a committed operation
    AlreadyCommitted,
}

impl fmt::Display for AtomicityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyCommitted => write!(f, "cannot rollback committed operation"),
        }
    }
}

/// Error from serialization guard operations.
#[derive(Debug, Clone, PartialEq)]
pub enum SerializationGuardError {
    /// Sequence number does not match expected order
    OutOfOrder,
}

impl fmt::Display for SerializationGuardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfOrder => write!(f, "out-of-order sequence"),
        }
    }
}

/// Error from rate limiter operations.
#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitError {
    /// Rate limit exceeded for current window
    Exceeded,
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exceeded => write!(f, "rate limit exceeded"),
        }
    }
}

/// Error from decomposition/mining depth operations.
#[derive(Debug, Clone, PartialEq)]
pub enum DepthError {
    /// Maximum recursion depth exceeded
    MaxDepthExceeded,
}

impl fmt::Display for DepthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaxDepthExceeded => write!(f, "max depth exceeded"),
        }
    }
}

// ============================================================================
// 1. ThreatSignature — T2-P: ∂ (Boundary) + μ (Mapping)
// ============================================================================

/// A pattern that identifies a boundary violation.
///
/// Transfers from: immunology (PAMP/DAMP), cybersecurity (IOC), ML (anomaly score).
/// Computing analog: any pattern that triggers a boundary-crossing response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatSignature {
    /// Identifier for the pattern class
    pub pattern_id: String,
    /// What boundary does this pattern threaten?
    pub boundary: String,
    /// Confidence that this signature represents a real threat (0.0-1.0)
    pub confidence: f64,
}

impl ThreatSignature {
    pub fn new(
        pattern_id: impl Into<String>,
        boundary: impl Into<String>,
        confidence: f64,
    ) -> Self {
        Self {
            pattern_id: pattern_id.into(),
            boundary: boundary.into(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Whether this signature exceeds a detection threshold
    #[must_use]
    pub fn exceeds_threshold(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }
}

impl fmt::Display for ThreatSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Threat({}, c={:.2})", self.pattern_id, self.confidence)
    }
}

// ============================================================================
// 2. ResourceRatio — T2-P: κ (Comparison) + N (Quantity)
// ============================================================================

/// A ratio between two quantities that governs system behavior.
///
/// Transfers from: biochemistry (ATP/ADP), economics (debt/equity), systems (load/capacity).
/// Computing analog: any ratio that determines regime or mode of operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceRatio {
    /// Numerator resource (available/active)
    pub available: f64,
    /// Denominator resource (total/capacity)
    pub capacity: f64,
}

impl ResourceRatio {
    pub fn new(available: f64, capacity: f64) -> Self {
        Self {
            available: available.max(0.0),
            capacity: capacity.max(f64::EPSILON),
        }
    }

    /// The ratio value (0.0 to +inf, typically 0.0-1.0)
    #[must_use]
    pub fn ratio(&self) -> f64 {
        self.available / self.capacity
    }

    /// Whether the ratio is above a threshold (resource sufficient)
    #[must_use]
    pub fn is_sufficient(&self, threshold: f64) -> bool {
        self.ratio() >= threshold
    }

    /// Whether the ratio indicates exhaustion (< 0.1)
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.ratio() < 0.1
    }
}

impl fmt::Display for ResourceRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.2}/{:.2} ({:.1}%)",
            self.available,
            self.capacity,
            self.ratio() * 100.0
        )
    }
}

// ============================================================================
// 3. PatternMatcher — T2-P: μ (Mapping) + κ (Comparison)
// ============================================================================

/// A rule that maps input to a match/no-match decision.
///
/// Transfers from: immunology (antibody), regex (pattern), ML (classifier).
/// Computing analog: any predicate that recognizes specific input patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternMatcher {
    /// Human-readable name for this matcher
    pub name: String,
    /// The pattern specification (domain-specific encoding)
    pub pattern: String,
    /// Sensitivity: probability of true positive (0.0-1.0)
    pub sensitivity: f64,
    /// Specificity: probability of true negative (0.0-1.0)
    pub specificity: f64,
}

impl PatternMatcher {
    pub fn new(name: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pattern: pattern.into(),
            sensitivity: 1.0,
            specificity: 1.0,
        }
    }

    pub fn with_performance(mut self, sensitivity: f64, specificity: f64) -> Self {
        self.sensitivity = sensitivity.clamp(0.0, 1.0);
        self.specificity = specificity.clamp(0.0, 1.0);
        self
    }

    /// F1-score of the matcher (harmonic mean of precision and recall)
    #[must_use]
    pub fn f1_score(&self) -> f64 {
        let precision = self.sensitivity; // simplified: precision ≈ sensitivity when balanced
        let recall = self.sensitivity;
        if precision + recall == 0.0 {
            return 0.0;
        }
        2.0 * precision * recall / (precision + recall)
    }
}

impl fmt::Display for PatternMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Matcher({}, sens={:.2})", self.name, self.sensitivity)
    }
}

// ============================================================================
// 4. ExploreExploit — T2-P: ρ (Recursion) + κ (Comparison)
// ============================================================================

/// A decision point between exploring new territory and exploiting known good.
///
/// Transfers from: RL (epsilon-greedy), game theory (minimax), search (BFS/DFS).
/// Computing analog: any iterative choice between discovery and optimization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExploreExploit {
    /// Exploration rate (0.0 = pure exploit, 1.0 = pure explore)
    pub epsilon: f64,
    /// Number of iterations performed
    pub iterations: u64,
    /// Best known value so far
    pub best_known: f64,
}

impl ExploreExploit {
    pub fn new(epsilon: f64) -> Self {
        Self {
            epsilon: epsilon.clamp(0.0, 1.0),
            iterations: 0,
            best_known: f64::NEG_INFINITY,
        }
    }

    /// Should the next action explore or exploit?
    #[must_use]
    pub fn should_explore(&self) -> bool {
        // Deterministic threshold (for testability)
        // In production, use random < epsilon
        self.epsilon > 0.5
    }

    /// Decay epsilon toward exploitation over iterations
    pub fn decay(&mut self, decay_rate: f64) {
        self.iterations += 1;
        self.epsilon *= (1.0 - decay_rate).max(0.0);
    }

    /// Update best known value
    pub fn observe(&mut self, value: f64) {
        if value > self.best_known {
            self.best_known = value;
        }
    }
}

impl fmt::Display for ExploreExploit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ε={:.2} (n={}, best={:.2})",
            self.epsilon, self.iterations, self.best_known
        )
    }
}

// ============================================================================
// 5. EventClassifier — T2-P: μ (Mapping) + κ (Comparison) + ∂ (Boundary)
// ============================================================================

/// Classifies an event into a category based on boundary rules.
///
/// Transfers from: ML (multiclass), triage (severity), monitoring (alert level).
/// Computing analog: any mapping from observation to discrete class via thresholds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventClassifier {
    /// Classifier name
    pub name: String,
    /// Number of classes
    pub num_classes: usize,
    /// Boundary thresholds between classes (sorted ascending)
    pub boundaries: Vec<f64>,
}

impl EventClassifier {
    pub fn new(name: impl Into<String>, boundaries: Vec<f64>) -> Self {
        let num_classes = boundaries.len() + 1;
        let mut sorted = boundaries;
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Self {
            name: name.into(),
            num_classes,
            boundaries: sorted,
        }
    }

    /// Classify a value into a class index (0-based)
    #[must_use]
    pub fn classify(&self, value: f64) -> usize {
        for (i, threshold) in self.boundaries.iter().enumerate() {
            if value < *threshold {
                return i;
            }
        }
        self.num_classes - 1
    }
}

impl fmt::Display for EventClassifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Classifier({}, {} classes)", self.name, self.num_classes)
    }
}

// ============================================================================
// 6. FeedbackLoop — T2-P: ρ (Recursion) + κ (Comparison) + → (Causality)
// ============================================================================

/// A self-correcting cycle that adjusts output based on measured error.
///
/// Transfers from: control theory (PID), cybernetics, biology (homeostasis).
/// Computing analog: any system that measures deviation and applies correction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedbackLoop {
    /// Target setpoint
    pub setpoint: f64,
    /// Current measured value
    pub current: f64,
    /// Gain factor for correction
    pub gain: f64,
}

impl FeedbackLoop {
    pub fn new(setpoint: f64, gain: f64) -> Self {
        Self {
            setpoint,
            current: 0.0,
            gain: gain.max(0.0),
        }
    }

    /// Error: difference between setpoint and current
    #[must_use]
    pub fn error(&self) -> f64 {
        self.setpoint - self.current
    }

    /// Correction signal: error * gain
    #[must_use]
    pub fn correction(&self) -> f64 {
        self.error() * self.gain
    }

    /// Apply one tick of the feedback loop
    pub fn tick(&mut self) {
        self.current += self.correction();
    }

    /// Whether the system has converged (error < tolerance)
    #[must_use]
    pub fn converged(&self, tolerance: f64) -> bool {
        self.error().abs() < tolerance
    }
}

impl fmt::Display for FeedbackLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FB(sp={:.1}, cur={:.1}, e={:.2})",
            self.setpoint,
            self.current,
            self.error()
        )
    }
}

// ============================================================================
// 7. SchemaContract — T2-P: μ (Mapping) + π (Persistence) + ∂ (Boundary)
// ============================================================================

/// A versioned agreement on data structure between producer and consumer.
///
/// Transfers from: databases (schema), APIs (contract), biology (ribosome).
/// Computing analog: any specification that constrains valid data shapes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaContract {
    /// Contract name/identifier
    pub name: String,
    /// Version (monotonically increasing)
    pub version: u32,
    /// Number of required fields
    pub required_fields: usize,
    /// Whether backward-compatible changes are allowed
    pub backward_compatible: bool,
}

impl SchemaContract {
    pub fn new(name: impl Into<String>, version: u32, required_fields: usize) -> Self {
        Self {
            name: name.into(),
            version,
            required_fields,
            backward_compatible: true,
        }
    }

    /// Check if a newer version is compatible with this one
    #[must_use]
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        if other.version < self.version {
            return false;
        }
        if !self.backward_compatible {
            return other.version == self.version;
        }
        // Backward compatible: newer version must have >= required fields
        other.required_fields >= self.required_fields
    }
}

impl fmt::Display for SchemaContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Schema({} v{}, {} fields)",
            self.name, self.version, self.required_fields
        )
    }
}

// ============================================================================
// 8. MessageBus — T2-P: σ (Sequence) + μ (Mapping) + → (Causality)
// ============================================================================

/// An ordered channel that routes messages from producers to consumers.
///
/// Transfers from: biology (cytokine), distributed systems (Kafka), hardware (bus).
/// Computing analog: any pub/sub or event dispatch mechanism.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageBus {
    /// Bus name
    pub name: String,
    /// Number of registered subscribers
    pub subscriber_count: usize,
    /// Total messages dispatched
    pub messages_dispatched: u64,
    /// Whether message ordering is guaranteed
    pub ordered: bool,
}

impl MessageBus {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            subscriber_count: 0,
            messages_dispatched: 0,
            ordered: true,
        }
    }

    pub fn add_subscriber(&mut self) {
        self.subscriber_count += 1;
    }

    pub fn dispatch(&mut self) {
        self.messages_dispatched += 1;
    }

    /// Fan-out ratio: messages per subscriber
    #[must_use]
    pub fn fan_out(&self) -> f64 {
        if self.subscriber_count == 0 {
            return 0.0;
        }
        self.messages_dispatched as f64 / self.subscriber_count as f64
    }
}

impl fmt::Display for MessageBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bus({}, {} subs, {} msgs)",
            self.name, self.subscriber_count, self.messages_dispatched
        )
    }
}

// ============================================================================
// 9. SpecializedWorker — T2-P: μ (Mapping) + σ (Sequence)
// ============================================================================

/// A unit of computation specialized for a specific task type.
///
/// Transfers from: biology (cell), microservices (service), manufacturing (station).
/// Computing analog: any processing unit with a defined input→output contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecializedWorker {
    /// Worker identifier
    pub id: String,
    /// What task type this worker handles
    pub specialization: String,
    /// Tasks completed
    pub tasks_completed: u64,
    /// Whether the worker is currently processing
    pub busy: bool,
}

impl SpecializedWorker {
    pub fn new(id: impl Into<String>, specialization: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            specialization: specialization.into(),
            tasks_completed: 0,
            busy: false,
        }
    }

    pub fn start_task(&mut self) {
        self.busy = true;
    }

    pub fn complete_task(&mut self) {
        self.busy = false;
        self.tasks_completed += 1;
    }

    /// Throughput: tasks per unit (abstract)
    #[must_use]
    pub fn throughput(&self) -> u64 {
        self.tasks_completed
    }
}

impl fmt::Display for SpecializedWorker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.busy { "busy" } else { "idle" };
        write!(f, "Worker({}/{}, {})", self.id, self.specialization, status)
    }
}

// ============================================================================
// 10. DecayFunction — T2-P: → (Causality) + N (Quantity) + ∝ (Irreversibility)
// ============================================================================

/// A quantity that decreases over time following a mathematical decay law.
///
/// Transfers from: physics (half-life), chemistry (degradation), caching (TTL).
/// Computing analog: any time-bounded resource that loses value or expires.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecayFunction {
    /// Initial value at t=0
    pub initial_value: f64,
    /// Half-life in abstract time units
    pub half_life: f64,
}

impl DecayFunction {
    pub fn new(initial_value: f64, half_life: f64) -> Self {
        Self {
            initial_value: initial_value.max(0.0),
            half_life: half_life.max(f64::EPSILON),
        }
    }

    /// Value remaining at time t
    #[must_use]
    pub fn value_at(&self, t: f64) -> f64 {
        self.initial_value * (0.5_f64).powf(t / self.half_life)
    }

    /// Time until value drops below threshold
    #[must_use]
    pub fn time_to_threshold(&self, threshold: f64) -> f64 {
        if threshold <= 0.0 || threshold >= self.initial_value {
            return 0.0;
        }
        self.half_life * (self.initial_value / threshold).log2()
    }

    /// Whether the value has decayed past a threshold at time t
    #[must_use]
    pub fn is_expired(&self, t: f64, threshold: f64) -> bool {
        self.value_at(t) < threshold
    }
}

impl fmt::Display for DecayFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Decay(v₀={:.1}, t½={:.1})",
            self.initial_value, self.half_life
        )
    }
}

// ============================================================================
// 11. Homeostasis — T2-P: ρ (Recursion) + κ (Comparison) + ∂ (Boundary)
// ============================================================================

/// Self-correcting feedback loop maintaining a setpoint within tolerance.
///
/// Transfers from: biology (thermoregulation), control theory (PID),
/// AI orchestration (Guardian SENSE→COMPARE→ACT), token budgeting (energy regimes).
/// Computing analog: watchdog/autoscaler with deadband.
///
/// Distinction from FeedbackLoop: FeedbackLoop is an open-ended P-controller.
/// Homeostasis adds a tolerance band (∂ boundary) — converge to a zone, not a point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Homeostasis {
    /// Target equilibrium value
    pub setpoint: f64,
    /// Acceptable deviation from setpoint (always > 0)
    pub tolerance: f64,
    /// Current measured value
    pub current: f64,
    /// Gain factor for correction signal
    pub correction_gain: f64,
}

impl Homeostasis {
    pub fn new(setpoint: f64, tolerance: f64, correction_gain: f64) -> Self {
        Self {
            setpoint,
            tolerance: tolerance.max(f64::EPSILON),
            current: setpoint,
            correction_gain: correction_gain.max(0.0),
        }
    }

    /// Error: difference between setpoint and current
    #[must_use]
    pub fn error(&self) -> f64 {
        self.setpoint - self.current
    }

    /// Whether the current value is within the tolerance band
    #[must_use]
    pub fn in_tolerance(&self) -> bool {
        self.error().abs() <= self.tolerance
    }

    /// Correction signal (zero if already in tolerance)
    #[must_use]
    pub fn correction(&self) -> f64 {
        if self.in_tolerance() {
            0.0
        } else {
            self.error() * self.correction_gain
        }
    }

    /// Apply one correction step
    pub fn tick(&mut self) {
        self.current += self.correction();
    }
}

impl fmt::Display for Homeostasis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.in_tolerance() { "OK" } else { "DRIFT" };
        write!(
            f,
            "Homeo(sp={:.1}±{:.1}, cur={:.1}, {})",
            self.setpoint, self.tolerance, self.current, status
        )
    }
}

// ============================================================================
// 12. StagedValidation — T2-P: σ (Sequence) + ∂ (Boundary) + π (Persistence)
// ============================================================================

/// Multi-stage gating with progressive evidence accumulation.
///
/// Transfers from: pharma (clinical trial phases I→II→III→IV),
/// code quality (CTVP 5-phase), skill validation (Diamond v2),
/// agentic hooks (exit 0→1→2 escalation).
/// Computing analog: pipeline with stage gates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StagedValidation {
    /// Total number of stages
    pub stages: usize,
    /// Current stage (0-indexed, 0 = not started, stages = complete)
    pub current_stage: usize,
    /// Evidence accumulated in current stage
    pub evidence_accumulated: f64,
    /// Evidence threshold required to advance each stage
    pub threshold_per_stage: f64,
}

impl StagedValidation {
    pub fn new(stages: usize, threshold_per_stage: f64) -> Self {
        Self {
            stages: stages.max(1),
            current_stage: 0,
            evidence_accumulated: 0.0,
            threshold_per_stage: threshold_per_stage.max(f64::EPSILON),
        }
    }

    /// Whether enough evidence has been accumulated to advance
    #[must_use]
    pub fn can_advance(&self) -> bool {
        self.evidence_accumulated >= self.threshold_per_stage && self.current_stage < self.stages
    }

    /// Advance to the next stage, resetting evidence. Returns Err if insufficient evidence.
    #[must_use]
    pub fn advance(&mut self) -> Result<usize, StagedValidationError> {
        if self.current_stage >= self.stages {
            return Err(StagedValidationError::AlreadyComplete);
        }
        if !self.can_advance() {
            return Err(StagedValidationError::InsufficientEvidence);
        }
        self.evidence_accumulated = 0.0;
        self.current_stage += 1;
        Ok(self.current_stage)
    }

    /// Whether all stages have been completed
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current_stage >= self.stages
    }

    /// Progress as a fraction (0.0 to 1.0)
    #[must_use]
    pub fn progress(&self) -> f64 {
        self.current_stage as f64 / self.stages as f64
    }
}

impl fmt::Display for StagedValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Stage {}/{} ({:.0}%)",
            self.current_stage,
            self.stages,
            self.progress() * 100.0
        )
    }
}

// ============================================================================
// 13. Atomicity — T2-P: Σ (Sum) + ∝ (Irreversibility)
// ============================================================================

/// Operation that completes fully or not at all — no partial state.
///
/// Transfers from: databases (transactions), Rust (Result<T,E>),
/// hook enforcement (exit 0 or non-zero), CTVP phase gates.
/// Computing analog: commit/rollback, all-or-nothing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Atomicity {
    /// Operation identifier
    pub operation: String,
    /// Whether the operation has been committed
    pub committed: bool,
}

impl Atomicity {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            committed: false,
        }
    }

    /// Mark the operation as committed (irreversible)
    pub fn commit(&mut self) {
        self.committed = true;
    }

    /// Roll back (only possible if not yet committed)
    #[must_use]
    pub fn rollback(&mut self) -> Result<(), AtomicityError> {
        if self.committed {
            return Err(AtomicityError::AlreadyCommitted);
        }
        Ok(())
    }

    /// Whether the operation is committed
    #[must_use]
    pub fn is_committed(&self) -> bool {
        self.committed
    }
}

impl fmt::Display for Atomicity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.committed {
            "COMMITTED"
        } else {
            "PENDING"
        };
        write!(f, "Atomic({}, {})", self.operation, status)
    }
}

// ============================================================================
// 14. CompareAndSwap — T2-P: κ (Comparison) + ∝ (Irreversibility)
// ============================================================================

/// Atomic conditional update: succeeds only if current value matches expected.
///
/// Transfers from: hardware (CPU CAS instruction), databases (optimistic locking),
/// biology (enzyme lock-and-key specificity), economics (fill-or-kill orders).
/// Computing analog: lock-free compare-and-swap, version-gated writes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompareAndSwap {
    /// The value expected at time of check
    pub expected: f64,
    /// The value to write if expected matches current
    pub desired: f64,
    /// The actual value witnessed during the operation
    pub witnessed: f64,
    /// Whether the swap succeeded
    pub succeeded: bool,
}

impl CompareAndSwap {
    pub fn new(expected: f64, desired: f64) -> Self {
        Self {
            expected,
            desired,
            witnessed: expected, // optimistic: assume match
            succeeded: false,
        }
    }

    /// Attempt the swap against a current value. Returns the witnessed value.
    #[must_use]
    pub fn execute(&mut self, current: f64) -> f64 {
        self.witnessed = current;
        if (current - self.expected).abs() < f64::EPSILON {
            self.succeeded = true;
        } else {
            self.succeeded = false;
        }
        self.witnessed
    }

    /// Whether the CAS succeeded
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.succeeded
    }

    /// The value that was actually observed (useful for retry loops)
    #[must_use]
    pub fn witnessed(&self) -> f64 {
        self.witnessed
    }
}

impl fmt::Display for CompareAndSwap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.succeeded { "OK" } else { "FAIL" };
        write!(
            f,
            "CAS({:.2}→{:.2}, {})",
            self.expected, self.desired, status
        )
    }
}

// ============================================================================
// 15. ToctouWindow — T2-P: σ (Sequence) + ∂ (Boundary) + ν (Frequency)
// ============================================================================

/// Time-of-check to time-of-use gap: models staleness between verification and action.
///
/// Transfers from: OS security (permission check then file open), PV (signal detected
/// but data changes before action), caching (TTL expiry between read and use),
/// distributed systems (lease expiry).
/// Computing analog: any check whose result can go stale before consumption.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToctouWindow {
    /// What was checked
    pub check_label: String,
    /// Time of check (abstract units)
    pub check_time: u64,
    /// Time of use (abstract units)
    pub use_time: u64,
    /// Maximum allowed gap before staleness
    pub max_gap: u64,
}

impl ToctouWindow {
    pub fn new(check_label: impl Into<String>, max_gap: u64) -> Self {
        Self {
            check_label: check_label.into(),
            check_time: 0,
            use_time: 0,
            max_gap: max_gap.max(1),
        }
    }

    /// Record the check timestamp
    pub fn check(&mut self, time: u64) {
        self.check_time = time;
    }

    /// Record the use timestamp
    pub fn use_at(&mut self, time: u64) {
        self.use_time = time;
    }

    /// The gap between check and use
    #[must_use]
    pub fn gap(&self) -> u64 {
        self.use_time.saturating_sub(self.check_time)
    }

    /// Whether the check result is stale at time of use
    #[must_use]
    pub fn is_stale(&self) -> bool {
        self.gap() > self.max_gap
    }
}

impl fmt::Display for ToctouWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_stale() { "STALE" } else { "FRESH" };
        write!(
            f,
            "TOCTOU({}, gap={}, {})",
            self.check_label,
            self.gap(),
            status
        )
    }
}

// ============================================================================
// 16. SerializationGuard — T2-P: σ (Sequence) + ∝ (Irreversibility) + π (Persistence)
// ============================================================================

/// Enforces total ordering of operations via monotonic sequence numbers.
///
/// Transfers from: databases (WAL log sequence numbers), distributed systems
/// (Lamport clocks), biology (DNA replication checkpoints), version control (commit DAG).
/// Computing analog: monotonic counter ensuring no operation reordering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializationGuard {
    /// Name of the guarded resource
    pub resource: String,
    /// Last committed sequence number
    pub last_committed: u64,
    /// Next expected sequence number
    pub next_expected: u64,
}

impl SerializationGuard {
    pub fn new(resource: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            last_committed: 0,
            next_expected: 1,
        }
    }

    /// Attempt to commit an operation with the given sequence number.
    /// Succeeds only if sequence matches next_expected (total order).
    #[must_use]
    pub fn commit(&mut self, sequence: u64) -> Result<u64, SerializationGuardError> {
        if sequence != self.next_expected {
            return Err(SerializationGuardError::OutOfOrder);
        }
        self.last_committed = sequence;
        self.next_expected = sequence + 1;
        Ok(sequence)
    }

    /// Whether a given sequence number is in the future (not yet committable)
    #[must_use]
    pub fn is_future(&self, sequence: u64) -> bool {
        sequence > self.next_expected
    }

    /// Whether a given sequence number has already been committed
    #[must_use]
    pub fn is_committed(&self, sequence: u64) -> bool {
        sequence <= self.last_committed
    }
}

impl fmt::Display for SerializationGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Guard({}, seq={})", self.resource, self.last_committed)
    }
}

// ============================================================================
// 17. RateLimiter — T2-P: ν (Frequency) + ∂ (Boundary) + N (Quantity)
// ============================================================================

/// Caps throughput to a maximum frequency over a time window.
///
/// Transfers from: API throttling (requests/sec), pharmacokinetics (dosing intervals),
/// neuroscience (refractory period), traffic engineering (flow control).
/// Computing analog: token bucket / sliding window rate limiter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimiter {
    /// Maximum allowed events in the window
    pub max_events: u64,
    /// Window size in abstract time units
    pub window_size: u64,
    /// Events consumed in the current window
    pub current_count: u64,
    /// Start of the current window
    pub window_start: u64,
}

impl RateLimiter {
    pub fn new(max_events: u64, window_size: u64) -> Self {
        Self {
            max_events: max_events.max(1),
            window_size: window_size.max(1),
            current_count: 0,
            window_start: 0,
        }
    }

    /// Try to consume one event at the given time. Returns Ok if allowed, Err if rate-limited.
    #[must_use]
    pub fn try_acquire(&mut self, time: u64) -> Result<u64, RateLimitError> {
        // Roll window forward if expired
        if time >= self.window_start + self.window_size {
            self.window_start = time;
            self.current_count = 0;
        }
        if self.current_count >= self.max_events {
            return Err(RateLimitError::Exceeded);
        }
        self.current_count += 1;
        Ok(self.remaining())
    }

    /// Remaining capacity in the current window
    #[must_use]
    pub fn remaining(&self) -> u64 {
        self.max_events.saturating_sub(self.current_count)
    }

    /// Whether the limiter is currently exhausted
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.current_count >= self.max_events
    }

    /// Utilization as a fraction (0.0 to 1.0)
    #[must_use]
    pub fn utilization(&self) -> f64 {
        self.current_count as f64 / self.max_events as f64
    }
}

impl fmt::Display for RateLimiter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RateLimit({}/{}, {:.0}%)",
            self.current_count,
            self.max_events,
            self.utilization() * 100.0
        )
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(100, 60) // 100 events per 60-unit window
    }
}

// ============================================================================
// 18. CircuitBreaker — T2-P: ς (State) + ∂ (Boundary) + κ (Comparison)
// ============================================================================

/// Three-state resilience pattern: Closed → Open → HalfOpen → Closed.
///
/// Transfers from: electrical engineering (fuse/breaker), microservices (resilience),
/// biology (nerve impulse gating / refractory period), economics (market circuit breakers).
/// Computing analog: fail-fast with recovery probe.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BreakerState {
    /// Normal operation — requests flow through
    Closed,
    /// Tripped — all requests rejected
    Open,
    /// Recovery probe — single request allowed to test
    HalfOpen,
}

impl fmt::Display for BreakerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => write!(f, "Closed"),
            Self::Open => write!(f, "Open"),
            Self::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircuitBreaker {
    /// Current breaker state
    pub state: BreakerState,
    /// Consecutive failures in Closed state
    pub failure_count: u64,
    /// Threshold of failures that trips the breaker
    pub failure_threshold: u64,
    /// Successes needed in HalfOpen to close again
    pub recovery_threshold: u64,
    /// Consecutive successes in HalfOpen
    pub recovery_count: u64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, recovery_threshold: u64) -> Self {
        Self {
            state: BreakerState::Closed,
            failure_count: 0,
            failure_threshold: failure_threshold.max(1),
            recovery_threshold: recovery_threshold.max(1),
            recovery_count: 0,
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        match self.state {
            BreakerState::Closed => {
                self.failure_count = 0;
            }
            BreakerState::HalfOpen => {
                self.recovery_count += 1;
                if self.recovery_count >= self.recovery_threshold {
                    self.state = BreakerState::Closed;
                    self.failure_count = 0;
                    self.recovery_count = 0;
                }
            }
            BreakerState::Open => {} // ignored in open state
        }
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        match self.state {
            BreakerState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = BreakerState::Open;
                }
            }
            BreakerState::HalfOpen => {
                self.state = BreakerState::Open;
                self.recovery_count = 0;
            }
            BreakerState::Open => {} // already open
        }
    }

    /// Transition from Open to HalfOpen (called after cooldown period)
    pub fn attempt_reset(&mut self) {
        if self.state == BreakerState::Open {
            self.state = BreakerState::HalfOpen;
            self.recovery_count = 0;
        }
    }

    /// Whether requests should be allowed through
    #[must_use]
    pub fn is_allowing(&self) -> bool {
        self.state != BreakerState::Open
    }
}

impl fmt::Display for CircuitBreaker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Breaker({:?}, {}/{})",
            self.state, self.failure_count, self.failure_threshold
        )
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(3, 1) // reasonable defaults
    }
}

// ============================================================================
// 19. Idempotency — T2-P: ∃ (Existence) + κ (Comparison)
// ============================================================================

/// Ensures applying an operation N times has the same effect as applying it once.
///
/// Transfers from: HTTP (PUT idempotency), databases (upsert), biology (vaccine
/// booster plateau), mathematics (projection operators f(f(x)) = f(x)).
/// Computing analog: deduplication key / idempotency token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Idempotency {
    /// Unique key identifying this operation
    pub key: String,
    /// How many times the operation has been applied
    pub application_count: u64,
    /// Whether the operation has been applied at least once
    pub applied: bool,
}

impl Idempotency {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            application_count: 0,
            applied: false,
        }
    }

    /// Apply the operation. Returns true only on the first application.
    #[must_use]
    pub fn apply(&mut self) -> bool {
        self.application_count += 1;
        if !self.applied {
            self.applied = true;
            return true; // first application: side effect occurs
        }
        false // subsequent: no-op
    }

    /// Whether this operation has already been applied
    #[must_use]
    pub fn is_applied(&self) -> bool {
        self.applied
    }

    /// How many times apply() has been called (including duplicates)
    #[must_use]
    pub fn attempts(&self) -> u64 {
        self.application_count
    }
}

impl fmt::Display for Idempotency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.applied { "applied" } else { "pending" };
        write!(f, "Idem({}, {})", self.key, status)
    }
}

// ============================================================================
// 20. NegativeEvidence — T2-P: ∅ (Void) + κ (Comparison)
// ============================================================================

/// Absence of expected evidence used as a signal.
///
/// Transfers from: pharmacovigilance (missing reports as signal),
/// security (absence of heartbeat), statistics (zero-count cells),
/// diagnostics (dog that didn't bark)
/// Computing analog: timeout, missing ACK, null result interpretation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NegativeEvidence {
    /// What was expected but absent
    pub expected: String,
    /// Duration or scope of observation
    pub observation_scope: f64,
    /// Threshold below which absence is meaningful
    pub significance_threshold: f64,
    /// Observed count (0 = pure absence)
    pub observed_count: u64,
}

impl NegativeEvidence {
    pub fn new(
        expected: impl Into<String>,
        observation_scope: f64,
        significance_threshold: f64,
    ) -> Self {
        Self {
            expected: expected.into(),
            observation_scope: observation_scope.max(0.0),
            significance_threshold: significance_threshold.max(0.0),
            observed_count: 0,
        }
    }

    /// Whether the absence is significant (count below threshold)
    #[must_use]
    pub fn is_significant(&self) -> bool {
        (self.observed_count as f64) < self.significance_threshold
    }

    /// Record an observation (reduces the "absence" signal)
    pub fn observe(&mut self) {
        self.observed_count += 1;
    }

    /// Ratio of observed to expected threshold (0.0 = total absence, 1.0+ = no absence)
    #[must_use]
    pub fn evidence_ratio(&self) -> f64 {
        if self.significance_threshold == 0.0 {
            return 1.0;
        }
        self.observed_count as f64 / self.significance_threshold
    }
}

impl fmt::Display for NegativeEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Absence({}, {}/{})",
            self.expected, self.observed_count, self.significance_threshold
        )
    }
}

// ============================================================================
// 21. TopologicalAddress — T2-P: λ (Location) + σ (Sequence)
// ============================================================================

/// A hierarchical address within a topology (tree, graph, namespace).
///
/// Transfers from: filesystem paths, DNS names, module paths,
/// organizational hierarchy, supply chain location, network topology
/// Computing analog: URI, dotted notation, tree coordinate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopologicalAddress {
    /// Ordered path segments from root to leaf
    pub segments: Vec<String>,
    /// Separator convention (e.g., "/", ".", "::")
    pub separator: String,
}

impl TopologicalAddress {
    pub fn new(segments: Vec<String>, separator: impl Into<String>) -> Self {
        Self {
            segments,
            separator: separator.into(),
        }
    }

    /// Parse from a string representation
    pub fn parse(address: &str, separator: &str) -> Self {
        let segments: Vec<String> = address.split(separator).map(String::from).collect();
        Self {
            segments,
            separator: separator.to_string(),
        }
    }

    /// Depth in the topology (0 = root)
    #[must_use]
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// Whether this address is a prefix (ancestor) of another
    #[must_use]
    pub fn is_ancestor_of(&self, other: &Self) -> bool {
        if self.segments.len() >= other.segments.len() {
            return false;
        }
        self.segments
            .iter()
            .zip(&other.segments)
            .all(|(a, b)| a == b)
    }

    /// Render to string representation
    #[must_use]
    pub fn render(&self) -> String {
        self.segments.join(&self.separator)
    }

    /// Get the leaf (last segment)
    #[must_use]
    pub fn leaf(&self) -> Option<&str> {
        self.segments.last().map(|s| s.as_str())
    }
}

impl fmt::Display for TopologicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

// ============================================================================
// 22. Accumulator — T2-P: N (Quantity) + σ (Sequence) + ∝ (Irreversibility)
// ============================================================================

/// A monotonically increasing running total.
///
/// Transfers from: accounting (ledger balances), metrics (counters),
/// event sourcing (running projections), statistics (cumulative sums)
/// Computing analog: atomic counter, write-ahead log position, sequence number
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Accumulator {
    /// Current accumulated value
    pub total: f64,
    /// Number of additions
    pub additions: u64,
}

impl Accumulator {
    pub fn new() -> Self {
        Self {
            total: 0.0,
            additions: 0,
        }
    }

    /// Add a non-negative value to the accumulator
    #[must_use]
    pub fn add(&mut self, value: f64) -> f64 {
        let clamped = value.max(0.0);
        self.total += clamped;
        self.additions += 1;
        self.total
    }

    /// Current running average
    #[must_use]
    pub fn average(&self) -> f64 {
        if self.additions == 0 {
            return 0.0;
        }
        self.total / self.additions as f64
    }

    /// Whether any additions have been made
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.additions == 0
    }
}

impl Default for Accumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Accumulator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Σ={:.2} (n={})", self.total, self.additions)
    }
}

// ============================================================================
// 23. Checkpoint — T2-P: π (Persistence) + ς (State) + ∝ (Irreversibility)
// ============================================================================

/// A crash-recoverable snapshot of state at a known-good point.
///
/// Transfers from: databases (WAL checkpoints), gaming (save points),
/// version control (commits), distributed systems (Chandy-Lamport snapshots),
/// clinical trials (interim analysis freeze)
/// Computing analog: snapshot, savepoint, bookmark
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Identifier for this checkpoint
    pub label: String,
    /// Monotonic sequence number
    pub sequence: u64,
    /// Whether this checkpoint has been confirmed durable
    pub durable: bool,
}

impl Checkpoint {
    pub fn new(label: impl Into<String>, sequence: u64) -> Self {
        Self {
            label: label.into(),
            sequence,
            durable: false,
        }
    }

    /// Mark this checkpoint as durably persisted
    pub fn confirm(&mut self) {
        self.durable = true;
    }

    /// Whether this checkpoint is safe to recover from
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        self.durable
    }

    /// Whether this checkpoint is newer than another
    #[must_use]
    pub fn is_newer_than(&self, other: &Self) -> bool {
        self.sequence > other.sequence
    }
}

impl fmt::Display for Checkpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.durable { "DURABLE" } else { "pending" };
        write!(f, "CP({} #{}, {})", self.label, self.sequence, status)
    }
}

impl Default for Checkpoint {
    fn default() -> Self {
        Self::new("default", 0)
    }
}

// ============================================================================
// 24. Decomposition — T2-P: Σ (Sum) + μ (Mapping) + ρ (Recursion)
// ============================================================================

/// Recursive breakdown of complex wholes into constituent parts.
///
/// Transfers from: chemistry (dissociation), linguistics (morpheme analysis),
/// finance (factor analysis), biology (catabolism), FORGE (primitive mining).
/// Computing analog: parser, factorization, tree traversal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decomposition {
    /// Current decomposition depth
    pub depth: u32,
    /// Maximum recursion depth before stopping
    pub max_depth: u32,
    /// Number of parts extracted so far
    pub parts_extracted: u32,
    /// Whether decomposition is complete (no more reducible parts)
    pub complete: bool,
}

impl Decomposition {
    /// Create a new decomposition context with max depth limit.
    pub fn new(max_depth: u32) -> Self {
        Self {
            depth: 0,
            max_depth,
            parts_extracted: 0,
            complete: false,
        }
    }

    /// Descend one level deeper. Returns Err if max depth exceeded.
    #[must_use]
    pub fn descend(&mut self) -> Result<u32, DepthError> {
        if self.depth >= self.max_depth {
            return Err(DepthError::MaxDepthExceeded);
        }
        self.depth += 1;
        Ok(self.depth)
    }

    /// Ascend one level (backtrack).
    pub fn ascend(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Record extraction of a part at the current depth.
    pub fn extract_part(&mut self) {
        self.parts_extracted += 1;
    }

    /// Mark decomposition as complete (irreducible).
    pub fn mark_complete(&mut self) {
        self.complete = true;
    }

    /// Whether we're at the root level.
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.depth == 0
    }

    /// Yield metric: parts per depth unit.
    #[must_use]
    pub fn yield_ratio(&self) -> f64 {
        if self.depth == 0 {
            return 0.0;
        }
        self.parts_extracted as f64 / self.depth as f64
    }
}

impl fmt::Display for Decomposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Decomp(d={}/{}, parts={})",
            self.depth, self.max_depth, self.parts_extracted
        )
    }
}

// ============================================================================
// 25. CodeGeneration — T2-C: μ (Mapping) + ς (State) + σ (Sequence) + ∃ (Existence)
// ============================================================================

/// Staged transformation from specification to executable artifact.
///
/// Transfers from: biology (protein synthesis: DNA→mRNA→protein),
/// manufacturing (CAD→CAM→part), law (statute→regulation→compliance),
/// FORGE (spec→AST→code).
/// Computing analog: compiler pipeline, template engine, code emitter.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenerationStage {
    /// Specification input (e.g., AST, schema, template)
    Specification,
    /// Intermediate representation
    Intermediate,
    /// Output generation in progress
    Emitting,
    /// Validation/verification of output
    Validating,
    /// Complete — output exists
    Complete,
    /// Failed — output does not exist
    Failed,
}

impl fmt::Display for GenerationStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Specification => "specification",
            Self::Intermediate => "intermediate",
            Self::Emitting => "emitting",
            Self::Validating => "validating",
            Self::Complete => "complete",
            Self::Failed => "failed",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeGeneration {
    /// Current generation stage
    pub stage: GenerationStage,
    /// Tokens/units emitted so far
    pub output_count: u64,
    /// Whether output artifact exists (compilation succeeded)
    pub output_exists: bool,
    /// Validation errors encountered
    pub error_count: u32,
}

impl CodeGeneration {
    /// Create a new code generation context.
    pub fn new() -> Self {
        Self {
            stage: GenerationStage::Specification,
            output_count: 0,
            output_exists: false,
            error_count: 0,
        }
    }

    /// Advance to the next stage. Returns the new stage.
    #[must_use]
    pub fn advance(&mut self) -> GenerationStage {
        self.stage = match self.stage {
            GenerationStage::Specification => GenerationStage::Intermediate,
            GenerationStage::Intermediate => GenerationStage::Emitting,
            GenerationStage::Emitting => GenerationStage::Validating,
            GenerationStage::Validating => {
                if self.error_count == 0 {
                    self.output_exists = true;
                    GenerationStage::Complete
                } else {
                    GenerationStage::Failed
                }
            }
            GenerationStage::Complete | GenerationStage::Failed => self.stage,
        };
        self.stage
    }

    /// Emit output tokens during Emitting stage.
    pub fn emit(&mut self, count: u64) {
        if self.stage == GenerationStage::Emitting {
            self.output_count += count;
        }
    }

    /// Record a validation error.
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    /// Whether generation succeeded (output exists).
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.output_exists && self.stage == GenerationStage::Complete
    }

    /// Whether generation is still in progress.
    #[must_use]
    pub fn is_in_progress(&self) -> bool {
        !matches!(
            self.stage,
            GenerationStage::Complete | GenerationStage::Failed
        )
    }
}

impl Default for CodeGeneration {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CodeGeneration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CodeGen({}, {} tokens)", self.stage, self.output_count)
    }
}

// ============================================================================
// 26. PrimitiveMining — T2-C: Σ (Sum) + ρ (Recursion) + κ (Comparison) + μ (Mapping)
// ============================================================================

/// Recursive extraction of irreducible concepts from complex systems.
///
/// Transfers from: chemistry (chromatography, mass spec), linguistics
/// (morpheme analysis), finance (factor analysis), ML (feature extraction),
/// FORGE (T1/T2/T3 decomposition).
/// Computing analog: parser, AST walker, primitive extractor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrimitiveMining {
    /// Total candidates under analysis
    pub candidates: u32,
    /// Confirmed primitives extracted
    pub confirmed: u32,
    /// Rejected (reducible) candidates
    pub rejected: u32,
    /// Current recursion depth
    pub depth: u32,
    /// Max depth before declaring irreducible
    pub max_depth: u32,
}

impl PrimitiveMining {
    /// Create a new mining context.
    pub fn new(max_depth: u32) -> Self {
        Self {
            candidates: 0,
            confirmed: 0,
            rejected: 0,
            depth: 0,
            max_depth,
        }
    }

    /// Submit a candidate for primitive testing.
    pub fn submit_candidate(&mut self) {
        self.candidates += 1;
    }

    /// Confirm a candidate as a true primitive.
    pub fn confirm_primitive(&mut self) {
        self.confirmed += 1;
    }

    /// Reject a candidate (it can be further decomposed).
    pub fn reject_reducible(&mut self) {
        self.rejected += 1;
    }

    /// Descend into a candidate for deeper analysis.
    #[must_use]
    pub fn descend(&mut self) -> Result<u32, DepthError> {
        if self.depth >= self.max_depth {
            return Err(DepthError::MaxDepthExceeded);
        }
        self.depth += 1;
        Ok(self.depth)
    }

    /// Ascend after analyzing a branch.
    pub fn ascend(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Primitive yield: confirmed / candidates.
    #[must_use]
    pub fn yield_ratio(&self) -> f64 {
        if self.candidates == 0 {
            return 0.0;
        }
        self.confirmed as f64 / self.candidates as f64
    }

    /// Rejection rate: rejected / candidates.
    #[must_use]
    pub fn rejection_rate(&self) -> f64 {
        if self.candidates == 0 {
            return 0.0;
        }
        self.rejected as f64 / self.candidates as f64
    }

    /// Unprocessed candidates remaining.
    #[must_use]
    pub fn pending(&self) -> u32 {
        self.candidates
            .saturating_sub(self.confirmed + self.rejected)
    }
}

impl fmt::Display for PrimitiveMining {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Mining({}/{} confirmed, d={}/{})",
            self.confirmed, self.candidates, self.depth, self.max_depth
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_signature_threshold() {
        let sig = ThreatSignature::new("malware-hash", "endpoint", 0.95);
        assert!(sig.exceeds_threshold(0.9));
        assert!(!sig.exceeds_threshold(0.99));
    }

    #[test]
    fn test_threat_signature_clamps_confidence() {
        let sig = ThreatSignature::new("test", "test", 1.5);
        assert!((sig.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_resource_ratio_basic() {
        let ratio = ResourceRatio::new(75.0, 100.0);
        assert!((ratio.ratio() - 0.75).abs() < 0.001);
        assert!(ratio.is_sufficient(0.5));
        assert!(!ratio.is_exhausted());
    }

    #[test]
    fn test_resource_ratio_exhausted() {
        let ratio = ResourceRatio::new(5.0, 100.0);
        assert!((ratio.ratio() - 0.05).abs() < 0.001);
        assert!(ratio.is_exhausted());
    }

    #[test]
    fn test_resource_ratio_display() {
        let ratio = ResourceRatio::new(75.0, 100.0);
        let display = format!("{}", ratio);
        assert!(display.contains("75.0%"));
    }

    #[test]
    fn test_pattern_matcher_f1() {
        let matcher = PatternMatcher::new("test", ".*error.*").with_performance(0.9, 0.95);
        assert!(matcher.f1_score() > 0.85);
    }

    #[test]
    fn test_explore_exploit_decay() {
        let mut ee = ExploreExploit::new(0.8);
        assert!(ee.should_explore());
        for _ in 0..10 {
            ee.decay(0.1);
        }
        assert!(!ee.should_explore());
        assert_eq!(ee.iterations, 10);
    }

    #[test]
    fn test_explore_exploit_observe() {
        let mut ee = ExploreExploit::new(0.5);
        ee.observe(10.0);
        ee.observe(5.0);
        assert!((ee.best_known - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_event_classifier_3class() {
        let classifier = EventClassifier::new("severity", vec![3.0, 7.0]);
        assert_eq!(classifier.classify(1.0), 0); // low
        assert_eq!(classifier.classify(5.0), 1); // medium
        assert_eq!(classifier.classify(9.0), 2); // high
        assert_eq!(classifier.num_classes, 3);
    }

    #[test]
    fn test_event_classifier_boundary() {
        let classifier = EventClassifier::new("binary", vec![0.5]);
        assert_eq!(classifier.classify(0.3), 0);
        assert_eq!(classifier.classify(0.5), 1);
        assert_eq!(classifier.classify(0.7), 1);
    }

    #[test]
    fn test_feedback_loop_convergence() {
        let mut fb = FeedbackLoop::new(100.0, 0.5);
        for _ in 0..20 {
            fb.tick();
        }
        assert!(fb.converged(1.0));
    }

    #[test]
    fn test_feedback_loop_error() {
        let fb = FeedbackLoop::new(10.0, 1.0);
        assert!((fb.error() - 10.0).abs() < f64::EPSILON);
        assert!((fb.correction() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_schema_contract_compatibility() {
        let v1 = SchemaContract::new("events", 1, 3);
        let v2 = SchemaContract::new("events", 2, 4);
        assert!(v1.is_compatible_with(&v2));
        assert!(!v2.is_compatible_with(&v1)); // downgrade not allowed
    }

    #[test]
    fn test_schema_contract_breaking_change() {
        let v1 = SchemaContract {
            name: "events".into(),
            version: 1,
            required_fields: 5,
            backward_compatible: false,
        };
        let v2 = SchemaContract::new("events", 2, 5);
        assert!(!v1.is_compatible_with(&v2)); // not backward compatible
    }

    #[test]
    fn test_message_bus_fanout() {
        let mut bus = MessageBus::new("events");
        bus.add_subscriber();
        bus.add_subscriber();
        for _ in 0..10 {
            bus.dispatch();
        }
        assert!((bus.fan_out() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_specialized_worker_lifecycle() {
        let mut worker = SpecializedWorker::new("w1", "signal-detection");
        assert!(!worker.busy);
        worker.start_task();
        assert!(worker.busy);
        worker.complete_task();
        assert!(!worker.busy);
        assert_eq!(worker.throughput(), 1);
    }

    #[test]
    fn test_decay_function_halflife() {
        let decay = DecayFunction::new(100.0, 10.0);
        let at_half = decay.value_at(10.0);
        assert!((at_half - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_decay_function_threshold() {
        let decay = DecayFunction::new(100.0, 10.0);
        let t = decay.time_to_threshold(25.0);
        assert!((t - 20.0).abs() < 0.01); // 2 half-lives
    }

    #[test]
    fn test_decay_function_expired() {
        let decay = DecayFunction::new(100.0, 10.0);
        assert!(!decay.is_expired(5.0, 50.0));
        assert!(decay.is_expired(15.0, 50.0));
    }

    #[test]
    fn test_decay_function_edge_cases() {
        let decay = DecayFunction::new(100.0, 10.0);
        assert!((decay.time_to_threshold(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((decay.time_to_threshold(200.0) - 0.0).abs() < f64::EPSILON);
    }

    // Homeostasis tests

    #[test]
    fn test_homeostasis_in_tolerance() {
        let h = Homeostasis::new(100.0, 5.0, 0.5);
        assert!(h.in_tolerance()); // starts at setpoint
        assert!((h.error()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_homeostasis_correction() {
        let mut h = Homeostasis::new(100.0, 5.0, 0.5);
        h.current = 80.0; // outside tolerance
        assert!(!h.in_tolerance());
        assert!((h.error() - 20.0).abs() < f64::EPSILON);
        assert!((h.correction() - 10.0).abs() < f64::EPSILON); // 20 * 0.5
    }

    #[test]
    fn test_homeostasis_no_correction_in_tolerance() {
        let mut h = Homeostasis::new(100.0, 5.0, 0.5);
        h.current = 97.0; // within tolerance
        assert!(h.in_tolerance());
        assert!((h.correction()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_homeostasis_tick_convergence() {
        let mut h = Homeostasis::new(100.0, 2.0, 0.3);
        h.current = 50.0;
        for _ in 0..50 {
            h.tick();
        }
        assert!(h.in_tolerance());
    }

    // StagedValidation tests

    #[test]
    fn test_staged_validation_progress() {
        let sv = StagedValidation::new(4, 10.0);
        assert!((sv.progress() - 0.0).abs() < f64::EPSILON);
        assert!(!sv.is_complete());
    }

    #[test]
    fn test_staged_validation_advance() {
        let mut sv = StagedValidation::new(3, 10.0);
        sv.evidence_accumulated = 10.0;
        assert!(sv.can_advance());
        let result = sv.advance();
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(1));
        assert!((sv.progress() - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_staged_validation_insufficient_evidence() {
        let mut sv = StagedValidation::new(3, 10.0);
        sv.evidence_accumulated = 5.0;
        assert!(!sv.can_advance());
        let result = sv.advance();
        assert_eq!(result, Err(StagedValidationError::InsufficientEvidence));
    }

    #[test]
    fn test_staged_validation_complete() {
        let mut sv = StagedValidation::new(2, 5.0);
        for _ in 0..2 {
            sv.evidence_accumulated = 5.0;
            #[allow(
                unused_results,
                reason = "best-effort advance in loop; error means already complete"
            )]
            let _ = sv.advance();
        }
        assert!(sv.is_complete());
        assert!((sv.progress() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_staged_validation_already_complete_error() {
        let mut sv = StagedValidation::new(1, 5.0);
        sv.evidence_accumulated = 5.0;
        #[allow(
            unused_results,
            reason = "first advance expected to succeed; result not needed"
        )]
        let _ = sv.advance();
        assert!(sv.is_complete());
        sv.evidence_accumulated = 5.0;
        let result = sv.advance();
        assert_eq!(result, Err(StagedValidationError::AlreadyComplete));
    }

    // Atomicity tests

    #[test]
    fn test_atomicity_lifecycle() {
        let mut a = Atomicity::new("deploy-v2");
        assert!(!a.is_committed());
        assert!(a.rollback().is_ok());
        a.commit();
        assert!(a.is_committed());
    }

    #[test]
    fn test_atomicity_no_rollback_after_commit() {
        let mut a = Atomicity::new("migrate-db");
        a.commit();
        assert_eq!(a.rollback(), Err(AtomicityError::AlreadyCommitted));
    }

    // CompareAndSwap tests

    #[test]
    fn test_cas_success() {
        let mut cas = CompareAndSwap::new(10.0, 20.0);
        cas.execute(10.0); // current matches expected
        assert!(cas.succeeded());
        assert!((cas.witnessed() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cas_failure() {
        let mut cas = CompareAndSwap::new(10.0, 20.0);
        cas.execute(15.0); // current != expected
        assert!(!cas.succeeded());
        assert!((cas.witnessed() - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cas_retry_pattern() {
        let mut cas = CompareAndSwap::new(10.0, 20.0);
        cas.execute(15.0); // fails
        assert!(!cas.succeeded());
        // Retry with witnessed value
        let mut cas2 = CompareAndSwap::new(cas.witnessed(), 20.0);
        cas2.execute(15.0); // now matches
        assert!(cas2.succeeded());
    }

    // ToctouWindow tests

    #[test]
    fn test_toctou_fresh() {
        let mut w = ToctouWindow::new("file-permission", 100);
        w.check(1000);
        w.use_at(1050);
        assert_eq!(w.gap(), 50);
        assert!(!w.is_stale());
    }

    #[test]
    fn test_toctou_stale() {
        let mut w = ToctouWindow::new("cache-entry", 10);
        w.check(100);
        w.use_at(200);
        assert_eq!(w.gap(), 100);
        assert!(w.is_stale());
    }

    #[test]
    fn test_toctou_boundary() {
        let mut w = ToctouWindow::new("lease", 50);
        w.check(0);
        w.use_at(50);
        assert_eq!(w.gap(), 50);
        assert!(!w.is_stale()); // exactly at boundary = not stale
    }

    // SerializationGuard tests

    #[test]
    fn test_serialization_in_order() {
        let mut g = SerializationGuard::new("wal-log");
        assert!(g.commit(1).is_ok());
        assert!(g.commit(2).is_ok());
        assert!(g.commit(3).is_ok());
        assert_eq!(g.last_committed, 3);
    }

    #[test]
    fn test_serialization_out_of_order() {
        let mut g = SerializationGuard::new("wal-log");
        assert!(g.commit(1).is_ok());
        assert_eq!(g.commit(3), Err(SerializationGuardError::OutOfOrder)); // skipped 2
        assert_eq!(g.last_committed, 1);
    }

    #[test]
    fn test_serialization_queries() {
        let mut g = SerializationGuard::new("commits");
        #[allow(
            unused_results,
            reason = "advancing guard state; success is an invariant here"
        )]
        let _ = g.commit(1);
        #[allow(
            unused_results,
            reason = "advancing guard state; success is an invariant here"
        )]
        let _ = g.commit(2);
        assert!(g.is_committed(1));
        assert!(g.is_committed(2));
        assert!(!g.is_committed(3));
        assert!(!g.is_future(3)); // 3 == next_expected, not future
        assert!(g.is_future(4));
    }

    // RateLimiter tests

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let mut rl = RateLimiter::new(3, 100);
        assert!(rl.try_acquire(0).is_ok());
        assert!(rl.try_acquire(10).is_ok());
        assert!(rl.try_acquire(20).is_ok());
        assert_eq!(rl.try_acquire(30), Err(RateLimitError::Exceeded)); // 4th in same window
    }

    #[test]
    fn test_rate_limiter_window_reset() {
        let mut rl = RateLimiter::new(2, 100);
        assert!(rl.try_acquire(0).is_ok());
        assert!(rl.try_acquire(50).is_ok());
        assert!(rl.try_acquire(80).is_err()); // exhausted
        assert!(rl.try_acquire(100).is_ok()); // new window
    }

    #[test]
    fn test_rate_limiter_utilization() {
        let mut rl = RateLimiter::new(4, 100);
        #[allow(
            unused_results,
            reason = "consuming slots to test utilization; success is the invariant"
        )]
        let _ = rl.try_acquire(0);
        #[allow(
            unused_results,
            reason = "consuming slots to test utilization; success is the invariant"
        )]
        let _ = rl.try_acquire(1);
        assert!((rl.utilization() - 0.5).abs() < f64::EPSILON);
        assert_eq!(rl.remaining(), 2);
    }

    // CircuitBreaker tests

    #[test]
    fn test_circuit_breaker_trips_on_failures() {
        let mut cb = CircuitBreaker::new(3, 1);
        assert!(cb.is_allowing());
        cb.record_failure();
        cb.record_failure();
        assert!(cb.is_allowing()); // still closed at 2
        cb.record_failure();
        assert!(!cb.is_allowing()); // tripped at 3
        assert_eq!(cb.state, BreakerState::Open);
    }

    #[test]
    fn test_circuit_breaker_recovery() {
        let mut cb = CircuitBreaker::new(2, 2);
        cb.record_failure();
        cb.record_failure(); // trips
        assert!(!cb.is_allowing());
        cb.attempt_reset(); // → HalfOpen
        assert_eq!(cb.state, BreakerState::HalfOpen);
        assert!(cb.is_allowing());
        cb.record_success();
        cb.record_success(); // recovery complete
        assert_eq!(cb.state, BreakerState::Closed);
    }

    #[test]
    fn test_circuit_breaker_halfopen_failure() {
        let mut cb = CircuitBreaker::new(1, 3);
        cb.record_failure(); // trips
        cb.attempt_reset(); // → HalfOpen
        cb.record_failure(); // back to Open
        assert_eq!(cb.state, BreakerState::Open);
    }

    #[test]
    fn test_circuit_breaker_success_resets_count() {
        let mut cb = CircuitBreaker::new(3, 1);
        cb.record_failure();
        cb.record_failure();
        cb.record_success(); // resets failure count
        cb.record_failure();
        assert!(cb.is_allowing()); // still closed (only 1 failure since reset)
    }

    // Idempotency tests

    #[test]
    fn test_idempotency_first_apply() {
        let mut idem = Idempotency::new("order-12345");
        assert!(idem.apply()); // first: true
        assert!(!idem.apply()); // second: false
        assert!(!idem.apply()); // third: false
        assert_eq!(idem.attempts(), 3);
    }

    #[test]
    fn test_idempotency_state() {
        let mut idem = Idempotency::new("tx-abc");
        assert!(!idem.is_applied());
        idem.apply();
        assert!(idem.is_applied());
    }

    // NegativeEvidence tests

    #[test]
    fn test_negative_evidence_significance() {
        let ne = NegativeEvidence::new("heartbeat", 60.0, 3.0);
        assert!(ne.is_significant()); // 0 < 3 → significant absence
    }

    #[test]
    fn test_negative_evidence_observe_reduces_signal() {
        let mut ne = NegativeEvidence::new("report", 30.0, 2.0);
        assert!(ne.is_significant());
        ne.observe();
        ne.observe();
        assert!(!ne.is_significant()); // 2 >= 2 → no longer significant
    }

    #[test]
    fn test_negative_evidence_ratio() {
        let mut ne = NegativeEvidence::new("ack", 10.0, 4.0);
        assert!((ne.evidence_ratio() - 0.0).abs() < f64::EPSILON);
        ne.observe();
        ne.observe();
        assert!((ne.evidence_ratio() - 0.5).abs() < f64::EPSILON);
    }

    // TopologicalAddress tests

    #[test]
    fn test_topological_address_parse_render() {
        let addr = TopologicalAddress::parse("crates::vigilance::pv", "::");
        assert_eq!(addr.depth(), 3);
        assert_eq!(addr.render(), "crates::vigilance::pv");
        assert_eq!(addr.leaf(), Some("pv"));
    }

    #[test]
    fn test_topological_address_ancestry() {
        let parent = TopologicalAddress::parse("/home/user", "/");
        let child = TopologicalAddress::parse("/home/user/docs", "/");
        assert!(parent.is_ancestor_of(&child));
        assert!(!child.is_ancestor_of(&parent));
        assert!(!parent.is_ancestor_of(&parent)); // not ancestor of self
    }

    // Accumulator tests

    #[test]
    fn test_accumulator_running_total() {
        let mut acc = Accumulator::new();
        assert!(acc.is_empty());
        assert_eq!(acc.add(10.0), 10.0);
        assert_eq!(acc.add(5.0), 15.0);
        assert!(!acc.is_empty());
        assert!((acc.average() - 7.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_accumulator_rejects_negative() {
        let mut acc = Accumulator::new();
        acc.add(-5.0); // clamped to 0.0
        assert!((acc.total - 0.0).abs() < f64::EPSILON);
        assert_eq!(acc.additions, 1); // still counted as an addition
    }

    #[test]
    fn test_accumulator_default() {
        let acc = Accumulator::default();
        assert!(acc.is_empty());
        assert!((acc.average() - 0.0).abs() < f64::EPSILON);
    }

    // Checkpoint tests

    #[test]
    fn test_checkpoint_durability() {
        let mut cp = Checkpoint::new("epoch-42", 42);
        assert!(!cp.is_recoverable());
        cp.confirm();
        assert!(cp.is_recoverable());
    }

    #[test]
    fn test_checkpoint_ordering() {
        let cp1 = Checkpoint::new("first", 1);
        let cp2 = Checkpoint::new("second", 2);
        assert!(cp2.is_newer_than(&cp1));
        assert!(!cp1.is_newer_than(&cp2));
    }

    // Decomposition tests

    #[test]
    fn test_decomposition_depth_limit() {
        let mut d = Decomposition::new(3);
        assert!(d.is_root());
        assert!(d.descend().is_ok());
        assert!(d.descend().is_ok());
        assert!(d.descend().is_ok());
        assert_eq!(d.descend(), Err(DepthError::MaxDepthExceeded)); // max depth exceeded
    }

    #[test]
    fn test_decomposition_extraction() {
        let mut d = Decomposition::new(5);
        #[allow(
            unused_results,
            reason = "descend side-effect is the depth increment; error impossible at depth 0"
        )]
        let _ = d.descend();
        d.extract_part();
        d.extract_part();
        #[allow(
            unused_results,
            reason = "descend side-effect is the depth increment; error impossible at depth 1"
        )]
        let _ = d.descend();
        d.extract_part();
        assert_eq!(d.parts_extracted, 3);
        assert!((d.yield_ratio() - 1.5).abs() < f64::EPSILON); // 3 parts / depth 2
    }

    #[test]
    fn test_decomposition_ascend() {
        let mut d = Decomposition::new(5);
        #[allow(
            unused_results,
            reason = "descend side-effect is the depth increment; not testing error path here"
        )]
        let _ = d.descend();
        #[allow(
            unused_results,
            reason = "descend side-effect is the depth increment; not testing error path here"
        )]
        let _ = d.descend();
        assert_eq!(d.depth, 2);
        d.ascend();
        assert_eq!(d.depth, 1);
        d.ascend();
        assert!(d.is_root());
    }

    // CodeGeneration tests

    #[test]
    fn test_code_generation_happy_path() {
        let mut codegen = CodeGeneration::new();
        assert!(codegen.is_in_progress());
        assert_eq!(codegen.stage, GenerationStage::Specification);

        codegen.advance(); // -> Intermediate
        assert_eq!(codegen.stage, GenerationStage::Intermediate);

        codegen.advance(); // -> Emitting
        codegen.emit(100);
        assert_eq!(codegen.output_count, 100);

        codegen.advance(); // -> Validating
        codegen.advance(); // -> Complete (no errors)

        assert!(codegen.succeeded());
        assert!(!codegen.is_in_progress());
    }

    #[test]
    fn test_code_generation_with_errors() {
        let mut codegen = CodeGeneration::new();
        codegen.advance(); // Intermediate
        codegen.advance(); // Emitting
        codegen.emit(50);
        codegen.advance(); // Validating
        codegen.record_error();
        codegen.advance(); // -> Failed

        assert!(!codegen.succeeded());
        assert_eq!(codegen.stage, GenerationStage::Failed);
    }

    #[test]
    fn test_code_generation_default() {
        let codegen = CodeGeneration::default();
        assert_eq!(codegen.stage, GenerationStage::Specification);
        assert_eq!(codegen.output_count, 0);
    }

    // PrimitiveMining tests

    #[test]
    fn test_primitive_mining_workflow() {
        let mut pm = PrimitiveMining::new(5);
        pm.submit_candidate();
        pm.submit_candidate();
        pm.submit_candidate();

        pm.confirm_primitive();
        pm.reject_reducible();

        assert_eq!(pm.pending(), 1);
        assert!((pm.yield_ratio() - (1.0 / 3.0)).abs() < 0.01);
        assert!((pm.rejection_rate() - (1.0 / 3.0)).abs() < 0.01);
    }

    #[test]
    fn test_primitive_mining_depth() {
        let mut pm = PrimitiveMining::new(3);
        assert!(pm.descend().is_ok());
        assert!(pm.descend().is_ok());
        assert!(pm.descend().is_ok());
        assert_eq!(pm.descend(), Err(DepthError::MaxDepthExceeded));

        pm.ascend();
        assert_eq!(pm.depth, 2);
    }

    #[test]
    fn test_primitive_mining_empty() {
        let pm = PrimitiveMining::new(10);
        assert!((pm.yield_ratio() - 0.0).abs() < f64::EPSILON);
        assert_eq!(pm.pending(), 0);
    }
}
