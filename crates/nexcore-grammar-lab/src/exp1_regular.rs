//! # Experiment 1: Type-3 Regular Grammar — State Transition Recognizer
//!
//! **Chomsky Level:** Type-3 (Regular)
//! **Primitives:** σ (Sequence) + Σ (Sum)
//! **STEM Traits:** Transit, Classify, Superpose
//!
//! ## What We Learn
//!
//! A finite automaton recognizes valid state transition sequences.
//! No stack. No recursion. Just: "Is this sequence of transitions legal?"
//!
//! ## Grammar (Regular)
//!
//! ```text
//! Path     → Transition Path | ε
//! Transition → StateId '→' StateId
//! ```
//!
//! This is a right-linear grammar — Type-3. It only needs σ (to chain)
//! and Σ (to choose which transition). No nesting possible.
//!
//! ## Practical Output
//!
//! `TransitionRecognizer` — validates that a sequence of state names
//! forms a legal path through a finite state machine.

#![allow(dead_code)]

use stem_core::Classify;
use stem_math::Transit;
use stem_phys::Superpose;

use std::collections::HashSet;

// ============================================================================
// Types (T2-P: grounded in σ + Σ)
// ============================================================================

/// A state identifier — atomic symbol in our grammar
/// Tier: T2-P (Sequence + Sum — it's a named element in an alternation)
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct StateSymbol(pub String);

impl StateSymbol {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

/// A transition rule: from → to
/// Tier: T2-P (grounded in σ Sequence — ordered pair)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransitionRule {
    pub from: StateSymbol,
    pub to: StateSymbol,
}

/// Recognition result — did the path match the grammar?
/// Tier: T2-P (grounded in Σ Sum — accepted | rejected)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recognition {
    /// Path is a valid sentence in the grammar
    Accepted { path_length: usize },
    /// Path violates the grammar at this position
    Rejected { position: usize, reason: String },
}

/// Token category for the classifier
/// Tier: T2-P (grounded in Σ Sum — alternation of token types)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenCategory {
    /// A state name
    State(StateSymbol),
    /// The arrow separator
    Arrow,
    /// End of input
    End,
    /// Invalid token
    Invalid(String),
}

// ============================================================================
// STEM Trait Implementations
// ============================================================================

/// The token classifier — implements Classify (T1: MAPPING μ)
/// Signal → Category: raw string token → TokenCategory
pub struct TokenClassifier;

impl Classify for TokenClassifier {
    type Signal = String;
    type Category = TokenCategory;

    fn classify(&self, signal: &String) -> TokenCategory {
        match signal.trim() {
            "->" | "→" => TokenCategory::Arrow,
            "" => TokenCategory::End,
            s if s.chars().all(|c| c.is_alphanumeric() || c == '_') => {
                TokenCategory::State(StateSymbol::new(s))
            }
            other => TokenCategory::Invalid(other.to_string()),
        }
    }
}

/// The transition relation — implements Transit (T1: SEQUENCE σ)
/// Checks: if A→B is legal and B→C is legal, then A→B→C is a legal path
///
/// This IS the grammar. The transition table defines which productions exist.
pub struct TransitionRelation {
    /// Adjacency set: (from, to) pairs that are legal
    rules: HashSet<(StateSymbol, StateSymbol)>,
    /// Initial states (start symbols of the grammar)
    initial: HashSet<StateSymbol>,
    /// Terminal states (accepting states)
    terminal: HashSet<StateSymbol>,
}

impl Transit for TransitionRelation {
    type Element = StateSymbol;

    /// Does state `a` directly transition to state `b`?
    fn relates(&self, a: &StateSymbol, b: &StateSymbol) -> bool {
        self.rules.contains(&(a.clone(), b.clone()))
    }

    /// Transitivity: if a→b and b→c, then the path a→b→c is valid
    /// This is exactly the closure property of regular grammars!
    fn is_transitive(&self, a: &StateSymbol, b: &StateSymbol, c: &StateSymbol) -> bool {
        self.relates(a, b) && self.relates(b, c)
    }
}

impl TransitionRelation {
    /// Build from a list of (from, to) string pairs
    #[must_use]
    pub fn new(rules: &[(&str, &str)], initial: &[&str], terminal: &[&str]) -> Self {
        Self {
            rules: rules
                .iter()
                .map(|(f, t)| (StateSymbol::new(f), StateSymbol::new(t)))
                .collect(),
            initial: initial.iter().map(|s| StateSymbol::new(s)).collect(),
            terminal: terminal.iter().map(|s| StateSymbol::new(s)).collect(),
        }
    }

    /// Is this an initial state?
    #[must_use]
    pub fn is_initial(&self, s: &StateSymbol) -> bool {
        self.initial.contains(s)
    }

    /// Is this a terminal (accepting) state?
    #[must_use]
    pub fn is_terminal(&self, s: &StateSymbol) -> bool {
        self.terminal.contains(s)
    }
}

/// Path superposition — implements Superpose (T1: SUM Σ)
/// Two valid paths can be combined (alternation in the grammar)
#[derive(Debug, Clone)]
pub struct ValidPath {
    pub states: Vec<StateSymbol>,
}

impl Superpose for ValidPath {
    /// Superpose two paths = merge their state sequences (union of paths)
    /// This models the Σ (Sum/alternation) in the grammar: Path1 | Path2
    fn superpose(&self, other: &Self) -> Self {
        let mut combined = self.states.clone();
        combined.extend(other.states.iter().cloned());
        Self { states: combined }
    }

    /// Superposition is valid if paths are associative under concatenation
    fn superposition_valid(&self, a: &Self, b: &Self) -> bool {
        // Concatenation is always associative — this is a monoid
        let ab_then_self = a.superpose(b).superpose(self);
        let a_then_b_self = a.superpose(&b.superpose(self));
        ab_then_self.states.len() == a_then_b_self.states.len()
    }
}

// ============================================================================
// The Recognizer (Experiment Core)
// ============================================================================

/// Finite automaton recognizer for state transition sequences.
///
/// This is a Type-3 grammar recognizer. It can only process:
/// - Linear sequences (σ)
/// - Choices between transitions (Σ)
///
/// It CANNOT handle:
/// - Nested states (needs ρ — Experiment 2)
/// - Context-dependent transitions (needs κ — Experiment 3)
/// - Computed transitions (needs ∃ — Experiment 4)
pub struct TransitionRecognizer {
    classifier: TokenClassifier,
    relation: TransitionRelation,
}

impl TransitionRecognizer {
    #[must_use]
    pub fn new(relation: TransitionRelation) -> Self {
        Self {
            classifier: TokenClassifier,
            relation,
        }
    }

    /// Tokenize an input string into classified tokens
    /// Grammar: split on whitespace, classify each token
    fn tokenize(&self, input: &str) -> Vec<TokenCategory> {
        // Split on arrows and whitespace, keeping arrows as tokens
        let normalized = input.replace("→", " → ").replace("->", " -> ");
        normalized
            .split_whitespace()
            .map(|tok| self.classifier.classify(&tok.to_string()))
            .collect()
    }

    /// Recognize: is this sequence a valid sentence in our grammar?
    ///
    /// The algorithm IS the grammar:
    /// 1. Tokenize (lexical analysis — regular grammar on characters)
    /// 2. Check sequence structure: State Arrow State Arrow State ...
    /// 3. Check each transition against the relation (Transit::relates)
    /// 4. Check initial/terminal constraints
    pub fn recognize(&self, input: &str) -> Recognition {
        let tokens = self.tokenize(input);

        if tokens.is_empty() {
            return Recognition::Rejected {
                position: 0,
                reason: "Empty input".to_string(),
            };
        }

        // Extract state sequence from token stream
        let mut states: Vec<StateSymbol> = Vec::new();
        let mut expect_state = true;

        for (i, token) in tokens.iter().enumerate() {
            match (token, expect_state) {
                (TokenCategory::State(s), true) => {
                    states.push(s.clone());
                    expect_state = false;
                }
                (TokenCategory::Arrow, false) => {
                    expect_state = true;
                }
                (TokenCategory::Invalid(s), _) => {
                    return Recognition::Rejected {
                        position: i,
                        reason: format!("Invalid token: '{s}'"),
                    };
                }
                (TokenCategory::State(_), false) => {
                    return Recognition::Rejected {
                        position: i,
                        reason: "Expected arrow between states".to_string(),
                    };
                }
                (TokenCategory::Arrow, true) => {
                    return Recognition::Rejected {
                        position: i,
                        reason: "Expected state, got arrow".to_string(),
                    };
                }
                (TokenCategory::End, _) => break,
            }
        }

        if states.len() < 2 {
            return Recognition::Rejected {
                position: 0,
                reason: "Need at least 2 states for a transition".to_string(),
            };
        }

        // Check initial state
        if !self.relation.is_initial(&states[0]) {
            return Recognition::Rejected {
                position: 0,
                reason: format!("'{}' is not an initial state", states[0].0),
            };
        }

        // Check each transition via Transit::relates (σ — sequential checking)
        for (i, window) in states.windows(2).enumerate() {
            if !self.relation.relates(&window[0], &window[1]) {
                return Recognition::Rejected {
                    position: i + 1,
                    reason: format!("No transition from '{}' to '{}'", window[0].0, window[1].0),
                };
            }
        }

        // Check terminal state
        let last = &states[states.len() - 1];
        if !self.relation.is_terminal(last) {
            return Recognition::Rejected {
                position: states.len() - 1,
                reason: format!("'{}' is not a terminal state", last.0),
            };
        }

        Recognition::Accepted {
            path_length: states.len(),
        }
    }

    /// Check transitive reachability: can we get from A to C through B?
    /// This uses Transit::is_transitive directly
    #[must_use]
    pub fn is_reachable_through(&self, from: &str, through: &str, to: &str) -> bool {
        let a = StateSymbol::new(from);
        let b = StateSymbol::new(through);
        let c = StateSymbol::new(to);
        self.relation.is_transitive(&a, &b, &c)
    }

    /// Get all reachable states from a given state (one step)
    #[must_use]
    pub fn reachable_from(&self, state: &str) -> Vec<String> {
        let s = StateSymbol::new(state);
        self.relation
            .rules
            .iter()
            .filter(|(from, _)| from == &s)
            .map(|(_, to)| to.0.clone())
            .collect()
    }
}

// ============================================================================
// Tests — Experimental Validation
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a simple case lifecycle FSM:
    /// New → Triaged → Assessed → Closed
    ///        ↘ Escalated → Assessed
    fn case_lifecycle() -> TransitionRecognizer {
        let relation = TransitionRelation::new(
            &[
                ("New", "Triaged"),
                ("Triaged", "Assessed"),
                ("Triaged", "Escalated"),
                ("Escalated", "Assessed"),
                ("Assessed", "Closed"),
            ],
            &["New"],
            &["Closed"],
        );
        TransitionRecognizer::new(relation)
    }

    #[test]
    fn test_happy_path_recognized() {
        let fsm = case_lifecycle();
        let result = fsm.recognize("New → Triaged → Assessed → Closed");
        assert_eq!(result, Recognition::Accepted { path_length: 4 });
    }

    #[test]
    fn test_alternate_path_via_escalation() {
        let fsm = case_lifecycle();
        let result = fsm.recognize("New → Triaged → Escalated → Assessed → Closed");
        assert_eq!(result, Recognition::Accepted { path_length: 5 });
    }

    #[test]
    fn test_arrow_syntax_both_forms() {
        let fsm = case_lifecycle();
        // ASCII arrow works too
        let result = fsm.recognize("New -> Triaged -> Assessed -> Closed");
        assert_eq!(result, Recognition::Accepted { path_length: 4 });
    }

    #[test]
    fn test_invalid_transition_rejected() {
        let fsm = case_lifecycle();
        // Can't go directly from New to Assessed
        let result = fsm.recognize("New → Assessed → Closed");
        assert!(matches!(result, Recognition::Rejected { position: 1, .. }));
    }

    #[test]
    fn test_non_initial_start_rejected() {
        let fsm = case_lifecycle();
        // Can't start from Triaged
        let result = fsm.recognize("Triaged → Assessed → Closed");
        assert!(matches!(result, Recognition::Rejected { position: 0, .. }));
    }

    #[test]
    fn test_non_terminal_end_rejected() {
        let fsm = case_lifecycle();
        // Must end at Closed
        let result = fsm.recognize("New → Triaged → Assessed");
        assert!(matches!(result, Recognition::Rejected { .. }));
    }

    #[test]
    fn test_transitivity_check() {
        let fsm = case_lifecycle();
        // New→Triaged and Triaged→Assessed: transitive path exists
        assert!(fsm.is_reachable_through("New", "Triaged", "Assessed"));
        // New→Triaged but Triaged does NOT go to Closed directly
        assert!(!fsm.is_reachable_through("New", "Triaged", "Closed"));
    }

    #[test]
    fn test_reachable_states() {
        let fsm = case_lifecycle();
        let mut reachable = fsm.reachable_from("Triaged");
        reachable.sort();
        assert_eq!(reachable, vec!["Assessed", "Escalated"]);
    }

    #[test]
    fn test_superpose_paths() {
        // Two valid paths can be superposed (alternation)
        let path1 = ValidPath {
            states: vec![
                StateSymbol::new("New"),
                StateSymbol::new("Triaged"),
                StateSymbol::new("Assessed"),
            ],
        };
        let path2 = ValidPath {
            states: vec![
                StateSymbol::new("New"),
                StateSymbol::new("Triaged"),
                StateSymbol::new("Escalated"),
            ],
        };
        let combined = path1.superpose(&path2);
        assert_eq!(combined.states.len(), 6); // Union of both paths
    }

    #[test]
    fn test_classify_tokens() {
        let c = TokenClassifier;
        assert_eq!(
            c.classify(&"Active".to_string()),
            TokenCategory::State(StateSymbol::new("Active"))
        );
        assert_eq!(c.classify(&"→".to_string()), TokenCategory::Arrow);
        assert_eq!(c.classify(&"->".to_string()), TokenCategory::Arrow);
        assert!(matches!(
            c.classify(&"not valid!".to_string()),
            TokenCategory::Invalid(_)
        ));
    }

    // ========================================================================
    // Key Experimental Observation
    // ========================================================================

    #[test]
    fn test_limitation_cannot_nest() {
        // This is the TYPE-3 LIMITATION.
        // We CANNOT express: "Active { Processing → Done }"
        // because that requires recursion (ρ) — Experiment 2!
        //
        // A regular grammar has no stack. It can only match flat sequences.
        // The moment you need to "match braces" or "nest states",
        // you've exceeded what σ + Σ can express.
        let fsm = case_lifecycle();
        let result = fsm.recognize("New → { Triaged → Assessed } → Closed");
        // Braces are invalid tokens in a regular grammar
        assert!(matches!(result, Recognition::Rejected { .. }));
    }
}
