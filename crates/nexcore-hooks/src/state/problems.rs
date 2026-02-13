//! Problem Registry - Persistent problem tracking
//!
//! Implements Dalio's principle: "Once you identify a problem, don't tolerate it."
//!
//! Problems are tracked with:
//! - Root cause (the REAL problem, not symptoms)
//! - Severity (1-10, distinguishing big from small)
//! - Resolution plan (concrete next action)
//! - Status (open, acknowledged, resolved)
//!
//! # Example
//!
//! ```rust,ignore
//! use nexcore_hooks::state::problems::{ProblemRegistry, Problem, Severity};
//!
//! let mut registry = ProblemRegistry::load();
//! let blocking = registry.blocking_problems();
//! println!("Blocking problems: {}", blocking.len());
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Problem severity levels (1-10)
///
/// Maps to decision thresholds:
/// - 1-3: Trivial/Low - informational only
/// - 4-6: Medium - warns but allows
/// - 7-10: High/Critical - blocks action
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Severity(pub u8);

impl Severity {
    /// Trivial severity (1) - informational only
    pub const TRIVIAL: Self = Self(1);
    /// Low severity (3) - minor concern
    pub const LOW: Self = Self(3);
    /// Medium severity (5) - should address soon
    pub const MEDIUM: Self = Self(5);
    /// High severity (7) - blocks progress
    pub const HIGH: Self = Self(7);
    /// Critical severity (9) - immediate action required
    pub const CRITICAL: Self = Self(9);

    /// Returns true if this severity should block actions (>= 7)
    pub fn should_block(&self) -> bool {
        self.0 >= 7
    }

    /// Returns true if this severity should warn (>= 4)
    pub fn should_warn(&self) -> bool {
        self.0 >= 4
    }

    /// Human-readable severity label
    pub fn label(&self) -> &'static str {
        match self.0 {
            1..=2 => "trivial",
            3..=4 => "low",
            5..=6 => "medium",
            7..=8 => "high",
            9..=10 => "critical",
            _ => "unknown",
        }
    }
}

/// Problem lifecycle status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProblemStatus {
    /// Newly identified, not yet addressed
    Open,
    /// Acknowledged with a resolution plan
    Acknowledged,
    /// Being actively worked on
    InProgress,
    /// Resolved
    Resolved,
    /// Explicitly accepted (with justification)
    AcceptedRisk,
}

/// A tracked problem in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    /// Unique identifier (e.g., "PROB-0001")
    pub id: String,
    /// When first identified (Unix timestamp)
    pub identified_at: i64,
    /// File where problem was found
    pub file_path: String,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Problem category
    pub category: ProblemCategory,
    /// What you're seeing (the symptom)
    pub symptom: String,
    /// The REAL underlying problem (root cause analysis)
    pub root_cause: Option<String>,
    /// Severity score (1-10)
    pub severity: Severity,
    /// Current lifecycle status
    pub status: ProblemStatus,
    /// Linked issue (e.g., "ISSUE-123")
    pub linked_issue: Option<String>,
    /// Concrete next action for resolution
    pub resolution_plan: Option<String>,
    /// Who's responsible for resolution
    pub owner: Option<String>,
    /// Last updated timestamp
    pub updated_at: i64,
}

/// Problem categories for classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProblemCategory {
    /// Untracked work markers (deferred work)
    TechnicalDebt,
    /// Panic paths without invariant documentation
    PanicPath,
    /// Silently discarded errors
    ErrorSwallowing,
    /// Suppressed warnings without justification
    SuppressedWarning,
    /// Disabled or ignored tests
    DisabledTest,
    /// Structural/design issues
    Architecture,
    /// Performance concerns
    Performance,
    /// Security vulnerabilities
    Security,
    /// Missing or inadequate documentation
    Documentation,
    /// Uncategorized
    Other,
}

impl ProblemCategory {
    /// Default severity for this category
    pub fn default_severity(&self) -> Severity {
        match self {
            Self::Security => Severity::CRITICAL,
            Self::PanicPath => Severity::HIGH,
            Self::ErrorSwallowing => Severity::MEDIUM,
            Self::TechnicalDebt => Severity::MEDIUM,
            Self::SuppressedWarning => Severity::LOW,
            Self::DisabledTest => Severity::MEDIUM,
            Self::Architecture => Severity::HIGH,
            Self::Performance => Severity::MEDIUM,
            Self::Documentation => Severity::LOW,
            Self::Other => Severity::MEDIUM,
        }
    }
}

/// Persistent problem registry
///
/// Stores all tracked problems across sessions.
/// Persisted to `~/.claude/problem_registry.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProblemRegistry {
    /// All tracked problems by ID
    pub problems: HashMap<String, Problem>,
    /// Counter for generating sequential IDs
    #[serde(default)]
    pub next_id: u32,
    /// Timestamp of last full codebase scan
    #[serde(default)]
    pub last_scan: i64,
}

impl ProblemRegistry {
    /// Registry file path
    fn registry_path() -> PathBuf {
        dirs::home_dir()
            .map(|h| h.join(".claude").join("problem_registry.json"))
            .unwrap_or_else(|| PathBuf::from(".claude/problem_registry.json"))
    }

    /// Load registry from disk, returning default if not found
    pub fn load() -> Self {
        let path = Self::registry_path();
        if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save registry to disk
    ///
    /// # Errors
    ///
    /// Returns IO error if file cannot be written.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::registry_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    /// Generate next sequential problem ID
    pub fn next_problem_id(&mut self) -> String {
        self.next_id += 1;
        format!("PROB-{:04}", self.next_id)
    }

    /// Add a new problem to the registry
    ///
    /// Returns the assigned problem ID.
    pub fn add_problem(&mut self, mut problem: Problem) -> String {
        let id = self.next_problem_id();
        problem.id = id.clone();
        self.problems.insert(id.clone(), problem);
        id
    }

    /// Get problem by ID
    pub fn get(&self, id: &str) -> Option<&Problem> {
        self.problems.get(id)
    }

    /// Get mutable problem by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Problem> {
        self.problems.get_mut(id)
    }

    /// Find all problems in a specific file
    pub fn find_by_file(&self, file_path: &str) -> Vec<&Problem> {
        self.problems
            .values()
            .filter(|p| p.file_path == file_path)
            .collect()
    }

    /// Find all open problems (not resolved or accepted)
    pub fn open_problems(&self) -> Vec<&Problem> {
        self.problems
            .values()
            .filter(|p| {
                !matches!(
                    p.status,
                    ProblemStatus::Resolved | ProblemStatus::AcceptedRisk
                )
            })
            .collect()
    }

    /// Find problems that should block (high severity + open)
    pub fn blocking_problems(&self) -> Vec<&Problem> {
        self.open_problems()
            .into_iter()
            .filter(|p| p.severity.should_block())
            .collect()
    }

    /// Find problems without resolution plans or linked issues
    pub fn unplanned_problems(&self) -> Vec<&Problem> {
        self.open_problems()
            .into_iter()
            .filter(|p| p.resolution_plan.is_none() && p.linked_issue.is_none())
            .collect()
    }

    /// Count open problems by severity level
    pub fn severity_counts(&self) -> HashMap<&'static str, usize> {
        let mut counts = HashMap::new();
        for problem in self.open_problems() {
            *counts.entry(problem.severity.label()).or_insert(0) += 1;
        }
        counts
    }

    /// Remove resolved problems older than threshold
    pub fn cleanup(&mut self, max_age_days: i64) {
        let now = chrono::Utc::now().timestamp();
        let threshold = now - (max_age_days * 24 * 60 * 60);

        self.problems.retain(|_, p| {
            // Keep if not resolved or if resolved recently
            !matches!(p.status, ProblemStatus::Resolved) || p.updated_at > threshold
        });
    }
}

/// A problem detected during code analysis (pre-registration)
#[derive(Debug, Clone)]
pub struct DetectedProblem {
    /// Problem category
    pub category: ProblemCategory,
    /// Description of what was detected
    pub symptom: String,
    /// Line number in source
    pub line: usize,
    /// Assessed severity
    pub severity: Severity,
    /// Pattern that triggered detection
    pub pattern: String,
    /// Suggested resolution
    pub suggestion: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_levels() {
        // Blocking threshold: >= 7
        assert!(Severity::CRITICAL.should_block());
        assert!(Severity::HIGH.should_block());
        assert!(!Severity::MEDIUM.should_block());
        assert!(!Severity::LOW.should_block());

        // Warning threshold: >= 4
        assert!(Severity::HIGH.should_warn());
        assert!(Severity::MEDIUM.should_warn());
        assert!(Severity(4).should_warn()); // Exactly 4 should warn
        assert!(!Severity::LOW.should_warn()); // 3 is below threshold
        assert!(!Severity::TRIVIAL.should_warn());
    }

    #[test]
    fn test_problem_registry() {
        let mut registry = ProblemRegistry::default();

        let problem = Problem {
            id: String::new(),
            identified_at: 0,
            file_path: "src/main.rs".to_string(),
            line: Some(42),
            category: ProblemCategory::TechnicalDebt,
            symptom: "Untracked work marker without issue reference".to_string(),
            root_cause: None,
            severity: Severity::MEDIUM,
            status: ProblemStatus::Open,
            linked_issue: None,
            resolution_plan: None,
            owner: None,
            updated_at: 0,
        };

        let id = registry.add_problem(problem);
        assert_eq!(id, "PROB-0001");

        let open = registry.open_problems();
        assert_eq!(open.len(), 1);
    }
}
