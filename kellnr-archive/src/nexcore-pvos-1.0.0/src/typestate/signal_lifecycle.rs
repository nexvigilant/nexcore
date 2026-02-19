//! # TypesafeSignal — Compile-Time Signal Lifecycle
//!
//! Signal lifecycle with compile-time state enforcement:
//! `Detected → Validated → Confirmed | Refuted`
//!
//! Branching terminal states (Confirmed vs Refuted) are type-safe.
//!
//! ## Primitive Grounding
//!
//! | Symbol | Role      | Weight |
//! |--------|-----------|--------|
//! | ς      | State     | 0.80 (dominant) |
//! | ∂      | Boundary  | 0.10   |
//! | ∅      | Void      | 0.05   |
//! | →      | Causality | 0.05   |
//!
//! ## ToV Axiom Mapping
//!
//! - **A1**: 4 states form finite decomposition
//! - **A2**: Detected < Validated < {Confirmed, Refuted} (DAG with fork)
//! - **A4**: Confirmed/Refuted are terminal (boundary)

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use super::LifecycleState;
use crate::state::StateContext;
use nexcore_lex_primitiva::{GroundsTo, LexPrimitiva, PrimitiveComposition};

// ═══════════════════════════════════════════════════════════
// STATE MARKERS
// ═══════════════════════════════════════════════════════════

/// Signal has been statistically detected.
///
/// Tier: T1 (ς)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalDetected;

impl LifecycleState for SignalDetected {
    fn name() -> &'static str {
        "detected"
    }
    fn is_terminal() -> bool {
        false
    }
    fn is_initial() -> bool {
        true
    }
}

/// Signal has been clinically validated.
///
/// Tier: T1 (ς)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalValidated;

impl LifecycleState for SignalValidated {
    fn name() -> &'static str {
        "validated"
    }
    fn is_terminal() -> bool {
        false
    }
    fn is_initial() -> bool {
        false
    }
}

/// Signal has been confirmed as real (terminal).
///
/// Tier: T1 (ς)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalConfirmed;

impl LifecycleState for SignalConfirmed {
    fn name() -> &'static str {
        "confirmed"
    }
    fn is_terminal() -> bool {
        true
    }
    fn is_initial() -> bool {
        false
    }
}

/// Signal has been refuted as spurious (terminal).
///
/// Tier: T1 (ς)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalRefuted;

impl LifecycleState for SignalRefuted {
    fn name() -> &'static str {
        "refuted"
    }
    fn is_terminal() -> bool {
        true
    }
    fn is_initial() -> bool {
        false
    }
}

// ═══════════════════════════════════════════════════════════
// TYPESAFE SIGNAL
// ═══════════════════════════════════════════════════════════

/// Signal lifecycle wrapper with compile-time state enforcement.
///
/// Tier: T2-C (ς + ∂ + ∅ + →)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypesafeSignal<S: LifecycleState> {
    /// Entity identifier.
    pub entity_id: u64,
    /// Drug associated with this signal.
    pub drug: String,
    /// Event associated with this signal.
    pub event: String,
    /// Context data.
    pub context: StateContext,
    /// Number of transitions applied.
    pub transition_count: u64,
    /// State marker.
    #[serde(skip)]
    _state: PhantomData<S>,
}

impl<S: LifecycleState> TypesafeSignal<S> {
    /// Returns the current state name.
    #[must_use]
    pub fn state_name(&self) -> &'static str {
        S::name()
    }

    /// Returns whether the signal is in a terminal state.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        S::is_terminal()
    }

    /// Returns the entity ID.
    #[must_use]
    pub fn entity_id(&self) -> u64 {
        self.entity_id
    }

    /// Returns the drug name.
    #[must_use]
    pub fn drug(&self) -> &str {
        &self.drug
    }

    /// Returns the event name.
    #[must_use]
    pub fn event(&self) -> &str {
        &self.event
    }

    /// Returns the transition count.
    #[must_use]
    pub fn transition_count(&self) -> u64 {
        self.transition_count
    }
}

impl TypesafeSignal<SignalDetected> {
    /// Creates a new signal in the Detected state.
    #[must_use]
    pub fn new(entity_id: u64, drug: &str, event: &str, timestamp: u64) -> Self {
        Self {
            entity_id,
            drug: drug.to_string(),
            event: event.to_string(),
            context: StateContext::new(entity_id, timestamp),
            transition_count: 0,
            _state: PhantomData,
        }
    }

    /// Validate the signal → transitions to Validated state.
    #[must_use]
    pub fn validate(self, timestamp: u64) -> TypesafeSignal<SignalValidated> {
        TypesafeSignal {
            entity_id: self.entity_id,
            drug: self.drug,
            event: self.event,
            context: StateContext::new(self.entity_id, timestamp),
            transition_count: self.transition_count + 1,
            _state: PhantomData,
        }
    }
}

impl TypesafeSignal<SignalValidated> {
    /// Confirm the signal → transitions to Confirmed (terminal).
    #[must_use]
    pub fn confirm(self, timestamp: u64) -> TypesafeSignal<SignalConfirmed> {
        TypesafeSignal {
            entity_id: self.entity_id,
            drug: self.drug,
            event: self.event,
            context: StateContext::new(self.entity_id, timestamp),
            transition_count: self.transition_count + 1,
            _state: PhantomData,
        }
    }

    /// Refute the signal → transitions to Refuted (terminal).
    #[must_use]
    pub fn refute(self, timestamp: u64) -> TypesafeSignal<SignalRefuted> {
        TypesafeSignal {
            entity_id: self.entity_id,
            drug: self.drug,
            event: self.event,
            context: StateContext::new(self.entity_id, timestamp),
            transition_count: self.transition_count + 1,
            _state: PhantomData,
        }
    }
}

// No transition methods on SignalConfirmed or SignalRefuted — terminal states.

impl<S: LifecycleState> GroundsTo for TypesafeSignal<S> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
            LexPrimitiva::Void,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::GroundingTier;

    #[test]
    fn test_typesafe_signal_grounding() {
        let comp = TypesafeSignal::<SignalDetected>::primitive_composition();
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
        assert_eq!(comp.unique().len(), 4);
    }

    #[test]
    fn test_signal_confirm_path() {
        let signal = TypesafeSignal::<SignalDetected>::new(200, "aspirin", "headache", 1000);
        assert_eq!(signal.state_name(), "detected");
        assert!(!signal.is_terminal());

        let signal = signal.validate(2000);
        assert_eq!(signal.state_name(), "validated");

        let signal = signal.confirm(3000);
        assert_eq!(signal.state_name(), "confirmed");
        assert!(signal.is_terminal());
        assert_eq!(signal.drug(), "aspirin");
        assert_eq!(signal.event(), "headache");
    }

    #[test]
    fn test_signal_refute_path() {
        let signal = TypesafeSignal::<SignalDetected>::new(201, "ibuprofen", "nausea", 1000);
        let signal = signal.validate(2000);
        let signal = signal.refute(3000);

        assert_eq!(signal.state_name(), "refuted");
        assert!(signal.is_terminal());
        assert_eq!(signal.transition_count(), 2);
    }

    #[test]
    fn test_state_markers() {
        assert!(SignalDetected::is_initial());
        assert!(!SignalValidated::is_initial());
        assert!(SignalConfirmed::is_terminal());
        assert!(SignalRefuted::is_terminal());
    }
}
