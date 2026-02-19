//! # Claude Code Specific Reproductive Types
//!
//! Types mapping the reproductive system specifically to Claude Code CI/CD,
//! deployment, and knowledge transfer patterns.

use serde::{Deserialize, Serialize};

// ══════════════════════════════════════════════════════════════════════════════
// CI Pipeline Types
// ══════════════════════════════════════════════════════════════════════════════

/// CI pipeline stage.
///
/// **Biological mapping**: Sequential developmental checkpoints.
///
/// **Type tier**: T2-P (Σ sum + σ sequence)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CiStage {
    /// Format checking (cargo fmt --check).
    Format,
    /// Lint checking (cargo clippy).
    Lint,
    /// Test execution (cargo test).
    Test,
    /// Documentation build (cargo doc).
    Docs,
    /// Binary build (cargo build).
    Build,
    /// Coverage analysis (cargo llvm-cov).
    Coverage,
}

/// CI pipeline execution.
///
/// **Biological mapping**: Developmental progression through validation stages.
///
/// **Type tier**: T2-C (σ sequence + κ comparison + N quantity)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CiPipeline {
    /// Ordered list of stages in the pipeline.
    pub stages: Vec<CiStage>,
    /// Index of the currently executing stage.
    pub current_stage: usize,
    /// Whether all stages have passed.
    pub all_passing: bool,
    /// Total execution duration in milliseconds.
    pub duration_ms: u64,
}

// ══════════════════════════════════════════════════════════════════════════════
// Deployment Types
// ══════════════════════════════════════════════════════════════════════════════

/// Deployment target environment.
///
/// **Biological mapping**: Environmental niche differentiation.
///
/// **Type tier**: T2-P (Σ sum + λ location)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeployTarget {
    /// Development environment.
    Dev,
    /// Staging environment.
    Staging,
    /// Production environment.
    Production,
}

/// Deployment event (birth of a service instance).
///
/// **Biological mapping**: Birth — emergence of fully-formed organism into environment.
///
/// **Type tier**: T2-C (∃ existence + → causality + λ location)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentBirth {
    /// Service being deployed (e.g., "guardian-gateway").
    pub service_name: String,
    /// Target environment.
    pub target: DeployTarget,
    /// Container image tag or binary version.
    pub image_tag: String,
    /// Number of replicas to deploy.
    pub replicas: u32,
    /// Whether the deployment succeeded.
    pub success: bool,
}

// ══════════════════════════════════════════════════════════════════════════════
// Mutation & Merge Types
// ══════════════════════════════════════════════════════════════════════════════

/// Feature branch mutation.
///
/// **Biological mapping**: Genetic mutation — variation from parent genome.
///
/// **Type tier**: T2-C (ρ recursion + ∝ irreversibility + N quantity)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchMutation {
    /// Branch name carrying the mutation.
    pub branch_name: String,
    /// Parent branch (typically main).
    pub parent_branch: String,
    /// Number of commits in the branch.
    pub commits: usize,
    /// Number of files modified.
    pub files_changed: usize,
    /// Whether the mutation improves fitness.
    pub beneficial: bool,
}

/// PR merge event.
///
/// **Biological mapping**: Fertilization — genetic recombination and viability testing.
///
/// **Type tier**: T2-C (→ causality + κ comparison + ∃ existence)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeEvent {
    /// Source branch being merged.
    pub source_branch: String,
    /// Target branch receiving the merge.
    pub target_branch: String,
    /// Number of conflicts resolved.
    pub conflicts_resolved: usize,
    /// Number of tests passing after merge.
    pub tests_after_merge: usize,
    /// Whether the merge produced a viable result.
    pub viable: bool,
}

// ══════════════════════════════════════════════════════════════════════════════
// Validation Gate Types
// ══════════════════════════════════════════════════════════════════════════════

/// Trimester validation gate.
///
/// **Biological mapping**: Stage-specific developmental checkpoint.
///
/// **Type tier**: T2-C (∂ boundary + κ comparison + N quantity)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrimesterGate {
    /// Trimester name (e.g., "First", "Second", "Third").
    pub trimester: String,
    /// List of checks performed at this gate.
    pub checks: Vec<String>,
    /// Number of checks that passed.
    pub passed: usize,
    /// Number of checks that failed.
    pub failed: usize,
    /// Whether the gate is open (all checks passed).
    pub gate_open: bool,
}

// ══════════════════════════════════════════════════════════════════════════════
// Scaling & Transfer Types
// ══════════════════════════════════════════════════════════════════════════════

/// Scaling event (replica count change).
///
/// **Biological mapping**: Mitosis — cell division and replication.
///
/// **Type tier**: T2-C (N quantity + × product + ∃ existence)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScalingEvent {
    /// Service being scaled.
    pub service: String,
    /// Starting replica count.
    pub from_replicas: u32,
    /// Target replica count.
    pub to_replicas: u32,
    /// Reason for scaling (e.g., "load_increase", "manual").
    pub trigger: String,
}

/// Knowledge transfer between sessions.
///
/// **Biological mapping**: Heredity — transfer of learned traits across generations.
///
/// **Type tier**: T2-C (π persistence + μ mapping + N quantity)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeTransfer {
    /// Source session identifier.
    pub source_session: String,
    /// Target session identifier.
    pub target_session: String,
    /// Number of artifacts transferred.
    pub artifacts_transferred: usize,
    /// Number of skills transferred.
    pub skills_transferred: usize,
}

// ══════════════════════════════════════════════════════════════════════════════
// System Health Types
// ══════════════════════════════════════════════════════════════════════════════

/// Overall reproductive system health for Claude Code.
///
/// **Biological mapping**: Reproductive fitness and fecundity.
///
/// **Type tier**: T2-C (ς state + κ comparison + ∂ boundary)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReproductiveSystemHealth {
    /// Whether CI pipeline is currently green.
    pub ci_pipeline_green: bool,
    /// Whether the last deployment succeeded.
    pub last_deploy_success: bool,
    /// Historical deployment success rate (0.0 to 1.0).
    pub deployment_success_rate: f64,
    /// Number of active feature branches.
    pub active_branches: usize,
    /// Whether all trimester gates are passing.
    pub trimester_gates_passing: bool,
}

// ══════════════════════════════════════════════════════════════════════════════
// Tests
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_stage_variants() {
        assert_eq!(CiStage::Format, CiStage::Format);
        assert_ne!(CiStage::Format, CiStage::Lint);
    }

    #[test]
    fn test_ci_pipeline() {
        let pipeline = CiPipeline {
            stages: vec![CiStage::Format, CiStage::Lint, CiStage::Test],
            current_stage: 1,
            all_passing: true,
            duration_ms: 45000,
        };
        assert_eq!(pipeline.stages.len(), 3);
        assert_eq!(pipeline.current_stage, 1);
        assert!(pipeline.all_passing);
    }

    #[test]
    fn test_deploy_target_variants() {
        assert_eq!(DeployTarget::Dev, DeployTarget::Dev);
        assert_ne!(DeployTarget::Dev, DeployTarget::Production);
    }

    #[test]
    fn test_deployment_birth() {
        let birth = DeploymentBirth {
            service_name: "guardian-gateway".to_string(),
            target: DeployTarget::Production,
            image_tag: "v2.0.0".to_string(),
            replicas: 5,
            success: true,
        };
        assert_eq!(birth.service_name, "guardian-gateway");
        assert_eq!(birth.target, DeployTarget::Production);
        assert!(birth.success);
        assert_eq!(birth.replicas, 5);
    }

    #[test]
    fn test_branch_mutation() {
        let mutation = BranchMutation {
            branch_name: "feature/signal-v2".to_string(),
            parent_branch: "main".to_string(),
            commits: 15,
            files_changed: 42,
            beneficial: true,
        };
        assert_eq!(mutation.commits, 15);
        assert_eq!(mutation.files_changed, 42);
        assert!(mutation.beneficial);
    }

    #[test]
    fn test_merge_event_viable() {
        let merge = MergeEvent {
            source_branch: "feature/fix".to_string(),
            target_branch: "main".to_string(),
            conflicts_resolved: 0,
            tests_after_merge: 4400,
            viable: true,
        };
        assert!(merge.viable);
        assert_eq!(merge.conflicts_resolved, 0);
    }

    #[test]
    fn test_merge_event_nonviable() {
        let merge = MergeEvent {
            source_branch: "feature/broken".to_string(),
            target_branch: "main".to_string(),
            conflicts_resolved: 8,
            tests_after_merge: 0,
            viable: false,
        };
        assert!(!merge.viable);
    }

    #[test]
    fn test_trimester_gate_open() {
        let gate = TrimesterGate {
            trimester: "First".to_string(),
            checks: vec!["fmt".to_string(), "clippy".to_string()],
            passed: 2,
            failed: 0,
            gate_open: true,
        };
        assert!(gate.gate_open);
        assert_eq!(gate.passed, 2);
        assert_eq!(gate.failed, 0);
    }

    #[test]
    fn test_trimester_gate_closed() {
        let gate = TrimesterGate {
            trimester: "Second".to_string(),
            checks: vec!["test".to_string(), "coverage".to_string()],
            passed: 1,
            failed: 1,
            gate_open: false,
        };
        assert!(!gate.gate_open);
        assert_eq!(gate.failed, 1);
    }

    #[test]
    fn test_scaling_event() {
        let scale = ScalingEvent {
            service: "nexcore-api".to_string(),
            from_replicas: 3,
            to_replicas: 10,
            trigger: "load_increase".to_string(),
        };
        assert_eq!(scale.from_replicas, 3);
        assert_eq!(scale.to_replicas, 10);
    }

    #[test]
    fn test_knowledge_transfer() {
        let transfer = KnowledgeTransfer {
            source_session: "session_001".to_string(),
            target_session: "session_002".to_string(),
            artifacts_transferred: 42,
            skills_transferred: 8,
        };
        assert_eq!(transfer.artifacts_transferred, 42);
        assert_eq!(transfer.skills_transferred, 8);
    }

    #[test]
    fn test_reproductive_system_health_healthy() {
        let health = ReproductiveSystemHealth {
            ci_pipeline_green: true,
            last_deploy_success: true,
            deployment_success_rate: 0.98,
            active_branches: 12,
            trimester_gates_passing: true,
        };
        assert!(health.ci_pipeline_green);
        assert!(health.last_deploy_success);
        assert!(health.trimester_gates_passing);
    }

    #[test]
    fn test_reproductive_system_health_degraded() {
        let health = ReproductiveSystemHealth {
            ci_pipeline_green: false,
            last_deploy_success: false,
            deployment_success_rate: 0.75,
            active_branches: 25,
            trimester_gates_passing: false,
        };
        assert!(!health.ci_pipeline_green);
        assert!(!health.last_deploy_success);
    }
}
