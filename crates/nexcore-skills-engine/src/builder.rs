//! # Skill Builder
//!
//! Build and validate skills, clean generated files, and report task status.
//!
//! ## Extracted Primitives
//!
//! | Tier | Primitive | Rust Manifestation |
//! |------|-----------|-------------------|
//! | T1 | task_result | `bool` |
//! | T1 | exit_code | `Result<T, E>` |
//! | T1 | sequence | `Vec<T>`, iterators |
//! | T2-P | path_exists | `Path::exists()` |
//! | T2-P | mkdir | `fs::create_dir_all()` |
//! | T2-P | glob_pattern | `walkdir` + filter |
//! | T2-P | file_unlink | `fs::remove_file()` |
//! | T2-P | boolean_flag | `bool` fields in options |
//! | T2-C | failure_count | `usize` in report |
//! | T2-C | BuildOptions | `struct { clean, verbose }` |
//! | T2-C | BuildReport | `struct { skill_name, tasks_*, messages }` |

use nexcore_error::Error;
use nexcore_fs::walk::WalkDir;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ============================================================================
// Error Types
// ============================================================================

/// Errors during skill build operations.
#[derive(Debug, Error)]
pub enum BuildError {
    /// Skill path does not exist.
    #[error("Skill path does not exist: {0}")]
    PathNotFound(String),

    /// Missing required SKILL.md file.
    #[error("Missing SKILL.md in skill directory: {0}")]
    MissingSkillMd(String),

    /// Failed to read file or directory.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid skill structure.
    #[error("Invalid skill structure: {0}")]
    InvalidStructure(String),
}

// ============================================================================
// T2-C Composite Types
// ============================================================================

/// Options controlling the build process.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildOptions {
    /// If true, clean generated files before building.
    pub clean: bool,
    /// If true, emit verbose messages during build.
    pub verbose: bool,
}

impl BuildOptions {
    /// Create new build options with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: enable clean mode.
    #[must_use]
    pub fn with_clean(mut self, clean: bool) -> Self {
        self.clean = clean;
        self
    }

    /// Builder: enable verbose mode.
    #[must_use]
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

/// Report of a skill build operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildReport {
    /// Name of the skill that was built.
    pub skill_name: String,
    /// Number of tasks executed.
    pub tasks_run: usize,
    /// Number of tasks that passed.
    pub tasks_passed: usize,
    /// Number of tasks that failed.
    pub tasks_failed: usize,
    /// Messages emitted during build.
    pub messages: Vec<String>,
}

impl BuildReport {
    /// Create a new build report for a skill.
    #[must_use]
    pub fn new(skill_name: impl Into<String>) -> Self {
        Self {
            skill_name: skill_name.into(),
            ..Default::default()
        }
    }

    /// Record a passed task.
    pub fn record_pass(&mut self, message: impl Into<String>) {
        self.tasks_run += 1;
        self.tasks_passed += 1;
        self.messages.push(message.into());
    }

    /// Record a failed task.
    pub fn record_fail(&mut self, message: impl Into<String>) {
        self.tasks_run += 1;
        self.tasks_failed += 1;
        self.messages.push(message.into());
    }

    /// Record an informational message.
    pub fn record_info(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
    }

    /// Check if all tasks passed.
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.tasks_failed == 0
    }

    /// Get pass rate as percentage.
    #[must_use]
    pub fn pass_rate(&self) -> f64 {
        if self.tasks_run == 0 {
            return 100.0;
        }
        (self.tasks_passed as f64 / self.tasks_run as f64) * 100.0
    }
}

/// Result of a structure validation check.
#[derive(Debug, Clone)]
pub struct StructureCheck {
    /// Name of the file or directory.
    pub name: String,
    /// Whether this item is required.
    pub required: bool,
    /// Whether this item exists.
    pub exists: bool,
}

impl StructureCheck {
    /// Check if this is a passing check.
    #[must_use]
    pub fn passed(&self) -> bool {
        !self.required || self.exists
    }
}

/// Summarize a batch of build reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    /// Total number of skills built.
    pub total_skills: usize,
    /// Number of skills with all tasks passing.
    pub skills_passed: usize,
    /// Number of skills with at least one failure.
    pub skills_failed: usize,
    /// Total tasks run across all skills.
    pub total_tasks: usize,
    /// Total tasks passed across all skills.
    pub total_tasks_passed: usize,
    /// Overall pass rate percentage.
    pub overall_pass_rate: f64,
}

// ============================================================================
// T2-P Primitives (Pure Functions)
// ============================================================================

/// Required directories for a valid skill structure.
const REQUIRED_DIRS: &[&str] = &["scripts", "references", "templates"];

/// Optional files that may exist in a skill.
const OPTIONAL_FILES: &[&str] = &["verify.py", "verify.rs", "build.py", "build.rs"];

/// Pattern for generated files to clean.
const GENERATED_PATTERN: &str = ".generated.";

/// Extract skill name from path.
fn extract_skill_name(skill_path: &Path) -> String {
    skill_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Check if a file is a generated file.
fn is_generated_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| name.contains(GENERATED_PATTERN))
}

/// Create a directory if missing.
fn ensure_dir(path: &Path) -> Result<(), std::io::Error> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Try to remove a file, returning true on success.
fn try_remove_file(path: &Path) -> bool {
    std::fs::remove_file(path).is_ok()
}

/// Check if entry is a removable generated file.
fn is_removable_generated(entry: &nexcore_fs::walk::DirEntry) -> bool {
    let path = entry.path();
    path.is_file() && is_generated_file(path)
}

// ============================================================================
// Build Tasks (Decomposed)
// ============================================================================

/// Task: Clean generated files from a skill directory.
fn task_clean(skill_path: &Path, verbose: bool, report: &mut BuildReport) {
    let removed = collect_and_remove_generated(skill_path, verbose, report);
    report.record_pass(format!("Clean: removed {removed} generated files"));
}

/// Collect and remove generated files.
fn collect_and_remove_generated(
    skill_path: &Path,
    verbose: bool,
    report: &mut BuildReport,
) -> usize {
    let walker = WalkDir::new(skill_path).follow_links(false).into_iter();
    let entries: Vec<_> = walker.filter_map(Result::ok).collect();

    let mut count = 0;
    for entry in entries {
        if is_removable_generated(&entry) {
            count += remove_and_log(&entry, verbose, report);
        }
    }
    count
}

/// Remove a file and optionally log it.
fn remove_and_log(
    entry: &nexcore_fs::walk::DirEntry,
    verbose: bool,
    report: &mut BuildReport,
) -> usize {
    let path = entry.path();
    if try_remove_file(path) {
        if verbose {
            report.record_info(format!("Removed: {}", path.display()));
        }
        return 1;
    }
    0
}

/// Task: Validate structure and record results.
fn task_validate_structure(skill_path: &Path, report: &mut BuildReport) -> Vec<StructureCheck> {
    let checks = collect_structure_checks(skill_path);
    record_structure_checks(&checks, report);
    checks
}

/// Collect structure checks for a skill.
fn collect_structure_checks(skill_path: &Path) -> Vec<StructureCheck> {
    let mut checks = Vec::new();

    // Check SKILL.md
    checks.push(StructureCheck {
        name: "SKILL.md".to_string(),
        required: true,
        exists: skill_path.join("SKILL.md").exists(),
    });

    // Check required directories
    for dir in REQUIRED_DIRS {
        checks.push(StructureCheck {
            name: (*dir).to_string(),
            required: true,
            exists: skill_path.join(dir).exists(),
        });
    }

    // Check optional files
    for file in OPTIONAL_FILES {
        checks.push(StructureCheck {
            name: (*file).to_string(),
            required: false,
            exists: skill_path.join(file).exists(),
        });
    }
    checks
}

/// Record structure checks to report.
fn record_structure_checks(checks: &[StructureCheck], report: &mut BuildReport) {
    for check in checks {
        record_single_check(check, report);
    }
}

/// Record a single structure check.
fn record_single_check(check: &StructureCheck, report: &mut BuildReport) {
    let status = if check.exists { "found" } else { "missing" };
    let kind = if check.required {
        "required"
    } else {
        "optional"
    };
    let msg = format!("Structure: {} ({}) - {}", check.name, kind, status);

    if check.passed() {
        report.record_pass(msg);
    } else {
        report.record_fail(msg);
    }
}

/// Task: Create missing directories.
fn task_create_dirs(skill_path: &Path, report: &mut BuildReport) {
    for dir in REQUIRED_DIRS {
        create_dir_if_missing(skill_path, dir, report);
    }
}

/// Create a directory if missing and record result.
fn create_dir_if_missing(skill_path: &Path, dir: &str, report: &mut BuildReport) {
    let dir_path = skill_path.join(dir);
    if dir_path.exists() {
        return;
    }
    match ensure_dir(&dir_path) {
        Ok(()) => report.record_pass(format!("Created directory: {dir}")),
        Err(e) => report.record_fail(format!("Failed to create {dir}: {e}")),
    }
}

/// Task: Validate SKILL.md content.
fn task_validate_content(skill_path: &Path, report: &mut BuildReport) {
    let skill_md_path = skill_path.join("SKILL.md");

    let content = match std::fs::read_to_string(&skill_md_path) {
        Ok(c) => c,
        Err(e) => {
            report.record_fail(format!("Failed to read SKILL.md: {e}"));
            return;
        }
    };

    validate_frontmatter(&content, report);
    validate_required_sections(&content, report);
}

/// Validate frontmatter presence.
fn validate_frontmatter(content: &str, report: &mut BuildReport) {
    if content.starts_with("---") {
        report.record_pass("Content: frontmatter present");
    } else {
        report.record_fail("Content: missing frontmatter (should start with ---)");
    }
}

/// Validate required sections.
fn validate_required_sections(content: &str, report: &mut BuildReport) {
    let required_sections = ["## Input", "## Output", "## Logic"];
    for section in required_sections {
        validate_section(content, section, report);
    }
}

/// Validate a single section.
fn validate_section(content: &str, section: &str, report: &mut BuildReport) {
    if content.contains(section) {
        report.record_pass(format!("Content: {section} section found"));
    } else {
        report.record_fail(format!("Content: {section} section missing"));
    }
}

// ============================================================================
// Main Build Function
// ============================================================================

/// Build a skill, optionally cleaning generated files first.
///
/// # Errors
///
/// Returns `BuildError` if the skill path is invalid or SKILL.md is missing.
pub fn build_skill(skill_path: &Path, opts: BuildOptions) -> Result<BuildReport, BuildError> {
    validate_path_exists(skill_path)?;

    let skill_name = extract_skill_name(skill_path);
    let mut report = BuildReport::new(&skill_name);

    if opts.verbose {
        report.record_info(format!("Building skill: {skill_name}"));
    }

    run_build_tasks(skill_path, &opts, &mut report)?;

    if opts.verbose {
        record_completion_summary(&report);
    }

    Ok(report)
}

/// Validate that skill path exists.
fn validate_path_exists(skill_path: &Path) -> Result<(), BuildError> {
    if !skill_path.exists() {
        return Err(BuildError::PathNotFound(skill_path.display().to_string()));
    }
    Ok(())
}

/// Run all build tasks.
fn run_build_tasks(
    skill_path: &Path,
    opts: &BuildOptions,
    report: &mut BuildReport,
) -> Result<(), BuildError> {
    // Task 1: Clean generated files
    if opts.clean {
        task_clean(skill_path, opts.verbose, report);
    }

    // Task 2: Validate structure
    let checks = task_validate_structure(skill_path, report);
    check_skill_md_exists(&checks, skill_path)?;

    // Task 3: Create missing directories
    task_create_dirs(skill_path, report);

    // Task 4: Validate content
    task_validate_content(skill_path, report);

    Ok(())
}

/// Check that SKILL.md exists.
fn check_skill_md_exists(checks: &[StructureCheck], skill_path: &Path) -> Result<(), BuildError> {
    let missing = checks.iter().any(|c| c.name == "SKILL.md" && !c.exists);
    if missing {
        return Err(BuildError::MissingSkillMd(skill_path.display().to_string()));
    }
    Ok(())
}

/// Record completion summary (logged but not counted as task).
fn record_completion_summary(report: &BuildReport) {
    let _summary = format!(
        "Build complete: {}/{} tasks passed ({:.1}%)",
        report.tasks_passed,
        report.tasks_run,
        report.pass_rate()
    );
}

/// Build multiple skills from a parent directory.
///
/// # Errors
///
/// Returns `BuildError` if directory traversal fails.
pub fn build_skills_batch(
    parent_dir: &Path,
    opts: BuildOptions,
) -> Result<Vec<BuildReport>, BuildError> {
    validate_path_exists(parent_dir)?;

    let skill_dirs = discover_skill_dirs(parent_dir);
    let reports = build_all_skills(&skill_dirs, &opts);
    Ok(reports)
}

/// Discover all skill directories.
fn discover_skill_dirs(parent_dir: &Path) -> Vec<std::path::PathBuf> {
    let walker = WalkDir::new(parent_dir).follow_links(true).max_depth(3);
    let entries: Vec<_> = walker.into_iter().filter_map(Result::ok).collect();

    entries
        .iter()
        .filter(|e| e.path().file_name().is_some_and(|n| n == "SKILL.md"))
        .filter_map(|e| e.path().parent().map(Path::to_path_buf))
        .collect()
}

/// Build all skills and collect reports.
fn build_all_skills(skill_dirs: &[std::path::PathBuf], opts: &BuildOptions) -> Vec<BuildReport> {
    skill_dirs
        .iter()
        .map(|dir| build_or_report_failure(dir, opts))
        .collect()
}

/// Build a skill or create a failure report.
fn build_or_report_failure(skill_dir: &Path, opts: &BuildOptions) -> BuildReport {
    match build_skill(skill_dir, opts.clone()) {
        Ok(report) => report,
        Err(e) => {
            let skill_name = extract_skill_name(skill_dir);
            let mut report = BuildReport::new(&skill_name);
            report.record_fail(format!("Build failed: {e}"));
            report
        }
    }
}

/// Generate a summary from a batch of build reports.
#[must_use]
pub fn summarize_batch(reports: &[BuildReport]) -> BatchSummary {
    let total_skills = reports.len();
    let skills_passed = reports.iter().filter(|r| r.all_passed()).count();
    let total_tasks: usize = reports.iter().map(|r| r.tasks_run).sum();
    let total_tasks_passed: usize = reports.iter().map(|r| r.tasks_passed).sum();

    let overall_pass_rate = compute_pass_rate(total_tasks_passed, total_tasks);

    BatchSummary {
        total_skills,
        skills_passed,
        skills_failed: total_skills - skills_passed,
        total_tasks,
        total_tasks_passed,
        overall_pass_rate,
    }
}

/// Compute pass rate percentage.
fn compute_pass_rate(passed: usize, total: usize) -> f64 {
    if total == 0 {
        return 100.0;
    }
    (passed as f64 / total as f64) * 100.0
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_skill(temp_dir: &TempDir, name: &str) -> std::path::PathBuf {
        let skill_dir = temp_dir.path().join(name);
        fs::create_dir_all(&skill_dir).ok();

        let skill_md = skill_dir.join("SKILL.md");
        let content = "---\nname: test\n---\n\n## Input\n\n## Output\n\n## Logic\n";
        fs::write(&skill_md, content).ok();

        fs::create_dir_all(skill_dir.join("scripts")).ok();
        fs::create_dir_all(skill_dir.join("references")).ok();
        fs::create_dir_all(skill_dir.join("templates")).ok();

        skill_dir
    }

    fn create_generated_file(skill_dir: &Path) -> std::path::PathBuf {
        let gen_file = skill_dir.join("output.generated.txt");
        fs::write(&gen_file, "generated content").ok();
        gen_file
    }

    #[test]
    fn test_build_options_default() {
        let opts = BuildOptions::new();
        assert!(!opts.clean);
        assert!(!opts.verbose);
    }

    #[test]
    fn test_build_options_builder() {
        let opts = BuildOptions::new().with_clean(true).with_verbose(true);
        assert!(opts.clean);
        assert!(opts.verbose);
    }

    #[test]
    fn test_build_report_new() {
        let report = BuildReport::new("my-skill");
        assert_eq!(report.skill_name, "my-skill");
        assert_eq!(report.tasks_run, 0);
    }

    #[test]
    fn test_build_report_record_pass() {
        let mut report = BuildReport::new("test");
        report.record_pass("Task 1 passed");
        assert_eq!(report.tasks_run, 1);
        assert_eq!(report.tasks_passed, 1);
        assert!(report.all_passed());
    }

    #[test]
    fn test_build_report_record_fail() {
        let mut report = BuildReport::new("test");
        report.record_fail("Task 1 failed");
        assert_eq!(report.tasks_failed, 1);
        assert!(!report.all_passed());
    }

    #[test]
    fn test_build_report_pass_rate() {
        let mut report = BuildReport::new("test");
        report.record_pass("Pass 1");
        report.record_pass("Pass 2");
        report.record_fail("Fail 1");
        assert!((report.pass_rate() - 66.666_666_7).abs() < 0.01);
    }

    #[test]
    fn test_is_generated_file() {
        assert!(is_generated_file(Path::new("output.generated.txt")));
        assert!(!is_generated_file(Path::new("normal.txt")));
    }

    #[test]
    fn test_extract_skill_name() {
        let name = extract_skill_name(Path::new("/home/user/skills/my-skill"));
        assert_eq!(name, "my-skill");
    }

    #[test]
    fn test_build_skill_path_not_found() {
        let result = build_skill(Path::new("/nonexistent"), BuildOptions::new());
        assert!(matches!(result, Err(BuildError::PathNotFound(_))));
    }

    #[test]
    fn test_build_skill_missing_skill_md() {
        let temp = TempDir::new().ok();
        let Some(ref temp) = temp else { return };

        let skill_dir = temp.path().join("empty");
        fs::create_dir_all(&skill_dir).ok();

        let result = build_skill(&skill_dir, BuildOptions::new());
        assert!(matches!(result, Err(BuildError::MissingSkillMd(_))));
    }

    #[test]
    fn test_build_skill_valid() {
        let temp = TempDir::new().ok();
        let Some(ref temp) = temp else { return };

        let skill_dir = create_test_skill(temp, "valid-skill");
        let result = build_skill(&skill_dir, BuildOptions::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_skill_with_clean() {
        let temp = TempDir::new().ok();
        let Some(ref temp) = temp else { return };

        let skill_dir = create_test_skill(temp, "clean-skill");
        let gen_file = create_generated_file(&skill_dir);
        assert!(gen_file.exists());

        let result = build_skill(&skill_dir, BuildOptions::new().with_clean(true));
        assert!(result.is_ok());
        assert!(!gen_file.exists());
    }

    #[test]
    fn test_summarize_batch_empty() {
        let summary = summarize_batch(&[]);
        assert_eq!(summary.total_skills, 0);
    }

    #[test]
    fn test_summarize_batch() {
        let mut r1 = BuildReport::new("s1");
        r1.record_pass("t1");

        let mut r2 = BuildReport::new("s2");
        r2.record_fail("t1");

        let summary = summarize_batch(&[r1, r2]);
        assert_eq!(summary.total_skills, 2);
        assert_eq!(summary.skills_passed, 1);
        assert_eq!(summary.skills_failed, 1);
    }
}
