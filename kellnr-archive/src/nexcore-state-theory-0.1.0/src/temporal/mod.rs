// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Temporal Logic for State Machines
//!
//! Specification and verification of temporal properties using:
//! - **LTL** (Linear Temporal Logic): Properties over infinite traces
//! - **CTL** (Computation Tree Logic): Properties over branching time
//! - **Safety/Liveness**: Classification of temporal properties
//!
//! ## Temporal Operators
//!
//! ### LTL Operators
//!
//! | Operator | Symbol | Meaning |
//! |----------|--------|---------|
//! | Next | X φ | φ holds in the next state |
//! | Eventually | F φ | φ holds at some future state |
//! | Globally | G φ | φ holds in all future states |
//! | Until | φ U ψ | φ holds until ψ becomes true |
//! | Release | φ R ψ | ψ holds until φ releases it |
//!
//! ### CTL Operators
//!
//! | Operator | Symbol | Meaning |
//! |----------|--------|---------|
//! | Exists Next | EX φ | Some successor satisfies φ |
//! | Forall Next | AX φ | All successors satisfy φ |
//! | Exists Eventually | EF φ | Some path reaches φ |
//! | Forall Eventually | AF φ | All paths reach φ |
//! | Exists Globally | EG φ | Some path maintains φ |
//! | Forall Globally | AG φ | All paths maintain φ |
//!
//! ## Safety vs Liveness
//!
//! - **Safety**: "Nothing bad happens" (G ¬bad)
//! - **Liveness**: "Something good eventually happens" (F good)

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::State;

// ═══════════════════════════════════════════════════════════
// ATOMIC PROPOSITIONS
// ═══════════════════════════════════════════════════════════

/// An atomic proposition about a state.
///
/// Propositions are the building blocks of temporal formulas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicProp {
    /// Proposition name.
    pub name: String,
    /// Proposition identifier.
    pub id: u32,
}

impl AtomicProp {
    /// Create a new atomic proposition.
    #[must_use]
    pub fn new(name: &str, id: u32) -> Self {
        Self {
            name: name.into(),
            id,
        }
    }

    /// Standard propositions.
    #[must_use]
    pub fn is_initial() -> Self {
        Self::new("initial", 0)
    }

    /// Terminal state proposition.
    #[must_use]
    pub fn is_terminal() -> Self {
        Self::new("terminal", 1)
    }

    /// Error state proposition.
    #[must_use]
    pub fn is_error() -> Self {
        Self::new("error", 2)
    }
}

// ═══════════════════════════════════════════════════════════
// LTL FORMULA
// ═══════════════════════════════════════════════════════════

/// Linear Temporal Logic formula.
///
/// LTL formulas describe properties of infinite traces (linear sequences).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LtlFormula {
    /// Atomic proposition.
    Atom(AtomicProp),

    /// Boolean true.
    True,

    /// Boolean false.
    False,

    /// Negation: ¬φ
    Not(Box<LtlFormula>),

    /// Conjunction: φ ∧ ψ
    And(Box<LtlFormula>, Box<LtlFormula>),

    /// Disjunction: φ ∨ ψ
    Or(Box<LtlFormula>, Box<LtlFormula>),

    /// Implication: φ → ψ
    Implies(Box<LtlFormula>, Box<LtlFormula>),

    /// Next: X φ (φ holds in next state)
    Next(Box<LtlFormula>),

    /// Eventually: F φ (φ holds at some future state)
    Eventually(Box<LtlFormula>),

    /// Globally: G φ (φ holds in all future states)
    Globally(Box<LtlFormula>),

    /// Until: φ U ψ (φ holds until ψ)
    Until(Box<LtlFormula>, Box<LtlFormula>),

    /// Release: φ R ψ (ψ holds until φ releases it)
    Release(Box<LtlFormula>, Box<LtlFormula>),

    /// Weak Until: φ W ψ (φ holds until ψ, or forever)
    WeakUntil(Box<LtlFormula>, Box<LtlFormula>),
}

impl LtlFormula {
    /// Create an atomic proposition formula.
    #[must_use]
    pub fn atom(name: &str, id: u32) -> Self {
        Self::Atom(AtomicProp::new(name, id))
    }

    /// Negation.
    #[must_use]
    pub fn not(self) -> Self {
        Self::Not(Box::new(self))
    }

    /// Conjunction.
    #[must_use]
    pub fn and(self, other: Self) -> Self {
        Self::And(Box::new(self), Box::new(other))
    }

    /// Disjunction.
    #[must_use]
    pub fn or(self, other: Self) -> Self {
        Self::Or(Box::new(self), Box::new(other))
    }

    /// Implication.
    #[must_use]
    pub fn implies(self, other: Self) -> Self {
        Self::Implies(Box::new(self), Box::new(other))
    }

    /// Next state.
    #[must_use]
    pub fn next(self) -> Self {
        Self::Next(Box::new(self))
    }

    /// Eventually (some future state).
    #[must_use]
    pub fn eventually(self) -> Self {
        Self::Eventually(Box::new(self))
    }

    /// Globally (all future states).
    #[must_use]
    pub fn globally(self) -> Self {
        Self::Globally(Box::new(self))
    }

    /// Until.
    #[must_use]
    pub fn until(self, other: Self) -> Self {
        Self::Until(Box::new(self), Box::new(other))
    }

    /// Release.
    #[must_use]
    pub fn release(self, other: Self) -> Self {
        Self::Release(Box::new(self), Box::new(other))
    }

    /// Whether this is a safety property.
    ///
    /// Safety = "bad thing never happens" = G ¬bad
    #[must_use]
    pub fn is_safety(&self) -> bool {
        matches!(self, Self::Globally(inner) if matches!(**inner, Self::Not(_)))
    }

    /// Whether this is a liveness property.
    ///
    /// Liveness = "good thing eventually happens" = F good
    #[must_use]
    pub fn is_liveness(&self) -> bool {
        matches!(self, Self::Eventually(_))
    }
}

// ═══════════════════════════════════════════════════════════
// CTL FORMULA
// ═══════════════════════════════════════════════════════════

/// Path quantifier for CTL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathQuantifier {
    /// Exists a path (E)
    Exists,
    /// For all paths (A)
    ForAll,
}

/// Computation Tree Logic formula.
///
/// CTL formulas describe properties of branching computation trees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CtlFormula {
    /// Atomic proposition.
    Atom(AtomicProp),

    /// Boolean true.
    True,

    /// Boolean false.
    False,

    /// Negation: ¬φ
    Not(Box<CtlFormula>),

    /// Conjunction: φ ∧ ψ
    And(Box<CtlFormula>, Box<CtlFormula>),

    /// Disjunction: φ ∨ ψ
    Or(Box<CtlFormula>, Box<CtlFormula>),

    /// EX φ: exists a next state satisfying φ
    ExistsNext(Box<CtlFormula>),

    /// AX φ: all next states satisfy φ
    ForAllNext(Box<CtlFormula>),

    /// EF φ: exists a path reaching φ
    ExistsEventually(Box<CtlFormula>),

    /// AF φ: all paths reach φ
    ForAllEventually(Box<CtlFormula>),

    /// EG φ: exists a path maintaining φ forever
    ExistsGlobally(Box<CtlFormula>),

    /// AG φ: all paths maintain φ forever
    ForAllGlobally(Box<CtlFormula>),

    /// E[φ U ψ]: exists a path where φ until ψ
    ExistsUntil(Box<CtlFormula>, Box<CtlFormula>),

    /// A[φ U ψ]: all paths have φ until ψ
    ForAllUntil(Box<CtlFormula>, Box<CtlFormula>),
}

impl CtlFormula {
    /// Create an atomic proposition formula.
    #[must_use]
    pub fn atom(name: &str, id: u32) -> Self {
        Self::Atom(AtomicProp::new(name, id))
    }

    /// EX: exists next.
    #[must_use]
    pub fn exists_next(self) -> Self {
        Self::ExistsNext(Box::new(self))
    }

    /// AX: forall next.
    #[must_use]
    pub fn forall_next(self) -> Self {
        Self::ForAllNext(Box::new(self))
    }

    /// EF: exists eventually (reachability).
    #[must_use]
    pub fn exists_eventually(self) -> Self {
        Self::ExistsEventually(Box::new(self))
    }

    /// AF: forall eventually (inevitable).
    #[must_use]
    pub fn forall_eventually(self) -> Self {
        Self::ForAllEventually(Box::new(self))
    }

    /// EG: exists globally.
    #[must_use]
    pub fn exists_globally(self) -> Self {
        Self::ExistsGlobally(Box::new(self))
    }

    /// AG: forall globally (invariant).
    #[must_use]
    pub fn forall_globally(self) -> Self {
        Self::ForAllGlobally(Box::new(self))
    }

    /// Whether this represents a reachability property.
    #[must_use]
    pub fn is_reachability(&self) -> bool {
        matches!(self, Self::ExistsEventually(_))
    }

    /// Whether this represents an inevitability property (AF φ).
    #[must_use]
    pub fn is_inevitability(&self) -> bool {
        matches!(self, Self::ForAllEventually(_))
    }

    /// Whether this is a liveness property (EF or AF).
    #[must_use]
    pub fn is_liveness(&self) -> bool {
        self.is_reachability() || self.is_inevitability()
    }

    /// Whether this represents an invariant property.
    #[must_use]
    pub fn is_invariant(&self) -> bool {
        matches!(self, Self::ForAllGlobally(_))
    }
}

// ═══════════════════════════════════════════════════════════
// PROPERTY CLASSIFICATION
// ═══════════════════════════════════════════════════════════

/// Classification of temporal properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyClass {
    /// Safety: bad things never happen.
    Safety,
    /// Liveness: good things eventually happen.
    Liveness,
    /// Mixed: combination of safety and liveness.
    Mixed,
}

/// A temporal property with its classification.
#[derive(Debug, Clone)]
pub struct TemporalProperty {
    /// Property name.
    pub name: String,
    /// Property description.
    pub description: String,
    /// Classification.
    pub class: PropertyClass,
    /// LTL formula (if expressible in LTL).
    pub ltl: Option<LtlFormula>,
    /// CTL formula (if expressible in CTL).
    pub ctl: Option<CtlFormula>,
}

impl TemporalProperty {
    /// Create a safety property.
    #[must_use]
    pub fn safety(name: &str, description: &str, formula: LtlFormula) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            class: PropertyClass::Safety,
            ltl: Some(formula),
            ctl: None,
        }
    }

    /// Create a liveness property.
    #[must_use]
    pub fn liveness(name: &str, description: &str, formula: LtlFormula) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            class: PropertyClass::Liveness,
            ltl: Some(formula),
            ctl: None,
        }
    }

    /// Create from CTL formula.
    #[must_use]
    pub fn from_ctl(name: &str, description: &str, formula: CtlFormula) -> Self {
        let class = if formula.is_invariant() {
            PropertyClass::Safety
        } else if formula.is_liveness() {
            PropertyClass::Liveness
        } else {
            PropertyClass::Mixed
        };

        Self {
            name: name.into(),
            description: description.into(),
            class,
            ltl: None,
            ctl: Some(formula),
        }
    }
}

// ═══════════════════════════════════════════════════════════
// STANDARD PROPERTIES
// ═══════════════════════════════════════════════════════════

/// Standard temporal properties for state machines.
pub struct StandardProperties;

impl StandardProperties {
    /// Deadlock freedom: AG(¬terminal → EX true)
    ///
    /// "From any non-terminal state, there exists a next state."
    #[must_use]
    pub fn deadlock_freedom() -> TemporalProperty {
        let terminal = CtlFormula::atom("terminal", 1);
        let can_progress = CtlFormula::True.exists_next();

        TemporalProperty::from_ctl(
            "deadlock_freedom",
            "Non-terminal states can always progress",
            CtlFormula::Or(Box::new(terminal), Box::new(can_progress)).forall_globally(),
        )
    }

    /// Termination: AF terminal
    ///
    /// "All paths eventually reach a terminal state."
    #[must_use]
    pub fn termination() -> TemporalProperty {
        TemporalProperty::from_ctl(
            "termination",
            "All executions eventually terminate",
            CtlFormula::atom("terminal", 1).forall_eventually(),
        )
    }

    /// Reachability: EF target
    ///
    /// "The target state is reachable from the initial state."
    #[must_use]
    pub fn reachability(target: &str, id: u32) -> TemporalProperty {
        TemporalProperty::from_ctl(
            "reachability",
            &alloc::format!("State '{}' is reachable", target),
            CtlFormula::atom(target, id).exists_eventually(),
        )
    }

    /// Safety: AG ¬error
    ///
    /// "Error states are never reached."
    #[must_use]
    pub fn error_freedom() -> TemporalProperty {
        TemporalProperty::from_ctl(
            "error_freedom",
            "Error states are never reached",
            CtlFormula::Not(Box::new(CtlFormula::atom("error", 2))).forall_globally(),
        )
    }

    /// Progress: G(request → F response)
    ///
    /// "Every request is eventually followed by a response."
    #[must_use]
    pub fn progress() -> TemporalProperty {
        let request = LtlFormula::atom("request", 10);
        let response = LtlFormula::atom("response", 11).eventually();

        TemporalProperty::liveness(
            "progress",
            "Every request gets a response",
            request.implies(response).globally(),
        )
    }

    /// Fairness: GF enabled → GF executed
    ///
    /// "If a transition is infinitely often enabled, it is infinitely often taken."
    #[must_use]
    pub fn fairness() -> TemporalProperty {
        let enabled = LtlFormula::atom("enabled", 20).eventually().globally();
        let executed = LtlFormula::atom("executed", 21).eventually().globally();

        TemporalProperty {
            name: "fairness".into(),
            description: "Enabled transitions are eventually taken".into(),
            class: PropertyClass::Liveness,
            ltl: Some(enabled.implies(executed)),
            ctl: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// TRACE
// ═══════════════════════════════════════════════════════════

/// A finite or infinite trace of states.
#[derive(Debug, Clone)]
pub struct Trace<S: State> {
    /// Sequence of states.
    pub states: Vec<u32>,
    /// Whether the trace is finite.
    pub is_finite: bool,
    /// Type marker.
    _marker: PhantomData<S>,
}

impl<S: State> Trace<S> {
    /// Create a finite trace.
    #[must_use]
    pub fn finite(states: Vec<u32>) -> Self {
        Self {
            states,
            is_finite: true,
            _marker: PhantomData,
        }
    }

    /// Create a trace representing an infinite loop.
    #[must_use]
    pub fn infinite_loop(prefix: Vec<u32>, loop_states: Vec<u32>) -> Self {
        let mut states = prefix;
        states.extend(loop_states);
        Self {
            states,
            is_finite: false,
            _marker: PhantomData,
        }
    }

    /// Trace length (for finite traces).
    #[must_use]
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Whether trace is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_prop() {
        let init = AtomicProp::is_initial();
        assert_eq!(init.name, "initial");
        assert_eq!(init.id, 0);
    }

    #[test]
    fn test_ltl_formula_construction() {
        let p = LtlFormula::atom("p", 1);
        let q = LtlFormula::atom("q", 2);

        let formula = p.clone().until(q.clone());
        assert!(matches!(formula, LtlFormula::Until(_, _)));

        let safety = LtlFormula::atom("bad", 3).not().globally();
        assert!(safety.is_safety());

        let liveness = LtlFormula::atom("good", 4).eventually();
        assert!(liveness.is_liveness());
    }

    #[test]
    fn test_ctl_formula_construction() {
        let target = CtlFormula::atom("target", 1);

        let reachable = target.clone().exists_eventually();
        assert!(reachable.is_reachability());

        let invariant = CtlFormula::atom("safe", 2).forall_globally();
        assert!(invariant.is_invariant());
    }

    #[test]
    fn test_standard_properties() {
        let deadlock = StandardProperties::deadlock_freedom();
        assert_eq!(deadlock.class, PropertyClass::Safety);

        let termination = StandardProperties::termination();
        assert_eq!(termination.class, PropertyClass::Liveness);

        let reach = StandardProperties::reachability("goal", 5);
        assert_eq!(reach.class, PropertyClass::Liveness);
    }

    #[test]
    fn test_trace() {
        struct TestState;
        impl State for TestState {
            fn name() -> &'static str {
                "test"
            }
            fn is_terminal() -> bool {
                false
            }
        }

        let trace: Trace<TestState> = Trace::finite(alloc::vec![0, 1, 2, 3]);
        assert!(trace.is_finite);
        assert_eq!(trace.len(), 4);

        let infinite: Trace<TestState> = Trace::infinite_loop(alloc::vec![0, 1], alloc::vec![2, 3]);
        assert!(!infinite.is_finite);
    }
}
