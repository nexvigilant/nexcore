//! # Gestation — Trimester-Gated Validation Pipeline
//!
//! Executes the full development lifecycle from conception (initial commit)
//! through birth (deployment), enforcing quality gates at each trimester.
//!
//! ## Biological Mapping
//!
//! | Phase | Biological | Claude Code |
//! |-------|-----------|-------------|
//! | Conception | Sperm meets egg | Feature branch created from main |
//! | Trimester 1 | Organ formation | Format + lint (structural correctness) |
//! | Trimester 2 | Growth & refinement | Tests + coverage (functional correctness) |
//! | Trimester 3 | Maturation | Build + integration (deployment readiness) |
//! | Birth | Delivery | Deployment to target environment |
//! | Postpartum | Neonatal care | Post-deployment monitoring |

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Trimester;

/// A single validation check within a trimester gate.
///
/// **T1 grounding**: κ(Comparison) + →(Causality)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateCheck {
    /// Check name (e.g., "cargo fmt --check", "cargo clippy").
    pub name: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Duration of the check in milliseconds.
    pub duration_ms: u64,
    /// Error output if the check failed.
    pub error_output: Option<String>,
}

/// Result of executing a trimester gate.
///
/// **T1 grounding**: ∂(Boundary) + κ(Comparison) + ×(Product)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateResult {
    /// Which trimester this gate belongs to.
    pub trimester: Trimester,
    /// Individual check results.
    pub checks: Vec<GateCheck>,
    /// Whether all checks passed (gate is open).
    pub passed: bool,
    /// When the gate was executed.
    pub executed_at: DateTime<Utc>,
}

impl GateResult {
    /// Creates a gate result from a list of checks.
    pub fn from_checks(trimester: Trimester, checks: Vec<GateCheck>) -> Self {
        let passed = checks.iter().all(|c| c.passed);
        Self {
            trimester,
            checks,
            passed,
            executed_at: Utc::now(),
        }
    }

    /// Total duration of all checks in this gate.
    pub fn total_duration_ms(&self) -> u64 {
        self.checks.iter().map(|c| c.duration_ms).sum()
    }

    /// Number of failing checks.
    pub fn failure_count(&self) -> usize {
        self.checks.iter().filter(|c| !c.passed).count()
    }
}

/// Overall phase of the gestation process.
///
/// **T1 grounding**: Σ(Sum) + σ(Sequence)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GestationPhase {
    /// Feature branch just created.
    Conception,
    /// Executing first trimester gates (format + lint).
    FirstTrimester,
    /// Executing second trimester gates (test + coverage).
    SecondTrimester,
    /// Executing third trimester gates (build + integration).
    ThirdTrimester,
    /// All gates passed, deploying.
    Birth,
    /// Post-deployment monitoring.
    Postpartum,
    /// Gestation failed at a gate.
    Miscarriage { trimester: Trimester },
}

/// A full gestation lifecycle for a release candidate.
///
/// **T1 grounding**: ∃(Existence) + ς(State) + σ(Sequence) + ∂(Boundary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gestation {
    /// Release candidate name.
    pub candidate: String,
    /// Current phase.
    pub phase: GestationPhase,
    /// Gate results accumulated so far.
    pub gate_results: Vec<GateResult>,
    /// When gestation started.
    pub conceived_at: DateTime<Utc>,
    /// When gestation ended (birth or miscarriage).
    pub completed_at: Option<DateTime<Utc>>,
}

impl Gestation {
    /// Creates a new gestation for a release candidate.
    pub fn conceive(candidate: String) -> Self {
        Self {
            candidate,
            phase: GestationPhase::Conception,
            gate_results: Vec::new(),
            conceived_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Submits a gate result and advances the phase accordingly.
    ///
    /// If the gate passes, advances to the next trimester.
    /// If it fails, transitions to Miscarriage.
    pub fn submit_gate(&mut self, result: GateResult) {
        let trimester = result.trimester;
        let passed = result.passed;
        self.gate_results.push(result);

        if !passed {
            self.phase = GestationPhase::Miscarriage { trimester };
            self.completed_at = Some(Utc::now());
            return;
        }

        self.phase = match trimester {
            Trimester::First => GestationPhase::SecondTrimester,
            Trimester::Second => GestationPhase::ThirdTrimester,
            Trimester::Third => GestationPhase::Birth,
        };
    }

    /// Marks birth as complete — deployment succeeded.
    pub fn deliver(&mut self) {
        if self.phase == GestationPhase::Birth {
            self.phase = GestationPhase::Postpartum;
            self.completed_at = Some(Utc::now());
        }
    }

    /// Returns true if gestation completed successfully (postpartum).
    pub fn is_born(&self) -> bool {
        self.phase == GestationPhase::Postpartum
    }

    /// Returns true if gestation failed.
    pub fn is_miscarriage(&self) -> bool {
        matches!(self.phase, GestationPhase::Miscarriage { .. })
    }

    /// Total validation time across all gates.
    pub fn total_validation_ms(&self) -> u64 {
        self.gate_results
            .iter()
            .map(|g| g.total_duration_ms())
            .sum()
    }

    /// Returns all errors from failed gates.
    pub fn all_errors(&self) -> Vec<String> {
        self.gate_results
            .iter()
            .flat_map(|g| &g.checks)
            .filter(|c| !c.passed)
            .filter_map(|c| c.error_output.clone())
            .collect()
    }
}

/// Standard trimester gate definitions for NexCore crates.
pub struct StandardGates;

impl StandardGates {
    /// First trimester: structural correctness.
    pub fn first_trimester_checks() -> Vec<String> {
        vec![
            "cargo fmt --check".into(),
            "cargo clippy -- -D warnings".into(),
        ]
    }

    /// Second trimester: functional correctness.
    pub fn second_trimester_checks() -> Vec<String> {
        vec!["cargo test --lib".into(), "cargo test --doc".into()]
    }

    /// Third trimester: deployment readiness.
    pub fn third_trimester_checks() -> Vec<String> {
        vec!["cargo build --release".into(), "cargo doc --no-deps".into()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passing_check(name: &str) -> GateCheck {
        GateCheck {
            name: name.into(),
            passed: true,
            duration_ms: 500,
            error_output: None,
        }
    }

    fn failing_check(name: &str, error: &str) -> GateCheck {
        GateCheck {
            name: name.into(),
            passed: false,
            duration_ms: 200,
            error_output: Some(error.into()),
        }
    }

    #[test]
    fn test_successful_gestation() {
        let mut g = Gestation::conceive("v2.0.0-rc1".into());
        assert_eq!(g.phase, GestationPhase::Conception);

        // First trimester
        let gate1 = GateResult::from_checks(
            Trimester::First,
            vec![passing_check("fmt"), passing_check("clippy")],
        );
        g.submit_gate(gate1);
        assert_eq!(g.phase, GestationPhase::SecondTrimester);

        // Second trimester
        let gate2 = GateResult::from_checks(
            Trimester::Second,
            vec![passing_check("test"), passing_check("doc-test")],
        );
        g.submit_gate(gate2);
        assert_eq!(g.phase, GestationPhase::ThirdTrimester);

        // Third trimester
        let gate3 = GateResult::from_checks(
            Trimester::Third,
            vec![passing_check("build"), passing_check("doc")],
        );
        g.submit_gate(gate3);
        assert_eq!(g.phase, GestationPhase::Birth);

        // Deliver
        g.deliver();
        assert!(g.is_born());
        assert!(!g.is_miscarriage());
        assert!(g.completed_at.is_some());
    }

    #[test]
    fn test_miscarriage_on_failure() {
        let mut g = Gestation::conceive("v2.0.0-rc2".into());

        let gate1 = GateResult::from_checks(
            Trimester::First,
            vec![
                passing_check("fmt"),
                failing_check("clippy", "error[E0308]: mismatched types"),
            ],
        );
        g.submit_gate(gate1);

        assert!(g.is_miscarriage());
        assert!(!g.is_born());

        let errors = g.all_errors();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("mismatched types"));
    }

    #[test]
    fn test_gate_result_metrics() {
        let gate = GateResult::from_checks(
            Trimester::First,
            vec![passing_check("fmt"), failing_check("clippy", "warning")],
        );
        assert!(!gate.passed);
        assert_eq!(gate.failure_count(), 1);
        assert_eq!(gate.total_duration_ms(), 700); // 500 + 200
    }

    #[test]
    fn test_standard_gates() {
        assert_eq!(StandardGates::first_trimester_checks().len(), 2);
        assert_eq!(StandardGates::second_trimester_checks().len(), 2);
        assert_eq!(StandardGates::third_trimester_checks().len(), 2);
    }

    #[test]
    fn test_total_validation_time() {
        let mut g = Gestation::conceive("rc".into());
        let gate = GateResult::from_checks(
            Trimester::First,
            vec![passing_check("a"), passing_check("b")],
        );
        g.submit_gate(gate);
        assert_eq!(g.total_validation_ms(), 1000); // 500 + 500
    }
}
