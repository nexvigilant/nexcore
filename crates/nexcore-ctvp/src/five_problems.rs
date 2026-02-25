//! Five Problems Analysis Protocol implementation.
//!
//! The Five Problems Protocol is a systematic method for discovering hidden
//! issues in any software deliverable by searching for exactly one problem
//! in each of five categories.

use crate::error::{CtvpError, CtvpResult};
use crate::types::*;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Category of problem in the Five Problems Protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProblemCategory {
    /// Safety problem (Phase 1 gap) - untested failure modes
    Safety,

    /// Efficacy problem (Phase 2 gap) - capability claim exceeds evidence
    Efficacy,

    /// Confirmation problem (Phase 3 gap) - untested production conditions
    Confirmation,

    /// Structural problem - architecture coupled to implementation details
    Structural,

    /// Functional problem - happy path assumptions hiding edge cases
    Functional,
}

impl ProblemCategory {
    /// Returns the discovery question for this category
    pub fn get_discovery_question(&self) -> &'static str {
        match self {
            Self::Safety => "What failure modes exist that we haven't tested?",
            Self::Efficacy => "Where does capability claim exceed evidence?",
            Self::Confirmation => "What real-world conditions haven't we tested against?",
            Self::Structural => "Where is architecture coupled to implementation details?",
            Self::Functional => "Where is happy path assumption hiding edge cases?",
        }
    }

    /// Returns the discovery method for this category
    pub fn get_discovery_method(&self) -> &'static str {
        match self {
            Self::Safety => "List external dependencies. For each: 'What happens when this fails?'",
            Self::Efficacy => "List stated capabilities. For each: 'What evidence validates this?'",
            Self::Confirmation => {
                "List production conditions. For each: 'Has this been validated at scale?'"
            }
            Self::Structural => {
                "Identify abstraction boundaries. For each: 'What happens when implementation changes?'"
            }
            Self::Functional => {
                "Trace primary code paths. For each: 'What assumptions are implicit?'"
            }
        }
    }

    /// Returns template patterns to check for this category
    pub fn get_check_patterns(&self) -> Vec<&'static str> {
        match self {
            Self::Safety => vec![
                "Untested dependency failures (database, cache, queue, API)",
                "Untested resource exhaustion (memory, CPU, disk, connections)",
                "Missing circuit breakers",
                "Missing retry limits (infinite retry = cascade)",
                "Missing timeouts",
                "Error messages that leak information",
            ],
            Self::Efficacy => vec![
                "Capabilities claimed but not measured",
                "Tests using mocks for critical paths",
                "Tests using fixtures instead of real data",
                "Missing SLO definitions",
                "Missing capability achievement metrics",
                "'Works on my machine' evidence",
            ],
            Self::Confirmation => vec![
                "No shadow deployment plan",
                "No canary deployment plan",
                "No rollback criteria defined",
                "No comparison metrics defined",
                "No statistical significance analysis",
                "Deploying directly to 100% after staging",
            ],
            Self::Structural => vec![
                "Hard-coded configuration",
                "Database-specific SQL without abstraction",
                "Vendor-specific API calls without interface",
                "Tight coupling between modules",
                "Circular dependencies",
                "God objects / classes doing too much",
            ],
            Self::Functional => vec![
                "Assumptions about input format/encoding",
                "Missing null/empty checks",
                "Missing bounds checks",
                "Assuming single-threaded execution",
                "Assuming ordered execution",
                "Assuming idempotency without verification",
            ],
        }
    }

    /// Returns all categories in order
    pub fn get_all() -> [Self; 5] {
        [
            Self::Safety,
            Self::Efficacy,
            Self::Confirmation,
            Self::Structural,
            Self::Functional,
        ]
    }
}

impl std::fmt::Display for ProblemCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Safety => write!(f, "Safety"),
            Self::Efficacy => write!(f, "Efficacy"),
            Self::Confirmation => write!(f, "Confirmation"),
            Self::Structural => write!(f, "Structural"),
            Self::Functional => write!(f, "Functional"),
        }
    }
}

/// A discovered problem in the Five Problems Protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Problem {
    /// Problem number (1-5)
    pub number: u8,

    /// Problem category
    pub category: ProblemCategory,

    /// Specific, concrete description of the problem
    pub description: String,

    /// How this problem was discovered
    pub evidence: String,

    /// Severity of the problem
    pub severity: DiagnosticLevel,

    /// Specific action to fix this problem
    pub remediation: String,

    /// Specific test that would catch this problem
    pub test_required: String,
}

impl Problem {
    /// Creates a new problem
    pub fn new(
        number: u8,
        category: ProblemCategory,
        description: impl Into<String>,
        severity: DiagnosticLevel,
    ) -> Self {
        Self {
            number,
            category,
            description: description.into(),
            evidence: String::new(),
            severity,
            remediation: String::new(),
            test_required: String::new(),
        }
    }

    /// Sets the evidence for this problem
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence = evidence.into();
        self
    }

    /// Sets the remediation for this problem
    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = remediation.into();
        self
    }

    /// Sets the test required for this problem
    pub fn with_test(mut self, test: impl Into<String>) -> Self {
        self.test_required = test.into();
        self
    }
}

/// Complete Five Problems Analysis result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FiveProblemsAnalysis {
    /// Deliverable that was analyzed
    pub deliverable: String,

    /// When the analysis was performed
    pub analysis_timestamp: DateTime,

    /// The five discovered problems (exactly 5)
    pub problems: Vec<Problem>,

    /// Overall severity assessment
    pub overall_severity: DiagnosticLevel,

    /// Prioritized remediation roadmap
    pub remediation_roadmap: Vec<RemediationStep>,
}

/// A step in the remediation roadmap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemediationStep {
    /// Priority (1 = highest)
    pub priority: u8,

    /// Which problem this addresses
    pub problem_number: u8,

    /// Action to take
    pub action: String,

    /// Estimated effort (optional)
    pub effort_estimate: Option<String>,
}

impl FiveProblemsAnalysis {
    /// Creates a new analysis with exactly 5 problems
    pub fn new(deliverable: impl Into<String>, problems: Vec<Problem>) -> CtvpResult<Self> {
        Self::validate_problem_count(&problems)?;
        Self::validate_categories(&problems)?;

        let overall_severity = Self::calculate_overall_severity(&problems);
        let roadmap = Self::build_remediation_roadmap(&problems);

        Ok(Self {
            deliverable: deliverable.into(),
            analysis_timestamp: DateTime::now(),
            problems,
            overall_severity,
            remediation_roadmap: roadmap,
        })
    }

    fn validate_problem_count(problems: &[Problem]) -> CtvpResult<()> {
        if problems.len() != 5 {
            return Err(CtvpError::Analysis(format!(
                "Five Problems Protocol requires exactly 5 problems, got {}",
                problems.len()
            )));
        }
        Ok(())
    }

    fn validate_categories(problems: &[Problem]) -> CtvpResult<()> {
        let mut seen = std::collections::HashSet::new();
        for p in problems {
            if !seen.insert(p.category) {
                return Err(CtvpError::Analysis(format!(
                    "Duplicate category: {}",
                    p.category
                )));
            }
        }
        Ok(())
    }

    fn calculate_overall_severity(problems: &[Problem]) -> DiagnosticLevel {
        problems
            .iter()
            .map(|p| p.severity)
            .max()
            .unwrap_or(DiagnosticLevel::Low)
    }

    fn build_remediation_roadmap(problems: &[Problem]) -> Vec<RemediationStep> {
        let mut roadmap: Vec<RemediationStep> = problems
            .iter()
            .filter(|p| !p.remediation.is_empty())
            .map(|p| RemediationStep {
                priority: 0,
                problem_number: p.number,
                action: p.remediation.clone(),
                effort_estimate: None,
            })
            .collect();

        roadmap.sort_by(|a, b| {
            let s_a = Self::find_severity(problems, a.problem_number);
            let s_b = Self::find_severity(problems, b.problem_number);
            s_b.cmp(&s_a)
        });

        for (i, step) in roadmap.iter_mut().enumerate() {
            step.priority = (i + 1) as u8;
        }
        roadmap
    }

    fn find_severity(problems: &[Problem], num: u8) -> DiagnosticLevel {
        problems
            .iter()
            .find(|p| p.number == num)
            .map(|p| p.severity)
            .unwrap_or(DiagnosticLevel::Low)
    }
}

impl From<Problem> for RemediationStep {
    fn from(p: Problem) -> Self {
        RemediationStep {
            priority: 0,
            problem_number: p.number,
            action: p.remediation,
            effort_estimate: None,
        }
    }
}

impl FiveProblemsAnalysis {
    /// Returns problems sorted by severity (critical first)
    pub fn get_problems_by_severity(&self) -> Vec<&Problem> {
        let mut sorted: Vec<&Problem> = self.problems.iter().collect();
        sorted.sort_by(|a, b| b.severity.cmp(&a.severity));
        sorted
    }

    /// Returns the problem for a specific category
    pub fn get_problem_for_category(&self, category: ProblemCategory) -> Option<&Problem> {
        self.problems.iter().find(|p| p.category == category)
    }

    /// Returns count of problems at each severity level
    pub fn get_severity_counts(&self) -> std::collections::HashMap<DiagnosticLevel, usize> {
        let mut counts = std::collections::HashMap::new();
        for problem in &self.problems {
            *counts.entry(problem.severity).or_insert(0) += 1;
        }
        counts
    }
}

/// Analyzer for discovering Five Problems.
pub struct FiveProblemsAnalyzer {
    /// Configuration
    config: AnalyzerConfig,
}

/// Configuration for the analyzer.
#[derive(Debug, Clone, Default)]
pub struct AnalyzerConfig {
    /// Enable verbose output
    pub verbose: bool,

    /// Include code snippets in evidence
    pub include_snippets: bool,

    /// Maximum files to scan
    pub max_files: Option<usize>,
}

impl FiveProblemsAnalyzer {
    /// Creates a new analyzer
    pub fn new() -> Self {
        Self {
            config: AnalyzerConfig::default(),
        }
    }

    /// Creates an analyzer with configuration
    pub fn with_config(config: AnalyzerConfig) -> Self {
        Self { config }
    }

    /// Discovers five problems from a deliverable path
    pub fn analyze(&self, path: &Path) -> CtvpResult<FiveProblemsAnalysis> {
        let deliverable = path.display().to_string();

        let mut problems = Vec::new();

        // Analyze each category
        for (i, category) in ProblemCategory::get_all().into_iter().enumerate() {
            let problem = self.discover_problem(category, path, (i + 1) as u8)?;
            problems.push(problem);
        }

        FiveProblemsAnalysis::new(deliverable, problems)
    }

    /// Discovers a single problem for a category
    fn discover_problem(
        &self,
        category: ProblemCategory,
        path: &Path,
        number: u8,
    ) -> CtvpResult<Problem> {
        // Scan for patterns relevant to this category
        let patterns = category.get_check_patterns();
        let findings = self.scan_for_patterns(path, &patterns)?;

        // Select the highest-severity finding
        let (description, severity, evidence) = if let Some(finding) = findings.first() {
            (
                finding.clone(),
                self.assess_severity(finding),
                format!("Pattern detected during code scan: {}", finding),
            )
        } else {
            // No specific finding - create generic problem
            (
                format!("No evidence found for {} validation", category),
                DiagnosticLevel::Medium,
                "No relevant patterns detected - may indicate missing validation".into(),
            )
        };

        let remediation = self.suggest_remediation(category, &description);
        let test_required = self.suggest_test(category, &description);

        Ok(Problem::new(number, category, description, severity)
            .with_evidence(evidence)
            .with_remediation(remediation)
            .with_test(test_required))
    }

    /// Scans files for patterns
    fn scan_for_patterns(&self, path: &Path, patterns: &[&str]) -> CtvpResult<Vec<String>> {
        use nexcore_fs::walk::WalkDir;

        let mut findings = Vec::new();
        let mut files_scanned = 0;

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Some(max) = self.config.max_files {
                if files_scanned >= max {
                    break;
                }
            }

            if self.process_file(entry.path(), patterns, &mut findings) {
                files_scanned += 1;
            }
        }

        // Deduplicate
        findings.sort();
        findings.dedup();

        Ok(findings)
    }

    fn process_file(&self, path: &Path, patterns: &[&str], findings: &mut Vec<String>) -> bool {
        if self.should_skip_file(path) {
            return false;
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            self.check_content_patterns(&content, patterns, findings);
            return true;
        }
        false
    }

    fn should_skip_file(&self, path: &Path) -> bool {
        // Skip non-code files and common directories
        if path.components().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            s == "target" || s == "node_modules" || s == ".git"
        }) {
            return true;
        }

        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            if !["rs", "toml", "yaml", "yml", "json"].contains(&ext_str.as_ref()) {
                return true;
            }
        } else {
            return true;
        }
        false
    }

    fn check_content_patterns(&self, content: &str, patterns: &[&str], findings: &mut Vec<String>) {
        let content_lower = content.to_lowercase();

        for p in patterns {
            if self.is_pattern_missing(p, &content_lower) {
                findings.push(p.to_string());
            }
        }
    }

    fn is_pattern_missing(&self, pattern: &str, content: &str) -> bool {
        let p_l = pattern.to_lowercase();
        if p_l.contains("missing") || p_l.contains("no ") {
            let thing = p_l
                .replace("missing ", "")
                .replace("no ", "")
                .replace("untested ", "");
            return !content.contains(&thing);
        }
        false
    }

    /// Assesses severity of a finding
    fn assess_severity(&self, finding: &str) -> DiagnosticLevel {
        let finding_lower = finding.to_lowercase();

        if finding_lower.contains("security")
            || finding_lower.contains("leak")
            || finding_lower.contains("cascade")
            || finding_lower.contains("data loss")
        {
            DiagnosticLevel::Critical
        } else if finding_lower.contains("failure")
            || finding_lower.contains("exhaustion")
            || finding_lower.contains("timeout")
        {
            DiagnosticLevel::High
        } else if finding_lower.contains("missing")
            || finding_lower.contains("no ")
            || finding_lower.contains("hard-coded")
        {
            DiagnosticLevel::Medium
        } else {
            DiagnosticLevel::Low
        }
    }

    /// Suggests remediation for a finding
    fn suggest_remediation(&self, category: ProblemCategory, finding: &str) -> String {
        match category {
            ProblemCategory::Safety => self.remedy_safety(finding),
            ProblemCategory::Efficacy => self.remedy_efficacy(finding),
            ProblemCategory::Confirmation => self.remedy_confirmation(finding),
            ProblemCategory::Structural => self.remedy_structural(finding),
            ProblemCategory::Functional => self.remedy_functional(finding),
        }
    }

    fn remedy_safety(&self, f: &str) -> String {
        format!(
            "Add fault injection test: Inject failure for {} and verify graceful degradation",
            f
        )
    }

    fn remedy_efficacy(&self, f: &str) -> String {
        format!(
            "Add capability validation: Measure {} with real production data",
            f
        )
    }

    fn remedy_confirmation(&self, f: &str) -> String {
        format!(
            "Add scale validation: Implement shadow/canary deployment to verify {}",
            f
        )
    }

    fn remedy_structural(&self, f: &str) -> String {
        format!("Refactor: Introduce abstraction layer to decouple {}", f)
    }

    fn remedy_functional(&self, f: &str) -> String {
        format!("Add edge case handling: Implement explicit check for {}", f)
    }

    /// Suggests test for a finding
    fn suggest_test(&self, category: ProblemCategory, finding: &str) -> String {
        let id = sanitize_identifier(finding);
        match category {
            ProblemCategory::Safety => self.test_safety(&id, finding),
            ProblemCategory::Efficacy => self.test_efficacy(&id),
            ProblemCategory::Confirmation => self.test_confirmation(&id),
            ProblemCategory::Structural => self.test_structural(&id),
            ProblemCategory::Functional => self.test_functional(&id),
        }
    }

    fn test_safety(&self, id: &str, finding: &str) -> String {
        format!(
            "#[test] fn test_{}_failure_handling() {{ /* Inject {} failure, assert recovery */ }}",
            id, finding
        )
    }

    fn test_efficacy(&self, id: &str) -> String {
        format!(
            "#[test] fn test_{}_with_real_data() {{ /* Use production data, measure CAR */ }}",
            id
        )
    }

    fn test_confirmation(&self, id: &str) -> String {
        format!(
            "#[test] fn test_{}_at_scale() {{ /* Shadow deploy, compare outputs */ }}",
            id
        )
    }

    fn test_structural(&self, id: &str) -> String {
        format!(
            "#[test] fn test_{}_abstraction() {{ /* Swap implementation, verify behavior unchanged */ }}",
            id
        )
    }

    fn test_functional(&self, id: &str) -> String {
        format!(
            "#[test] fn test_{}_edge_case() {{ /* Input: edge case, assert handled */ }}",
            id
        )
    }
}

impl Default for FiveProblemsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Sanitizes a string for use as a Rust identifier
fn sanitize_identifier(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problem_categories_complete() {
        let categories = ProblemCategory::get_all();
        assert_eq!(categories.len(), 5);
    }

    #[test]
    fn test_five_problems_analysis_requires_five() {
        let problems = vec![
            Problem::new(1, ProblemCategory::Safety, "Test", DiagnosticLevel::Low),
            Problem::new(2, ProblemCategory::Efficacy, "Test", DiagnosticLevel::Low),
        ];

        let result = FiveProblemsAnalysis::new("test", problems);
        assert!(result.is_err());
    }

    #[test]
    fn test_five_problems_analysis_no_duplicates() {
        let problems = vec![
            Problem::new(1, ProblemCategory::Safety, "Test", DiagnosticLevel::Low),
            Problem::new(
                2,
                ProblemCategory::Safety,
                "Duplicate",
                DiagnosticLevel::Low,
            ), // Duplicate!
            Problem::new(
                3,
                ProblemCategory::Confirmation,
                "Test",
                DiagnosticLevel::Low,
            ),
            Problem::new(4, ProblemCategory::Structural, "Test", DiagnosticLevel::Low),
            Problem::new(5, ProblemCategory::Functional, "Test", DiagnosticLevel::Low),
        ];

        let result = FiveProblemsAnalysis::new("test", problems);
        assert!(result.is_err());
    }

    #[test]
    fn test_five_problems_analysis_valid() -> CtvpResult<()> {
        let problems = vec![
            Problem::new(
                1,
                ProblemCategory::Safety,
                "Safety issue",
                DiagnosticLevel::Critical,
            ),
            Problem::new(
                2,
                ProblemCategory::Efficacy,
                "Efficacy issue",
                DiagnosticLevel::High,
            ),
            Problem::new(
                3,
                ProblemCategory::Confirmation,
                "Confirmation issue",
                DiagnosticLevel::Medium,
            ),
            Problem::new(
                4,
                ProblemCategory::Structural,
                "Structural issue",
                DiagnosticLevel::Low,
            ),
            Problem::new(
                5,
                ProblemCategory::Functional,
                "Functional issue",
                DiagnosticLevel::Low,
            ),
        ];

        let analysis = FiveProblemsAnalysis::new("test", problems)?;

        assert_eq!(analysis.problems.len(), 5);
        assert_eq!(analysis.overall_severity, DiagnosticLevel::Critical);
        Ok(())
    }

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("Hello World!"), "hello_world");
        assert_eq!(sanitize_identifier("test-case"), "test_case");
        assert_eq!(sanitize_identifier("  spaces  "), "spaces");
    }
}
