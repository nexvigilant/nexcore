// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # nexcore-model-checker
//!
//! Model checking algorithms for verifying temporal properties of state machines.
//!
//! Provides CTL (Computation Tree Logic) and LTL (Linear Temporal Logic) model
//! checking against Kripke structures, producing verified `ProofCertificate`s
//! or counterexample traces.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────┐     ┌──────────────┐
//! │ CTL Checker   │     │ LTL Bounded  │
//! │ (fixpoint)    │     │  Checker     │
//! └──────┬───────┘     └──────┬───────┘
//!        │                     │
//!        └────────┬────────────┘
//!                 │
//!        ┌────────▼────────┐
//!        │ Kripke Structure │
//!        │ (S, S0, R, L)   │
//!        └─────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_model_checker::prelude::*;
//! use nexcore_state_theory::temporal::{AtomicProp, CtlFormula};
//!
//! // Build a Kripke structure
//! let mut b = KripkeStructure::builder(3);
//! b.initial(0);
//! b.transition(0, 1).transition(1, 2).transition(2, 0);
//! b.label(0, 10).label(1, 11).label(2, 12);
//! let model = b.build().ok().unwrap_or_else(|| unreachable!());
//!
//! // Check CTL: AG ¬error
//! let formula = CtlFormula::Not(Box::new(CtlFormula::Atom(AtomicProp::new("error", 99))))
//!     .forall_globally();
//! let checker = CtlChecker::new(&model);
//! let result = checker.check(&formula);
//! assert!(result.is_satisfied());
//! ```
//!
//! ## Primitive Grounding
//!
//! | Type | Tier | Dominant | Primitives |
//! |------|------|----------|------------|
//! | `KripkeStructure` | T3 | ρ | ρ ∂ → κ ∃ ς |
//! | `CtlChecker` | T2-C | ρ | ρ ∂ κ → |
//! | `LtlBoundedChecker` | T2-C | σ | σ ∂ κ ρ |
//! | `CheckResult` | T2-P | κ | κ ∂ |
//! | `Counterexample` | T2-C | σ | σ → ∂ |
//! | `Witness` | T2-P | N | N κ |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod ctl;
pub mod grounding;
pub mod kripke;
pub mod ltl;
pub mod result;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::ctl::CtlChecker;
    pub use crate::kripke::{KripkeBuilder, KripkeError, KripkeStructure, PropId, StateId};
    pub use crate::ltl::{LtlBoundedChecker, check_ltl};
    pub use crate::result::{CheckResult, Counterexample, Witness};
}

#[cfg(test)]
mod integration_tests {
    use crate::prelude::*;
    use nexcore_state_theory::temporal::{AtomicProp, CtlFormula, LtlFormula};

    /// Mutual exclusion protocol: two processes, critical sections can't overlap.
    fn mutex_protocol() -> KripkeStructure {
        // States: (P1, P2) where each is {idle, trying, critical}
        // 0=(I,I), 1=(T,I), 2=(C,I), 3=(I,T), 4=(T,T), 5=(I,C)
        // We omit (C,C) — that's the mutual exclusion violation
        // Props: 100=p1_critical, 101=p2_critical, 102=mutex_ok
        let mut b = KripkeStructure::builder(6);
        b.initial(0);

        // (I,I) → (T,I) or (I,T)
        b.transition(0, 1).transition(0, 3);
        // (T,I) → (C,I)
        b.transition(1, 2);
        // (C,I) → (I,I) — release
        b.transition(2, 0);
        // (I,T) → (I,C)
        b.transition(3, 5);
        // (T,T) → (C,I) or (I,C) — nondeterministic grant
        b.transition(4, 2).transition(4, 5);
        // (I,C) → (I,I) — release
        b.transition(5, 0);

        // Also (T,I) can go to (T,T) if P2 starts trying
        b.transition(1, 4);
        // And (I,T) can go to (T,T) if P1 starts trying
        b.transition(3, 4);

        // Labels
        b.label(2, 100).prop_name(100, "p1_critical");
        b.label(5, 101).prop_name(101, "p2_critical");
        // mutex_ok in all states (we enforce it by not having (C,C))
        for i in 0..6 {
            b.label(i, 102);
        }
        b.prop_name(102, "mutex_ok");

        b.build().ok().unwrap_or_else(|| unreachable!())
    }

    #[test]
    fn test_mutex_mutual_exclusion() {
        let model = mutex_protocol();
        let checker = CtlChecker::new(&model);

        // AG ¬(p1_critical ∧ p2_critical): both can't be in critical section
        let both_critical = CtlFormula::And(
            Box::new(CtlFormula::Atom(AtomicProp::new("p1_critical", 100))),
            Box::new(CtlFormula::Atom(AtomicProp::new("p2_critical", 101))),
        );
        let formula = CtlFormula::Not(Box::new(both_critical)).forall_globally();
        let result = checker.check(&formula);
        assert!(result.is_satisfied(), "Mutual exclusion should hold");
    }

    #[test]
    fn test_mutex_liveness_p1() {
        let model = mutex_protocol();
        let checker = CtlChecker::new(&model);

        // EF p1_critical: P1 can eventually reach critical section
        let formula = CtlFormula::Atom(AtomicProp::new("p1_critical", 100)).exists_eventually();
        let result = checker.check(&formula);
        assert!(
            result.is_satisfied(),
            "P1 should be able to reach critical section"
        );
    }

    #[test]
    fn test_mutex_liveness_p2() {
        let model = mutex_protocol();
        let checker = CtlChecker::new(&model);

        // EF p2_critical: P2 can eventually reach critical section
        let formula = CtlFormula::Atom(AtomicProp::new("p2_critical", 101)).exists_eventually();
        let result = checker.check(&formula);
        assert!(
            result.is_satisfied(),
            "P2 should be able to reach critical section"
        );
    }

    #[test]
    fn test_mutex_ag_mutex_ok() {
        let model = mutex_protocol();
        let checker = CtlChecker::new(&model);

        // AG mutex_ok: the mutex invariant always holds
        let formula = CtlFormula::Atom(AtomicProp::new("mutex_ok", 102)).forall_globally();
        let result = checker.check(&formula);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_ltl_mutex_safety() {
        let model = mutex_protocol();

        // G ¬(p1_critical ∧ p2_critical) — safety via LTL
        let both = LtlFormula::And(
            Box::new(LtlFormula::Atom(AtomicProp::new("p1_critical", 100))),
            Box::new(LtlFormula::Atom(AtomicProp::new("p2_critical", 101))),
        );
        let formula = LtlFormula::Not(Box::new(both)).globally();
        let result = check_ltl(&model, &formula, 20);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_ltl_traffic_light_cycle() {
        let mut b = KripkeStructure::builder(3);
        b.initial(0);
        b.transition(0, 1).transition(1, 2).transition(2, 0);
        b.label(0, 10).label(1, 11).label(2, 12);
        let model = b.build().ok().unwrap_or_else(|| unreachable!());

        // red → X green: after red, next is always green
        let formula = LtlFormula::Implies(
            Box::new(LtlFormula::Atom(AtomicProp::new("red", 10))),
            Box::new(LtlFormula::Atom(AtomicProp::new("green", 11)).next()),
        );
        let result = check_ltl(&model, &formula, 10);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_ctl_and_ltl_agree_on_safety() {
        // Both checkers should agree on a simple safety property
        let mut b = KripkeStructure::builder(2);
        b.initial(0);
        b.transition(0, 1).self_loop(1);
        b.label(0, 50).label(1, 51);
        let model = b.build().ok().unwrap_or_else(|| unreachable!());

        // AG ¬(prop_50 ∧ prop_51): no state has both props
        let ctl_formula = CtlFormula::Not(Box::new(CtlFormula::And(
            Box::new(CtlFormula::Atom(AtomicProp::new("a", 50))),
            Box::new(CtlFormula::Atom(AtomicProp::new("b", 51))),
        )))
        .forall_globally();

        let ltl_formula = LtlFormula::Not(Box::new(LtlFormula::And(
            Box::new(LtlFormula::Atom(AtomicProp::new("a", 50))),
            Box::new(LtlFormula::Atom(AtomicProp::new("b", 51))),
        )))
        .globally();

        let ctl_result = CtlChecker::new(&model).check(&ctl_formula);
        let ltl_result = check_ltl(&model, &ltl_formula, 10);

        assert_eq!(ctl_result.is_satisfied(), ltl_result.is_satisfied());
    }

    #[test]
    fn test_deadlock_freedom_standard_property() {
        let mut b = KripkeStructure::builder(3);
        b.initial(0);
        b.transition(0, 1).transition(1, 2).self_loop(2);
        // Label state 2 as terminal
        b.label(2, 1); // prop 1 = terminal
        let model = b.build().ok().unwrap_or_else(|| unreachable!());

        let checker = CtlChecker::new(&model);

        // AG(terminal ∨ EX true): non-terminal states can always progress
        // Since state 2 has self-loop, EX true holds everywhere
        let terminal = CtlFormula::Atom(AtomicProp::new("terminal", 1));
        let can_progress = CtlFormula::True.exists_next();
        let formula = CtlFormula::Or(Box::new(terminal), Box::new(can_progress)).forall_globally();

        let result = checker.check(&formula);
        assert!(result.is_satisfied());
    }
}
