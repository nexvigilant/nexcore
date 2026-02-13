// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # LTL Bounded Model Checker
//!
//! Verifies LTL formulas against Kripke structures using bounded path exploration.
//!
//! ## Algorithm
//!
//! Bounded model checking explores all paths up to depth `k`:
//! 1. Enumerate all paths of length ≤ k from initial states
//! 2. Check each path against the LTL formula
//! 3. Report violation if any path fails
//!
//! This is sound for safety properties (if a violation exists within depth k,
//! it will be found) but incomplete for liveness (a deeper path may violate).
//!
//! ## Primitive Grounding
//!
//! `LtlBoundedChecker` is T2-C (σ-dominant):
//! - σ Sequence: path enumeration
//! - ∂ Boundary: depth bound
//! - κ Comparison: formula satisfaction
//! - ρ Recursion: recursive formula evaluation

use std::collections::BTreeSet;

use nexcore_state_theory::temporal::LtlFormula;

use crate::kripke::{KripkeStructure, StateId};
use crate::result::{CheckResult, Counterexample, Witness};

/// LTL bounded model checker.
///
/// Explores all paths up to a given depth bound.
///
/// ## Tier: T2-C (σ + ∂ + κ + ρ)
pub struct LtlBoundedChecker<'a> {
    model: &'a KripkeStructure,
    /// Maximum path depth to explore.
    bound: usize,
}

impl<'a> LtlBoundedChecker<'a> {
    /// Create a bounded checker with the given depth limit.
    #[must_use]
    pub fn new(model: &'a KripkeStructure, bound: usize) -> Self {
        Self { model, bound }
    }

    /// Check an LTL formula against all paths from initial states.
    ///
    /// Returns `Satisfied` if no violation found within the bound,
    /// or `Violated` with a counterexample path.
    #[must_use]
    pub fn check(&self, formula: &LtlFormula) -> CheckResult {
        for &init in &self.model.initial_states {
            let path = vec![init];
            if let Some(cex) = self.explore_path(path, formula, self.bound) {
                return CheckResult::Violated {
                    counterexample: cex,
                };
            }
        }

        CheckResult::Satisfied {
            witness: Witness {
                satisfied_states: self.model.state_count(),
                total_states: self.model.state_count(),
            },
        }
    }

    /// Recursively explore paths up to the bound, checking the formula.
    ///
    /// The formula is evaluated at position 0 on the complete (maximal-depth) path.
    /// Temporal operators (G, F, U, X) look ahead through the full path from that position.
    fn explore_path(
        &self,
        path: Vec<StateId>,
        formula: &LtlFormula,
        remaining: usize,
    ) -> Option<Counterexample> {
        if remaining == 0 {
            // Maximal depth reached — evaluate formula at position 0 over the full path
            if !self.eval_at_state(formula, path[0], &path, 0) {
                return Some(Counterexample {
                    path,
                    is_loop: false,
                    description: "LTL formula violated on path".to_string(),
                });
            }
            return None;
        }

        // Explore successors
        let current = path[path.len() - 1];
        for &succ in self.model.successors(current) {
            let mut next_path = path.clone();
            next_path.push(succ);
            if let Some(cex) = self.explore_path(next_path, formula, remaining - 1) {
                return Some(cex);
            }
        }

        None
    }

    /// Evaluate an LTL formula at a specific position in a path.
    ///
    /// This is a bounded evaluation — temporal operators look ahead
    /// only within the path boundaries.
    fn eval_at_state(
        &self,
        formula: &LtlFormula,
        _state: StateId,
        path: &[StateId],
        pos: usize,
    ) -> bool {
        match formula {
            LtlFormula::Atom(ap) => self.model.has_prop(path[pos], ap.id),
            LtlFormula::True => true,
            LtlFormula::False => false,
            LtlFormula::Not(inner) => !self.eval_at_state(inner, path[pos], path, pos),
            LtlFormula::And(l, r) => {
                self.eval_at_state(l, path[pos], path, pos)
                    && self.eval_at_state(r, path[pos], path, pos)
            }
            LtlFormula::Or(l, r) => {
                self.eval_at_state(l, path[pos], path, pos)
                    || self.eval_at_state(r, path[pos], path, pos)
            }
            LtlFormula::Implies(l, r) => {
                !self.eval_at_state(l, path[pos], path, pos)
                    || self.eval_at_state(r, path[pos], path, pos)
            }
            LtlFormula::Next(inner) => {
                if pos + 1 < path.len() {
                    self.eval_at_state(inner, path[pos + 1], path, pos + 1)
                } else {
                    true // Beyond bound — optimistic
                }
            }
            LtlFormula::Globally(inner) => {
                // G φ: φ holds at all positions from pos to end of path
                (pos..path.len()).all(|i| self.eval_at_state(inner, path[i], path, i))
            }
            LtlFormula::Eventually(inner) => {
                // F φ: φ holds at some position from pos to end of path
                (pos..path.len()).any(|i| self.eval_at_state(inner, path[i], path, i))
            }
            LtlFormula::Until(phi, psi) => {
                // φ U ψ: ψ holds at some future pos j, and φ holds at all positions [pos, j)
                for j in pos..path.len() {
                    if self.eval_at_state(psi, path[j], path, j) {
                        // Check φ holds at all positions [pos, j)
                        let phi_holds = (pos..j).all(|i| self.eval_at_state(phi, path[i], path, i));
                        if phi_holds {
                            return true;
                        }
                    }
                }
                false
            }
            LtlFormula::Release(phi, psi) => {
                // φ R ψ ≡ ¬(¬φ U ¬ψ): ψ holds until φ releases it (or forever)
                // ψ must hold at every position until (and including) the one where φ holds
                for j in pos..path.len() {
                    if !self.eval_at_state(psi, path[j], path, j) {
                        // ψ stopped holding — φ must have held at some point before
                        return (pos..j).any(|i| self.eval_at_state(phi, path[i], path, i));
                    }
                    if self.eval_at_state(phi, path[j], path, j) {
                        return true; // φ releases while ψ still holds
                    }
                }
                true // ψ held throughout (φ never needed)
            }
            LtlFormula::WeakUntil(phi, psi) => {
                // φ W ψ ≡ (φ U ψ) ∨ (G φ): like Until but φ can hold forever
                let until = self.eval_at_state(
                    &LtlFormula::Until(Box::new(*phi.clone()), Box::new(*psi.clone())),
                    path[pos],
                    path,
                    pos,
                );
                let globally =
                    self.eval_at_state(&LtlFormula::Globally(phi.clone()), path[pos], path, pos);
                until || globally
            }
        }
    }
}

/// Convenience function: check an LTL property with default bound.
#[must_use]
pub fn check_ltl(model: &KripkeStructure, formula: &LtlFormula, bound: usize) -> CheckResult {
    LtlBoundedChecker::new(model, bound).check(formula)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kripke::KripkeStructure;
    use nexcore_state_theory::temporal::{AtomicProp, LtlFormula};

    fn traffic_light() -> KripkeStructure {
        let mut b = KripkeStructure::builder(3);
        b.initial(0);
        b.transition(0, 1).transition(1, 2).transition(2, 0);
        b.label(0, 10).label(1, 11).label(2, 12);
        b.build().ok().unwrap_or_else(|| unreachable!())
    }

    fn microwave() -> KripkeStructure {
        let mut b = KripkeStructure::builder(4);
        b.initial(0);
        b.transition(0, 1).transition(1, 2).transition(1, 3);
        b.transition(2, 0).self_loop(3);
        b.label(0, 20).label(1, 21).label(2, 22).label(3, 23);
        b.build().ok().unwrap_or_else(|| unreachable!())
    }

    #[test]
    fn test_atom_satisfaction() {
        let k = traffic_light();
        // Red holds at initial state
        let red = LtlFormula::Atom(AtomicProp::new("red", 10));
        let result = check_ltl(&k, &red, 5);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_globally_no_error() {
        let k = traffic_light();
        // G ¬error: error never occurs
        let formula =
            LtlFormula::Not(Box::new(LtlFormula::Atom(AtomicProp::new("error", 99)))).globally();
        let result = check_ltl(&k, &formula, 10);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_globally_violated() {
        let k = microwave();
        // G ¬error: should fail because error state is reachable
        let formula =
            LtlFormula::Not(Box::new(LtlFormula::Atom(AtomicProp::new("error", 23)))).globally();
        let result = check_ltl(&k, &formula, 5);
        assert!(result.is_violated());
    }

    #[test]
    fn test_eventually_green() {
        let k = traffic_light();
        // F green: green is eventually reached
        let formula = LtlFormula::Atom(AtomicProp::new("green", 11)).eventually();
        let result = check_ltl(&k, &formula, 5);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_next() {
        let k = traffic_light();
        // X green: next state from Red is Green
        let formula = LtlFormula::Atom(AtomicProp::new("green", 11)).next();
        let result = check_ltl(&k, &formula, 1);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_until() {
        let k = microwave();
        // idle U cooking: idle holds until cooking starts
        let formula = LtlFormula::Until(
            Box::new(LtlFormula::Atom(AtomicProp::new("idle", 20))),
            Box::new(LtlFormula::Atom(AtomicProp::new("cooking", 21))),
        );
        let result = check_ltl(&k, &formula, 5);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_implies() {
        let k = traffic_light();
        // red → X green: if red then next is green
        let formula = LtlFormula::Implies(
            Box::new(LtlFormula::Atom(AtomicProp::new("red", 10))),
            Box::new(LtlFormula::Atom(AtomicProp::new("green", 11)).next()),
        );
        let result = check_ltl(&k, &formula, 3);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_counterexample_path() {
        let k = microwave();
        let formula =
            LtlFormula::Not(Box::new(LtlFormula::Atom(AtomicProp::new("error", 23)))).globally();
        let result = check_ltl(&k, &formula, 5);
        assert!(result.is_violated());
        let cex = result.counterexample();
        assert!(cex.is_some());
        if let Some(c) = cex {
            assert!(!c.is_empty());
            // Path should include error state (23)
            assert!(c.path.contains(&3));
        }
    }
}
