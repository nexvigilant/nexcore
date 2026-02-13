//! # Forensic Path Tracer (Phase 9B)
//!
//! Root-cause analysis engine for execution failures.
//!
//! ## Theory
//!
//! When a skill chain fails, we need to understand:
//! 1. **Root causes**: What was the original failure?
//! 2. **Critical path**: Which failure chain had maximum impact?
//! 3. **Recovery plan**: How can we resume execution?
//!
//! ## Components
//!
//! - **`FailureCause`**: A single failure with upstream/downstream links
//! - **`ForensicReport`**: Complete analysis of a failed execution
//! - **`RecoveryStep`**: Action to recover from failure
//! - **`ForensicAnalyzer`**: Main analysis engine

use super::models::{Chain, ExecutionResult, ExecutionStatus};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════════════
// FAILURE CAUSE
// ═══════════════════════════════════════════════════════════════════════════

/// A single failure in the execution chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureCause {
    /// Skill that failed
    pub skill_id: String,
    /// Error message or description
    pub error: String,
    /// Skills that failed before this one (upstream causes)
    pub upstream_causes: Vec<String>,
    /// Skills that were blocked by this failure (downstream impact)
    pub downstream_impact: Vec<String>,
    /// Depth in the causal chain (0 = root cause)
    pub causal_depth: usize,
}

impl FailureCause {
    /// Check if this is a root cause (no upstream failures).
    #[must_use]
    pub fn is_root_cause(&self) -> bool {
        self.upstream_causes.is_empty()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RECOVERY ACTION
// ═══════════════════════════════════════════════════════════════════════════

/// Type of recovery action to take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Retry the failed skill
    Retry,
    /// Skip the failed skill and continue
    Skip,
    /// Rollback to a previous state
    Rollback,
    /// Requires manual intervention
    Manual,
    /// Substitute with an alternative skill
    Substitute,
}

/// A single recovery step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStep {
    /// Target skill for recovery
    pub skill_id: String,
    /// Recommended action
    pub action: RecoveryAction,
    /// Optional rollback command (for Rollback action)
    pub rollback_command: Option<String>,
    /// Alternative skill name (for Substitute action)
    pub substitute_skill: Option<String>,
    /// Reason for this recommendation
    pub rationale: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// FORENSIC REPORT
// ═══════════════════════════════════════════════════════════════════════════

/// Complete forensic analysis of a failed execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicReport {
    /// True root causes (skills with no upstream failures)
    pub root_causes: Vec<FailureCause>,
    /// All failures in causal order
    pub all_failures: Vec<FailureCause>,
    /// Critical path (longest failure chain)
    pub critical_path: Vec<String>,
    /// Length of the critical path
    pub critical_path_length: usize,
    /// Suggested recovery plan
    pub recovery_plan: Vec<RecoveryStep>,
    /// Total number of affected skills
    pub total_impact: usize,
}

impl ForensicReport {
    /// Get a summary of the root causes.
    #[must_use]
    pub fn root_cause_summary(&self) -> String {
        if self.root_causes.is_empty() {
            return "No failures detected".to_string();
        }

        let causes: Vec<_> = self
            .root_causes
            .iter()
            .map(|c| format!("{}: {}", c.skill_id, c.error))
            .collect();

        format!(
            "{} root cause(s): {}",
            self.root_causes.len(),
            causes.join("; ")
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FORENSIC ANALYZER
// ═══════════════════════════════════════════════════════════════════════════

/// Forensic analysis engine.
///
/// Analyzes execution failures to identify root causes, critical paths,
/// and recovery strategies.
#[derive(Debug, Default)]
pub struct ForensicAnalyzer {
    /// Known transient failures that can be retried
    transient_patterns: Vec<String>,
    /// Known fatal failures that require manual intervention
    fatal_patterns: Vec<String>,
}

impl ForensicAnalyzer {
    /// Create a new analyzer with default patterns.
    #[must_use]
    pub fn new() -> Self {
        Self {
            transient_patterns: vec![
                "timeout".to_string(),
                "connection refused".to_string(),
                "rate limit".to_string(),
                "temporary".to_string(),
            ],
            fatal_patterns: vec![
                "permission denied".to_string(),
                "not found".to_string(),
                "invalid".to_string(),
                "authentication".to_string(),
            ],
        }
    }

    /// Analyze a failed execution result.
    #[must_use]
    pub fn analyze(&self, chain: &Chain, result: &ExecutionResult) -> ForensicReport {
        // Build dependency graph
        let deps = self.build_dependency_map(chain);

        // Find all failures
        let all_failures = self.trace_all_failures(chain, result, &deps);

        // Identify root causes
        let root_causes: Vec<_> = all_failures
            .iter()
            .filter(|f| f.is_root_cause())
            .cloned()
            .collect();

        // Find critical path
        let critical_path = self.find_critical_path(&all_failures, &deps);
        let critical_path_length = critical_path.len();

        // Plan recovery
        let recovery_plan = self.plan_recovery(&root_causes, &all_failures);

        // Count total impact
        let total_impact = all_failures
            .iter()
            .map(|f| 1 + f.downstream_impact.len())
            .sum();

        ForensicReport {
            root_causes,
            all_failures,
            critical_path,
            critical_path_length,
            recovery_plan,
            total_impact,
        }
    }

    /// Build a map of skill dependencies from the chain.
    fn build_dependency_map(&self, chain: &Chain) -> HashMap<String, Vec<String>> {
        let mut deps = HashMap::new();
        for node in &chain.nodes {
            deps.insert(node.skill_name.clone(), node.dependencies.clone());
        }
        deps
    }

    /// Trace all failures in the execution.
    fn trace_all_failures(
        &self,
        _chain: &Chain,
        result: &ExecutionResult,
        deps: &HashMap<String, Vec<String>>,
    ) -> Vec<FailureCause> {
        let mut failures = Vec::new();
        let failed_skills: HashSet<_> = result
            .skill_results
            .iter()
            .filter(|r| r.status == ExecutionStatus::Failed)
            .map(|r| r.skill_name.clone())
            .collect();

        for skill_result in &result.skill_results {
            if skill_result.status != ExecutionStatus::Failed {
                continue;
            }

            // Find upstream causes (failed dependencies)
            let skill_deps = deps
                .get(&skill_result.skill_name)
                .cloned()
                .unwrap_or_default();
            let upstream_causes: Vec<_> = skill_deps
                .iter()
                .filter(|d| failed_skills.contains(*d))
                .cloned()
                .collect();

            // Find downstream impact (skills that depend on this one)
            let downstream_impact: Vec<_> = deps
                .iter()
                .filter(|(_, v)| v.contains(&skill_result.skill_name))
                .filter(|(k, _)| failed_skills.contains(*k))
                .map(|(k, _)| k.clone())
                .collect();

            let causal_depth = self.calculate_depth(&skill_result.skill_name, deps, &failed_skills);

            failures.push(FailureCause {
                skill_id: skill_result.skill_name.clone(),
                error: skill_result.error.clone().unwrap_or_default(),
                upstream_causes,
                downstream_impact,
                causal_depth,
            });
        }

        // Sort by causal depth (root causes first)
        failures.sort_by_key(|f| f.causal_depth);
        failures
    }

    /// Calculate the depth of a failure in the causal chain.
    fn calculate_depth(
        &self,
        skill: &str,
        deps: &HashMap<String, Vec<String>>,
        failed: &HashSet<String>,
    ) -> usize {
        let skill_deps = match deps.get(skill) {
            Some(d) => d,
            None => return 0,
        };

        let failed_deps: Vec<_> = skill_deps.iter().filter(|d| failed.contains(*d)).collect();

        if failed_deps.is_empty() {
            0
        } else {
            1 + failed_deps
                .iter()
                .map(|d| self.calculate_depth(d, deps, failed))
                .max()
                .unwrap_or(0)
        }
    }

    /// Find the critical path (longest failure chain).
    fn find_critical_path(
        &self,
        failures: &[FailureCause],
        deps: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        if failures.is_empty() {
            return Vec::new();
        }

        let failed_set: HashSet<_> = failures.iter().map(|f| f.skill_id.clone()).collect();

        // Find the deepest failure
        let deepest = failures.iter().max_by_key(|f| f.causal_depth);

        match deepest {
            Some(end) => self.trace_path_to_root(&end.skill_id, deps, &failed_set),
            None => Vec::new(),
        }
    }

    /// Trace path from a failure back to its root cause.
    fn trace_path_to_root(
        &self,
        skill: &str,
        deps: &HashMap<String, Vec<String>>,
        failed: &HashSet<String>,
    ) -> Vec<String> {
        let mut path = vec![skill.to_string()];

        let skill_deps = match deps.get(skill) {
            Some(d) => d,
            None => return path,
        };

        // Find the first failed dependency
        for dep in skill_deps {
            if failed.contains(dep) {
                let mut upstream_path = self.trace_path_to_root(dep, deps, failed);
                upstream_path.push(skill.to_string());
                if upstream_path.len() > path.len() {
                    path = upstream_path;
                }
            }
        }

        path
    }

    /// Plan recovery steps for the failures.
    #[must_use]
    pub fn plan_recovery(
        &self,
        root_causes: &[FailureCause],
        _all_failures: &[FailureCause],
    ) -> Vec<RecoveryStep> {
        let mut plan = Vec::new();

        for cause in root_causes {
            let action = self.classify_failure(&cause.error);
            let rationale = self.explain_action(action, &cause.error);

            plan.push(RecoveryStep {
                skill_id: cause.skill_id.clone(),
                action,
                rollback_command: None, // Would be populated from skill metadata
                substitute_skill: None,
                rationale,
            });
        }

        plan
    }

    /// Classify a failure error to determine recovery action.
    fn classify_failure(&self, error: &str) -> RecoveryAction {
        let error_lower = error.to_lowercase();

        // Check for transient failures (can retry)
        for pattern in &self.transient_patterns {
            if error_lower.contains(pattern) {
                return RecoveryAction::Retry;
            }
        }

        // Check for fatal failures (need manual intervention)
        for pattern in &self.fatal_patterns {
            if error_lower.contains(pattern) {
                return RecoveryAction::Manual;
            }
        }

        // Default to skip for unknown errors
        RecoveryAction::Skip
    }

    /// Generate explanation for the recovery action.
    fn explain_action(&self, action: RecoveryAction, error: &str) -> String {
        match action {
            RecoveryAction::Retry => {
                format!("Transient error detected ({}), retry recommended", error)
            }
            RecoveryAction::Skip => {
                format!("Non-critical error ({}), can skip and continue", error)
            }
            RecoveryAction::Rollback => "State corruption detected, rollback required".to_string(),
            RecoveryAction::Manual => {
                format!("Fatal error ({}), manual intervention required", error)
            }
            RecoveryAction::Substitute => "Alternative skill available".to_string(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::super::models::{AndonSignal, ChainNode, ChainOperator, SkillResult};
    use super::*;

    fn make_chain() -> Chain {
        Chain {
            nodes: vec![
                ChainNode {
                    skill_name: "fetch-data".to_string(),
                    operator: ChainOperator::Sequential,
                    level: 0,
                    dependencies: vec![],
                },
                ChainNode {
                    skill_name: "transform".to_string(),
                    operator: ChainOperator::Sequential,
                    level: 1,
                    dependencies: vec!["fetch-data".to_string()],
                },
                ChainNode {
                    skill_name: "validate".to_string(),
                    operator: ChainOperator::End,
                    level: 2,
                    dependencies: vec!["transform".to_string()],
                },
            ],
            analysis: None,
            confidence: 0.9,
            preset_name: None,
            safety_manifold: None,
        }
    }

    fn make_result_with_failure(failed_skill: &str, error: &str) -> ExecutionResult {
        let chain = make_chain();
        ExecutionResult {
            chain,
            status: ExecutionStatus::Failed,
            skill_results: vec![
                SkillResult {
                    skill_name: "fetch-data".to_string(),
                    status: if failed_skill == "fetch-data" {
                        ExecutionStatus::Failed
                    } else {
                        ExecutionStatus::Completed
                    },
                    signal: AndonSignal::Green,
                    output: serde_json::Value::Null,
                    artifacts: vec![],
                    error: if failed_skill == "fetch-data" {
                        Some(error.to_string())
                    } else {
                        None
                    },
                    duration_ms: 100,
                },
                SkillResult {
                    skill_name: "transform".to_string(),
                    status: if failed_skill == "fetch-data" || failed_skill == "transform" {
                        ExecutionStatus::Failed
                    } else {
                        ExecutionStatus::Completed
                    },
                    signal: AndonSignal::Yellow,
                    output: serde_json::Value::Null,
                    artifacts: vec![],
                    error: if failed_skill == "transform" {
                        Some(error.to_string())
                    } else if failed_skill == "fetch-data" {
                        Some("dependency failed".to_string())
                    } else {
                        None
                    },
                    duration_ms: 50,
                },
            ],
            total_duration_seconds: 0.15,
            halt_reason: None,
            context_accumulated: None,
        }
    }

    #[test]
    fn test_root_cause_identification() {
        let analyzer = ForensicAnalyzer::new();
        let chain = make_chain();
        let result = make_result_with_failure("fetch-data", "connection timeout");

        let report = analyzer.analyze(&chain, &result);

        assert_eq!(report.root_causes.len(), 1);
        assert_eq!(report.root_causes[0].skill_id, "fetch-data");
    }

    #[test]
    fn test_recovery_plan_retry() {
        let analyzer = ForensicAnalyzer::new();
        let chain = make_chain();
        let result = make_result_with_failure("fetch-data", "connection timeout");

        let report = analyzer.analyze(&chain, &result);

        assert!(!report.recovery_plan.is_empty());
        assert_eq!(report.recovery_plan[0].action, RecoveryAction::Retry);
    }

    #[test]
    fn test_recovery_plan_manual() {
        let analyzer = ForensicAnalyzer::new();
        let chain = make_chain();
        let result = make_result_with_failure("fetch-data", "permission denied");

        let report = analyzer.analyze(&chain, &result);

        assert!(!report.recovery_plan.is_empty());
        assert_eq!(report.recovery_plan[0].action, RecoveryAction::Manual);
    }
}
