//! # nexcore-reproductive
//!
//! The Reproductive System for CI/CD, deployment, replication with variation,
//! and trimester-gated validation.
//!
//! ## Biological Mapping
//!
//! | Biological Component | Claude Code Analog | Type |
//! |---------------------|-------------------|------|
//! | Gamete | Build artifact (binary, container) | `Gamete` |
//! | Fertilization | Merge + test (PR merge) | `Fertilization` |
//! | Trimester | Validation gate (fmt → clippy → test → build) | `Trimester` |
//! | Embryo | Candidate release | `Embryo` |
//! | Differentiation | Environment-specific config (dev/staging/prod) | `Differentiation` |
//! | Birth | Deployment (Cloud Run, binary release) | `Birth` |
//! | Mutation | Feature branch variation | `Mutation` |
//! | ReproductiveHealth | Pipeline diagnostic | `ReproductiveHealth` |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use serde::{Deserialize, Serialize};

pub mod claude_code;
pub mod genetics;
pub mod grounding;
pub mod mitosis;
pub mod phenotypes;
pub mod validation;

#[cfg(test)]
mod cellular_integration;

// ══════════════════════════════════════════════════════════════════════════════
// Core Types
// ══════════════════════════════════════════════════════════════════════════════

/// Build artifact health status.
///
/// **Biological mapping**: Gamete viability assessment.
///
/// **Type tier**: T2-P (Σ sum + κ comparison)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameteFitness {
    /// Build passed all tests and validations.
    Viable,
    /// Build failed one or more validations.
    NonViable,
    /// Fitness not yet determined.
    Unknown,
}

/// Build artifact (binary, container image, release candidate).
///
/// **Biological mapping**: Gamete — reproductive cell carrying genetic material.
///
/// **Type tier**: T2-C (∃ existence + N quantity + κ comparison)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gamete {
    /// Artifact identifier (e.g., "nexcore-mcp", "guardian-gateway").
    pub artifact_name: String,
    /// Semantic version of the artifact.
    pub version: String,
    /// Viability assessment of the artifact.
    pub fitness: GameteFitness,
    /// Size of the artifact in bytes.
    pub size_bytes: u64,
    /// Number of tests that passed.
    pub tests_passed: usize,
    /// Total number of tests executed.
    pub tests_total: usize,
}

impl Gamete {
    /// Creates a new Gamete with Unknown fitness.
    pub fn new(artifact_name: String, version: String) -> Self {
        Self {
            artifact_name,
            version,
            fitness: GameteFitness::Unknown,
            size_bytes: 0,
            tests_passed: 0,
            tests_total: 0,
        }
    }

    /// Returns true if the gamete is viable (all tests passed).
    pub fn is_viable(&self) -> bool {
        self.fitness == GameteFitness::Viable && self.tests_passed == self.tests_total
    }
}

/// PR merge event with conflict resolution and testing.
///
/// **Biological mapping**: Fertilization — fusion of gametes with genetic recombination.
///
/// **Type tier**: T2-C (→ causality + κ comparison + ∃ existence)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fertilization {
    /// Source branch (feature branch).
    pub branch_source: String,
    /// Target branch (typically main or develop).
    pub branch_target: String,
    /// Number of merge conflicts encountered.
    pub conflicts: usize,
    /// Number of tests that passed after merge.
    pub tests_passed_after_merge: usize,
    /// Whether the merge produced a viable result.
    pub viable: bool,
}

/// CI/CD validation phase gates.
///
/// **Biological mapping**: Trimester — developmental stages with phase-specific validations.
///
/// **Type tier**: T2-P (Σ sum + σ sequence)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trimester {
    /// First trimester: format + lint checks.
    First,
    /// Second trimester: test + coverage validation.
    Second,
    /// Third trimester: build + integration testing.
    Third,
}

/// Candidate release undergoing trimester-gated validation.
///
/// **Biological mapping**: Embryo — developing organism progressing through stages.
///
/// **Type tier**: T2-C (∃ existence + ς state + σ sequence)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Embryo {
    /// Release candidate identifier.
    pub name: String,
    /// Current validation trimester.
    pub trimester: Trimester,
    /// Whether the embryo is viable.
    pub viable: bool,
    /// List of detected defects (validation failures).
    pub defects: Vec<String>,
}

impl Embryo {
    /// Creates a new Embryo starting in the First trimester.
    pub fn new(name: String, trimester: Trimester) -> Self {
        Self {
            name,
            trimester,
            viable: true,
            defects: Vec::new(),
        }
    }

    /// Advances to the next trimester if no defects exist.
    ///
    /// Returns true if advancement occurred, false if defects block progression.
    pub fn advance_trimester(&mut self) -> bool {
        if !self.defects.is_empty() {
            return false;
        }

        match self.trimester {
            Trimester::First => {
                self.trimester = Trimester::Second;
                true
            }
            Trimester::Second => {
                self.trimester = Trimester::Third;
                true
            }
            Trimester::Third => false, // Already at final stage
        }
    }
}

/// Environment-specific configuration adaptation.
///
/// **Biological mapping**: Differentiation — cell specialization for specific functions.
///
/// **Type tier**: T2-C (μ mapping + ∂ boundary + ς state)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Differentiation {
    /// Base configuration template.
    pub base_config: String,
    /// Target environment (dev, staging, production).
    pub environment: String,
    /// List of environment-specific adaptations applied.
    pub adaptations: Vec<String>,
}

/// Feature branch variation from main line.
///
/// **Biological mapping**: Mutation — genetic variation enabling evolution.
///
/// **Type tier**: T2-C (ρ recursion + ∝ irreversibility + ∃ existence)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mutation {
    /// Description of the mutation (feature/change).
    pub description: String,
    /// Whether the mutation is beneficial (improves fitness).
    pub beneficial: bool,
    /// Branch name carrying the mutation.
    pub branch_name: String,
    /// Number of files modified by the mutation.
    pub files_changed: usize,
}

/// Overall health of the reproductive/CI system.
///
/// **Biological mapping**: Reproductive health metrics.
///
/// **Type tier**: T2-C (ς state + κ comparison + ∂ boundary)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReproductiveHealth {
    /// Whether the CI pipeline is currently passing.
    pub pipeline_passing: bool,
    /// Success rate of deployments (0.0 to 1.0).
    pub deployment_success_rate: f64,
    /// Number of trimester gates in the validation pipeline.
    pub trimester_gate_count: usize,
    /// Whether the last deployment was successful.
    pub last_birth_success: bool,
    /// Rate of beneficial mutations (feature branches) (0.0 to 1.0).
    pub mutation_rate: f64,
}

impl ReproductiveHealth {
    /// Returns true if the reproductive system is healthy.
    pub fn is_healthy(&self) -> bool {
        self.pipeline_passing && self.deployment_success_rate > 0.9 && self.last_birth_success
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Tests
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamete_fitness_variants() {
        assert_eq!(GameteFitness::Viable, GameteFitness::Viable);
        assert_ne!(GameteFitness::Viable, GameteFitness::NonViable);
    }

    #[test]
    fn test_gamete_new() {
        let gamete = Gamete::new("nexcore-mcp".to_string(), "1.0.0".to_string());
        assert_eq!(gamete.artifact_name, "nexcore-mcp");
        assert_eq!(gamete.version, "1.0.0");
        assert_eq!(gamete.fitness, GameteFitness::Unknown);
        assert_eq!(gamete.tests_passed, 0);
        assert_eq!(gamete.tests_total, 0);
    }

    #[test]
    fn test_gamete_is_viable_success() {
        let mut gamete = Gamete::new("test".to_string(), "1.0.0".to_string());
        gamete.fitness = GameteFitness::Viable;
        gamete.tests_passed = 100;
        gamete.tests_total = 100;
        assert!(gamete.is_viable());
    }

    #[test]
    fn test_gamete_is_viable_failure_fitness() {
        let mut gamete = Gamete::new("test".to_string(), "1.0.0".to_string());
        gamete.fitness = GameteFitness::NonViable;
        gamete.tests_passed = 100;
        gamete.tests_total = 100;
        assert!(!gamete.is_viable());
    }

    #[test]
    fn test_gamete_is_viable_failure_tests() {
        let mut gamete = Gamete::new("test".to_string(), "1.0.0".to_string());
        gamete.fitness = GameteFitness::Viable;
        gamete.tests_passed = 99;
        gamete.tests_total = 100;
        assert!(!gamete.is_viable());
    }

    #[test]
    fn test_fertilization_viable() {
        let fert = Fertilization {
            branch_source: "feature/new".to_string(),
            branch_target: "main".to_string(),
            conflicts: 0,
            tests_passed_after_merge: 200,
            viable: true,
        };
        assert!(fert.viable);
        assert_eq!(fert.conflicts, 0);
    }

    #[test]
    fn test_trimester_progression() {
        assert_ne!(Trimester::First, Trimester::Second);
        assert_ne!(Trimester::Second, Trimester::Third);
    }

    #[test]
    fn test_embryo_new() {
        let embryo = Embryo::new("v2.0.0-rc1".to_string(), Trimester::First);
        assert_eq!(embryo.name, "v2.0.0-rc1");
        assert_eq!(embryo.trimester, Trimester::First);
        assert!(embryo.viable);
        assert!(embryo.defects.is_empty());
    }

    #[test]
    fn test_embryo_advance_trimester_success() {
        let mut embryo = Embryo::new("rc1".to_string(), Trimester::First);
        assert!(embryo.advance_trimester());
        assert_eq!(embryo.trimester, Trimester::Second);
        assert!(embryo.advance_trimester());
        assert_eq!(embryo.trimester, Trimester::Third);
        assert!(!embryo.advance_trimester()); // Cannot advance past Third
    }

    #[test]
    fn test_embryo_advance_trimester_blocked_by_defects() {
        let mut embryo = Embryo::new("rc1".to_string(), Trimester::First);
        embryo.defects.push("clippy error".to_string());
        assert!(!embryo.advance_trimester());
        assert_eq!(embryo.trimester, Trimester::First);
    }

    #[test]
    fn test_differentiation() {
        let diff = Differentiation {
            base_config: "config.base.yaml".to_string(),
            environment: "production".to_string(),
            adaptations: vec!["enable_ssl".to_string(), "set_replicas=10".to_string()],
        };
        assert_eq!(diff.environment, "production");
        assert_eq!(diff.adaptations.len(), 2);
    }

    #[test]
    fn test_mutation() {
        let mutation = Mutation {
            description: "Add new signal detection algorithm".to_string(),
            beneficial: true,
            branch_name: "feature/improved-signal".to_string(),
            files_changed: 12,
        };
        assert!(mutation.beneficial);
        assert_eq!(mutation.files_changed, 12);
    }

    #[test]
    fn test_reproductive_health_is_healthy_success() {
        let health = ReproductiveHealth {
            pipeline_passing: true,
            deployment_success_rate: 0.95,
            trimester_gate_count: 3,
            last_birth_success: true,
            mutation_rate: 0.15,
        };
        assert!(health.is_healthy());
    }

    #[test]
    fn test_reproductive_health_is_healthy_failure_rate() {
        let health = ReproductiveHealth {
            pipeline_passing: true,
            deployment_success_rate: 0.85,
            trimester_gate_count: 3,
            last_birth_success: true,
            mutation_rate: 0.15,
        };
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_reproductive_health_is_healthy_failure_last_birth() {
        let health = ReproductiveHealth {
            pipeline_passing: true,
            deployment_success_rate: 0.95,
            trimester_gate_count: 3,
            last_birth_success: false,
            mutation_rate: 0.15,
        };
        assert!(!health.is_healthy());
    }
}
