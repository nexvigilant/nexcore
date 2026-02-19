//! # STARK Verification Framework
//!
//! Skill/Atom Runtime Kernel for verification and audit tracking.
//!
//! ## Purpose
//!
//! STARK provides a structured framework for tracking verification operations
//! with full audit trails. It integrates with the Conservation Law validators
//! in nexcore-pv to provide comprehensive system state verification.
//!
//! ## Architecture
//!
//! - **ExecutionContext** - Tracks a verification session with unique ID
//! - **AtomResult** - Records the outcome of a single verification operation
//! - **VerificationAudit** - Comprehensive audit report combining multiple checks
//!
//! ## Example
//!
//! ```
//! use nexcore_vigilance::stark::{ExecutionContext, AtomResult, AtomStatus};
//!
//! // Create a verification context
//! let mut ctx = ExecutionContext::new(Some("project-123".into()), false);
//!
//! // Record verification results
//! let result = AtomResult::success("validation_check", serde_json::json!({"passed": true}));
//! ctx.record(result);
//!
//! assert!(ctx.all_successful());
//! ```

pub mod audit_confidence;
pub mod cognitive_event;
pub mod cognitive_power;

pub use audit_confidence::{
    ConfidenceAssessment, EvidenceCheck, Recommendation, WilsonInterval,
    calculate_weighted_confidence, wilson_confidence_interval,
};
pub use cognitive_event::CognitiveEvent;
pub use cognitive_power::CognitivePowerAnalyzer;

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

/// Status of a verification atom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AtomStatus {
    /// Verification passed
    Success,
    /// Verification failed
    Failure,
    /// Verification skipped (dry run or precondition not met)
    Skipped,
    /// Status could not be determined
    Unknown,
}

impl AtomStatus {
    /// Returns true if this status represents success.
    pub fn is_success(&self) -> bool {
        *self == Self::Success
    }

    /// Returns true if this status represents failure.
    pub fn is_failure(&self) -> bool {
        *self == Self::Failure
    }
}

/// Result of a single verification operation (atom).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomResult {
    /// Unique identifier for this atom
    pub atom_id: String,
    /// Verification status
    pub status: AtomStatus,
    /// Output data (JSON-serializable)
    pub output: Option<serde_json::Value>,
    /// When the verification was performed
    pub timestamp: DateTime<Utc>,
    /// Optional rollback command if this operation needs reversal
    pub rollback_command: Option<String>,
    /// Error message if verification failed
    pub error: Option<String>,
}

impl AtomResult {
    /// Create a successful atom result.
    pub fn success(atom_id: &str, output: serde_json::Value) -> Self {
        Self {
            atom_id: atom_id.to_string(),
            status: AtomStatus::Success,
            output: Some(output),
            timestamp: Utc::now(),
            rollback_command: None,
            error: None,
        }
    }

    /// Create a failed atom result.
    pub fn failure(atom_id: &str, error: String) -> Self {
        Self {
            atom_id: atom_id.to_string(),
            status: AtomStatus::Failure,
            output: None,
            timestamp: Utc::now(),
            rollback_command: None,
            error: Some(error),
        }
    }

    /// Create a skipped atom result.
    pub fn skipped(atom_id: &str, reason: &str) -> Self {
        Self {
            atom_id: atom_id.to_string(),
            status: AtomStatus::Skipped,
            output: Some(serde_json::json!({ "reason": reason })),
            timestamp: Utc::now(),
            rollback_command: None,
            error: None,
        }
    }
}

/// Execution context for a verification session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Unique identifier for this execution
    pub execution_id: String,
    /// Optional project scope
    pub project_id: Option<String>,
    /// Whether this is a dry run (preview only)
    pub dry_run: bool,
    /// Optional workflow identifier
    pub workflow_id: Option<String>,
    /// All atom results recorded in this context
    pub results: Vec<AtomResult>,
    /// When the context was created
    pub created_at: DateTime<Utc>,
}

impl ExecutionContext {
    /// Create a new execution context.
    pub fn new(project_id: Option<String>, dry_run: bool) -> Self {
        Self {
            execution_id: NexId::v4().to_string(),
            project_id,
            dry_run,
            workflow_id: None,
            results: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Create a new context with a specific workflow ID.
    pub fn with_workflow(project_id: Option<String>, workflow_id: String, dry_run: bool) -> Self {
        let mut ctx = Self::new(project_id, dry_run);
        ctx.workflow_id = Some(workflow_id);
        ctx
    }

    /// Record an atom result in this context.
    pub fn record(&mut self, result: AtomResult) {
        self.results.push(result);
    }

    /// Returns true if all recorded atoms succeeded (excludes skipped).
    pub fn all_successful(&self) -> bool {
        self.results.iter().all(|r| r.status.is_success())
    }

    /// Returns true if no atoms failed (skipped is acceptable).
    pub fn no_failures(&self) -> bool {
        !self.results.iter().any(|r| r.status.is_failure())
    }

    /// Returns the number of failed atoms.
    pub fn failure_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status.is_failure())
            .count()
    }

    /// Returns the number of successful atoms.
    pub fn success_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status.is_success())
            .count()
    }

    /// Returns failed atom IDs.
    pub fn failed_atoms(&self) -> Vec<&str> {
        self.results
            .iter()
            .filter(|r| r.status.is_failure())
            .map(|r| r.atom_id.as_str())
            .collect()
    }

    /// Generate an audit summary.
    pub fn audit_summary(&self) -> VerificationAudit {
        VerificationAudit {
            execution_id: self.execution_id.clone(),
            project_id: self.project_id.clone(),
            workflow_id: self.workflow_id.clone(),
            total_atoms: self.results.len(),
            successful: self.success_count(),
            failed: self.failure_count(),
            skipped: self
                .results
                .iter()
                .filter(|r| r.status == AtomStatus::Skipped)
                .count(),
            all_passed: self.no_failures(),
            created_at: self.created_at,
            completed_at: Utc::now(),
            failed_atoms: self.failed_atoms().into_iter().map(String::from).collect(),
        }
    }
}

/// Comprehensive verification audit report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationAudit {
    /// Execution ID
    pub execution_id: String,
    /// Project scope
    pub project_id: Option<String>,
    /// Workflow ID
    pub workflow_id: Option<String>,
    /// Total atoms executed
    pub total_atoms: usize,
    /// Number of successful atoms
    pub successful: usize,
    /// Number of failed atoms
    pub failed: usize,
    /// Number of skipped atoms
    pub skipped: usize,
    /// Whether all atoms passed
    pub all_passed: bool,
    /// When verification started
    pub created_at: DateTime<Utc>,
    /// When verification completed
    pub completed_at: DateTime<Utc>,
    /// List of failed atom IDs
    pub failed_atoms: Vec<String>,
}

impl VerificationAudit {
    /// Returns the pass rate as a percentage.
    pub fn pass_rate(&self) -> f64 {
        if self.total_atoms == 0 {
            100.0
        } else {
            ((self.successful as f64) / (self.total_atoms as f64)) * 100.0
        }
    }

    /// Returns the duration of the verification in seconds.
    pub fn duration_seconds(&self) -> f64 {
        ((self.completed_at - self.created_at).num_milliseconds() as f64) / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_creation() {
        let ctx = ExecutionContext::new(Some("test-project".into()), false);
        assert!(!ctx.execution_id.is_empty());
        assert_eq!(ctx.project_id, Some("test-project".into()));
        assert!(!ctx.dry_run);
    }

    #[test]
    fn test_atom_result_success() {
        let result = AtomResult::success("test_atom", serde_json::json!({"value": 42}));
        assert!(result.status.is_success());
        assert!(!result.status.is_failure());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_atom_result_failure() {
        let result = AtomResult::failure("test_atom", "Something went wrong".into());
        assert!(result.status.is_failure());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_context_recording() {
        let mut ctx = ExecutionContext::new(None, false);

        ctx.record(AtomResult::success("atom_1", serde_json::json!({})));
        ctx.record(AtomResult::success("atom_2", serde_json::json!({})));
        ctx.record(AtomResult::failure("atom_3", "failed".into()));

        assert_eq!(ctx.results.len(), 3);
        assert_eq!(ctx.success_count(), 2);
        assert_eq!(ctx.failure_count(), 1);
        assert!(!ctx.all_successful());
        assert_eq!(ctx.failed_atoms(), vec!["atom_3"]);
    }

    #[test]
    fn test_audit_summary() {
        let mut ctx = ExecutionContext::new(Some("project".into()), false);
        ctx.workflow_id = Some("workflow-123".into());

        ctx.record(AtomResult::success("check_1", serde_json::json!({})));
        ctx.record(AtomResult::success("check_2", serde_json::json!({})));
        ctx.record(AtomResult::skipped("check_3", "not applicable"));

        let audit = ctx.audit_summary();

        assert_eq!(audit.total_atoms, 3);
        assert_eq!(audit.successful, 2);
        assert_eq!(audit.skipped, 1);
        assert_eq!(audit.failed, 0);
        assert!(audit.all_passed);
        assert!((audit.pass_rate() - 66.666).abs() < 1.0);
    }

    #[test]
    fn test_dry_run_context() {
        let ctx = ExecutionContext::new(None, true);
        assert!(ctx.dry_run);
    }
}
