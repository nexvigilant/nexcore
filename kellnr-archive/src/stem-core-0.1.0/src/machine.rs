//! # Machine Primitive (T3)
//!
//! A system that transforms inputs into outputs through a deterministic
//! sequence of operations governed by a mechanism.
//!
//! ## Decomposition
//!
//! ```text
//! Machine (T3)
//! ├─ transformation (T1: μ)  — stem_chem::Transform
//! ├─ sequence (T1: σ)        — ordered steps
//! ├─ state (T1: ς)           — instantaneous configuration
//! ├─ component (T2-P)        — functional part→whole  [NEW]
//! ├─ operation (T2-P)        — discrete action→effect [NEW]
//! ├─ mechanism (T2-C)        — sequence(cause→effect) [NEW]
//! ├─ determinism (T2-P)      — repeatability score    [NEW]
//! ├─ input (T2-C)            — state(before)
//! └─ output (T2-C)           — state(after)
//! ```
//!
//! ## Grounding Path
//!
//! `Machine → {T2-C} → {T2-P} → {T1: σ, μ, ς, →, κ}`
//!
//! ## Heisenberg Acknowledgment
//!
//! Observing a Machine's intermediate state may alter its execution
//! (e.g., debug probes affect timing). This module makes no guarantee
//! of zero-disturbance observation of in-flight state.

use serde::{Deserialize, Serialize};

use crate::{Confidence, Measured, Tier};

// ============================================================================
// Component (T2-P): Functional part→whole
// ============================================================================

/// T2-P: A functional part that contributes to a larger whole.
///
/// Grounded in T1 mereology (part/whole) with added functional semantics.
/// Cross-domain: engineering parts, biological organs, software modules.
pub trait Component {
    /// What this component produces when it operates.
    type Output;

    /// Unique identifier within the containing whole.
    fn id(&self) -> ComponentId;

    /// Execute this component's function, producing output.
    ///
    /// # Errors
    ///
    /// Returns `MachineError::ComponentFailure` if the component cannot operate.
    fn operate(&self) -> Result<Self::Output, MachineError>;
}

/// T2-P: Unique identifier for a component within a machine.
///
/// Wrapped per Codex IV (no naked primitives for domain values).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(u64);

impl ComponentId {
    /// Create a new component identifier.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw identifier value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl From<ComponentId> for u64 {
    fn from(id: ComponentId) -> Self {
        id.0
    }
}

// ============================================================================
// Operation (T2-P): Discrete action→effect
// ============================================================================

/// T2-P: A single discrete action that produces an effect.
///
/// Grounded in T1 Causality (→): action triggers effect.
/// Cross-domain: math operators, CPU instructions, enzymatic steps.
pub trait Operation {
    /// The input consumed by this operation.
    type Input;
    /// The output produced by this operation.
    type Output;

    /// Execute the operation: input → output.
    ///
    /// # Errors
    ///
    /// Returns `MachineError::OperationFailed` on failure.
    fn execute(&self, input: Self::Input) -> Result<Self::Output, MachineError>;
}

// ============================================================================
// Determinism (T2-P): Repeatability score
// ============================================================================

/// T2-P: Quantified repeatability of a transformation.
///
/// Grounded in T1 Comparison (κ): same input → same output.
///
/// `score = 1.0` means perfectly deterministic (pure function).
/// `score = 0.0` means completely stochastic (random oracle).
///
/// Cross-domain: physics (laws), CS (pure functions), biology (enzyme fidelity).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Determinism {
    /// Repeatability in [0.0, 1.0]. Codex IV: wrapped, not naked f64.
    score: Confidence,
}

impl Determinism {
    /// Perfectly deterministic (score = 1.0).
    pub const DETERMINISTIC: Self = Self {
        score: Confidence::PERFECT,
    };

    /// Completely stochastic (score = 0.0).
    pub const STOCHASTIC: Self = Self {
        score: Confidence::NONE,
    };

    /// Create with explicit score, clamped to [0.0, 1.0].
    #[must_use]
    pub fn new(score: f64) -> Self {
        Self {
            score: Confidence::new(score),
        }
    }

    /// Get the repeatability score.
    #[must_use]
    pub fn score(&self) -> Confidence {
        self.score
    }

    /// Is this above the deterministic threshold (≥ 0.95)?
    #[must_use]
    pub fn is_deterministic(&self) -> bool {
        self.score.value() >= 0.95
    }

    /// Is this below the stochastic threshold (< 0.05)?
    #[must_use]
    pub fn is_stochastic(&self) -> bool {
        self.score.value() < 0.05
    }

    /// Tier classification for this primitive.
    #[must_use]
    pub const fn tier() -> Tier {
        Tier::T2Primitive
    }
}

impl From<Determinism> for f64 {
    /// Codex I: every quality becomes a quantity via `From`.
    fn from(d: Determinism) -> Self {
        d.score.value()
    }
}

// ============================================================================
// Mechanism (T2-C): Sequence(Cause → Effect)
// ============================================================================

/// T2-C: An ordered chain of operations that produces an aggregate effect.
///
/// Grounded in:
/// - **σ (Sequence)**: Steps have defined order.
/// - **→ (Causality)**: Each step's output causes the next step's input.
///
/// Cross-domain: biology (metabolic pathways), physics (force chains),
/// chemistry (reaction mechanisms), software (middleware pipelines).
pub struct Mechanism<I, O> {
    steps: Vec<Box<dyn Operation<Input = I, Output = O>>>,
    determinism: Determinism,
}

impl<I, O> Mechanism<I, O> {
    /// Create a mechanism from ordered steps.
    #[must_use]
    pub fn new(
        steps: Vec<Box<dyn Operation<Input = I, Output = O>>>,
        determinism: Determinism,
    ) -> Self {
        Self { steps, determinism }
    }

    /// Number of steps in the causal chain.
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Is the mechanism empty (no steps)?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get the mechanism's determinism rating.
    #[must_use]
    pub fn determinism(&self) -> Determinism {
        self.determinism
    }

    /// Tier classification.
    #[must_use]
    pub const fn tier() -> Tier {
        Tier::T2Composite
    }
}

// ============================================================================
// Input / Output wrappers (T2-C)
// ============================================================================

/// T2-C: A value entering a machine, with provenance tracking.
///
/// Grounded in T1 State (ς) + directionality.
/// Wraps raw state to distinguish "before transformation" from arbitrary state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input<T> {
    /// The data entering the machine.
    pub data: T,
    /// Monotonic sequence number for ordering.
    pub seq: u64,
}

/// T2-C: A value exiting a machine, with confidence.
///
/// Grounded in T1 State (ς) + directionality + measurement.
/// Every output carries the machine's confidence in the result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output<T> {
    /// The data produced by the machine.
    pub data: T,
    /// Sequence number linking to the input that produced this.
    pub source_seq: u64,
    /// Confidence in this output (Codex IX: all values carry uncertainty).
    pub confidence: Confidence,
}

// ============================================================================
// Machine (T3): The complete abstraction
// ============================================================================

/// T3: A system that transforms inputs to outputs through a mechanism.
///
/// Composes all sub-primitives:
/// - `Component` (T2-P) via the step list
/// - `Operation` (T2-P) via each step's execute
/// - `Mechanism` (T2-C) as the ordered causal chain
/// - `Determinism` (T2-P) as the repeatability guarantee
/// - `Input`/`Output` (T2-C) as directional state wrappers
///
/// ## Gödel Acknowledgment
///
/// A Machine that takes Machine descriptions as input and produces
/// Machine descriptions as output will encounter undecidable cases
/// (halting problem). This type does not attempt to detect such
/// self-reference; the caller must bound execution externally.
pub struct Machine<I, O> {
    /// The causal chain that performs transformation.
    mechanism: Mechanism<I, O>,
    /// Monotonic counter for input sequencing (σ).
    next_seq: u64,
}

impl<I, O> Machine<I, O> {
    /// Construct a machine from its mechanism.
    #[must_use]
    pub fn new(mechanism: Mechanism<I, O>) -> Self {
        Self {
            mechanism,
            next_seq: 0,
        }
    }

    /// Execute the first step of the mechanism on the input data.
    ///
    /// For single-step mechanisms this is a complete transformation.
    /// For multi-step homogeneous machines (`I == O`), prefer [`Machine::pipeline`].
    ///
    /// # Errors
    ///
    /// Returns `MachineError::EmptyMechanism` if no steps exist, or
    /// propagates the step's error on failure.
    pub fn process(&mut self, data: I) -> Result<Measured<Output<O>>, MachineError> {
        let seq = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);

        let step = self
            .mechanism
            .steps
            .first()
            .ok_or(MachineError::EmptyMechanism)?;
        let result = step.execute(data)?;
        let output = Output {
            data: result,
            source_seq: seq,
            confidence: self.mechanism.determinism.score(),
        };
        Ok(Measured::new(output, self.mechanism.determinism.score()))
    }

    /// Get the mechanism's determinism rating.
    #[must_use]
    pub fn determinism(&self) -> Determinism {
        self.mechanism.determinism()
    }

    /// Number of steps in the underlying mechanism.
    #[must_use]
    pub fn step_count(&self) -> usize {
        self.mechanism.len()
    }

    /// How many inputs have been processed.
    #[must_use]
    pub fn processed_count(&self) -> u64 {
        self.next_seq
    }

    /// Tier classification.
    #[must_use]
    pub const fn tier() -> Tier {
        Tier::T3DomainSpecific
    }
}

/// Homogeneous pipeline: when Input type == Output type, chain steps.
impl<T> Machine<T, T> {
    /// Feed input through ALL mechanism steps sequentially (pipeline mode).
    ///
    /// Each step's output becomes the next step's input.
    ///
    /// # Errors
    ///
    /// Returns `MachineError` if any step fails. Reports which step failed.
    pub fn pipeline(&mut self, data: T) -> Result<Measured<Output<T>>, MachineError> {
        let seq = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);

        if self.mechanism.is_empty() {
            return Err(MachineError::EmptyMechanism);
        }

        let mut current = data;
        for (i, step) in self.mechanism.steps.iter().enumerate() {
            current = step
                .execute(current)
                .map_err(|_| MachineError::StepFailed {
                    step_index: i,
                    step_count: self.mechanism.steps.len(),
                })?;
        }

        let output = Output {
            data: current,
            source_seq: seq,
            confidence: self.mechanism.determinism.score(),
        };
        Ok(Measured::new(output, self.mechanism.determinism.score()))
    }
}

// ============================================================================
// Transfer Confidence (T2-C)
// ============================================================================

/// T2-C: Cross-domain transfer confidence for the Machine concept.
///
/// Formula: `combined = structural × 0.4 + functional × 0.4 + contextual × 0.2`
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransferConfidence {
    /// How well the structure maps (0.0–1.0).
    pub structural: Confidence,
    /// How well the function maps (0.0–1.0).
    pub functional: Confidence,
    /// How well the context maps (0.0–1.0).
    pub contextual: Confidence,
}

impl TransferConfidence {
    /// Compute the combined transfer confidence.
    #[must_use]
    pub fn combined(&self) -> Confidence {
        let score = self.structural.value() * 0.4
            + self.functional.value() * 0.4
            + self.contextual.value() * 0.2;
        Confidence::new(score)
    }

    /// Identify the limiting dimension (lowest score).
    ///
    /// Tie-breaking: structural > functional > contextual (documented priority).
    #[must_use]
    pub fn limiting_factor(&self) -> &'static str {
        let s = self.structural.value();
        let f = self.functional.value();
        let c = self.contextual.value();
        let min = s.min(f).min(c);
        if (s - min).abs() < f64::EPSILON {
            "structural"
        } else if (f - min).abs() < f64::EPSILON {
            "functional"
        } else {
            "contextual"
        }
    }
}

// ============================================================================
// Errors (Codex XI: all state is correctable)
// ============================================================================

/// Errors during machine operation.
///
/// Each variant identifies the failure point for correction (Codex XI).
#[derive(Debug, thiserror::Error)]
pub enum MachineError {
    /// A component failed to operate.
    #[error("component {0} failed")]
    ComponentFailure(ComponentId),

    /// An operation within the mechanism failed.
    #[error("operation failed: {0}")]
    OperationFailed(String),

    /// A specific pipeline step failed.
    #[error("step {step_index}/{step_count} failed")]
    StepFailed {
        /// Zero-indexed step that failed.
        step_index: usize,
        /// Total steps in mechanism.
        step_count: usize,
    },

    /// Mechanism has no steps — cannot transform.
    #[error("mechanism has no steps")]
    EmptyMechanism,
}

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "component-{}", self.0)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Test Fixtures ==========

    struct Doubler;
    impl Operation for Doubler {
        type Input = i64;
        type Output = i64;
        fn execute(&self, input: i64) -> Result<i64, MachineError> {
            Ok(input * 2)
        }
    }

    struct Incrementer;
    impl Operation for Incrementer {
        type Input = i64;
        type Output = i64;
        fn execute(&self, input: i64) -> Result<i64, MachineError> {
            Ok(input + 1)
        }
    }

    struct Failer;
    impl Operation for Failer {
        type Input = i64;
        type Output = i64;
        fn execute(&self, _input: i64) -> Result<i64, MachineError> {
            Err(MachineError::OperationFailed("intentional".into()))
        }
    }

    // -- Component test fixture --
    struct Valve {
        id: ComponentId,
        open: bool,
    }

    impl Component for Valve {
        type Output = bool;

        fn id(&self) -> ComponentId {
            self.id
        }

        fn operate(&self) -> Result<bool, MachineError> {
            if self.open {
                Ok(true)
            } else {
                Err(MachineError::ComponentFailure(self.id))
            }
        }
    }

    // ========== Component Trait (Finding #2) ==========

    #[test]
    fn component_operates_success() {
        let valve = Valve {
            id: ComponentId::new(1),
            open: true,
        };
        assert_eq!(valve.id(), ComponentId::new(1));
        assert!(valve.operate().is_ok());
    }

    #[test]
    fn component_operates_failure() {
        let valve = Valve {
            id: ComponentId::new(2),
            open: false,
        };
        let err = valve.operate().unwrap_err();
        assert!(matches!(err, MachineError::ComponentFailure(id) if id == ComponentId::new(2)));
    }

    #[test]
    fn component_failure_error_display() {
        let err = MachineError::ComponentFailure(ComponentId::new(7));
        assert_eq!(format!("{err}"), "component component-7 failed");
    }

    // ========== Determinism ==========

    #[test]
    fn determinism_deterministic() {
        let d = Determinism::DETERMINISTIC;
        assert!(d.is_deterministic());
        assert!(!d.is_stochastic());
        assert!((f64::from(d) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn determinism_stochastic() {
        let d = Determinism::STOCHASTIC;
        assert!(!d.is_deterministic());
        assert!(d.is_stochastic());
    }

    #[test]
    fn determinism_clamps() {
        let d = Determinism::new(1.5);
        assert!((d.score().value() - 1.0).abs() < f64::EPSILON);
        let d2 = Determinism::new(-0.5);
        assert!(d2.score().value().abs() < f64::EPSILON);
    }

    #[test]
    fn determinism_tier() {
        assert_eq!(Determinism::tier(), Tier::T2Primitive);
    }

    // ========== ComponentId ==========

    #[test]
    fn component_id_roundtrip() {
        let id = ComponentId::new(42);
        assert_eq!(id.value(), 42);
        assert_eq!(u64::from(id), 42);
        assert_eq!(format!("{id}"), "component-42");
    }

    // ========== Input / Output (Finding #4) ==========

    #[test]
    fn input_output_construction() {
        let input = Input {
            data: 42_i64,
            seq: 0,
        };
        assert_eq!(input.data, 42);
        assert_eq!(input.seq, 0);

        let output = Output {
            data: 84_i64,
            source_seq: 0,
            confidence: Confidence::PERFECT,
        };
        assert_eq!(output.data, 84);
        assert_eq!(output.source_seq, 0);
        assert!((output.confidence.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn input_output_serde_roundtrip() {
        let output = Output {
            data: 99_i64,
            source_seq: 5,
            confidence: Confidence::new(0.75),
        };
        let json = serde_json::to_string(&output).expect("INVARIANT: serializable");
        let restored: Output<i64> = serde_json::from_str(&json).expect("INVARIANT: deserializable");
        assert_eq!(restored.data, 99);
        assert_eq!(restored.source_seq, 5);
        assert!((restored.confidence.value() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn output_with_zero_confidence() {
        let output = Output {
            data: 0_i64,
            source_seq: 0,
            confidence: Confidence::NONE,
        };
        assert!(output.confidence.value().abs() < f64::EPSILON);
    }

    // ========== Machine: Single-step ==========

    #[test]
    fn machine_single_step_process() {
        let steps: Vec<Box<dyn Operation<Input = i64, Output = i64>>> = vec![Box::new(Doubler)];
        let mechanism = Mechanism::new(steps, Determinism::DETERMINISTIC);
        let mut machine = Machine::new(mechanism);

        let result = machine.process(21);
        assert!(result.is_ok());
        let measured = result.ok().expect("INVARIANT: just checked is_ok");
        assert_eq!(measured.value.data, 42);
        assert_eq!(measured.value.source_seq, 0);
        assert!(measured.confidence.value() >= 0.95);
    }

    // ========== Machine: Pipeline ==========

    #[test]
    fn machine_pipeline_chains_steps() {
        let steps: Vec<Box<dyn Operation<Input = i64, Output = i64>>> =
            vec![Box::new(Doubler), Box::new(Incrementer)];
        let mechanism = Mechanism::new(steps, Determinism::new(0.99));
        let mut machine = Machine::new(mechanism);

        // 10 → double(10)=20 → increment(20)=21
        let result = machine.pipeline(10);
        assert!(result.is_ok());
        let measured = result.ok().expect("INVARIANT: just checked is_ok");
        assert_eq!(measured.value.data, 21);
    }

    // ========== Machine: Failure (Finding #3: no panic) ==========

    #[test]
    fn machine_pipeline_reports_failed_step() {
        let steps: Vec<Box<dyn Operation<Input = i64, Output = i64>>> =
            vec![Box::new(Doubler), Box::new(Failer), Box::new(Incrementer)];
        let mechanism = Mechanism::new(steps, Determinism::DETERMINISTIC);
        let mut machine = Machine::new(mechanism);

        let err = machine.pipeline(5).unwrap_err();
        assert!(matches!(
            err,
            MachineError::StepFailed {
                step_index: 1,
                step_count: 3
            }
        ));
    }

    // ========== Machine: Empty ==========

    #[test]
    fn machine_empty_mechanism_errors() {
        let steps: Vec<Box<dyn Operation<Input = i64, Output = i64>>> = vec![];
        let mechanism = Mechanism::new(steps, Determinism::DETERMINISTIC);
        let mut machine = Machine::new(mechanism);

        assert!(matches!(
            machine.process(1),
            Err(MachineError::EmptyMechanism)
        ));
        assert!(matches!(
            machine.pipeline(1),
            Err(MachineError::EmptyMechanism)
        ));
    }

    // ========== Machine: Sequence tracking ==========

    #[test]
    fn machine_tracks_sequence_numbers() {
        let steps: Vec<Box<dyn Operation<Input = i64, Output = i64>>> = vec![Box::new(Incrementer)];
        let mechanism = Mechanism::new(steps, Determinism::DETERMINISTIC);
        let mut machine = Machine::new(mechanism);

        let r0 = machine.process(0).ok().expect("INVARIANT: valid");
        let r1 = machine.process(0).ok().expect("INVARIANT: valid");
        assert_eq!(r0.value.source_seq, 0);
        assert_eq!(r1.value.source_seq, 1);
        assert_eq!(machine.processed_count(), 2);
    }

    // ========== Transfer Confidence (Finding #1: tie-breaking) ==========

    #[test]
    fn transfer_confidence_formula() {
        let tc = TransferConfidence {
            structural: Confidence::new(0.95),
            functional: Confidence::new(0.95),
            contextual: Confidence::new(0.90),
        };
        assert!((tc.combined().value() - 0.94).abs() < f64::EPSILON);
    }

    #[test]
    fn transfer_confidence_limiting_factor_contextual() {
        let tc = TransferConfidence {
            structural: Confidence::new(0.95),
            functional: Confidence::new(0.90),
            contextual: Confidence::new(0.60),
        };
        assert_eq!(tc.limiting_factor(), "contextual");
    }

    #[test]
    fn transfer_confidence_limiting_factor_with_tie() {
        // structural == functional, both higher than contextual
        let tc = TransferConfidence {
            structural: Confidence::new(0.8),
            functional: Confidence::new(0.8),
            contextual: Confidence::new(0.6),
        };
        assert_eq!(tc.limiting_factor(), "contextual");
    }

    #[test]
    fn transfer_confidence_all_equal() {
        // All equal → documented priority: structural wins
        let tc = TransferConfidence {
            structural: Confidence::new(0.5),
            functional: Confidence::new(0.5),
            contextual: Confidence::new(0.5),
        };
        assert_eq!(tc.limiting_factor(), "structural");
    }

    // ========== Tier classifications ==========

    #[test]
    fn tier_classifications_correct() {
        assert_eq!(Determinism::tier(), Tier::T2Primitive);
        assert_eq!(Mechanism::<(), ()>::tier(), Tier::T2Composite);
        assert_eq!(Machine::<(), ()>::tier(), Tier::T3DomainSpecific);
    }

    // ========== Mechanism properties ==========

    #[test]
    fn mechanism_len_and_empty() {
        let empty: Vec<Box<dyn Operation<Input = i64, Output = i64>>> = vec![];
        let m = Mechanism::new(empty, Determinism::DETERMINISTIC);
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);

        let steps: Vec<Box<dyn Operation<Input = i64, Output = i64>>> =
            vec![Box::new(Doubler), Box::new(Incrementer)];
        let m2 = Mechanism::new(steps, Determinism::new(0.9));
        assert!(!m2.is_empty());
        assert_eq!(m2.len(), 2);
        assert!((m2.determinism().score().value() - 0.9).abs() < f64::EPSILON);
    }
}
