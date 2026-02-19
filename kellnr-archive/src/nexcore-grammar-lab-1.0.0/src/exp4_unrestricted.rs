//! # Experiment 4: Type-0 Unrestricted Grammar — State Machine Interpreter
//!
//! **Chomsky Level:** Type-0 (Unrestricted / Recursively Enumerable)
//! **Primitives:** σ + Σ + ρ + κ + ∃ (Existence)
//! **STEM Traits:** Normalize, Experiment, Prove (+ all from Exp 1-3)
//!
//! ## What Changes From Experiment 3
//!
//! We add ONE primitive: ∃ (Existence).
//! This gives us the power to CREATE new values during execution.
//! The grammar becomes unrestricted: any string → any string.
//! This is a Turing machine.
//!
//! ## Grammar (Unrestricted)
//!
//! ```text
//! α → β   (where α, β are arbitrary strings of terminals/nonterminals)
//! ```
//!
//! No restrictions on production shape. The machine can rewrite anything.
//! This is full computation.
//!
//! ## Practical Output
//!
//! `StateMachineInterpreter` — executes state machines with:
//! - Bayesian belief updates on transition (Normalize)
//! - Action-outcome experiments (Experiment)
//! - Logical proof of invariants (Prove)

#![allow(dead_code)]

use stem_core::{Confidence, Experiment, Normalize};
use stem_math::Prove;

use std::collections::HashMap;

// ============================================================================
// Types (T3: all 5 primitives = full domain specificity)
// ============================================================================

/// A belief about the current system state
/// Tier: T3 (σ + Σ + ρ + κ + ∃ — fully domain-specific)
#[derive(Debug, Clone)]
pub struct Belief {
    /// Name of what we believe about
    pub subject: String,
    /// Prior probability [0.0, 1.0]
    pub probability: f64,
    /// How confident we are in this belief
    pub confidence: Confidence,
    /// Evidence count supporting this belief
    pub evidence_count: u32,
}

impl Belief {
    #[must_use]
    pub fn new(subject: &str, probability: f64) -> Self {
        Self {
            subject: subject.to_string(),
            probability,
            confidence: Confidence::new(0.5),
            evidence_count: 0,
        }
    }
}

/// Evidence that updates a belief
#[derive(Debug, Clone)]
pub struct Evidence {
    pub source: String,
    /// Likelihood ratio: P(evidence | hypothesis) / P(evidence | ¬hypothesis)
    pub likelihood_ratio: f64,
    /// Weight of this evidence
    pub weight: f64,
}

/// An action that can be performed during state transition
#[derive(Debug, Clone)]
pub struct TransitionAction {
    pub name: String,
    /// Side effects to apply
    pub effects: Vec<Effect>,
}

/// Side effect of an action
#[derive(Debug, Clone)]
pub enum Effect {
    /// Update a belief with evidence
    UpdateBelief {
        subject: String,
        likelihood_ratio: f64,
    },
    /// Set a variable
    SetVariable { name: String, value: f64 },
    /// Log a message
    Log(String),
}

/// Outcome of executing an action
#[derive(Debug, Clone)]
pub struct ActionOutcome {
    pub action_name: String,
    pub success: bool,
    pub effects_applied: usize,
    pub beliefs_updated: usize,
}

/// A premise in a logical proof
#[derive(Debug, Clone, PartialEq)]
pub enum Premise {
    /// A belief exceeds a threshold
    BeliefAbove { subject: String, threshold: f64 },
    /// A variable has a specific value
    VariableEquals { name: String, value: f64 },
    /// A state is currently active
    InState(String),
    /// Multiple premises must hold (conjunction)
    All(Vec<Premise>),
}

/// A conclusion derived from premises
#[derive(Debug, Clone)]
pub enum Conclusion {
    /// Transition is permitted
    TransitionAllowed(String),
    /// Transition is denied with reason
    TransitionDenied(String),
    /// A new belief should be formed
    FormBelief { subject: String, probability: f64 },
}

// ============================================================================
// STEM Trait Implementations
// ============================================================================

/// Bayesian belief updater — implements Normalize (T1: STATE ς)
///
/// Prior × Evidence → Posterior
/// This is the quintessential state transformation: the system's belief
/// state changes based on new evidence. This cannot happen in Type-1
/// because creating new probability values requires ∃ (Existence).
pub struct BayesianUpdater;

impl Normalize for BayesianUpdater {
    type Prior = Belief;
    type Evidence = Evidence;
    type Posterior = Belief;

    /// Update belief using Bayes' rule (simplified):
    /// P(H|E) = P(E|H) × P(H) / P(E)
    ///
    /// Using likelihood ratio form:
    /// posterior_odds = likelihood_ratio × prior_odds
    fn normalize(&self, prior: Belief, evidence: &Evidence) -> Belief {
        // Convert probability to odds
        let prior_odds = prior.probability / (1.0 - prior.probability.max(0.001));

        // Apply likelihood ratio
        let posterior_odds = prior_odds * evidence.likelihood_ratio;

        // Convert back to probability
        let posterior_prob = posterior_odds / (1.0 + posterior_odds);

        // Clamp to valid range
        let clamped = posterior_prob.clamp(0.001, 0.999);

        // Update confidence based on evidence weight
        let new_confidence = Confidence::new(
            (prior.confidence.value() * 0.8 + evidence.weight * 0.2).clamp(0.0, 1.0),
        );

        Belief {
            subject: prior.subject.clone(),
            probability: clamped,
            confidence: new_confidence,
            evidence_count: prior.evidence_count + 1,
        }
    }
}

/// Action executor — implements Experiment (T1: SEQUENCE σ)
///
/// Action → Outcome: execute a transition action and observe the result.
/// This is the ∃ primitive at work: new values come into EXISTENCE
/// as a result of executing actions.
pub struct ActionExecutor {
    /// Current variable state
    pub variables: HashMap<String, f64>,
}

impl Experiment for ActionExecutor {
    type Action = TransitionAction;
    type Outcome = ActionOutcome;

    /// Execute an action and observe its outcome
    fn experiment(&mut self, action: TransitionAction) -> ActionOutcome {
        let mut effects_applied = 0;
        let mut beliefs_updated = 0;

        for effect in &action.effects {
            match effect {
                Effect::SetVariable { name, value } => {
                    self.variables.insert(name.clone(), *value);
                    effects_applied += 1;
                }
                Effect::UpdateBelief { .. } => {
                    // Belief updates are handled by the interpreter
                    beliefs_updated += 1;
                }
                Effect::Log(_msg) => {
                    effects_applied += 1;
                }
            }
        }

        ActionOutcome {
            action_name: action.name.clone(),
            success: true,
            effects_applied,
            beliefs_updated,
        }
    }
}

impl Default for ActionExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

/// Logical prover — implements Prove (T1: SEQUENCE σ)
///
/// Premises → Conclusion: derive whether a transition is allowed
/// based on the current system state.
pub struct TransitionProver {
    /// Current beliefs
    beliefs: HashMap<String, f64>,
    /// Current variables
    variables: HashMap<String, f64>,
    /// Current state name
    current_state: String,
}

impl Prove for TransitionProver {
    type Premise = Premise;
    type Conclusion = Conclusion;

    /// Check if premises imply a conclusion
    fn implies(&self, premises: &[Premise]) -> Option<Conclusion> {
        for premise in premises {
            if !self.evaluate_premise(premise) {
                return Some(Conclusion::TransitionDenied(format!(
                    "Premise failed: {premise:?}"
                )));
            }
        }
        Some(Conclusion::TransitionAllowed(
            "All premises satisfied".to_string(),
        ))
    }
}

impl TransitionProver {
    #[must_use]
    pub fn new(current_state: &str) -> Self {
        Self {
            beliefs: HashMap::new(),
            variables: HashMap::new(),
            current_state: current_state.to_string(),
        }
    }

    pub fn set_belief(&mut self, subject: &str, probability: f64) {
        self.beliefs.insert(subject.to_string(), probability);
    }

    pub fn set_variable(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn set_state(&mut self, state: &str) {
        self.current_state = state.to_string();
    }

    fn evaluate_premise(&self, premise: &Premise) -> bool {
        match premise {
            Premise::BeliefAbove { subject, threshold } => {
                self.beliefs.get(subject).copied().unwrap_or(0.0) > *threshold
            }
            Premise::VariableEquals { name, value } => self
                .variables
                .get(name)
                .map(|v| (v - value).abs() < 0.001)
                .unwrap_or(false),
            Premise::InState(state) => self.current_state == *state,
            Premise::All(premises) => premises.iter().all(|p| self.evaluate_premise(p)),
        }
    }
}

// ============================================================================
// The Interpreter (Full Turing Machine)
// ============================================================================

/// Full state machine interpreter — Type-0 (Unrestricted) grammar.
///
/// Combines ALL four Chomsky levels:
/// - Type-3 (σ+Σ): Sequential transition recognition
/// - Type-2 (+ρ): Recursive/nested state handling
/// - Type-1 (+κ): Context-sensitive guard checking
/// - Type-0 (+∃): Computation — belief updates, action execution, proofs
///
/// This is a Turing machine for state machines.
pub struct StateMachineInterpreter {
    updater: BayesianUpdater,
    executor: ActionExecutor,
    prover: TransitionProver,
    /// Current beliefs
    beliefs: HashMap<String, Belief>,
    /// Execution trace
    trace: Vec<String>,
}

impl StateMachineInterpreter {
    #[must_use]
    pub fn new(initial_state: &str) -> Self {
        Self {
            updater: BayesianUpdater,
            executor: ActionExecutor::new(),
            prover: TransitionProver::new(initial_state),
            beliefs: HashMap::new(),
            trace: vec![format!("INIT: {initial_state}")],
        }
    }

    /// Add an initial belief
    pub fn add_belief(&mut self, subject: &str, probability: f64) {
        let belief = Belief::new(subject, probability);
        self.prover.set_belief(subject, probability);
        self.beliefs.insert(subject.to_string(), belief);
    }

    /// Attempt a transition with full Type-0 capabilities
    ///
    /// 1. PROVE: Check premises (κ — context-sensitive)
    /// 2. EXPERIMENT: Execute actions (∃ — create new values)
    /// 3. NORMALIZE: Update beliefs with evidence (ς — state transformation)
    pub fn transition(
        &mut self,
        to_state: &str,
        premises: &[Premise],
        action: TransitionAction,
        evidence: Option<Evidence>,
    ) -> Result<ActionOutcome, String> {
        // Step 1: PROVE — can we make this transition?
        match self.prover.implies(premises) {
            Some(Conclusion::TransitionAllowed(_)) => {
                self.trace
                    .push(format!("PROVE: transition to '{to_state}' allowed"));
            }
            Some(Conclusion::TransitionDenied(reason)) => {
                self.trace.push(format!(
                    "PROVE: transition to '{to_state}' DENIED: {reason}"
                ));
                return Err(reason);
            }
            _ => {
                return Err("Proof inconclusive".to_string());
            }
        }

        // Step 2: EXPERIMENT — execute the transition action
        // Extract metadata before consuming action (ownership transfer to experiment)
        let action_name = action.name.clone();
        let action_effects = action.effects.clone();
        let outcome = self.executor.experiment(action);
        self.trace.push(format!(
            "EXPERIMENT: '{}' → {} effects, {} belief updates",
            action_name, outcome.effects_applied, outcome.beliefs_updated
        ));

        // Apply variable effects to prover
        for effect in &action_effects {
            if let Effect::SetVariable { name, value } = effect {
                self.prover.set_variable(name, *value);
            }
        }

        // Step 3: NORMALIZE — update beliefs with evidence
        if let Some(ev) = evidence {
            // Collect belief updates to apply (avoid borrow conflict with self.beliefs)
            let mut updates: Vec<(String, Belief)> = Vec::new();
            for effect in &action_effects {
                if let Effect::UpdateBelief {
                    subject,
                    likelihood_ratio,
                } = effect
                {
                    if let Some(prior) = self.beliefs.get(subject) {
                        let evidence_for_belief = Evidence {
                            source: ev.source.clone(),
                            likelihood_ratio: *likelihood_ratio,
                            weight: ev.weight,
                        };
                        let prior_prob = prior.probability;
                        let posterior = self.updater.normalize(prior.clone(), &evidence_for_belief);
                        self.trace.push(format!(
                            "NORMALIZE: '{}' belief {:.3} → {:.3}",
                            subject, prior_prob, posterior.probability
                        ));
                        updates.push((subject.clone(), posterior));
                    }
                }
            }
            // Apply all belief updates
            for (subject, posterior) in updates {
                self.prover.set_belief(&subject, posterior.probability);
                self.beliefs.insert(subject, posterior);
            }
        }

        // Update state
        self.prover.set_state(to_state);
        self.trace.push(format!("STATE: → {to_state}"));

        Ok(outcome)
    }

    /// Get the execution trace
    #[must_use]
    pub fn trace(&self) -> &[String] {
        &self.trace
    }

    /// Get current belief probability
    #[must_use]
    pub fn belief_probability(&self, subject: &str) -> Option<f64> {
        self.beliefs.get(subject).map(|b| b.probability)
    }

    /// Get current variable value
    #[must_use]
    pub fn variable(&self, name: &str) -> Option<f64> {
        self.executor.variables.get(name).copied()
    }
}

// ============================================================================
// Tests — Experimental Validation
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bayesian_update() {
        let updater = BayesianUpdater;
        let prior = Belief::new("signal_real", 0.5); // 50/50

        // Strong evidence supporting the hypothesis (LR = 4.0)
        let evidence = Evidence {
            source: "clinical_trial".to_string(),
            likelihood_ratio: 4.0,
            weight: 0.9,
        };

        let posterior = updater.normalize(prior, &evidence);

        // With LR=4 and prior=0.5:
        // prior_odds = 0.5/0.5 = 1.0
        // posterior_odds = 4.0 * 1.0 = 4.0
        // posterior_prob = 4.0/5.0 = 0.8
        assert!((posterior.probability - 0.8).abs() < 0.01);
        assert_eq!(posterior.evidence_count, 1);
    }

    #[test]
    fn test_sequential_evidence_accumulation() {
        let updater = BayesianUpdater;
        let mut belief = Belief::new("drug_causes_event", 0.1); // Low prior

        // Three pieces of confirming evidence
        for _ in 0..3 {
            let evidence = Evidence {
                source: "case_report".to_string(),
                likelihood_ratio: 3.0,
                weight: 0.7,
            };
            belief = updater.normalize(belief, &evidence);
        }

        // After 3 updates with LR=3, belief should be substantially higher
        assert!(belief.probability > 0.7);
        assert_eq!(belief.evidence_count, 3);
    }

    #[test]
    fn test_disconfirming_evidence() {
        let updater = BayesianUpdater;
        let prior = Belief::new("signal_real", 0.8); // Strong prior

        // Disconfirming evidence (LR < 1)
        let evidence = Evidence {
            source: "controlled_study".to_string(),
            likelihood_ratio: 0.2,
            weight: 0.9,
        };

        let prior_prob = prior.probability;
        let posterior = updater.normalize(prior, &evidence);
        assert!(posterior.probability < prior_prob);
    }

    #[test]
    fn test_full_interpreter_workflow() {
        let mut interp = StateMachineInterpreter::new("New");

        // Initialize beliefs
        interp.add_belief("signal_real", 0.3);
        interp.add_belief("causality_established", 0.1);

        // Transition: New → Triaged (requires being in "New" state)
        let result = interp.transition(
            "Triaged",
            &[Premise::InState("New".to_string())],
            TransitionAction {
                name: "triage_case".to_string(),
                effects: vec![
                    Effect::SetVariable {
                        name: "priority".to_string(),
                        value: 2.0,
                    },
                    Effect::UpdateBelief {
                        subject: "signal_real".to_string(),
                        likelihood_ratio: 2.0,
                    },
                ],
            },
            Some(Evidence {
                source: "triage_assessment".to_string(),
                likelihood_ratio: 2.0,
                weight: 0.6,
            }),
        );
        assert!(result.is_ok());

        // Check belief was updated
        let signal_belief = interp.belief_probability("signal_real");
        assert!(signal_belief.is_some());
        assert!(signal_belief.map(|b| b > 0.3).unwrap_or(false)); // Should have increased

        // Check variable was set
        assert_eq!(interp.variable("priority"), Some(2.0));
    }

    #[test]
    fn test_premise_failure_blocks_transition() {
        let mut interp = StateMachineInterpreter::new("New");
        interp.add_belief("signal_real", 0.2);

        // Try transition that requires belief > 0.5 — should fail
        let result = interp.transition(
            "Confirmed",
            &[Premise::BeliefAbove {
                subject: "signal_real".to_string(),
                threshold: 0.5,
            }],
            TransitionAction {
                name: "confirm_signal".to_string(),
                effects: vec![],
            },
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_compound_premises() {
        let mut interp = StateMachineInterpreter::new("Triaged");
        interp.add_belief("signal_real", 0.8);

        // Compound: must be in Triaged AND belief > 0.7
        let result = interp.transition(
            "Assessed",
            &[Premise::All(vec![
                Premise::InState("Triaged".to_string()),
                Premise::BeliefAbove {
                    subject: "signal_real".to_string(),
                    threshold: 0.7,
                },
            ])],
            TransitionAction {
                name: "assess_signal".to_string(),
                effects: vec![Effect::Log("Assessment complete".to_string())],
            },
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_trace_records_all_operations() {
        let mut interp = StateMachineInterpreter::new("Start");
        interp.add_belief("test_belief", 0.5);

        let _ = interp.transition(
            "End",
            &[Premise::InState("Start".to_string())],
            TransitionAction {
                name: "finish".to_string(),
                effects: vec![Effect::UpdateBelief {
                    subject: "test_belief".to_string(),
                    likelihood_ratio: 3.0,
                }],
            },
            Some(Evidence {
                source: "test".to_string(),
                likelihood_ratio: 3.0,
                weight: 0.8,
            }),
        );

        let trace = interp.trace();
        assert!(trace.len() >= 4); // INIT, PROVE, EXPERIMENT, NORMALIZE, STATE
        assert!(trace[0].starts_with("INIT"));
        assert!(trace.iter().any(|t| t.starts_with("PROVE")));
        assert!(trace.iter().any(|t| t.starts_with("EXPERIMENT")));
        assert!(trace.iter().any(|t| t.starts_with("NORMALIZE")));
    }

    // ========================================================================
    // The Grand Unification Test
    // ========================================================================

    #[test]
    fn test_all_four_levels_in_one_workflow() {
        // This test exercises ALL four Chomsky levels:
        //
        // Type-3 (σ+Σ): The sequence of states New → Triaged → Assessed
        // Type-2 (+ρ):  Recursive belief updates (each update feeds the next)
        // Type-1 (+κ):  Context-sensitive premises (InState, BeliefAbove)
        // Type-0 (+∃):  New values created (beliefs updated, variables set)

        let mut interp = StateMachineInterpreter::new("New");
        interp.add_belief("drug_causes_event", 0.1);

        // Transition 1: New → Triaged (σ: sequence)
        let r1 = interp.transition(
            "Triaged",
            &[Premise::InState("New".to_string())], // κ: context check
            TransitionAction {
                name: "initial_triage".to_string(),
                effects: vec![
                    Effect::SetVariable {
                        name: "cases_reviewed".to_string(),
                        value: 1.0,
                    },
                    Effect::UpdateBelief {
                        subject: "drug_causes_event".to_string(),
                        likelihood_ratio: 2.5, // ∃: new value created
                    },
                ],
            },
            Some(Evidence {
                source: "case_report_1".to_string(),
                likelihood_ratio: 2.5,
                weight: 0.6,
            }),
        );
        assert!(r1.is_ok());

        let belief_after_1 = interp.belief_probability("drug_causes_event");
        assert!(belief_after_1.map(|b| b > 0.1).unwrap_or(false));

        // Transition 2: Triaged → Assessed (ρ: builds on previous update)
        let r2 = interp.transition(
            "Assessed",
            &[Premise::All(vec![
                Premise::InState("Triaged".to_string()),
                Premise::VariableEquals {
                    name: "cases_reviewed".to_string(),
                    value: 1.0,
                },
            ])],
            TransitionAction {
                name: "full_assessment".to_string(),
                effects: vec![
                    Effect::SetVariable {
                        name: "cases_reviewed".to_string(),
                        value: 2.0,
                    },
                    Effect::UpdateBelief {
                        subject: "drug_causes_event".to_string(),
                        likelihood_ratio: 3.0, // More evidence
                    },
                ],
            },
            Some(Evidence {
                source: "epidemiological_study".to_string(),
                likelihood_ratio: 3.0,
                weight: 0.8,
            }),
        );
        assert!(r2.is_ok());

        let belief_after_2 = interp.belief_probability("drug_causes_event");
        // After two rounds of confirming evidence, belief should be substantially higher
        assert!(
            belief_after_2
                .map(|b| b > belief_after_1.unwrap_or(0.0))
                .unwrap_or(false)
        );

        // The trace tells the full story
        let trace = interp.trace();
        assert!(trace.len() >= 8); // Multiple operations per transition
    }
}
