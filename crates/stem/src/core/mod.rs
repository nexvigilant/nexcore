//! # stem-core: SCIENCE as Rust Traits
//!
//! Cross-domain T2-P primitives derived from the scientific method.
//!
//! ## The SCIENCE Composite (T2-C)
//!
//! ```text
//! S - SENSE      : Environment → Signal       (T1: MAPPING)
//! C - CLASSIFY   : Signal → Category          (T1: MAPPING)
//! I - INFER      : Pattern × Data → Prediction (T1: RECURSION)
//! E - EXPERIMENT : Action → Outcome           (T1: SEQUENCE)
//! N - NORMALIZE  : Prior × Evidence → Posterior (T1: STATE)
//! C - CODIFY     : Belief → Representation    (T1: MAPPING)
//! E - EXTEND     : Representation → NewDomain (T1: MAPPING)
//! ```
//!
//! ## Tier Classification
//!
//! - **T1**: Universal primitives (Sequence, Mapping, State, Recursion, Void, Boundary, Frequency, Existence, Persistence, Causality, Comparison, Quantity, Location, Irreversibility)
//! - **T2-P**: Cross-domain primitives (these traits)
//! - **T2-C**: Cross-domain composites (Science trait)
//! - **T3**: Domain-specific implementations
//!
//! ## The Three Unfixable Limits
//!
//! 1. **Heisenberg**: `Sense` changes the observed (acknowledged in trait docs)
//! 2. **Gödel**: `Science` implementing itself creates incompleteness
//! 3. **Shannon**: `Codify` has irreducible information loss

pub mod machine;

use serde::{Deserialize, Serialize};

/// T2-P: Convert environment state to signal.
///
/// Grounded in T1 MAPPING: world → representation.
///
/// # Heisenberg Acknowledgment
///
/// The act of sensing may alter the environment. This trait makes no
/// guarantee of zero-disturbance observation. Implementors should
/// document their observation effects.
///
/// # Examples
///
/// ```ignore
/// impl Sense for Thermometer {
///     type Environment = Room;
///     type Signal = Temperature;
///
///     fn sense(&self, env: &Room) -> Temperature {
///         env.current_temperature()
///     }
/// }
/// ```
pub trait Sense {
    /// The environment being observed
    type Environment;
    /// The signal produced by observation
    type Signal;

    /// Convert environment state to signal.
    ///
    /// May alter environment (Heisenberg applies).
    fn sense(&self, env: &Self::Environment) -> Self::Signal;
}

/// T2-P: Partition signal into categories.
///
/// Grounded in T1 MAPPING: signal → category.
///
/// Categories must form a valid partition (exhaustive, mutually exclusive)
/// or explicitly handle the "unknown" case.
pub trait Classify {
    /// The input signal type
    type Signal;
    /// The output category type (often an enum)
    type Category;

    /// Assign signal to category.
    fn classify(&self, signal: &Self::Signal) -> Self::Category;
}

/// T2-P: Derive prediction from pattern and data.
///
/// Grounded in T1 RECURSION: pattern applied to data produces prediction.
///
/// Unlike deduction, inference operates under uncertainty.
/// Implementors should quantify prediction confidence.
pub trait Infer {
    /// The pattern or model used for inference
    type Pattern;
    /// The observed data
    type Data;
    /// The prediction produced
    type Prediction;

    /// Apply pattern to data, producing prediction.
    fn infer(&self, pattern: &Self::Pattern, data: &Self::Data) -> Self::Prediction;
}

/// T2-P: Apply action and observe outcome.
///
/// Grounded in T1 SEQUENCE: action → observation.
///
/// Experiments are interventions that test causal hypotheses.
/// The outcome should be reproducible given the same action
/// and initial state.
pub trait Experiment {
    /// The intervention applied
    type Action;
    /// The observed result
    type Outcome;

    /// Execute action, return observed outcome.
    ///
    /// Note: `&mut self` because experiments may change internal state.
    fn experiment(&mut self, action: Self::Action) -> Self::Outcome;
}

/// T2-P: Update belief given evidence (Bayesian core).
///
/// Grounded in T1 STATE: prior → posterior.
///
/// This is the learning primitive. Posterior becomes next iteration's prior.
pub trait Normalize {
    /// The prior belief state
    type Prior;
    /// New evidence
    type Evidence;
    /// Updated belief state
    type Posterior;

    /// Update prior belief with evidence to produce posterior.
    fn normalize(&self, prior: Self::Prior, evidence: &Self::Evidence) -> Self::Posterior;
}

/// T2-P: Compress belief to transferable representation.
///
/// Grounded in T1 MAPPING: tacit → explicit.
///
/// # Shannon Acknowledgment
///
/// Codification has irreducible information loss. The representation
/// cannot capture 100% of the original belief. Implementors should
/// document what information is preserved vs lost.
pub trait Codify {
    /// The internal belief state
    type Belief;
    /// The external representation
    type Representation;

    /// Compress belief to transferable form.
    ///
    /// Information loss is unavoidable (Shannon applies).
    fn codify(&self, belief: &Self::Belief) -> Self::Representation;
}

/// T2-P: Apply representation across domain boundary.
///
/// Grounded in T1 MAPPING: source_domain → target_domain.
///
/// This is the transfer primitive. It enables cross-domain application
/// of codified knowledge.
pub trait Extend {
    /// Representation from source domain
    type Source;
    /// Instantiation in target domain
    type Target;

    /// Apply source representation to produce target instantiation.
    fn extend(&self, source: &Self::Source) -> Self::Target;
}

/// T2-C: The complete scientific method as a composite trait.
///
/// Combines all seven T2-P primitives into a coherent methodology.
///
/// # Gödel Acknowledgment
///
/// A `Science` implementation studying itself will encounter
/// incompleteness. Self-referential science has fundamental limits.
///
/// # The Loop
///
/// ```text
/// SENSE → CLASSIFY → INFER → EXPERIMENT → NORMALIZE → CODIFY → EXTEND
///   ↑                                                              │
///   └──────────────────────────────────────────────────────────────┘
/// ```
pub trait Science: Sense + Classify + Infer + Experiment + Normalize + Codify + Extend {
    /// Execute one iteration of the scientific method.
    ///
    /// Returns the codified representation produced by this cycle.
    fn cycle(&mut self) -> <Self as Codify>::Representation
    where
        Self: Sized,
        <Self as Sense>::Signal: Clone,
        <Self as Classify>::Category: Clone,
        <Self as Infer>::Prediction: Into<<Self as Experiment>::Action>,
        <Self as Experiment>::Outcome: Into<<Self as Normalize>::Evidence>,
        <Self as Normalize>::Posterior: Into<<Self as Codify>::Belief>;
}

// ============================================================================
// Bedrock Types (re-exported from nexcore-constants)
// ============================================================================

pub use nexcore_constants::{Confidence, Correction, Measured, Tier};

// ============================================================================
// Integrity Gate Composite (ς + κ + ∂)
// ============================================================================

/// T2-C: A gate that ensures state integrity via comparison and boundary enforcement.
///
/// Grounded in:
/// - **ς (State)**: Holds the inner data.
/// - **κ (Comparison)**: Validates actual vs required constraints.
/// - **∂ (Boundary)**: Returns `Result`, preventing invalid data from entering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integrity<T> {
    /// The inner data that has crossed the validation boundary.
    value: T,
    /// When this data was last validated.
    validated_at: std::time::SystemTime,
}

impl<T> Integrity<T> {
    /// Create a new integrity gate for a value.
    ///
    /// # Errors
    /// Returns `IntegrityError` if the initial value fails validation.
    pub fn new<V: Validate<T>>(value: T, validator: &V) -> Result<Self, IntegrityError> {
        validator.validate(&value)?;
        Ok(Self {
            value,
            validated_at: std::time::SystemTime::now(),
        })
    }

    /// Access the inner data.
    ///
    /// Since the data is inside an `Integrity` gate, it is guaranteed to
    /// have passed its validation criteria at creation time.
    #[must_use]
    pub fn get_value(&self) -> &T {
        &self.value
    }

    /// Update the inner data by passing through the gate again.
    ///
    /// # Errors
    /// Returns `IntegrityError` if the new value fails validation.
    pub fn update<V: Validate<T>>(
        &mut self,
        new_value: T,
        validator: &V,
    ) -> Result<(), IntegrityError> {
        validator.validate(&new_value)?;
        self.value = new_value;
        self.validated_at = std::time::SystemTime::now();
        Ok(())
    }

    /// Map the inner value to a new type while preserving integrity.
    ///
    /// # Errors
    /// Returns `IntegrityError` if the transformed value fails the NEW validation.
    pub fn map_integrity<U, V: Validate<U>>(
        self,
        f: impl FnOnce(T) -> U,
        validator: &V,
    ) -> Result<Integrity<U>, IntegrityError> {
        let new_value = f(self.value);
        Integrity::new(new_value, validator)
    }
}

/// T2-P: Trait for comparison-based validation (κ).
pub trait Validate<T> {
    /// Validate that actual matches required constraints.
    ///
    /// # Errors
    /// Returns `IntegrityError` with details on failure.
    fn validate(&self, value: &T) -> Result<(), IntegrityError>;
}

/// Errors for integrity gate failures (∂).
#[derive(Debug, nexcore_error::Error)]
pub enum IntegrityError {
    /// Comparison failed: value out of bounds.
    #[error("boundary violation: {0}")]
    BoundaryViolation(String),
    /// Comparison failed: value malformed.
    #[error("structural violation: {0}")]
    StructuralViolation(String),
    /// Comparison failed: invariant broken.
    #[error("invariant violation: {0}")]
    InvariantViolation(String),
}

// ============================================================================
// Autonomous Loop Composite (f + → + ρ)
// ============================================================================

/// T2-C: A self-referential loop that manages action frequency and causality.
///
/// Grounded in:
/// - **f (Frequency)**: Limits the number of retries/rate.
/// - **→ (Causality)**: Triggers the next attempt based on result.
/// - **ρ (Recursion)**: Re-enters the action logic until boundary is met.
pub struct AutonomousLoop {
    /// Maximum number of attempts allowed (f).
    max_attempts: u32,
    /// Current attempt count (f).
    current_attempt: u32,
}

impl AutonomousLoop {
    /// Create a new autonomous loop with a retry limit.
    #[must_use]
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            current_attempt: 0,
        }
    }

    /// Execute an action with autonomous retry logic.
    ///
    /// # Errors
    /// Returns the last error if all attempts fail (∂).
    pub async fn run<T, E, F, Fut>(&mut self, mut action: F) -> Result<T, E>
    where
        F: FnMut(u32) -> Fut,
        Fut: std::future::Future<Output = Result<T, LoopOutcome<E>>>,
    {
        loop {
            self.current_attempt += 1;

            match action(self.current_attempt).await {
                Ok(value) => return Ok(value),
                Err(LoopOutcome::Terminal(e)) => return Err(e),
                Err(LoopOutcome::Retryable(e)) => {
                    if self.current_attempt >= self.max_attempts {
                        return Err(e);
                    }
                    tracing::warn!("Attempt {} failed, retrying...", self.current_attempt);
                }
            }
        }
    }

    /// Get current attempt count.
    #[must_use]
    pub fn get_current_attempt(&self) -> u32 {
        self.current_attempt
    }
}

/// T2-P: Classification of an action outcome for the autonomous loop (Σ).
pub enum LoopOutcome<E> {
    /// The error is terminal and the loop should exit (∂).
    Terminal(E),
    /// The error is transient and the loop may retry (ρ).
    Retryable(E),
}

// ============================================================================
// Context Injection Composite (π + ς + σ)
// ============================================================================

/// T2-C: A pattern that injects persisted context into a sequential execution.
///
/// Grounded in:
/// - **π (Persistence)**: The long-term storage of context.
/// - **ς (State)**: The active snapshot of context.
/// - **σ (Sequence)**: The ordered operations using the context.
pub struct ContextInjection<C> {
    /// The active context snapshot (ς).
    context: C,
}

impl<C: Clone> ContextInjection<C> {
    /// Create a new context injection from a persisted source (π).
    ///
    /// # Errors
    /// Returns error if context retrieval fails.
    pub async fn inject<S: ContextSource<C>>(source: &S) -> Result<Self, S::Error> {
        let context = source.fetch_context().await?;
        Ok(Self { context })
    }

    /// Execute a sequence of operations using the injected context (σ).
    pub async fn execute<T, E, F, Fut>(&self, mut action: F) -> Result<T, E>
    where
        F: FnMut(C) -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        action(self.context.clone()).await
    }

    /// Get the current context snapshot (ς).
    #[must_use]
    pub fn get_context(&self) -> &C {
        &self.context
    }
}

/// T2-P: Trait for fetching persisted context (π).
pub trait ContextSource<C> {
    /// Error type for retrieval failures.
    type Error;

    /// Fetch the context from persistent storage.
    fn fetch_context(&self) -> impl std::future::Future<Output = Result<C, Self::Error>> + Send;
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during scientific method execution
#[derive(Debug, nexcore_error::Error)]
pub enum ScienceError {
    /// Sensing failed to produce signal
    #[error("sensing failed: {0}")]
    SensingFailed(String),

    /// Classification produced unknown category
    #[error("classification failed: unknown category for signal")]
    ClassificationFailed,

    /// Inference could not produce prediction
    #[error("inference failed: {0}")]
    InferenceFailed(String),

    /// Experiment could not be executed
    #[error("experiment failed: {0}")]
    ExperimentFailed(String),

    /// Normalization failed (e.g., invalid evidence)
    #[error("normalization failed: {0}")]
    NormalizationFailed(String),

    /// Codification lost too much information
    #[error("codification failed: {0}")]
    CodificationFailed(String),

    /// Extension to target domain failed
    #[error("extension failed: {0}")]
    ExtensionFailed(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_clamps_to_range() {
        assert!((Confidence::new(1.5).value() - 1.0).abs() < f64::EPSILON);
        assert!((Confidence::new(-0.5).value() - 0.0).abs() < f64::EPSILON);
        assert!((Confidence::new(0.7).value() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_combines_multiplicatively() {
        let a = Confidence::new(0.8);
        let b = Confidence::new(0.9);
        let combined = a.combine(b);
        assert!((combined.value() - 0.72).abs() < f64::EPSILON);
    }

    #[test]
    fn tier_transfer_multipliers() {
        assert!((Tier::T1Universal.transfer_multiplier() - 1.0).abs() < f64::EPSILON);
        assert!((Tier::T2Primitive.transfer_multiplier() - 0.9).abs() < f64::EPSILON);
        assert!((Tier::T2Composite.transfer_multiplier() - 0.7).abs() < f64::EPSILON);
        assert!((Tier::T3DomainSpecific.transfer_multiplier() - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn measured_map_preserves_confidence() {
        let m = Measured::new(5, Confidence::new(0.8));
        let doubled = m.map(|x| x * 2);
        assert_eq!(doubled.value, 10);
        assert!((doubled.confidence.value() - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_cmp_total_provides_ordering() {
        let a = Confidence::new(0.3);
        let b = Confidence::new(0.7);
        assert_eq!(a.cmp_total(b), std::cmp::Ordering::Less);
        assert_eq!(b.cmp_total(a), std::cmp::Ordering::Greater);
        assert_eq!(a.cmp_total(a), std::cmp::Ordering::Equal);
    }

    #[test]
    fn correction_records_change() {
        let correction = Correction::now(10, 20, "Fixed off-by-one");
        assert_eq!(*correction.original(), 10);
        assert_eq!(*correction.corrected(), 20);
        assert_eq!(correction.reason, "Fixed off-by-one");
        assert_eq!(correction.apply(), 20);
    }

    // ========== Integrity Gate Tests ==========

    struct PositiveValidator;
    impl Validate<i32> for PositiveValidator {
        fn validate(&self, value: &i32) -> Result<(), IntegrityError> {
            if *value > 0 {
                Ok(())
            } else {
                Err(IntegrityError::BoundaryViolation(
                    "value must be positive".to_string(),
                ))
            }
        }
    }

    #[test]
    fn test_integrity_gate_allows_valid() {
        let gate = Integrity::new(10, &PositiveValidator).unwrap(); // INVARIANT: valid input
        assert_eq!(*gate.get_value(), 10);
    }

    #[test]
    fn test_integrity_gate_blocks_invalid() {
        let result = Integrity::new(-5, &PositiveValidator);
        assert!(result.is_err());
    }

    #[test]
    fn test_integrity_update() {
        let mut gate = Integrity::new(10, &PositiveValidator).unwrap(); // INVARIANT: valid input
        assert!(gate.update(20, &PositiveValidator).is_ok());
        assert_eq!(*gate.get_value(), 20);
        assert!(gate.update(-1, &PositiveValidator).is_err());
        assert_eq!(*gate.get_value(), 20); // Value preserved on failure
    }

    // ========== Autonomous Loop Tests ==========

    #[tokio::test]
    async fn test_autonomous_loop_success() {
        let mut loop_ctrl = AutonomousLoop::new(3);
        let result: Result<i32, String> = loop_ctrl.run(|_| async { Ok(42) }).await;

        assert_eq!(result.unwrap(), 42); // INVARIANT: returns Ok
        assert_eq!(loop_ctrl.get_current_attempt(), 1);
    }

    #[tokio::test]
    async fn test_autonomous_loop_retries_then_succeeds() {
        let mut loop_ctrl = AutonomousLoop::new(3);
        let result: Result<i32, String> = loop_ctrl
            .run(|attempt| async move {
                if attempt < 2 {
                    Err(LoopOutcome::Retryable("transient".to_string()))
                } else {
                    Ok(42)
                }
            })
            .await;

        assert_eq!(result.unwrap(), 42); // INVARIANT: succeeds on 2nd attempt
        assert_eq!(loop_ctrl.get_current_attempt(), 2);
    }

    #[tokio::test]
    async fn test_autonomous_loop_fails_at_limit() {
        let mut loop_ctrl = AutonomousLoop::new(2);
        let result: Result<i32, String> = loop_ctrl
            .run(|_| async { Err(LoopOutcome::Retryable("fail".to_string())) })
            .await;

        assert_eq!(result.unwrap_err(), "fail"); // INVARIANT: returns Err
        assert_eq!(loop_ctrl.get_current_attempt(), 2);
    }

    #[tokio::test]
    async fn test_autonomous_loop_terminal_exit() {
        let mut loop_ctrl = AutonomousLoop::new(5);
        let result: Result<i32, String> = loop_ctrl
            .run(|_| async { Err(LoopOutcome::Terminal("fatal".to_string())) })
            .await;

        assert_eq!(result.unwrap_err(), "fatal"); // INVARIANT: returns Err
        assert_eq!(loop_ctrl.get_current_attempt(), 1);
    }

    // ========== Context Injection Tests ==========

    #[derive(Clone, PartialEq, Debug)]
    struct MockContext {
        user_id: String,
        permissions: Vec<String>,
    }

    struct MockSource;
    impl ContextSource<MockContext> for MockSource {
        type Error = String;
        async fn fetch_context(&self) -> Result<MockContext, Self::Error> {
            Ok(MockContext {
                user_id: "user-123".to_string(),
                permissions: vec!["read".to_string()],
            })
        }
    }

    #[tokio::test]
    async fn test_context_injection_flow() {
        let source = MockSource;
        let injector = ContextInjection::inject(&source).await.unwrap(); // INVARIANT: valid source

        assert_eq!(injector.get_context().user_id, "user-123");

        let result: Result<bool, String> = injector
            .execute(|ctx| async move { Ok(ctx.permissions.contains(&"read".to_string())) })
            .await;

        assert!(result.unwrap()); // INVARIANT: contains read permission
    }
}
