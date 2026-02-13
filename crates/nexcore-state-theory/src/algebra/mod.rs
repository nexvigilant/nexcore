//! # State Algebra
//!
//! Mathematical operations on state machines: products, coproducts,
//! composition, and morphisms.
//!
//! ## Categorical View
//!
//! State machines form a category where:
//! - **Objects**: State machine types
//! - **Morphisms**: State-preserving transformations
//! - **Products**: Parallel composition (both machines run)
//! - **Coproducts**: Choice composition (one machine runs)
//!
//! ## Key Operations
//!
//! | Operation | Symbol | Meaning |
//! |-----------|--------|---------|
//! | Product | M₁ × M₂ | Run both machines in parallel |
//! | Coproduct | M₁ + M₂ | Run one machine or the other |
//! | Sequence | M₁ ; M₂ | Run M₁ then M₂ |
//! | Iteration | M* | Run M zero or more times |

use crate::State;
use core::marker::PhantomData;

// ═══════════════════════════════════════════════════════════
// STATE PRODUCT
// ═══════════════════════════════════════════════════════════

/// Product of two states: both must be satisfied simultaneously.
///
/// This is the categorical product in the category of state machines.
///
/// ## Example
///
/// ```rust
/// use nexcore_state_theory::algebra::StateProduct;
/// use nexcore_state_theory::State;
///
/// struct Running;
/// struct Connected;
///
/// impl State for Running {
///     fn name() -> &'static str { "running" }
///     fn is_terminal() -> bool { false }
/// }
///
/// impl State for Connected {
///     fn name() -> &'static str { "connected" }
///     fn is_terminal() -> bool { false }
/// }
///
/// // Product state: Running AND Connected
/// type ActiveState = StateProduct<Running, Connected>;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateProduct<A: State, B: State> {
    _a: PhantomData<A>,
    _b: PhantomData<B>,
}

impl<A: State, B: State> StateProduct<A, B> {
    /// Create a product state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _a: PhantomData,
            _b: PhantomData,
        }
    }

    /// Name of the first component.
    #[must_use]
    pub fn first_name() -> &'static str {
        A::name()
    }

    /// Name of the second component.
    #[must_use]
    pub fn second_name() -> &'static str {
        B::name()
    }
}

impl<A: State, B: State> Default for StateProduct<A, B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: State, B: State> State for StateProduct<A, B> {
    fn name() -> &'static str {
        // Static string concatenation not possible, return placeholder
        "product"
    }

    fn is_terminal() -> bool {
        // Product is terminal iff both components are terminal
        A::is_terminal() && B::is_terminal()
    }

    fn is_initial() -> bool {
        // Product is initial iff both components are initial
        A::is_initial() && B::is_initial()
    }
}

// ═══════════════════════════════════════════════════════════
// STATE COPRODUCT
// ═══════════════════════════════════════════════════════════

/// Which component of a coproduct is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoproductSide {
    /// Left component is active.
    Left,
    /// Right component is active.
    Right,
}

/// Coproduct of two states: exactly one is active at a time.
///
/// This is the categorical coproduct (disjoint union).
///
/// ## Example
///
/// ```rust
/// use nexcore_state_theory::algebra::{StateCoproduct, CoproductSide};
/// use nexcore_state_theory::State;
///
/// struct Success;
/// struct Failure;
///
/// impl State for Success {
///     fn name() -> &'static str { "success" }
///     fn is_terminal() -> bool { true }
/// }
///
/// impl State for Failure {
///     fn name() -> &'static str { "failure" }
///     fn is_terminal() -> bool { true }
/// }
///
/// // Coproduct state: Success OR Failure
/// let outcome: StateCoproduct<Success, Failure> = StateCoproduct::left();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateCoproduct<A: State, B: State> {
    side: CoproductSide,
    _a: PhantomData<A>,
    _b: PhantomData<B>,
}

impl<A: State, B: State> StateCoproduct<A, B> {
    /// Create a coproduct with left component active.
    #[must_use]
    pub const fn left() -> Self {
        Self {
            side: CoproductSide::Left,
            _a: PhantomData,
            _b: PhantomData,
        }
    }

    /// Create a coproduct with right component active.
    #[must_use]
    pub const fn right() -> Self {
        Self {
            side: CoproductSide::Right,
            _a: PhantomData,
            _b: PhantomData,
        }
    }

    /// Which side is active.
    #[must_use]
    pub const fn active_side(&self) -> CoproductSide {
        self.side
    }

    /// Whether left is active.
    #[must_use]
    pub const fn is_left(&self) -> bool {
        matches!(self.side, CoproductSide::Left)
    }

    /// Whether right is active.
    #[must_use]
    pub const fn is_right(&self) -> bool {
        matches!(self.side, CoproductSide::Right)
    }

    /// Active state name.
    #[must_use]
    pub fn active_name(&self) -> &'static str {
        match self.side {
            CoproductSide::Left => A::name(),
            CoproductSide::Right => B::name(),
        }
    }
}

// Note: StateCoproduct doesn't implement State because the active
// component is a runtime value, not a type-level distinction.

// ═══════════════════════════════════════════════════════════
// STATE COMPOSITION
// ═══════════════════════════════════════════════════════════

/// Sequential composition of two state machines.
///
/// The second machine begins when the first reaches a terminal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositionPhase {
    /// Running the first machine.
    First,
    /// Running the second machine.
    Second,
}

/// Sequential composition: run M₁, then M₂.
///
/// ## Semantics
///
/// - Starts in `First` phase with M₁'s initial state
/// - When M₁ reaches terminal, transitions to `Second` phase
/// - Ends when M₂ reaches terminal
#[derive(Debug, Clone)]
pub struct StateComposition<S1: State, S2: State> {
    phase: CompositionPhase,
    _s1: PhantomData<S1>,
    _s2: PhantomData<S2>,
}

impl<S1: State, S2: State> StateComposition<S1, S2> {
    /// Create in first phase.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            phase: CompositionPhase::First,
            _s1: PhantomData,
            _s2: PhantomData,
        }
    }

    /// Current phase.
    #[must_use]
    pub const fn phase(&self) -> CompositionPhase {
        self.phase
    }

    /// Advance to second phase.
    #[must_use]
    pub fn advance(self) -> Self {
        Self {
            phase: CompositionPhase::Second,
            _s1: PhantomData,
            _s2: PhantomData,
        }
    }
}

impl<S1: State, S2: State> Default for StateComposition<S1, S2> {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════
// STATE ITERATION
// ═══════════════════════════════════════════════════════════

/// Kleene star: run a state machine zero or more times.
///
/// M* = ε + M + MM + MMM + ...
#[derive(Debug, Clone)]
pub struct StateIteration<S: State> {
    iteration_count: u64,
    _s: PhantomData<S>,
}

impl<S: State> StateIteration<S> {
    /// Create with zero iterations (ε).
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            iteration_count: 0,
            _s: PhantomData,
        }
    }

    /// Number of completed iterations.
    #[must_use]
    pub const fn count(&self) -> u64 {
        self.iteration_count
    }

    /// Complete one iteration.
    #[must_use]
    pub const fn iterate(self) -> Self {
        Self {
            iteration_count: self.iteration_count + 1,
            _s: PhantomData,
        }
    }
}

impl<S: State> Default for StateIteration<S> {
    fn default() -> Self {
        Self::zero()
    }
}

// ═══════════════════════════════════════════════════════════
// MORPHISMS
// ═══════════════════════════════════════════════════════════

/// A morphism between state machines.
///
/// Maps states of M₁ to states of M₂ while preserving structure.
pub trait StateMorphism<From: State, To: State> {
    /// Apply the morphism.
    fn apply(&self) -> To;
}

/// Identity morphism: maps a state to itself.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityMorphism<S: State> {
    _s: PhantomData<S>,
}

impl<S: State> IdentityMorphism<S> {
    /// Create identity morphism.
    #[must_use]
    pub const fn new() -> Self {
        Self { _s: PhantomData }
    }
}

/// Composition of two morphisms.
#[derive(Debug, Clone)]
pub struct ComposedMorphism<A: State, B: State, C: State, M1, M2>
where
    M1: StateMorphism<A, B>,
    M2: StateMorphism<B, C>,
{
    first: M1,
    second: M2,
    _a: PhantomData<A>,
    _b: PhantomData<B>,
    _c: PhantomData<C>,
}

impl<A: State, B: State, C: State, M1, M2> ComposedMorphism<A, B, C, M1, M2>
where
    M1: StateMorphism<A, B>,
    M2: StateMorphism<B, C>,
{
    /// Compose two morphisms.
    #[must_use]
    pub fn compose(first: M1, second: M2) -> Self {
        Self {
            first,
            second,
            _a: PhantomData,
            _b: PhantomData,
            _c: PhantomData,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    struct StateA;
    struct StateB;
    struct TerminalState;

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

    impl State for TerminalState {
        fn name() -> &'static str {
            "terminal"
        }
        fn is_terminal() -> bool {
            true
        }
    }

    #[test]
    fn test_state_product() {
        type ProductAB = StateProduct<StateA, StateB>;

        assert!(!ProductAB::is_terminal());
        // Product is initial only if BOTH components are initial
        // StateA is initial, StateB is not → product is NOT initial
        assert!(!ProductAB::is_initial());

        type ProductInitial = StateProduct<StateA, StateA>;
        assert!(ProductInitial::is_initial()); // Both A are initial

        type ProductTerminal = StateProduct<TerminalState, TerminalState>;
        assert!(ProductTerminal::is_terminal());
    }

    #[test]
    fn test_state_coproduct() {
        let left: StateCoproduct<StateA, StateB> = StateCoproduct::left();
        assert!(left.is_left());
        assert_eq!(left.active_name(), "A");

        let right: StateCoproduct<StateA, StateB> = StateCoproduct::right();
        assert!(right.is_right());
        assert_eq!(right.active_name(), "B");
    }

    #[test]
    fn test_state_composition() {
        let comp: StateComposition<StateA, StateB> = StateComposition::new();
        assert!(matches!(comp.phase(), CompositionPhase::First));

        let comp = comp.advance();
        assert!(matches!(comp.phase(), CompositionPhase::Second));
    }

    #[test]
    fn test_state_iteration() {
        let iter: StateIteration<StateA> = StateIteration::zero();
        assert_eq!(iter.count(), 0);

        let iter = iter.iterate().iterate().iterate();
        assert_eq!(iter.count(), 3);
    }
}
