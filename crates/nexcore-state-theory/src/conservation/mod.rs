//! # Conservation Laws
//!
//! Universal invariants that state machines must preserve.
//!
//! ## The Three Fundamental Laws
//!
//! | Law | Name | Invariant |
//! |-----|------|-----------|
//! | **L3** | Single State | Machine occupies exactly one state at any time |
//! | **L4** | Non-Terminal Flux | Non-terminal states have ≥1 outgoing transition |
//! | **L11** | Structure Immutability | State count fixed after construction |
//!
//! ## Verification Strategy
//!
//! Laws are verified through:
//! 1. **Compile-time**: Typestate pattern enforces L3/L4 structurally
//! 2. **Runtime**: `ConservationVerifier` checks invariants dynamically
//! 3. **Formal**: Kani harnesses prove bounded correctness
//!
//! ## Connection to ToV Axioms
//!
//! Conservation laws implement Axiom A3 (Conservation):
//! - L3 → A4 (Safety Manifold) interior property
//! - L4 → A4 boundary property
//! - L11 → A1 (Finite Decomposition) stability

use alloc::string::String;
use alloc::vec::Vec;

// ═══════════════════════════════════════════════════════════
// INVARIANT TRAIT
// ═══════════════════════════════════════════════════════════

/// A named invariant that can be checked.
pub trait Invariant {
    /// Invariant identifier.
    fn id(&self) -> &'static str;

    /// Human-readable name.
    fn name(&self) -> &'static str;

    /// The invariant statement.
    fn statement(&self) -> &'static str;
}

// ═══════════════════════════════════════════════════════════
// CONSERVATION LAW TRAIT
// ═══════════════════════════════════════════════════════════

/// Result of verifying a conservation law.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LawVerification {
    /// Law is satisfied.
    Satisfied,
    /// Law is violated with explanation.
    Violated(String),
    /// Law cannot be checked (insufficient information).
    Indeterminate(String),
}

impl LawVerification {
    /// Whether the law is satisfied.
    #[must_use]
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied)
    }

    /// Whether the law is violated.
    #[must_use]
    pub fn is_violated(&self) -> bool {
        matches!(self, Self::Violated(_))
    }
}

/// Trait for conservation laws that can be verified against a state machine.
pub trait ConservationLaw: Invariant {
    /// The input type for verification.
    type Input;

    /// Verify the law against the given input.
    fn verify(&self, input: &Self::Input) -> LawVerification;
}

// ═══════════════════════════════════════════════════════════
// L3: SINGLE STATE
// ═══════════════════════════════════════════════════════════

/// **L3: Single State Law**
///
/// *A state machine occupies exactly one state at any given time.*
///
/// This is enforced structurally by the typestate pattern:
/// - The type `Machine<S>` can only have one `S`
/// - Transitions consume `self`, preventing cloning of state
///
/// ## Verification
///
/// At compile time: The type system ensures single state.
/// At runtime: Check that state count == 1 in any snapshot.
#[derive(Debug, Clone, Copy, Default)]
pub struct L3SingleState;

impl Invariant for L3SingleState {
    fn id(&self) -> &'static str {
        "L3"
    }
    fn name(&self) -> &'static str {
        "Single State"
    }
    fn statement(&self) -> &'static str {
        "Machine occupies exactly one state at any time"
    }
}

/// Input for L3 verification.
#[derive(Debug, Clone)]
pub struct L3Input {
    /// Number of currently active states.
    pub active_state_count: usize,
}

impl ConservationLaw for L3SingleState {
    type Input = L3Input;

    fn verify(&self, input: &Self::Input) -> LawVerification {
        if input.active_state_count == 1 {
            LawVerification::Satisfied
        } else {
            LawVerification::Violated(alloc::format!(
                "Expected 1 active state, found {}",
                input.active_state_count
            ))
        }
    }
}

// ═══════════════════════════════════════════════════════════
// L4: NON-TERMINAL FLUX
// ═══════════════════════════════════════════════════════════

/// **L4: Non-Terminal Flux Law**
///
/// *Every non-terminal state has at least one outgoing transition.*
///
/// This ensures no "dead ends" except for intentional terminal states.
///
/// ## Verification
///
/// At compile time: Methods exist on non-terminal state impls.
/// At runtime: Check outgoing transition count per state.
#[derive(Debug, Clone, Copy, Default)]
pub struct L4NonTerminalFlux;

impl Invariant for L4NonTerminalFlux {
    fn id(&self) -> &'static str {
        "L4"
    }
    fn name(&self) -> &'static str {
        "Non-Terminal Flux"
    }
    fn statement(&self) -> &'static str {
        "Non-terminal states have ≥1 outgoing transition"
    }
}

/// Input for L4 verification.
#[derive(Debug, Clone)]
pub struct L4Input {
    /// List of (state_id, is_terminal, outgoing_count).
    pub states: Vec<(u32, bool, usize)>,
}

impl ConservationLaw for L4NonTerminalFlux {
    type Input = L4Input;

    fn verify(&self, input: &Self::Input) -> LawVerification {
        for (state_id, is_terminal, outgoing) in &input.states {
            if !is_terminal && *outgoing == 0 {
                return LawVerification::Violated(alloc::format!(
                    "Non-terminal state {} has no outgoing transitions",
                    state_id
                ));
            }
        }
        LawVerification::Satisfied
    }
}

// ═══════════════════════════════════════════════════════════
// L11: STRUCTURE IMMUTABILITY
// ═══════════════════════════════════════════════════════════

/// **L11: Structure Immutability Law**
///
/// *The state count is fixed after construction.*
///
/// Once a state machine is defined, you cannot add or remove states.
/// This is enforced by Rust's type system: states are types, not values.
///
/// ## Verification
///
/// At compile time: The generic parameter `S` is fixed.
/// At runtime: Compare expected vs actual state count.
#[derive(Debug, Clone)]
pub struct L11StructureImmutability {
    /// Expected number of states.
    pub expected_state_count: usize,
    /// Expected number of transitions.
    pub expected_transition_count: usize,
}

impl L11StructureImmutability {
    /// Create with expected counts.
    #[must_use]
    pub fn new(states: usize, transitions: usize) -> Self {
        Self {
            expected_state_count: states,
            expected_transition_count: transitions,
        }
    }
}

impl Invariant for L11StructureImmutability {
    fn id(&self) -> &'static str {
        "L11"
    }
    fn name(&self) -> &'static str {
        "Structure Immutability"
    }
    fn statement(&self) -> &'static str {
        "State count and transition count are fixed after construction"
    }
}

/// Input for L11 verification.
#[derive(Debug, Clone)]
pub struct L11Input {
    /// Actual state count.
    pub actual_state_count: usize,
    /// Actual transition count.
    pub actual_transition_count: usize,
}

impl ConservationLaw for L11StructureImmutability {
    type Input = L11Input;

    fn verify(&self, input: &Self::Input) -> LawVerification {
        if input.actual_state_count != self.expected_state_count {
            return LawVerification::Violated(alloc::format!(
                "State count changed: expected {}, found {}",
                self.expected_state_count,
                input.actual_state_count
            ));
        }
        if input.actual_transition_count != self.expected_transition_count {
            return LawVerification::Violated(alloc::format!(
                "Transition count changed: expected {}, found {}",
                self.expected_transition_count,
                input.actual_transition_count
            ));
        }
        LawVerification::Satisfied
    }
}

// ═══════════════════════════════════════════════════════════
// VERIFIER
// ═══════════════════════════════════════════════════════════

/// Aggregated verification results.
#[derive(Debug, Clone)]
pub struct VerificationReport {
    /// Results keyed by law ID.
    pub results: Vec<(String, LawVerification)>,
}

impl VerificationReport {
    /// Create empty report.
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Add a verification result.
    pub fn add(&mut self, law_id: &str, result: LawVerification) {
        self.results.push((law_id.into(), result));
    }

    /// Whether all laws are satisfied.
    #[must_use]
    pub fn all_satisfied(&self) -> bool {
        self.results.iter().all(|(_, r)| r.is_satisfied())
    }

    /// Get violations.
    #[must_use]
    pub fn violations(&self) -> Vec<(&str, &str)> {
        self.results
            .iter()
            .filter_map(|(id, r)| {
                if let LawVerification::Violated(msg) = r {
                    Some((id.as_str(), msg.as_str()))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for VerificationReport {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l3_satisfied() {
        let law = L3SingleState;
        let input = L3Input {
            active_state_count: 1,
        };
        assert!(law.verify(&input).is_satisfied());
    }

    #[test]
    fn test_l3_violated() {
        let law = L3SingleState;
        let input = L3Input {
            active_state_count: 2,
        };
        assert!(law.verify(&input).is_violated());
    }

    #[test]
    fn test_l4_satisfied() {
        let law = L4NonTerminalFlux;
        let input = L4Input {
            states: alloc::vec![
                (0, false, 2), // non-terminal with outgoing
                (1, false, 1), // non-terminal with outgoing
                (2, true, 0),  // terminal with no outgoing (OK)
            ],
        };
        assert!(law.verify(&input).is_satisfied());
    }

    #[test]
    fn test_l4_violated() {
        let law = L4NonTerminalFlux;
        let input = L4Input {
            states: alloc::vec![
                (0, false, 0), // non-terminal with NO outgoing (violation)
            ],
        };
        assert!(law.verify(&input).is_violated());
    }

    #[test]
    fn test_l11_satisfied() {
        let law = L11StructureImmutability::new(4, 3);
        let input = L11Input {
            actual_state_count: 4,
            actual_transition_count: 3,
        };
        assert!(law.verify(&input).is_satisfied());
    }

    #[test]
    fn test_l11_violated_states() {
        let law = L11StructureImmutability::new(4, 3);
        let input = L11Input {
            actual_state_count: 5,
            actual_transition_count: 3,
        };
        assert!(law.verify(&input).is_violated());
    }

    #[test]
    fn test_verification_report() {
        let mut report = VerificationReport::new();
        report.add("L3", LawVerification::Satisfied);
        report.add("L4", LawVerification::Violated("test".into()));

        assert!(!report.all_satisfied());
        assert_eq!(report.violations().len(), 1);
    }
}
