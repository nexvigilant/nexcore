//! The GROUNDED feedback loop: hypothesis → experiment → outcome → learning → persist.
//!
//! This is the minimum viable primitive from the specification:
//! ```text
//! loop {
//!     let hypothesis = claude.reason(context);
//!     let experiment = claude.design_test(hypothesis);
//!     let outcome = world.execute(experiment);
//!     let learning = claude.integrate(hypothesis, outcome);
//!     context.update(learning);
//! }
//! ```
//!
//! Grounds to: ρ(Recursion) + →(Causality) + π(Persistence) + ς(State)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::GroundedError;
use crate::confidence::Confidence;
use crate::uncertain::Uncertain;

/// A testable claim derived from reasoning about context.
///
/// Grounds to: →(Causality) + κ(Comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    /// Human-readable statement of the hypothesis.
    pub claim: String,
    /// Prior confidence before testing.
    pub prior: Confidence,
    /// What would falsify this hypothesis?
    pub falsification_criteria: String,
    /// When was this hypothesis generated?
    pub generated_at: DateTime<Utc>,
}

/// A concrete test that can produce observable outcomes.
///
/// Grounds to: ∃(Existence) + μ(Mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    /// What test to run.
    pub description: String,
    /// The hypothesis being tested.
    pub hypothesis_claim: String,
    /// Expected outcome if hypothesis is true.
    pub expected_if_true: String,
    /// Expected outcome if hypothesis is false.
    pub expected_if_false: String,
}

/// The result of executing an experiment in the world.
///
/// Grounds to: →(Causality) + ∂(Boundary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    /// What actually happened.
    pub observation: String,
    /// Did the experiment succeed in producing a clear signal?
    pub conclusive: bool,
    /// Raw data or evidence.
    pub evidence: serde_json::Value,
    /// When was this observed?
    pub observed_at: DateTime<Utc>,
}

/// Knowledge gained from comparing hypothesis to outcome.
///
/// Grounds to: μ(Mapping) + π(Persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learning {
    /// What was learned.
    pub insight: String,
    /// Updated confidence (posterior).
    pub posterior: Confidence,
    /// Was the hypothesis supported, refuted, or inconclusive?
    pub verdict: Verdict,
    /// The hypothesis-outcome pair that produced this learning.
    pub hypothesis_claim: String,
    pub observation: String,
    /// When was this learning generated?
    pub learned_at: DateTime<Utc>,
}

/// The result of comparing prediction to reality.
///
/// Grounds to: Σ(Sum) — one of three exclusive outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// Evidence supports the hypothesis.
    Supported,
    /// Evidence contradicts the hypothesis.
    Refuted,
    /// Evidence is insufficient to decide.
    Inconclusive,
}

/// Trait for anything that can serve as reasoning context.
///
/// Grounds to: ς(State)
pub trait Context {
    /// Reason about the current context to produce a hypothesis.
    fn reason(&self) -> Result<Hypothesis, GroundedError>;

    /// Design an experiment to test a hypothesis.
    fn design_test(&self, hypothesis: &Hypothesis) -> Result<Experiment, GroundedError>;

    /// Integrate a hypothesis and outcome into a learning.
    fn integrate(
        &self,
        hypothesis: &Hypothesis,
        outcome: &Outcome,
    ) -> Result<Learning, GroundedError>;

    /// Update context with new learning.
    fn update(&mut self, learning: &Learning) -> Result<(), GroundedError>;
}

/// Trait for anything that can execute experiments.
///
/// Grounds to: →(Causality) — cause produces observable effect.
pub trait World {
    /// Execute an experiment and observe the outcome.
    fn execute(&self, experiment: &Experiment) -> Result<Outcome, GroundedError>;
}

/// Persistent storage for learnings.
///
/// Grounds to: π(Persistence)
pub trait ExperienceStore {
    /// Persist a learning for future retrieval.
    fn persist(&mut self, learning: &Learning) -> Result<(), GroundedError>;

    /// Retrieve all learnings.
    fn all_learnings(&self) -> Result<Vec<Learning>, GroundedError>;

    /// Retrieve learnings related to a claim.
    fn learnings_for(&self, claim: &str) -> Result<Vec<Learning>, GroundedError>;
}

/// The GROUNDED feedback loop.
///
/// This is the core primitive: a cycle of hypothesis → experiment → outcome → learning → persist.
/// Each iteration refines understanding through contact with reality.
///
/// Grounds to: ρ(Recursion) + →(Causality) + π(Persistence)
pub struct GroundedLoop<C: Context, W: World, E: ExperienceStore> {
    context: C,
    world: W,
    store: E,
    iterations: u64,
}

impl<C: Context, W: World, E: ExperienceStore> GroundedLoop<C, W, E> {
    /// Create a new feedback loop.
    pub fn new(context: C, world: W, store: E) -> Self {
        Self {
            context,
            world,
            store,
            iterations: 0,
        }
    }

    /// Execute one iteration of the feedback loop.
    ///
    /// This is the minimum viable primitive:
    /// reason → design_test → execute → integrate → update → persist
    pub fn iterate(&mut self) -> Result<Uncertain<Learning>, GroundedError> {
        // 1. Reason: generate hypothesis from context
        let hypothesis = self.context.reason()?;
        let prior = hypothesis.prior;

        // 2. Design: create experiment to test hypothesis
        let experiment = self.context.design_test(&hypothesis)?;

        // 3. Execute: run experiment in the world
        let outcome = self.world.execute(&experiment)?;

        // 4. Integrate: compare hypothesis to outcome
        let learning = self.context.integrate(&hypothesis, &outcome)?;
        let posterior = learning.posterior;

        // 5. Update: modify context with new learning
        self.context.update(&learning)?;

        // 6. Persist: store learning for future reference
        self.store.persist(&learning)?;

        self.iterations += 1;

        // Return the learning wrapped in its confidence
        Ok(Uncertain::with_provenance(
            learning,
            posterior,
            format!("grounded-loop iteration {}, prior={prior}", self.iterations),
        ))
    }

    /// Run multiple iterations, stopping early if confidence exceeds threshold.
    pub fn iterate_until(
        &mut self,
        max_iterations: u64,
        confidence_threshold: Confidence,
    ) -> Result<Vec<Uncertain<Learning>>, GroundedError> {
        let mut learnings = Vec::new();

        for _ in 0..max_iterations {
            let learning = self.iterate()?;
            let reached_threshold = learning.confidence() >= confidence_threshold;
            learnings.push(learning);

            if reached_threshold {
                break;
            }
        }

        Ok(learnings)
    }

    /// How many iterations have been executed.
    pub fn iterations(&self) -> u64 {
        self.iterations
    }

    /// Access the experience store.
    pub fn store(&self) -> &E {
        &self.store
    }

    /// Access the context.
    pub fn context(&self) -> &C {
        &self.context
    }
}

/// In-memory experience store for testing and prototyping.
#[derive(Debug, Default)]
pub struct MemoryStore {
    learnings: Vec<Learning>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ExperienceStore for MemoryStore {
    fn persist(&mut self, learning: &Learning) -> Result<(), GroundedError> {
        self.learnings.push(learning.clone());
        Ok(())
    }

    fn all_learnings(&self) -> Result<Vec<Learning>, GroundedError> {
        Ok(self.learnings.clone())
    }

    fn learnings_for(&self, claim: &str) -> Result<Vec<Learning>, GroundedError> {
        Ok(self
            .learnings
            .iter()
            .filter(|l| l.hypothesis_claim.contains(claim))
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext {
        hypothesis_count: u32,
    }

    impl Context for TestContext {
        fn reason(&self) -> Result<Hypothesis, GroundedError> {
            Ok(Hypothesis {
                claim: format!("hypothesis-{}", self.hypothesis_count),
                prior: Confidence::new(0.5)
                    .map_err(|e| GroundedError::ExperimentFailed(e.to_string()))?,
                falsification_criteria: "outcome != expected".into(),
                generated_at: Utc::now(),
            })
        }

        fn design_test(&self, h: &Hypothesis) -> Result<Experiment, GroundedError> {
            Ok(Experiment {
                description: format!("test for {}", h.claim),
                hypothesis_claim: h.claim.clone(),
                expected_if_true: "success".into(),
                expected_if_false: "failure".into(),
            })
        }

        fn integrate(&self, h: &Hypothesis, o: &Outcome) -> Result<Learning, GroundedError> {
            let verdict = if o.conclusive {
                Verdict::Supported
            } else {
                Verdict::Inconclusive
            };
            let posterior = if verdict == Verdict::Supported {
                Confidence::new(0.8).map_err(|e| GroundedError::IntegrationFailed(e.to_string()))?
            } else {
                h.prior
            };

            Ok(Learning {
                insight: format!("learned from {}", h.claim),
                posterior,
                verdict,
                hypothesis_claim: h.claim.clone(),
                observation: o.observation.clone(),
                learned_at: Utc::now(),
            })
        }

        fn update(&mut self, _learning: &Learning) -> Result<(), GroundedError> {
            self.hypothesis_count += 1;
            Ok(())
        }
    }

    struct TestWorld;

    impl World for TestWorld {
        fn execute(&self, _experiment: &Experiment) -> Result<Outcome, GroundedError> {
            Ok(Outcome {
                observation: "experiment produced expected result".into(),
                conclusive: true,
                evidence: serde_json::json!({"result": "pass"}),
                observed_at: Utc::now(),
            })
        }
    }

    #[test]
    fn grounded_loop_single_iteration() {
        let ctx = TestContext {
            hypothesis_count: 0,
        };
        let world = TestWorld;
        let store = MemoryStore::new();

        let mut grounded = GroundedLoop::new(ctx, world, store);
        let result = grounded.iterate();
        assert!(result.is_ok());

        let learning = result.ok();
        assert!(learning.is_some());
        assert_eq!(grounded.iterations(), 1);
        assert_eq!(
            grounded.store().all_learnings().unwrap_or_default().len(),
            1
        );
    }

    #[test]
    fn grounded_loop_multiple_iterations() {
        let ctx = TestContext {
            hypothesis_count: 0,
        };
        let world = TestWorld;
        let store = MemoryStore::new();

        let mut grounded = GroundedLoop::new(ctx, world, store);
        let threshold = Confidence::new(0.75).unwrap_or(Confidence::NONE);
        let learnings = grounded.iterate_until(10, threshold);

        assert!(learnings.is_ok());
        let learnings = learnings.unwrap_or_default();
        // Should stop after first iteration since TestWorld always returns conclusive=true
        // and integrate gives posterior=0.8 which >= 0.75 threshold
        assert_eq!(learnings.len(), 1);
        assert_eq!(grounded.iterations(), 1);
    }

    #[test]
    fn memory_store_query() {
        let mut store = MemoryStore::new();
        let learning = Learning {
            insight: "test insight".into(),
            posterior: Confidence::new(0.9).unwrap_or(Confidence::NONE),
            verdict: Verdict::Supported,
            hypothesis_claim: "the sky is blue".into(),
            observation: "looked up, saw blue".into(),
            learned_at: Utc::now(),
        };
        let _ = store.persist(&learning);

        let found = store.learnings_for("sky").unwrap_or_default();
        assert_eq!(found.len(), 1);

        let not_found = store.learnings_for("ocean").unwrap_or_default();
        assert_eq!(not_found.len(), 0);
    }
}
