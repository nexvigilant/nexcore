//! Validation engine for academy content.
//!
//! Validates generated academy content against:
//! - Schema rules (R1-R8): structural correctness
//! - Accuracy rules (R9-R14): factual correctness vs IR
//! - Conventions rules (R15-R19): consistency
//! - Progression rules (R20-R23): difficulty curve
//! - Experiential rules (R24-R27): learning theory compliance

pub mod accuracy;
pub mod alo;
pub mod conventions;
pub mod experiential;
pub mod progression;
pub mod schema;

use serde::{Deserialize, Serialize};

use crate::ir::DomainAnalysis;

/// Severity level for a validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Must fix — content is structurally broken or factually wrong.
    Error,
    /// Should fix — content is inconsistent or incomplete.
    Warning,
    /// Consider fixing — advisory for better quality.
    Advisory,
}

/// A single validation finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFinding {
    /// Rule identifier (e.g., "R1", "R14").
    pub rule: String,
    /// Severity level.
    pub severity: Severity,
    /// Human-readable description.
    pub message: String,
    /// JSON path to the offending field (if applicable).
    pub field_path: Option<String>,
}

/// Complete validation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Whether the content passes all error-level rules.
    pub passed: bool,
    /// Total number of findings.
    pub total_findings: usize,
    /// Error count.
    pub error_count: usize,
    /// Warning count.
    pub warning_count: usize,
    /// Advisory count.
    pub advisory_count: usize,
    /// All findings.
    pub findings: Vec<ValidationFinding>,
}

/// Validate academy content against all rules.
///
/// If `domain` is provided, accuracy rules (R9-R14) are also checked.
pub fn validate(content: &serde_json::Value, domain: Option<&DomainAnalysis>) -> ValidationReport {
    let mut findings = Vec::new();

    // Schema rules (R1-R8)
    findings.extend(schema::validate_schema(content));

    // Accuracy rules (R9-R14) — only if domain IR is available
    if let Some(domain) = domain {
        findings.extend(accuracy::validate_accuracy(content, domain));
    }

    // Conventions rules (R15-R19)
    findings.extend(conventions::validate_conventions(content));

    // Progression rules (R20-R23)
    findings.extend(progression::validate_progression(content));

    // Experiential rules (R24-R27)
    findings.extend(experiential::validate_experiential(content));

    let error_count = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Error))
        .count();
    let warning_count = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Warning))
        .count();
    let advisory_count = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Advisory))
        .count();

    ValidationReport {
        passed: error_count == 0,
        total_findings: findings.len(),
        error_count,
        warning_count,
        advisory_count,
        findings,
    }
}
