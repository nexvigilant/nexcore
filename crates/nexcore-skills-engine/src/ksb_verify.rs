//! # KSB (Knowledge/Skills/Behaviors) Verification
//!
//! Verify skill compliance with KSB classification, growth tracking protocols,
//! and framework completeness requirements.
//!
//! ## Extracted Primitives
//!
//! | Tier | Primitive | Rust Manifestation |
//! |------|-----------|-------------------|
//! | T1 | exists | `Path::exists()` |
//! | T1 | sequence | `Vec<T>`, iterators |
//! | T1 | threshold | `const MIN: usize` |
//! | T1 | if-then | `if`, `match`, `?` |
//! | T2-P | contains | `str.contains()` |
//! | T2-P | count | `.filter().count()` |
//! | T2-P | read_text | `std::fs::read_to_string()` |
//! | T2-C | CheckResult | `struct { passed, message }` |
//! | T2-C | ValidationReport | `struct { skill, checks, passed }` |

use nexcore_chrono::DateTime;
use nexcore_error::Error;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::validation::ComplianceLevel;

// ============================================================================
// T1 Constants (Thresholds)
// ============================================================================

/// Minimum required KSB categories documented.
const MIN_KSB_CATEGORIES: usize = 2;

/// Minimum required growth protocol elements.
const MIN_GROWTH_PROTOCOLS: usize = 2;

/// Minimum required framework elements.
const MIN_FRAMEWORK_ELEMENTS: usize = 3;

// ============================================================================
// Error Types
// ============================================================================

/// Errors during KSB verification.
#[derive(Debug, Error)]
pub enum KsbError {
    /// Skill path does not exist.
    #[error("Skill path does not exist: {0}")]
    PathNotFound(String),

    /// Missing SKILL.md file.
    #[error("Missing SKILL.md in skill directory")]
    MissingSkillMd,

    /// Failed to read file.
    #[error("Failed to read file: {0}")]
    ReadError(#[from] std::io::Error),

    /// Invalid content structure.
    #[error("Invalid content structure: {0}")]
    InvalidStructure(String),
}

// ============================================================================
// T2-C Composite Types
// ============================================================================

/// Result of a single verification check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Name of the check.
    pub name: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Human-readable message.
    pub message: String,
    /// Items found (for keyword/pattern checks).
    pub found_items: Vec<String>,
    /// Expected items (for completeness checks).
    pub expected_items: Vec<String>,
    /// Match ratio (found/expected).
    pub match_ratio: f64,
}

impl CheckResult {
    /// Create a new check result.
    #[must_use]
    pub fn new(name: impl Into<String>, passed: bool, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed,
            message: message.into(),
            found_items: Vec::new(),
            expected_items: Vec::new(),
            match_ratio: if passed { 1.0 } else { 0.0 },
        }
    }

    /// Builder: add found and expected items with computed ratio.
    #[must_use]
    pub fn with_items(mut self, found: Vec<String>, expected: Vec<String>) -> Self {
        self.match_ratio = compute_ratio(found.len(), expected.len());
        self.found_items = found;
        self.expected_items = expected;
        self
    }
}

/// Full KSB validation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbValidation {
    /// Skill name (from path).
    pub skill_name: String,
    /// Timestamp of validation.
    pub timestamp: DateTime,
    /// All check results.
    pub checks: Vec<CheckResult>,
    /// Overall pass status.
    pub passed: bool,
    /// Number of checks passed.
    pub passed_count: usize,
    /// Total number of checks.
    pub total_count: usize,
    /// Compliance level achieved.
    pub compliance_level: ComplianceLevel,
    /// Suggestions for improvement.
    pub suggestions: Vec<String>,
}

impl KsbValidation {
    /// Calculate pass ratio as percentage.
    #[must_use]
    pub fn pass_percentage(&self) -> f64 {
        compute_ratio(self.passed_count, self.total_count) * 100.0
    }
}

// ============================================================================
// T2-P Primitives (Pure Functions)
// ============================================================================

/// Compute ratio (found / expected), returns 0.0 if denominator is 0.
#[inline]
fn compute_ratio(found: usize, expected: usize) -> f64 {
    if expected == 0 {
        0.0
    } else {
        found as f64 / expected as f64
    }
}

/// Check if any keyword from list exists in content.
#[inline]
fn has_any_keyword(content: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| content.contains(kw))
}

/// Map pass percentage to compliance level.
fn map_to_compliance_level(pass_pct: f64) -> ComplianceLevel {
    match pass_pct as u32 {
        90..=100 => ComplianceLevel::Diamond,
        80..=89 => ComplianceLevel::Platinum,
        70..=79 => ComplianceLevel::Gold,
        50..=69 => ComplianceLevel::Silver,
        _ => ComplianceLevel::Bronze,
    }
}

/// Build check result from found/expected comparison.
fn build_check_result(
    name: &str,
    found: Vec<String>,
    expected: Vec<String>,
    min: usize,
) -> CheckResult {
    let passed = found.len() >= min;
    let message = format_check_message(&found, &expected, min, passed);
    CheckResult::new(name, passed, message).with_items(found, expected)
}

/// Format message for check result.
fn format_check_message(found: &[String], expected: &[String], min: usize, passed: bool) -> String {
    if passed {
        format!("{} of {} elements found", found.len(), expected.len())
    } else {
        format!("{} of {} found (need {})", found.len(), expected.len(), min)
    }
}

// ============================================================================
// Keyword Definitions
// ============================================================================

/// Keywords for KSB categories.
mod ksb_keywords {
    pub const KNOWLEDGE: &[&str] = &[
        "knowledge",
        "understand",
        "concepts",
        "theory",
        "principles",
    ];
    pub const SKILLS: &[&str] = &["skill", "technique", "method", "procedure", "implement"];
    pub const BEHAVIORS: &[&str] = &["behavior", "habit", "pattern", "attitude", "mindset"];
}

/// Keywords for growth protocols.
mod growth_keywords {
    pub const PAIN_BUTTON: &[&str] = &["pain button", "pain point", "friction", "blocker"];
    pub const FIVE_WHYS: &[&str] = &["5 why", "five why", "root cause", "why?"];
    pub const PATTERN_EXTRACTION: &[&str] = &["pattern", "extract", "recognize", "recurring"];
}

/// Keywords for framework elements.
mod framework_keywords {
    pub const CODE_PATTERNS: &[&str] = &["code pattern", "implementation", "algorithm", "api"];
    pub const TESTING: &[&str] = &["test", "assert", "verify", "validate", "coverage"];
    pub const REVIEW: &[&str] = &["review", "audit", "inspect", "evaluate", "criteria"];
    pub const LOCATIONS: &[&str] = &["location", "directory", "path", "file", "module"];
}

// ============================================================================
// Check Functions (Decomposed)
// ============================================================================

/// Check KSB classification.
fn check_ksb_classification(content: &str) -> CheckResult {
    let lower = content.to_lowercase();
    let mut found = Vec::new();
    let expected = vec!["Knowledge".into(), "Skills".into(), "Behaviors".into()];

    if has_any_keyword(&lower, ksb_keywords::KNOWLEDGE) {
        found.push("Knowledge".into());
    }
    if has_any_keyword(&lower, ksb_keywords::SKILLS) {
        found.push("Skills".into());
    }
    if has_any_keyword(&lower, ksb_keywords::BEHAVIORS) {
        found.push("Behaviors".into());
    }

    build_check_result("ksb_classification", found, expected, MIN_KSB_CATEGORIES)
}

/// Check growth tracking protocols.
fn check_growth_tracking(content: &str) -> CheckResult {
    let lower = content.to_lowercase();
    let mut found = Vec::new();
    let expected = vec![
        "Pain Button".into(),
        "5 Whys".into(),
        "Pattern Extraction".into(),
    ];

    if has_any_keyword(&lower, growth_keywords::PAIN_BUTTON) {
        found.push("Pain Button".into());
    }
    if has_any_keyword(&lower, growth_keywords::FIVE_WHYS) {
        found.push("5 Whys".into());
    }
    if has_any_keyword(&lower, growth_keywords::PATTERN_EXTRACTION) {
        found.push("Pattern Extraction".into());
    }

    build_check_result("growth_tracking", found, expected, MIN_GROWTH_PROTOCOLS)
}

/// Check framework completeness.
fn check_framework_completeness(content: &str) -> CheckResult {
    let lower = content.to_lowercase();
    let mut found = Vec::new();
    let expected = vec![
        "Code Patterns".into(),
        "Testing".into(),
        "Review".into(),
        "Locations".into(),
    ];

    if has_any_keyword(&lower, framework_keywords::CODE_PATTERNS) {
        found.push("Code Patterns".into());
    }
    if has_any_keyword(&lower, framework_keywords::TESTING) {
        found.push("Testing".into());
    }
    if has_any_keyword(&lower, framework_keywords::REVIEW) {
        found.push("Review".into());
    }
    if has_any_keyword(&lower, framework_keywords::LOCATIONS) {
        found.push("Locations".into());
    }

    build_check_result(
        "framework_completeness",
        found,
        expected,
        MIN_FRAMEWORK_ELEMENTS,
    )
}

/// Check directory structure.
fn check_directory_structure(skill_path: &Path) -> CheckResult {
    let expected = vec![
        "SKILL.md".into(),
        "scripts/".into(),
        "references/".into(),
        "templates/".into(),
    ];
    let found = collect_existing_paths(skill_path);
    let passed = found.contains(&"SKILL.md".to_string());
    let message = if passed {
        format!("{} of {} elements present", found.len(), expected.len())
    } else {
        "Missing required SKILL.md file".to_string()
    };
    CheckResult::new("directory_structure", passed, message).with_items(found, expected)
}

/// Collect existing paths from skill directory.
fn collect_existing_paths(skill_path: &Path) -> Vec<String> {
    let mut found = Vec::new();
    if skill_path.join("SKILL.md").exists() {
        found.push("SKILL.md".into());
    }
    if skill_path.join("scripts").exists() {
        found.push("scripts/".into());
    }
    if skill_path.join("references").exists() {
        found.push("references/".into());
    }
    if skill_path.join("templates").exists() {
        found.push("templates/".into());
    }
    found
}

/// Check frontmatter presence.
fn check_frontmatter(content: &str) -> CheckResult {
    let expected = vec!["name".into(), "version".into(), "description".into()];
    let found = extract_frontmatter_fields(content);
    let has_frontmatter = content.starts_with("---");
    let passed = has_frontmatter && !found.is_empty();
    let message = format_frontmatter_message(has_frontmatter, &found, &expected);
    CheckResult::new("frontmatter", passed, message).with_items(found, expected)
}

/// Extract frontmatter fields from content.
fn extract_frontmatter_fields(content: &str) -> Vec<String> {
    if !content.starts_with("---") {
        return Vec::new();
    }
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 2 {
        return Vec::new();
    }

    let fm = parts[1].to_lowercase();
    let mut found = Vec::new();
    if fm.contains("name") {
        found.push("name".into());
    }
    if fm.contains("version") {
        found.push("version".into());
    }
    if fm.contains("description") || fm.contains("desc") {
        found.push("description".into());
    }
    found
}

/// Format frontmatter check message.
fn format_frontmatter_message(has_fm: bool, found: &[String], expected: &[String]) -> String {
    if has_fm && !found.is_empty() {
        format!(
            "Frontmatter with {} of {} fields",
            found.len(),
            expected.len()
        )
    } else if has_fm {
        "Frontmatter present but missing standard fields".to_string()
    } else {
        "Missing YAML frontmatter (should start with ---)".to_string()
    }
}

// ============================================================================
// Main Verification Function
// ============================================================================

/// Verify a skill against KSB requirements.
pub fn verify_ksb(skill_path: &Path) -> Result<KsbValidation, KsbError> {
    validate_skill_path(skill_path)?;
    let content = std::fs::read_to_string(skill_path.join("SKILL.md"))?;
    let skill_name = extract_skill_name(skill_path);
    let checks = run_all_checks(skill_path, &content);
    Ok(build_validation(skill_name, checks))
}

/// Validate that skill path and SKILL.md exist.
fn validate_skill_path(skill_path: &Path) -> Result<(), KsbError> {
    if !skill_path.exists() {
        return Err(KsbError::PathNotFound(
            skill_path.to_string_lossy().to_string(),
        ));
    }
    if !skill_path.join("SKILL.md").exists() {
        return Err(KsbError::MissingSkillMd);
    }
    Ok(())
}

/// Extract skill name from path.
fn extract_skill_name(skill_path: &Path) -> String {
    skill_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Run all verification checks.
fn run_all_checks(skill_path: &Path, content: &str) -> Vec<CheckResult> {
    vec![
        check_directory_structure(skill_path),
        check_frontmatter(content),
        check_ksb_classification(content),
        check_growth_tracking(content),
        check_framework_completeness(content),
    ]
}

/// Build validation result from checks.
fn build_validation(skill_name: String, checks: Vec<CheckResult>) -> KsbValidation {
    let passed_count = checks.iter().filter(|c| c.passed).count();
    let total_count = checks.len();
    let passed = passed_count > total_count / 2;
    let pass_pct = compute_ratio(passed_count, total_count) * 100.0;
    let suggestions = generate_suggestions(&checks);

    KsbValidation {
        skill_name,
        timestamp: DateTime::now(),
        checks,
        passed,
        passed_count,
        total_count,
        compliance_level: map_to_compliance_level(pass_pct),
        suggestions,
    }
}

/// Generate suggestions for failed checks.
fn generate_suggestions(checks: &[CheckResult]) -> Vec<String> {
    checks
        .iter()
        .filter(|c| !c.passed)
        .map(|c| format!("Fix: {}", c.message))
        .collect()
}

/// Verify multiple skills at once.
pub fn verify_ksb_batch<I, P>(skill_paths: I) -> Vec<Result<KsbValidation, KsbError>>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    skill_paths
        .into_iter()
        .map(|p| verify_ksb(p.as_ref()))
        .collect()
}

/// Summary statistics for batch validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbBatchSummary {
    /// Total skills validated.
    pub total: usize,
    /// Skills that passed.
    pub passed: usize,
    /// Skills that failed.
    pub failed: usize,
    /// Skills with errors.
    pub errors: usize,
    /// Pass rate percentage.
    pub pass_rate: f64,
    /// Compliance level distribution.
    pub level_distribution: std::collections::HashMap<String, usize>,
}

/// Generate summary statistics from batch results.
#[must_use]
pub fn summarize_batch(results: &[Result<KsbValidation, KsbError>]) -> KsbBatchSummary {
    let mut summary = KsbBatchSummary {
        total: results.len(),
        passed: 0,
        failed: 0,
        errors: 0,
        pass_rate: 0.0,
        level_distribution: std::collections::HashMap::new(),
    };
    aggregate_results(results, &mut summary);
    summary.pass_rate = compute_ratio(summary.passed, summary.total) * 100.0;
    summary
}

/// Aggregate results into summary.
fn aggregate_results(results: &[Result<KsbValidation, KsbError>], summary: &mut KsbBatchSummary) {
    for result in results {
        match result {
            Ok(v) if v.passed => {
                summary.passed += 1;
                *summary
                    .level_distribution
                    .entry(v.compliance_level.to_string())
                    .or_insert(0) += 1;
            }
            Ok(v) => {
                summary.failed += 1;
                *summary
                    .level_distribution
                    .entry(v.compliance_level.to_string())
                    .or_insert(0) += 1;
            }
            Err(_) => summary.errors += 1,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_skill(content: &str) -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("create temp dir");
        let skill_path = dir.path().to_path_buf();
        std::fs::write(skill_path.join("SKILL.md"), content).expect("write");
        (dir, skill_path)
    }

    #[test]
    fn test_compute_ratio() {
        assert!((compute_ratio(2, 4) - 0.5).abs() < f64::EPSILON);
        assert!((compute_ratio(0, 0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_has_any_keyword() {
        assert!(has_any_keyword("hello world", &["world", "foo"]));
        assert!(!has_any_keyword("hello", &["world", "foo"]));
    }

    #[test]
    fn test_check_ksb_full() {
        let content = "knowledge skills behavior patterns";
        let result = check_ksb_classification(content);
        assert!(result.passed);
        assert_eq!(result.found_items.len(), 3);
    }

    #[test]
    fn test_check_ksb_partial() {
        let result = check_ksb_classification("just knowledge here");
        assert!(!result.passed);
    }

    #[test]
    fn test_check_growth_tracking() {
        let content = "pain point friction root cause pattern extract";
        let result = check_growth_tracking(content);
        assert!(result.passed);
    }

    #[test]
    fn test_check_framework() {
        let content = "api test review directory";
        let result = check_framework_completeness(content);
        assert!(result.passed);
    }

    #[test]
    fn test_check_frontmatter_valid() {
        let content = "---\nname: test\nversion: 1.0\ndescription: x\n---\n# Test";
        let result = check_frontmatter(content);
        assert!(result.passed);
    }

    #[test]
    fn test_check_frontmatter_missing() {
        let result = check_frontmatter("# No frontmatter");
        assert!(!result.passed);
    }

    #[test]
    fn test_verify_ksb_missing_path() {
        let result = verify_ksb(Path::new("/nonexistent"));
        assert!(matches!(result, Err(KsbError::PathNotFound(_))));
    }

    #[test]
    fn test_verify_ksb_minimal() {
        let (_dir, path) = create_test_skill("---\nname: x\n---\n# Minimal");
        let result = verify_ksb(&path).expect("valid");
        assert!(!result.passed);
    }

    #[test]
    fn test_verify_ksb_comprehensive() {
        let content = "---\nname: x\nversion: 1\ndescription: y\n---\n\
            # Skill\nknowledge skills behavior pain point root cause pattern api test review directory";
        let (_dir, path) = create_test_skill(content);
        let result = verify_ksb(&path).expect("valid");
        assert!(result.passed);
    }

    #[test]
    fn test_batch_and_summary() {
        let (_d1, p1) = create_test_skill("---\nname: a\n---\nknowledge skills");
        let (_d2, p2) = create_test_skill("---\nname: b\n---\n# Empty");
        let results = verify_ksb_batch([p1.as_path(), p2.as_path()]);
        let summary = summarize_batch(&results);
        assert_eq!(summary.total, 2);
    }

    #[test]
    fn test_compliance_mapping() {
        assert_eq!(map_to_compliance_level(95.0), ComplianceLevel::Diamond);
        assert_eq!(map_to_compliance_level(85.0), ComplianceLevel::Platinum);
        assert_eq!(map_to_compliance_level(75.0), ComplianceLevel::Gold);
        assert_eq!(map_to_compliance_level(55.0), ComplianceLevel::Silver);
        assert_eq!(map_to_compliance_level(30.0), ComplianceLevel::Bronze);
    }
}
