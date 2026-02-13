//! FAERS Data Validation Module
//!
//! Validates FAERS (FDA Adverse Event Reporting System) data before signal detection.
//!
//! # Key Validations
//!
//! 1. **Structure** - Required fields present
//! 2. **Completeness** - Missing data assessment
//! 3. **Semantic** - Value ranges and formats
//! 4. **Referential** - Drug/event coding consistency
//! 5. **Temporal** - Date logic and reporting delays

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Validation Severity
// =============================================================================

/// Severity levels for validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationSeverity {
    /// Critical - data cannot be used
    Error,
    /// Concerning - may affect analysis quality
    Warning,
    /// Informational - good to know
    Info,
}

impl ValidationSeverity {
    /// Check if this severity blocks validation
    #[must_use]
    pub const fn is_blocking(&self) -> bool {
        matches!(self, Self::Error)
    }
}

impl std::fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warning => write!(f, "WARNING"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

// =============================================================================
// Validation Category
// =============================================================================

/// Categories of validation checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationCategory {
    /// Required fields present
    Structure,
    /// Missing data assessment
    Completeness,
    /// Value ranges and formats
    Semantic,
    /// Coding consistency
    Referential,
    /// Date logic
    Temporal,
}

impl std::fmt::Display for ValidationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Structure => write!(f, "STRUCTURE"),
            Self::Completeness => write!(f, "COMPLETENESS"),
            Self::Semantic => write!(f, "SEMANTIC"),
            Self::Referential => write!(f, "REFERENTIAL"),
            Self::Temporal => write!(f, "TEMPORAL"),
        }
    }
}

// =============================================================================
// Validation Issue
// =============================================================================

/// A single validation issue found in the data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Category of the validation check
    pub category: ValidationCategory,
    /// Severity of the issue
    pub severity: ValidationSeverity,
    /// Field that failed validation
    pub field: String,
    /// Human-readable description
    pub message: String,
    /// Optional value that caused the issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl ValidationIssue {
    /// Create a new validation issue
    #[must_use]
    pub fn new(
        category: ValidationCategory,
        severity: ValidationSeverity,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            severity,
            field: field.into(),
            message: message.into(),
            value: None,
        }
    }

    /// Create a new validation issue with a value
    #[must_use]
    pub fn with_value(
        category: ValidationCategory,
        severity: ValidationSeverity,
        field: impl Into<String>,
        message: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            category,
            severity,
            field: field.into(),
            message: message.into(),
            value: Some(value.into()),
        }
    }

    /// Create an error issue
    #[must_use]
    pub fn error(
        category: ValidationCategory,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::new(category, ValidationSeverity::Error, field, message)
    }

    /// Create a warning issue
    #[must_use]
    pub fn warning(
        category: ValidationCategory,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::new(category, ValidationSeverity::Warning, field, message)
    }

    /// Create an info issue
    #[must_use]
    pub fn info(
        category: ValidationCategory,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::new(category, ValidationSeverity::Info, field, message)
    }
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {} - {}",
            self.severity, self.category, self.field, self.message
        )?;
        if let Some(ref value) = self.value {
            write!(f, " (value: {value})")?;
        }
        Ok(())
    }
}

// =============================================================================
// Validation Result
// =============================================================================

/// Result of validating a FAERS report or dataset.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed (no errors)
    pub is_valid: bool,
    /// List of all issues found
    pub issues: Vec<ValidationIssue>,
    /// Total records validated
    pub record_count: u64,
    /// Records that passed validation
    pub valid_count: u64,
    /// Number of error-level issues
    pub error_count: u64,
    /// Number of warning-level issues
    pub warning_count: u64,
}

impl ValidationResult {
    /// Create a new validation result (initially valid)
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_valid: true,
            issues: Vec::new(),
            record_count: 0,
            valid_count: 0,
            error_count: 0,
            warning_count: 0,
        }
    }

    /// Create a new validation result for a single record
    #[must_use]
    pub fn for_record() -> Self {
        Self {
            is_valid: true,
            issues: Vec::new(),
            record_count: 1,
            valid_count: 0,
            error_count: 0,
            warning_count: 0,
        }
    }

    /// Add a validation issue
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        match issue.severity {
            ValidationSeverity::Error => {
                self.error_count += 1;
                self.is_valid = false;
            }
            ValidationSeverity::Warning => {
                self.warning_count += 1;
            }
            ValidationSeverity::Info => {}
        }
        self.issues.push(issue);
    }

    /// Add an error issue
    pub fn add_error(
        &mut self,
        category: ValidationCategory,
        field: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.add_issue(ValidationIssue::error(category, field, message));
    }

    /// Add a warning issue
    pub fn add_warning(
        &mut self,
        category: ValidationCategory,
        field: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.add_issue(ValidationIssue::warning(category, field, message));
    }

    /// Add an info issue
    pub fn add_info(
        &mut self,
        category: ValidationCategory,
        field: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.add_issue(ValidationIssue::info(category, field, message));
    }

    /// Finalize the result (update valid_count based on errors)
    pub fn finalize(&mut self) {
        if self.error_count == 0 {
            self.valid_count = self.record_count;
        }
    }

    /// Merge another validation result into this one
    pub fn merge(&mut self, other: ValidationResult) {
        self.issues.extend(other.issues);
        self.record_count += other.record_count;
        self.valid_count += other.valid_count;
        self.error_count += other.error_count;
        self.warning_count += other.warning_count;
        self.is_valid = self.error_count == 0;
    }

    /// Get issues by category
    #[must_use]
    pub fn issues_by_category(&self, category: ValidationCategory) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.category == category)
            .collect()
    }

    /// Get issues by severity
    #[must_use]
    pub fn issues_by_severity(&self, severity: ValidationSeverity) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .collect()
    }

    /// Get issues by field
    #[must_use]
    pub fn issues_by_field(&self, field: &str) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|i| i.field == field).collect()
    }

    /// Generate summary statistics
    #[must_use]
    pub fn summary(&self) -> ValidationSummary {
        let mut by_category: HashMap<ValidationCategory, u64> = HashMap::new();
        let mut by_field: HashMap<String, u64> = HashMap::new();

        for issue in &self.issues {
            *by_category.entry(issue.category).or_default() += 1;
            *by_field.entry(issue.field.clone()).or_default() += 1;
        }

        ValidationSummary {
            is_valid: self.is_valid,
            record_count: self.record_count,
            valid_count: self.valid_count,
            error_count: self.error_count,
            warning_count: self.warning_count,
            issues_by_category: by_category,
            issues_by_field: by_field,
        }
    }
}

/// Summary statistics for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Whether validation passed
    pub is_valid: bool,
    /// Total records
    pub record_count: u64,
    /// Valid records
    pub valid_count: u64,
    /// Error count
    pub error_count: u64,
    /// Warning count
    pub warning_count: u64,
    /// Issues grouped by category
    pub issues_by_category: HashMap<ValidationCategory, u64>,
    /// Issues grouped by field
    pub issues_by_field: HashMap<String, u64>,
}

// =============================================================================
// Contingency Table Validation
// =============================================================================

/// Validate a 2x2 contingency table for signal detection.
///
/// # Arguments
///
/// * `a` - Drug + Event count
/// * `b` - Drug + No Event count
/// * `c` - No Drug + Event count
/// * `d` - No Drug + No Event count
///
/// # Returns
///
/// `ValidationResult` with any issues found
#[must_use]
pub fn validate_contingency_table(a: i64, b: i64, c: i64, d: i64) -> ValidationResult {
    let mut result = ValidationResult::for_record();

    // All values must be non-negative
    if a < 0 || b < 0 || c < 0 || d < 0 {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Error,
            "contingency_table",
            "Contingency table values must be non-negative",
            format!("a={a}, b={b}, c={c}, d={d}"),
        ));
    }

    // Total must be positive
    let total = a + b + c + d;
    if total == 0 {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Error,
            "contingency_table",
            "Contingency table total is zero",
            format!("a={a}, b={b}, c={c}, d={d}"),
        ));
    }

    // Warn if margins are zero (undefined ratios)
    if a + b == 0 {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Warning,
            "contingency_table",
            "No drug reports (a+b=0)",
            format!("a={a}, b={b}"),
        ));
    }

    if c + d == 0 {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Warning,
            "contingency_table",
            "No non-drug reports (c+d=0)",
            format!("c={c}, d={d}"),
        ));
    }

    if a + c == 0 {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Warning,
            "contingency_table",
            "No event reports (a+c=0)",
            format!("a={a}, c={c}"),
        ));
    }

    // Warn if cell counts are low (unstable estimates)
    if a >= 0 && a < 3 {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Warning,
            "contingency_table",
            format!("Low drug+event count (a={a}), estimates may be unstable"),
            format!("a={a}"),
        ));
    }

    result.finalize();
    result
}

/// Validate age value is within reasonable bounds
///
/// # Arguments
///
/// * `age` - Age value (already normalized to years)
///
/// # Returns
///
/// `ValidationResult` with any issues found
#[must_use]
pub fn validate_age(age: f64) -> ValidationResult {
    let mut result = ValidationResult::for_record();

    if age < 0.0 {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Error,
            "age",
            "Age cannot be negative",
            format!("{age}"),
        ));
    } else if age > 150.0 {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Error,
            "age",
            "Age exceeds maximum plausible value",
            format!("{age}"),
        ));
    } else if age > 120.0 {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Warning,
            "age",
            "Age is unusually high",
            format!("{age}"),
        ));
    }

    result.finalize();
    result
}

/// Validate weight value is within reasonable bounds
///
/// # Arguments
///
/// * `weight` - Weight value (already normalized to kg)
///
/// # Returns
///
/// `ValidationResult` with any issues found
#[must_use]
pub fn validate_weight(weight: f64) -> ValidationResult {
    let mut result = ValidationResult::for_record();

    if weight <= 0.0 {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Error,
            "weight",
            "Weight must be positive",
            format!("{weight}"),
        ));
    } else if weight > 700.0 {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Error,
            "weight",
            "Weight exceeds maximum plausible value",
            format!("{weight} kg"),
        ));
    } else if weight < 0.5 {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Warning,
            "weight",
            "Weight is unusually low (premature infant?)",
            format!("{weight} kg"),
        ));
    }

    result.finalize();
    result
}

/// Validate sex code
#[must_use]
pub fn validate_sex(sex: &str) -> ValidationResult {
    let mut result = ValidationResult::for_record();

    let valid_codes = ["M", "F", "MALE", "FEMALE", "UNK", "UNKNOWN", "NS"];
    if !valid_codes.contains(&sex.to_uppercase().as_str()) {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Warning,
            "sex",
            "Non-standard sex value",
            sex.to_string(),
        ));
    }

    result.finalize();
    result
}

/// Validate drug role code
#[must_use]
pub fn validate_drug_role(role: &str) -> ValidationResult {
    let mut result = ValidationResult::for_record();

    let valid_roles = ["PS", "SS", "C", "I"];
    if !valid_roles.contains(&role.to_uppercase().as_str()) {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Warning,
            "drug_role",
            "Invalid drug role code",
            role.to_string(),
        ));
    }

    result.finalize();
    result
}

/// Validate country code (ISO 3166-1 alpha-2)
#[must_use]
pub fn validate_country_code(code: &str) -> ValidationResult {
    let mut result = ValidationResult::for_record();

    // Basic format check: 2 uppercase letters
    let trimmed = code.trim().to_uppercase();
    if trimmed.len() != 2 || !trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        result.add_issue(ValidationIssue::with_value(
            ValidationCategory::Semantic,
            ValidationSeverity::Info,
            "country_code",
            "Non-ISO 3166-1 alpha-2 country code",
            code.to_string(),
        ));
    }

    result.finalize();
    result
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_severity_display() {
        assert_eq!(format!("{}", ValidationSeverity::Error), "ERROR");
        assert_eq!(format!("{}", ValidationSeverity::Warning), "WARNING");
        assert_eq!(format!("{}", ValidationSeverity::Info), "INFO");
    }

    #[test]
    fn test_validation_severity_is_blocking() {
        assert!(ValidationSeverity::Error.is_blocking());
        assert!(!ValidationSeverity::Warning.is_blocking());
        assert!(!ValidationSeverity::Info.is_blocking());
    }

    #[test]
    fn test_validation_result_add_issue() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid);
        assert_eq!(result.error_count, 0);

        result.add_error(ValidationCategory::Structure, "test_field", "Test error");
        assert!(!result.is_valid);
        assert_eq!(result.error_count, 1);

        result.add_warning(ValidationCategory::Semantic, "test_field", "Test warning");
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.issues.len(), 2);
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::new();
        result1.record_count = 10;
        result1.add_error(ValidationCategory::Structure, "field1", "Error 1");

        let mut result2 = ValidationResult::new();
        result2.record_count = 5;
        result2.add_warning(ValidationCategory::Semantic, "field2", "Warning 1");

        result1.merge(result2);

        assert_eq!(result1.record_count, 15);
        assert_eq!(result1.error_count, 1);
        assert_eq!(result1.warning_count, 1);
        assert_eq!(result1.issues.len(), 2);
        assert!(!result1.is_valid);
    }

    #[test]
    fn test_contingency_table_valid() {
        let result = validate_contingency_table(100, 200, 150, 5000);
        assert!(result.is_valid);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_contingency_table_negative() {
        let result = validate_contingency_table(-1, 200, 150, 5000);
        assert!(!result.is_valid);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_contingency_table_zero_total() {
        let result = validate_contingency_table(0, 0, 0, 0);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_contingency_table_low_count_warning() {
        let result = validate_contingency_table(2, 200, 150, 5000);
        assert!(result.is_valid); // Low count is a warning, not error
        assert!(result.warning_count > 0);
    }

    #[test]
    fn test_validate_age() {
        // Valid ages
        assert!(validate_age(25.0).is_valid);
        assert!(validate_age(0.5).is_valid);
        assert!(validate_age(100.0).is_valid);

        // Invalid ages
        assert!(!validate_age(-5.0).is_valid);
        assert!(!validate_age(200.0).is_valid);

        // Warning for unusual but possible ages
        let result = validate_age(125.0);
        assert!(result.is_valid);
        assert!(result.warning_count > 0);
    }

    #[test]
    fn test_validate_weight() {
        // Valid weights
        assert!(validate_weight(70.0).is_valid);
        assert!(validate_weight(5.0).is_valid);
        assert!(validate_weight(150.0).is_valid);

        // Invalid weights
        assert!(!validate_weight(-10.0).is_valid);
        assert!(!validate_weight(0.0).is_valid);
        assert!(!validate_weight(800.0).is_valid);

        // Warning for very low weight
        let result = validate_weight(0.3);
        assert!(result.is_valid);
        assert!(result.warning_count > 0);
    }

    #[test]
    fn test_validate_sex() {
        assert!(validate_sex("M").is_valid);
        assert!(validate_sex("F").is_valid);
        assert!(validate_sex("UNK").is_valid);
        assert!(validate_sex("male").is_valid);

        // Non-standard value gets warning
        let result = validate_sex("X");
        assert!(result.is_valid);
        assert!(result.warning_count > 0);
    }

    #[test]
    fn test_validate_drug_role() {
        assert!(validate_drug_role("PS").is_valid);
        assert!(validate_drug_role("SS").is_valid);
        assert!(validate_drug_role("C").is_valid);
        assert!(validate_drug_role("I").is_valid);
        assert!(validate_drug_role("ps").is_valid); // case insensitive

        // Invalid role gets warning
        let result = validate_drug_role("X");
        assert!(result.is_valid);
        assert!(result.warning_count > 0);
    }

    #[test]
    fn test_validate_country_code() {
        assert!(validate_country_code("US").is_valid);
        assert!(validate_country_code("GB").is_valid);
        assert!(validate_country_code("jp").is_valid); // will be uppercased

        // Non-ISO codes get info
        let result = validate_country_code("USA");
        assert!(result.is_valid);
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_validation_summary() {
        let mut result = ValidationResult::new();
        result.record_count = 100;
        result.add_error(ValidationCategory::Structure, "field1", "Error 1");
        result.add_error(ValidationCategory::Structure, "field1", "Error 2");
        result.add_warning(ValidationCategory::Semantic, "field2", "Warning 1");

        let summary = result.summary();

        assert!(!summary.is_valid);
        assert_eq!(summary.error_count, 2);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(
            summary
                .issues_by_category
                .get(&ValidationCategory::Structure),
            Some(&2)
        );
        assert_eq!(summary.issues_by_field.get("field1"), Some(&2));
    }
}
