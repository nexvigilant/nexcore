//! # Typestate Patterns
//!
//! Compile-time state enforcement using Rust's type system.
//!
//! ## Core Pattern
//!
//! The typestate pattern uses:
//! 1. **State markers**: Zero-sized types implementing `State`
//! 2. **Wrapper struct**: Generic over state marker, contains data
//! 3. **Transition methods**: `impl` blocks only on valid source states
//!
//! ```text
//! struct Machine<S: State> {
//!     data: T,
//!     _state: PhantomData<S>,
//! }
//!
//! impl Machine<StateA> {
//!     fn transition(self) -> Machine<StateB> { ... }
//! }
//! // Machine<StateC>.transition() → compile error
//! ```
//!
//! ## Sealed Trait Pattern
//!
//! To prevent external implementations of states:
//!
//! ```rust,ignore
//! mod private {
//!     pub trait Sealed {}
//!     impl Sealed for super::StateA {}
//!     impl Sealed for super::StateB {}
//! }
//!
//! pub trait State: private::Sealed { ... }
//! ```

use alloc::string::String;
use core::marker::PhantomData;

use crate::State;

// ═══════════════════════════════════════════════════════════
// TYPESTATE WRAPPER
// ═══════════════════════════════════════════════════════════

/// Generic typestate wrapper that can hold any payload.
///
/// This is a building block for creating domain-specific typestate machines.
///
/// ## Type Parameters
///
/// - `S`: The current state (must implement `State`)
/// - `T`: The payload type
///
/// ## Example
///
/// ```rust
/// use nexcore_state_theory::typestate::TypesafeWrapper;
/// use nexcore_state_theory::State;
///
/// struct Draft;
/// struct Published;
///
/// impl State for Draft {
///     fn name() -> &'static str { "draft" }
///     fn is_terminal() -> bool { false }
///     fn is_initial() -> bool { true }
/// }
///
/// impl State for Published {
///     fn name() -> &'static str { "published" }
///     fn is_terminal() -> bool { true }
/// }
///
/// type Document<S> = TypesafeWrapper<S, String>;
/// ```
#[derive(Debug, Clone)]
pub struct TypesafeWrapper<S: State, T> {
    /// The wrapped payload.
    pub payload: T,
    /// Transition counter.
    pub transitions: u64,
    /// State marker (zero-sized).
    _state: PhantomData<S>,
}

impl<S: State, T> TypesafeWrapper<S, T> {
    /// Current state name.
    #[must_use]
    pub fn state_name(&self) -> &'static str {
        S::name()
    }

    /// Whether in terminal state.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        S::is_terminal()
    }

    /// Whether in initial state.
    #[must_use]
    pub fn is_initial(&self) -> bool {
        S::is_initial()
    }

    /// Number of transitions applied.
    #[must_use]
    pub fn transition_count(&self) -> u64 {
        self.transitions
    }

    /// Access the payload.
    #[must_use]
    pub fn payload(&self) -> &T {
        &self.payload
    }

    /// Mutably access the payload.
    pub fn payload_mut(&mut self) -> &mut T {
        &mut self.payload
    }

    /// Transform to a new state (internal helper).
    #[must_use]
    fn transition_to<N: State>(self) -> TypesafeWrapper<N, T> {
        TypesafeWrapper {
            payload: self.payload,
            transitions: self.transitions + 1,
            _state: PhantomData,
        }
    }
}

impl<S: State, T: Default> TypesafeWrapper<S, T> {
    /// Create with default payload.
    #[must_use]
    pub fn new() -> Self {
        Self {
            payload: T::default(),
            transitions: 0,
            _state: PhantomData,
        }
    }
}

impl<S: State, T: Default> Default for TypesafeWrapper<S, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: State, T> TypesafeWrapper<S, T> {
    /// Create with specific payload.
    #[must_use]
    pub fn with_payload(payload: T) -> Self {
        Self {
            payload,
            transitions: 0,
            _state: PhantomData,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// TRANSITION BUILDER
// ═══════════════════════════════════════════════════════════

/// Builder for defining state machine transitions.
///
/// This allows fluent construction of typestate machines.
#[derive(Debug)]
pub struct TransitionBuilder<S: State, T> {
    wrapper: TypesafeWrapper<S, T>,
}

impl<S: State, T> TransitionBuilder<S, T> {
    /// Start building from a wrapper.
    #[must_use]
    pub fn from(wrapper: TypesafeWrapper<S, T>) -> Self {
        Self { wrapper }
    }

    /// Apply a transformation to the payload before transitioning.
    #[must_use]
    pub fn map<F: FnOnce(T) -> T>(mut self, f: F) -> Self {
        self.wrapper.payload = f(self.wrapper.payload);
        self
    }

    /// Transition to a new state.
    #[must_use]
    pub fn to<N: State>(self) -> TypesafeWrapper<N, T> {
        self.wrapper.transition_to()
    }

    /// Consume and return the wrapper without transitioning.
    #[must_use]
    pub fn build(self) -> TypesafeWrapper<S, T> {
        self.wrapper
    }
}

// ═══════════════════════════════════════════════════════════
// TRANSITION TRAIT
// ═══════════════════════════════════════════════════════════

/// Trait for types that can perform state transitions.
///
/// Implement this on your wrapper type to define valid transitions.
pub trait Transitionable<From: State, To: State>: Sized {
    /// Perform the transition.
    fn transition(self) -> Self;
}

// ═══════════════════════════════════════════════════════════
// STATE MACHINE METADATA
// ═══════════════════════════════════════════════════════════

/// Runtime metadata about a state machine.
///
/// Since typestate information is erased at runtime, this struct
/// captures state machine properties for logging/debugging.
#[derive(Debug, Clone)]
pub struct StateMachineMetadata {
    /// Name of the state machine.
    pub name: String,
    /// Current state name.
    pub current_state: String,
    /// Total number of states.
    pub state_count: usize,
    /// Number of transitions performed.
    pub transition_count: u64,
    /// Whether currently in terminal state.
    pub is_terminal: bool,
}

impl StateMachineMetadata {
    /// Create metadata for a typestate wrapper.
    #[must_use]
    pub fn from_wrapper<S: State, T>(
        name: &str,
        wrapper: &TypesafeWrapper<S, T>,
        state_count: usize,
    ) -> Self {
        Self {
            name: name.into(),
            current_state: S::name().into(),
            state_count,
            transition_count: wrapper.transitions,
            is_terminal: S::is_terminal(),
        }
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // Test states
    struct StateA;
    struct StateB;
    struct StateC;

    impl State for StateA {
        fn name() -> &'static str {
            "A"
        }
        fn is_terminal() -> bool {
            false
        }
        fn is_initial() -> bool {
            true
        }
    }

    impl State for StateB {
        fn name() -> &'static str {
            "B"
        }
        fn is_terminal() -> bool {
            false
        }
    }

    impl State for StateC {
        fn name() -> &'static str {
            "C"
        }
        fn is_terminal() -> bool {
            true
        }
    }

    #[test]
    fn test_typestate_wrapper() {
        let w: TypesafeWrapper<StateA, u32> = TypesafeWrapper::with_payload(42);
        assert_eq!(w.state_name(), "A");
        assert!(w.is_initial());
        assert!(!w.is_terminal());
        assert_eq!(*w.payload(), 42);
    }

    #[test]
    fn test_transition_builder() {
        let w: TypesafeWrapper<StateA, u32> = TypesafeWrapper::with_payload(10);

        let w2: TypesafeWrapper<StateB, u32> = TransitionBuilder::from(w).map(|x| x * 2).to();

        assert_eq!(w2.state_name(), "B");
        assert_eq!(*w2.payload(), 20);
        assert_eq!(w2.transition_count(), 1);
    }

    #[test]
    fn test_multiple_transitions() {
        let w: TypesafeWrapper<StateA, String> = TypesafeWrapper::with_payload("hello".into());

        let w2: TypesafeWrapper<StateB, String> = w.transition_to();
        let w3: TypesafeWrapper<StateC, String> = w2.transition_to();

        assert_eq!(w3.state_name(), "C");
        assert!(w3.is_terminal());
        assert_eq!(w3.transition_count(), 2);
    }

    #[test]
    fn test_metadata() {
        let w: TypesafeWrapper<StateB, ()> = TypesafeWrapper::new();
        let meta = StateMachineMetadata::from_wrapper("TestMachine", &w, 3);

        assert_eq!(meta.name, "TestMachine");
        assert_eq!(meta.current_state, "B");
        assert_eq!(meta.state_count, 3);
        assert!(!meta.is_terminal);
    }
}
