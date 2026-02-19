// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Model Checking Results
//!
//! Result types for property verification, including witnesses and counterexamples.
//!
//! ## Primitive Grounding
//!
//! - `CheckResult`: T2-P (κ-dominant) — binary satisfaction verdict
//! - `Counterexample`: T2-C (σ + → + ∂) — path through state space to violation
//! - `Witness`: T2-P (N + κ) — quantified satisfaction evidence

use crate::kripke::StateId;

/// Result of a model checking query.
///
/// ## Tier: T2-P (κ-dominant)
#[derive(Debug, Clone)]
pub enum CheckResult {
    /// The property is satisfied in all initial states.
    Satisfied {
        /// Witness information about the satisfaction.
        witness: Witness,
    },
    /// The property is violated. Includes a counterexample.
    Violated {
        /// A counterexample demonstrating the violation.
        counterexample: Counterexample,
    },
}

impl CheckResult {
    /// Whether the property is satisfied.
    #[must_use]
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied { .. })
    }

    /// Whether the property is violated.
    #[must_use]
    pub fn is_violated(&self) -> bool {
        matches!(self, Self::Violated { .. })
    }

    /// Get the counterexample, if violated.
    #[must_use]
    pub fn counterexample(&self) -> Option<&Counterexample> {
        match self {
            Self::Violated { counterexample } => Some(counterexample),
            Self::Satisfied { .. } => None,
        }
    }
}

/// A counterexample trace demonstrating a property violation.
///
/// For safety violations: a finite path to a bad state.
/// For liveness violations: a path ending in a loop (lasso-shaped).
///
/// ## Tier: T2-C (σ + → + ∂)
#[derive(Debug, Clone)]
pub struct Counterexample {
    /// Sequence of state IDs forming the counterexample path.
    pub path: Vec<StateId>,
    /// Whether the path ends in a loop (lasso shape for liveness).
    pub is_loop: bool,
    /// Human-readable description of the violation.
    pub description: String,
}

impl Counterexample {
    /// Length of the counterexample path.
    #[must_use]
    pub fn len(&self) -> usize {
        self.path.len()
    }

    /// Whether the counterexample is empty (shouldn't happen in practice).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    /// The initial state of the counterexample.
    #[must_use]
    pub fn start(&self) -> Option<StateId> {
        self.path.first().copied()
    }

    /// The final state of the counterexample.
    #[must_use]
    pub fn end(&self) -> Option<StateId> {
        self.path.last().copied()
    }
}

/// Witness information for a satisfied property.
///
/// ## Tier: T2-P (N + κ)
#[derive(Debug, Clone)]
pub struct Witness {
    /// Number of states satisfying the property.
    pub satisfied_states: usize,
    /// Total number of states in the model.
    pub total_states: usize,
}

impl Witness {
    /// Fraction of states satisfying the property.
    #[must_use]
    pub fn satisfaction_ratio(&self) -> f64 {
        if self.total_states == 0 {
            0.0
        } else {
            self.satisfied_states as f64 / self.total_states as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_satisfied() {
        let result = CheckResult::Satisfied {
            witness: Witness {
                satisfied_states: 5,
                total_states: 5,
            },
        };
        assert!(result.is_satisfied());
        assert!(!result.is_violated());
        assert!(result.counterexample().is_none());
    }

    #[test]
    fn test_check_result_violated() {
        let result = CheckResult::Violated {
            counterexample: Counterexample {
                path: vec![0, 1, 2],
                is_loop: false,
                description: "safety violation".into(),
            },
        };
        assert!(!result.is_satisfied());
        assert!(result.is_violated());
        let cex = result.counterexample();
        assert!(cex.is_some());
        if let Some(c) = cex {
            assert_eq!(c.len(), 3);
            assert_eq!(c.start(), Some(0));
            assert_eq!(c.end(), Some(2));
        }
    }

    #[test]
    fn test_witness_ratio() {
        let w = Witness {
            satisfied_states: 3,
            total_states: 4,
        };
        let ratio = w.satisfaction_ratio();
        assert!((ratio - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_witness_ratio_empty() {
        let w = Witness {
            satisfied_states: 0,
            total_states: 0,
        };
        assert!((w.satisfaction_ratio()).abs() < f64::EPSILON);
    }
}
