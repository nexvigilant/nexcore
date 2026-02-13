//! # Primitive Validation
//!
//! Validate primitives, compositions, and grounding integrity.
//!
//! ## Tier: T2-C (Comparison + Boundary + Causality)

use crate::bedrock::BedrockAtom;
use crate::graph::DependencyGraph;
use crate::primitiva::{LexPrimitiva, PrimitiveComposition};
use crate::tier::Tier;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Classification of validation/schema issue severity.
///
/// Tier: T2-P (κ + ∂ — comparison with boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    /// Informational note.
    Info,
    /// Warning - may indicate issues.
    Warning,
    /// Error - validation failed.
    Error,
}

/// Backward-compatible alias.
#[deprecated(note = "use DiagnosticLevel — F2 equivocation fix")]
pub type Severity = DiagnosticLevel;

impl std::fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// A single validation issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Diagnostic level of the issue.
    pub severity: DiagnosticLevel,
    /// Issue code for programmatic handling.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Context about where the issue occurred.
    pub context: Option<String>,
}

impl ValidationIssue {
    /// Create a new issue.
    #[must_use]
    pub fn new(severity: DiagnosticLevel, code: &str, message: &str) -> Self {
        Self {
            severity,
            code: code.to_string(),
            message: message.to_string(),
            context: None,
        }
    }

    /// Add context to the issue.
    #[must_use]
    pub fn with_context(mut self, ctx: &str) -> Self {
        self.context = Some(ctx.to_string());
        self
    }

    /// Create an info issue.
    #[must_use]
    pub fn info(code: &str, message: &str) -> Self {
        Self::new(DiagnosticLevel::Info, code, message)
    }

    /// Create a warning issue.
    #[must_use]
    pub fn warning(code: &str, message: &str) -> Self {
        Self::new(DiagnosticLevel::Warning, code, message)
    }

    /// Create an error issue.
    #[must_use]
    pub fn error(code: &str, message: &str) -> Self {
        Self::new(DiagnosticLevel::Error, code, message)
    }
}

/// Result of validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Subject being validated.
    pub subject: String,
    /// All issues found.
    pub issues: Vec<ValidationIssue>,
    /// Overall pass/fail status.
    pub passed: bool,
    /// Summary statistics.
    pub stats: ValidationStats,
}

/// Validation statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationStats {
    /// Number of checks run.
    pub checks_run: usize,
    /// Number of info messages.
    pub info_count: usize,
    /// Number of warnings.
    pub warning_count: usize,
    /// Number of errors.
    pub error_count: usize,
}

impl ValidationReport {
    /// Create a new report.
    #[must_use]
    pub fn new(subject: &str) -> Self {
        Self {
            subject: subject.to_string(),
            issues: Vec::new(),
            passed: true,
            stats: ValidationStats::default(),
        }
    }

    /// Add an issue.
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.update_stats(issue.severity);
        self.issues.push(issue);
    }

    fn update_stats(&mut self, severity: DiagnosticLevel) {
        match severity {
            DiagnosticLevel::Info => self.stats.info_count += 1,
            DiagnosticLevel::Warning => self.stats.warning_count += 1,
            DiagnosticLevel::Error => {
                self.stats.error_count += 1;
                self.passed = false;
            }
        }
    }

    /// Increment checks run.
    pub fn check(&mut self) {
        self.stats.checks_run += 1;
    }

    /// Get errors only.
    #[must_use]
    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == DiagnosticLevel::Error)
            .collect()
    }

    /// Get warnings only.
    #[must_use]
    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == DiagnosticLevel::Warning)
            .collect()
    }

    /// Format as text report.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Validation Report: {}\n", self.subject));
        out.push_str(&format!(
            "Status: {}\n",
            if self.passed { "PASSED" } else { "FAILED" }
        ));
        out.push_str(&format!(
            "Checks: {} | Errors: {} | Warnings: {} | Info: {}\n",
            self.stats.checks_run,
            self.stats.error_count,
            self.stats.warning_count,
            self.stats.info_count
        ));
        self.append_issues(&mut out);
        out
    }

    fn append_issues(&self, out: &mut String) {
        if self.issues.is_empty() {
            return;
        }
        out.push_str("\nIssues:\n");
        for issue in &self.issues {
            out.push_str(&format!(
                "  [{}] {}: {}\n",
                issue.severity, issue.code, issue.message
            ));
            if let Some(ctx) = &issue.context {
                out.push_str(&format!("       Context: {}\n", ctx));
            }
        }
    }
}

/// Context for cycle detection traversal.
struct CycleContext {
    visited: HashSet<LexPrimitiva>,
    stack: HashSet<LexPrimitiva>,
}

impl CycleContext {
    fn new() -> Self {
        Self {
            visited: HashSet::new(),
            stack: HashSet::new(),
        }
    }

    fn check_node(&mut self, node: LexPrimitiva) -> Option<bool> {
        if self.stack.contains(&node) {
            return Some(true);
        }
        if self.visited.contains(&node) {
            return Some(false);
        }
        None
    }

    fn enter(&mut self, node: LexPrimitiva) {
        self.visited.insert(node);
        self.stack.insert(node);
    }

    fn exit(&mut self, node: LexPrimitiva) {
        self.stack.remove(&node);
    }
}

/// Validator for Lex Primitiva structures.
#[derive(Debug, Clone, Default)]
pub struct PrimitivaValidator {
    /// Whether to include info-level checks.
    pub include_info: bool,
    /// Whether to check grounding integrity.
    pub check_grounding: bool,
}

impl PrimitivaValidator {
    /// Create a new validator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            include_info: true,
            check_grounding: true,
        }
    }

    /// Create a strict validator (errors only).
    #[must_use]
    pub fn strict() -> Self {
        Self {
            include_info: false,
            check_grounding: true,
        }
    }

    /// Validate a single primitive.
    #[must_use]
    pub fn validate_primitive(&self, primitive: LexPrimitiva) -> ValidationReport {
        let mut report = ValidationReport::new(&format!("Primitive: {}", primitive.name()));
        self.check_primitive_symbol(&mut report, primitive);
        self.check_primitive_grounding(&mut report, primitive);
        self.check_primitive_atoms(&mut report, primitive);
        report
    }

    /// Validate a composition.
    #[must_use]
    pub fn validate_composition(&self, composition: &PrimitiveComposition) -> ValidationReport {
        let tier = Tier::classify(composition);
        let mut report = ValidationReport::new(&format!("Composition: {} tier", tier.code()));
        self.check_composition_non_empty(&mut report, composition);
        self.check_composition_dominant(&mut report, composition);
        self.check_composition_confidence(&mut report, composition);
        self.check_composition_tier(&mut report, tier);
        report
    }

    /// Validate the entire system.
    #[must_use]
    pub fn validate_system(&self) -> ValidationReport {
        let mut report = ValidationReport::new("Lex Primitiva System");
        self.check_root_count(&mut report);
        self.check_dag_acyclicity(&mut report);
        self.check_all_grounded(&mut report);
        self.check_atom_coverage(&mut report);
        self.check_constant_coverage(&mut report);
        report
    }

    fn check_primitive_symbol(&self, report: &mut ValidationReport, p: LexPrimitiva) {
        report.check();
        if p.symbol().is_empty() {
            report.add_issue(ValidationIssue::error(
                "LP-001",
                "Primitive has empty symbol",
            ));
        }
        if self.include_info {
            let issue = ValidationIssue::info("LP-INFO", &format!("Symbol: {}", p.symbol()));
            report.add_issue(issue.with_context(&p.name()));
        }
    }

    fn check_primitive_grounding(&self, report: &mut ValidationReport, p: LexPrimitiva) {
        if !self.check_grounding {
            return;
        }
        report.check();
        let constants = DependencyGraph::constants_for_primitive(p);
        self.check_constants_exist(report, p, &constants);
        self.check_root_reachable(report, p, &constants);
    }

    fn check_constants_exist(
        &self,
        report: &mut ValidationReport,
        p: LexPrimitiva,
        constants: &HashSet<&str>,
    ) {
        if constants.is_empty() {
            let issue = ValidationIssue::error("LP-002", "Primitive has no grounding constants");
            report.add_issue(issue.with_context(&p.name()));
        }
    }

    fn check_root_reachable(
        &self,
        report: &mut ValidationReport,
        p: LexPrimitiva,
        constants: &HashSet<&str>,
    ) {
        let has_root = constants.contains("0") || constants.contains("1");
        if !has_root {
            let msg = format!("{} grounds to: {:?}", p.name(), constants);
            let issue = ValidationIssue::error(
                "LP-003",
                "Primitive does not ground to root constants {0, 1}",
            );
            report.add_issue(issue.with_context(&msg));
        }
    }

    fn check_primitive_atoms(&self, report: &mut ValidationReport, p: LexPrimitiva) {
        report.check();
        let atoms = BedrockAtom::for_primitive(p);
        if atoms.len() != 5 {
            let issue = ValidationIssue::error(
                "LP-004",
                &format!("Expected 5 atoms, found {}", atoms.len()),
            );
            report.add_issue(issue.with_context(&p.name()));
        }
    }

    fn check_composition_non_empty(
        &self,
        report: &mut ValidationReport,
        comp: &PrimitiveComposition,
    ) {
        report.check();
        if comp.primitives.is_empty() {
            report.add_issue(ValidationIssue::error("COMP-001", "Composition is empty"));
        }
    }

    fn check_composition_dominant(
        &self,
        report: &mut ValidationReport,
        comp: &PrimitiveComposition,
    ) {
        report.check();
        let Some(dom) = comp.dominant else {
            return;
        };
        if !comp.primitives.contains(&dom) {
            let issue = ValidationIssue::error("COMP-002", "Dominant not in primitives");
            report.add_issue(issue.with_context(&format!("{:?}", dom)));
        }
    }

    fn check_composition_confidence(
        &self,
        report: &mut ValidationReport,
        comp: &PrimitiveComposition,
    ) {
        report.check();
        if !(0.0..=1.0).contains(&comp.confidence) {
            let issue = ValidationIssue::error("COMP-003", "Confidence out of range [0, 1]");
            report.add_issue(issue.with_context(&format!("{}", comp.confidence)));
        }
        if comp.confidence < 0.5 && self.include_info {
            report.add_issue(ValidationIssue::warning(
                "COMP-004",
                "Low confidence composition",
            ));
        }
    }

    fn check_composition_tier(&self, report: &mut ValidationReport, tier: Tier) {
        report.check();
        if tier == Tier::T3DomainSpecific && self.include_info {
            report.add_issue(ValidationIssue::warning(
                "COMP-005",
                "T3-DomainSpecific tier has low transfer confidence (0.4)",
            ));
        }
    }

    fn check_root_count(&self, report: &mut ValidationReport) {
        report.check();
        let roots: Vec<_> = LexPrimitiva::all()
            .into_iter()
            .filter(|p| p.is_root())
            .collect();
        if roots.len() != 2 {
            report.add_issue(ValidationIssue::error(
                "SYS-001",
                &format!("Expected 2 roots, found {}", roots.len()),
            ));
        }
    }

    fn check_dag_acyclicity(&self, report: &mut ValidationReport) {
        report.check();
        for p in LexPrimitiva::all() {
            let mut ctx = CycleContext::new();
            if has_cycle(p, &mut ctx) {
                let issue = ValidationIssue::error("SYS-002", "Cycle detected in dependency graph");
                report.add_issue(issue.with_context(&p.name()));
            }
        }
    }

    fn check_all_grounded(&self, report: &mut ValidationReport) {
        report.check();
        for p in LexPrimitiva::all() {
            let constants = DependencyGraph::constants_for_primitive(p);
            let has_root = constants.contains("0") || constants.contains("1");
            if !has_root {
                report.add_issue(ValidationIssue::error(
                    "SYS-003",
                    &format!("{} not grounded to {{0, 1}}", p.name()),
                ));
            }
        }
    }

    fn check_atom_coverage(&self, report: &mut ValidationReport) {
        report.check();
        let total: usize = LexPrimitiva::all()
            .iter()
            .map(|p| BedrockAtom::for_primitive(*p).len())
            .sum();
        if total != 80 {
            report.add_issue(ValidationIssue::error(
                "SYS-004",
                &format!("Expected 80 atoms, found {}", total),
            ));
        }
    }

    fn check_constant_coverage(&self, report: &mut ValidationReport) {
        report.check();
        let constants = crate::constants::MathConstant::all();
        if constants.len() != 10 {
            report.add_issue(ValidationIssue::error(
                "SYS-005",
                &format!("Expected 10 constants, found {}", constants.len()),
            ));
        }
    }
}

/// Check for cycles in dependency graph (extracted for low nesting).
fn has_cycle(node: LexPrimitiva, ctx: &mut CycleContext) -> bool {
    if let Some(result) = ctx.check_node(node) {
        return result;
    }
    ctx.enter(node);
    let found = node.derives_from().iter().any(|dep| has_cycle(*dep, ctx));
    ctx.exit(node);
    found
}

/// Quick validation helper.
#[must_use]
pub fn validate_primitive(primitive: LexPrimitiva) -> bool {
    PrimitivaValidator::strict()
        .validate_primitive(primitive)
        .passed
}

/// Quick composition validation.
#[must_use]
pub fn validate_composition(composition: &PrimitiveComposition) -> bool {
    PrimitivaValidator::strict()
        .validate_composition(composition)
        .passed
}

/// Full system validation.
#[must_use]
pub fn validate_system() -> ValidationReport {
    PrimitivaValidator::new().validate_system()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_primitive_quantity() {
        let report = PrimitivaValidator::new().validate_primitive(LexPrimitiva::Quantity);
        assert!(report.passed);
    }

    #[test]
    fn test_validate_primitive_all() {
        let validator = PrimitivaValidator::strict();
        for p in LexPrimitiva::all() {
            let report = validator.validate_primitive(p);
            assert!(report.passed, "Failed for {:?}: {:?}", p, report.errors());
        }
    }

    #[test]
    fn test_validate_composition_valid() {
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Mapping]);
        let report = PrimitivaValidator::new().validate_composition(&comp);
        assert!(report.passed);
    }

    #[test]
    fn test_validate_composition_empty() {
        let comp = PrimitiveComposition::new(vec![]);
        let report = PrimitivaValidator::strict().validate_composition(&comp);
        assert!(!report.passed);
        assert!(report.errors().iter().any(|e| e.code == "COMP-001"));
    }

    #[test]
    fn test_validate_system() {
        let report = validate_system();
        assert!(
            report.passed,
            "System validation failed: {:?}",
            report.errors()
        );
    }

    #[test]
    fn test_diagnostic_level_ordering() {
        assert!(DiagnosticLevel::Info < DiagnosticLevel::Warning);
        assert!(DiagnosticLevel::Warning < DiagnosticLevel::Error);
    }

    #[test]
    fn test_issue_with_context() {
        let issue = ValidationIssue::error("TEST-001", "Test error").with_context("test context");
        assert_eq!(issue.context, Some("test context".to_string()));
    }

    #[test]
    fn test_report_text_output() {
        let report = PrimitivaValidator::new().validate_primitive(LexPrimitiva::Void);
        let text = report.to_text();
        assert!(text.contains("Validation Report"));
        assert!(text.contains("Primitive: Void"));
    }

    #[test]
    fn test_quick_validate_primitive() {
        assert!(validate_primitive(LexPrimitiva::Sequence));
    }

    #[test]
    fn test_quick_validate_composition() {
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        assert!(validate_composition(&comp));
    }
}
