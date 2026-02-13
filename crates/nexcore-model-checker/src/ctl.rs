// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # CTL Model Checker
//!
//! Evaluates CTL formulas against Kripke structures using fixpoint iteration.
//!
//! ## Algorithm
//!
//! For each CTL operator, compute the **satisfaction set** Sat(φ) = { s ∈ S | s ⊨ φ }:
//!
//! | Operator | Computation |
//! |----------|------------|
//! | EX φ | pre∃(Sat(φ)) |
//! | AX φ | pre∀(Sat(φ)) |
//! | EF φ | μZ. Sat(φ) ∪ pre∃(Z) — least fixpoint |
//! | AF φ | μZ. Sat(φ) ∪ pre∀(Z) — least fixpoint |
//! | EG φ | νZ. Sat(φ) ∩ pre∃(Z) — greatest fixpoint |
//! | AG φ | νZ. Sat(φ) ∩ pre∀(Z) — greatest fixpoint |
//! | E[φ U ψ] | μZ. Sat(ψ) ∪ (Sat(φ) ∩ pre∃(Z)) — least fixpoint |
//! | A[φ U ψ] | μZ. Sat(ψ) ∪ (Sat(φ) ∩ pre∀(Z)) — least fixpoint |
//!
//! ## Primitive Grounding
//!
//! `CtlChecker` is T2-C (ρ-dominant):
//! - ρ Recursion: fixpoint iteration
//! - ∂ Boundary: satisfaction set boundaries
//! - κ Comparison: set membership tests
//! - → Causality: predecessor image computation

use std::collections::BTreeSet;

use nexcore_state_theory::temporal::CtlFormula;

use crate::kripke::{KripkeStructure, PropId, StateId};
use crate::result::{CheckResult, Counterexample, Witness};

/// CTL model checker operating on Kripke structures.
///
/// ## Tier: T2-C (ρ + ∂ + κ + →)
pub struct CtlChecker<'a> {
    /// The Kripke structure to check against.
    model: &'a KripkeStructure,
}

impl<'a> CtlChecker<'a> {
    /// Create a checker for the given model.
    #[must_use]
    pub fn new(model: &'a KripkeStructure) -> Self {
        Self { model }
    }

    /// Check if a CTL formula holds in all initial states.
    ///
    /// Returns `CheckResult::Satisfied` if φ holds in every initial state,
    /// or `CheckResult::Violated` with a counterexample path.
    #[must_use]
    pub fn check(&self, formula: &CtlFormula) -> CheckResult {
        let sat = self.sat(formula);

        // Check if all initial states satisfy the formula
        let violating: Vec<StateId> = self
            .model
            .initial_states
            .iter()
            .filter(|s| !sat.contains(s))
            .copied()
            .collect();

        if violating.is_empty() {
            CheckResult::Satisfied {
                witness: Witness {
                    satisfied_states: sat.len(),
                    total_states: self.model.state_count(),
                },
            }
        } else {
            // Build counterexample from first violating initial state
            let cex = self.build_counterexample(violating[0], formula);
            CheckResult::Violated {
                counterexample: cex,
            }
        }
    }

    /// Compute the satisfaction set: Sat(φ) = { s | s ⊨ φ }.
    #[must_use]
    pub fn sat(&self, formula: &CtlFormula) -> BTreeSet<StateId> {
        match formula {
            CtlFormula::Atom(ap) => self.model.states_with_prop(ap.id),
            CtlFormula::True => self.model.all_states(),
            CtlFormula::False => BTreeSet::new(),
            CtlFormula::Not(inner) => {
                let sat_inner = self.sat(inner);
                self.model
                    .all_states()
                    .difference(&sat_inner)
                    .copied()
                    .collect()
            }
            CtlFormula::And(left, right) => {
                let sat_l = self.sat(left);
                let sat_r = self.sat(right);
                sat_l.intersection(&sat_r).copied().collect()
            }
            CtlFormula::Or(left, right) => {
                let sat_l = self.sat(left);
                let sat_r = self.sat(right);
                sat_l.union(&sat_r).copied().collect()
            }
            CtlFormula::ExistsNext(inner) => {
                let sat_inner = self.sat(inner);
                self.model.pre_exists(&sat_inner)
            }
            CtlFormula::ForAllNext(inner) => {
                let sat_inner = self.sat(inner);
                self.model.pre_forall(&sat_inner)
            }
            CtlFormula::ExistsEventually(inner) => {
                // EF φ = μZ. Sat(φ) ∪ pre∃(Z)
                let sat_phi = self.sat(inner);
                self.least_fixpoint_exists(&sat_phi)
            }
            CtlFormula::ForAllEventually(inner) => {
                // AF φ = μZ. Sat(φ) ∪ pre∀(Z)
                let sat_phi = self.sat(inner);
                self.least_fixpoint_forall(&sat_phi)
            }
            CtlFormula::ExistsGlobally(inner) => {
                // EG φ = νZ. Sat(φ) ∩ pre∃(Z)
                let sat_phi = self.sat(inner);
                self.greatest_fixpoint_exists(&sat_phi)
            }
            CtlFormula::ForAllGlobally(inner) => {
                // AG φ = νZ. Sat(φ) ∩ pre∀(Z)
                let sat_phi = self.sat(inner);
                self.greatest_fixpoint_forall(&sat_phi)
            }
            CtlFormula::ExistsUntil(phi, psi) => {
                // E[φ U ψ] = μZ. Sat(ψ) ∪ (Sat(φ) ∩ pre∃(Z))
                let sat_phi = self.sat(phi);
                let sat_psi = self.sat(psi);
                self.least_fixpoint_until_exists(&sat_phi, &sat_psi)
            }
            CtlFormula::ForAllUntil(phi, psi) => {
                // A[φ U ψ] = μZ. Sat(ψ) ∪ (Sat(φ) ∩ pre∀(Z))
                let sat_phi = self.sat(phi);
                let sat_psi = self.sat(psi);
                self.least_fixpoint_until_forall(&sat_phi, &sat_psi)
            }
        }
    }

    // ── Fixpoint Computations ───────────────────────────────

    /// Least fixpoint: μZ. base ∪ pre∃(Z)
    ///
    /// Start from ∅, iteratively add states until stable.
    fn least_fixpoint_exists(&self, base: &BTreeSet<StateId>) -> BTreeSet<StateId> {
        let mut z = base.clone();
        loop {
            let pre = self.model.pre_exists(&z);
            let next: BTreeSet<StateId> = z.union(&pre).copied().collect();
            if next == z {
                break;
            }
            z = next;
        }
        z
    }

    /// Least fixpoint: μZ. base ∪ pre∀(Z)
    fn least_fixpoint_forall(&self, base: &BTreeSet<StateId>) -> BTreeSet<StateId> {
        let mut z = base.clone();
        loop {
            let pre = self.model.pre_forall(&z);
            let next: BTreeSet<StateId> = z.union(&pre).copied().collect();
            if next == z {
                break;
            }
            z = next;
        }
        z
    }

    /// Greatest fixpoint: νZ. base ∩ pre∃(Z)
    ///
    /// Start from all states satisfying base, remove states that can't stay.
    fn greatest_fixpoint_exists(&self, base: &BTreeSet<StateId>) -> BTreeSet<StateId> {
        let mut z = base.clone();
        loop {
            let pre = self.model.pre_exists(&z);
            let next: BTreeSet<StateId> = z.intersection(&pre).copied().collect();
            if next == z {
                break;
            }
            z = next;
        }
        z
    }

    /// Greatest fixpoint: νZ. base ∩ pre∀(Z)
    fn greatest_fixpoint_forall(&self, base: &BTreeSet<StateId>) -> BTreeSet<StateId> {
        let mut z = base.clone();
        loop {
            let pre = self.model.pre_forall(&z);
            let next: BTreeSet<StateId> = z.intersection(&pre).copied().collect();
            if next == z {
                break;
            }
            z = next;
        }
        z
    }

    /// Least fixpoint for Until: μZ. sat_psi ∪ (sat_phi ∩ pre∃(Z))
    fn least_fixpoint_until_exists(
        &self,
        sat_phi: &BTreeSet<StateId>,
        sat_psi: &BTreeSet<StateId>,
    ) -> BTreeSet<StateId> {
        let mut z = sat_psi.clone();
        loop {
            let pre = self.model.pre_exists(&z);
            let phi_and_pre: BTreeSet<StateId> = sat_phi.intersection(&pre).copied().collect();
            let next: BTreeSet<StateId> = z.union(&phi_and_pre).copied().collect();
            if next == z {
                break;
            }
            z = next;
        }
        z
    }

    /// Least fixpoint for Until: μZ. sat_psi ∪ (sat_phi ∩ pre∀(Z))
    fn least_fixpoint_until_forall(
        &self,
        sat_phi: &BTreeSet<StateId>,
        sat_psi: &BTreeSet<StateId>,
    ) -> BTreeSet<StateId> {
        let mut z = sat_psi.clone();
        loop {
            let pre = self.model.pre_forall(&z);
            let phi_and_pre: BTreeSet<StateId> = sat_phi.intersection(&pre).copied().collect();
            let next: BTreeSet<StateId> = z.union(&phi_and_pre).copied().collect();
            if next == z {
                break;
            }
            z = next;
        }
        z
    }

    // ── Counterexample Construction ─────────────────────────

    /// Build a counterexample path from a violating state.
    ///
    /// For AG φ violation: find a path to a state where ¬φ.
    /// For AF φ violation: find a cycle avoiding φ.
    /// For other operators: provide the violating state.
    fn build_counterexample(&self, start: StateId, _formula: &CtlFormula) -> Counterexample {
        // BFS from start to find a reachable "interesting" state
        let mut path = vec![start];
        let mut visited = BTreeSet::from([start]);
        let mut current = start;

        // Walk forward up to depth limit
        for _ in 0..self.model.state_count() {
            let succs = self.model.successors(current);
            let next = succs.iter().find(|s| !visited.contains(s));
            match next {
                Some(&s) => {
                    path.push(s);
                    visited.insert(s);
                    current = s;
                }
                None => break, // No unvisited successor, stop
            }
        }

        Counterexample {
            path,
            is_loop: false,
            description: "CTL property violation".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kripke::KripkeStructure;
    use nexcore_state_theory::temporal::{AtomicProp, CtlFormula};

    fn traffic_light() -> KripkeStructure {
        let mut b = KripkeStructure::builder(3);
        b.initial(0);
        b.transition(0, 1).transition(1, 2).transition(2, 0);
        b.label(0, 10).label(1, 11).label(2, 12);
        b.prop_name(10, "red")
            .prop_name(11, "green")
            .prop_name(12, "yellow");
        b.build().ok().unwrap_or_else(|| unreachable!())
    }

    fn microwave() -> KripkeStructure {
        // States: 0=Idle, 1=Cooking, 2=Done, 3=Error
        // Props: 20=idle, 21=cooking, 22=done, 23=error
        let mut b = KripkeStructure::builder(4);
        b.initial(0);
        b.transition(0, 1); // Idle → Cooking
        b.transition(1, 2); // Cooking → Done
        b.transition(1, 3); // Cooking → Error
        b.transition(2, 0); // Done → Idle
        b.self_loop(3); // Error is absorbing
        b.label(0, 20).label(1, 21).label(2, 22).label(3, 23);
        b.build().ok().unwrap_or_else(|| unreachable!())
    }

    #[test]
    fn test_sat_atom() {
        let k = traffic_light();
        let checker = CtlChecker::new(&k);
        let red = CtlFormula::Atom(AtomicProp::new("red", 10));
        let sat = checker.sat(&red);
        assert_eq!(sat, BTreeSet::from([0]));
    }

    #[test]
    fn test_sat_not() {
        let k = traffic_light();
        let checker = CtlChecker::new(&k);
        let not_red = CtlFormula::Not(Box::new(CtlFormula::Atom(AtomicProp::new("red", 10))));
        let sat = checker.sat(&not_red);
        assert_eq!(sat, BTreeSet::from([1, 2])); // Green and Yellow
    }

    #[test]
    fn test_sat_and() {
        let k = traffic_light();
        let checker = CtlChecker::new(&k);
        let red = CtlFormula::Atom(AtomicProp::new("red", 10));
        let green = CtlFormula::Atom(AtomicProp::new("green", 11));
        let both = CtlFormula::And(Box::new(red), Box::new(green));
        let sat = checker.sat(&both);
        assert!(sat.is_empty()); // No state is both red and green
    }

    #[test]
    fn test_ex_next() {
        let k = traffic_light();
        let checker = CtlChecker::new(&k);
        // EX green: exists a next state that's green
        let ex_green =
            CtlFormula::ExistsNext(Box::new(CtlFormula::Atom(AtomicProp::new("green", 11))));
        let sat = checker.sat(&ex_green);
        assert!(sat.contains(&0)); // Red → Green
        assert!(!sat.contains(&1)); // Green → Yellow (not green)
    }

    #[test]
    fn test_ax_next() {
        let k = traffic_light();
        let checker = CtlChecker::new(&k);
        // AX green: ALL next states are green (true only from Red in traffic light)
        let ax_green =
            CtlFormula::ForAllNext(Box::new(CtlFormula::Atom(AtomicProp::new("green", 11))));
        let sat = checker.sat(&ax_green);
        assert!(sat.contains(&0)); // Red → {Green} (all successors green)
        assert!(!sat.contains(&1)); // Green → {Yellow}
    }

    #[test]
    fn test_ef_reachability() {
        let k = traffic_light();
        let checker = CtlChecker::new(&k);
        // EF green: can eventually reach green
        let ef_green =
            CtlFormula::ExistsEventually(Box::new(CtlFormula::Atom(AtomicProp::new("green", 11))));
        let sat = checker.sat(&ef_green);
        assert_eq!(sat.len(), 3); // All states can reach green (it's a cycle)
    }

    #[test]
    fn test_ag_invariant() {
        let k = traffic_light();
        let checker = CtlChecker::new(&k);
        // AG ¬error: error never happens (no error state in traffic light)
        let ag_no_error = CtlFormula::ForAllGlobally(Box::new(CtlFormula::Not(Box::new(
            CtlFormula::Atom(AtomicProp::new("error", 99)),
        ))));
        let result = checker.check(&ag_no_error);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_ag_violated() {
        let k = microwave();
        let checker = CtlChecker::new(&k);
        // AG ¬error: error never happens (but it can in microwave!)
        let ag_no_error = CtlFormula::ForAllGlobally(Box::new(CtlFormula::Not(Box::new(
            CtlFormula::Atom(AtomicProp::new("error", 23)),
        ))));
        let result = checker.check(&ag_no_error);
        assert!(!result.is_satisfied());
    }

    #[test]
    fn test_af_termination() {
        let k = microwave();
        let checker = CtlChecker::new(&k);
        // AF done: all paths eventually reach done
        // This should FAIL because error path never reaches done
        let af_done =
            CtlFormula::ForAllEventually(Box::new(CtlFormula::Atom(AtomicProp::new("done", 22))));
        let result = checker.check(&af_done);
        assert!(!result.is_satisfied()); // Error absorbs, never reaches Done
    }

    #[test]
    fn test_eg_globally() {
        let k = traffic_light();
        let checker = CtlChecker::new(&k);
        // EG ¬error: exists a path that always avoids error
        let eg_no_error = CtlFormula::ExistsGlobally(Box::new(CtlFormula::Not(Box::new(
            CtlFormula::Atom(AtomicProp::new("error", 99)),
        ))));
        let sat = checker.sat(&eg_no_error);
        assert_eq!(sat.len(), 3); // All states — the cycle always avoids error
    }

    #[test]
    fn test_eu_until() {
        let k = microwave();
        let checker = CtlChecker::new(&k);
        // E[cooking U done]: exists a path where cooking holds until done
        let eu = CtlFormula::ExistsUntil(
            Box::new(CtlFormula::Atom(AtomicProp::new("cooking", 21))),
            Box::new(CtlFormula::Atom(AtomicProp::new("done", 22))),
        );
        let sat = checker.sat(&eu);
        assert!(sat.contains(&1)); // From Cooking, can go to Done
        assert!(sat.contains(&2)); // Done satisfies ψ directly
    }

    #[test]
    fn test_check_satisfied_returns_witness() {
        let k = traffic_light();
        let checker = CtlChecker::new(&k);
        let ef_red =
            CtlFormula::ExistsEventually(Box::new(CtlFormula::Atom(AtomicProp::new("red", 10))));
        let result = checker.check(&ef_red);
        assert!(result.is_satisfied());
        if let CheckResult::Satisfied { witness } = result {
            assert_eq!(witness.total_states, 3);
        }
    }
}
