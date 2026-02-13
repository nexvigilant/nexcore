//! Core types for universal validation

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// The five canonical validation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationLevel {
    /// L1: Coherence - Is this internally consistent?
    L1Coherence = 1,
    /// L2: Structural - Is this built correctly per spec?
    L2Structural = 2,
    /// L3: Functional - Does this produce correct outputs?
    L3Functional = 3,
    /// L4: Operational - Does this work reliably?
    L4Operational = 4,
    /// L5: Impact - Does this achieve outcomes?
    L5Impact = 5,
}

impl ValidationLevel {
    /// Returns the level number (1-5)
    #[must_use]
    pub const fn number(&self) -> u8 {
        *self as u8
    }

    /// Returns the level name
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::L1Coherence => "Coherence",
            Self::L2Structural => "Structural",
            Self::L3Functional => "Functional",
            Self::L4Operational => "Operational",
            Self::L5Impact => "Impact",
        }
    }

    /// Returns the core question this level answers
    #[must_use]
    pub const fn question(&self) -> &'static str {
        match self {
            Self::L1Coherence => "Is this internally consistent?",
            Self::L2Structural => "Is this built correctly per spec?",
            Self::L3Functional => "Does this produce correct outputs?",
            Self::L4Operational => "Does this work reliably?",
            Self::L5Impact => "Does this achieve outcomes?",
        }
    }

    /// Returns the typical timeframe for this level
    #[must_use]
    pub const fn timeframe(&self) -> &'static str {
        match self {
            Self::L1Coherence => "ms-seconds",
            Self::L2Structural => "seconds-minutes",
            Self::L3Functional => "hours-days",
            Self::L4Operational => "days-weeks",
            Self::L5Impact => "weeks-months",
        }
    }

    /// Returns the weight for score calculation (percentage)
    #[must_use]
    pub const fn weight(&self) -> u8 {
        match self {
            Self::L1Coherence => 10,
            Self::L2Structural => 20,
            Self::L3Functional => 30,
            Self::L4Operational => 25,
            Self::L5Impact => 15,
        }
    }

    /// Returns all levels in order
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::L1Coherence,
            Self::L2Structural,
            Self::L3Functional,
            Self::L4Operational,
            Self::L5Impact,
        ]
    }

    /// Returns levels up to and including the specified level
    #[must_use]
    pub fn up_to(max: Self) -> Vec<Self> {
        Self::all().into_iter().filter(|l| *l <= max).collect()
    }
}

impl fmt::Display for ValidationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "L{} {}", self.number(), self.name())
    }
}

/// Andon signal colors for validation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ValidationStatus {
    /// All checks passed
    Green,
    /// Passed with warnings
    Yellow,
    /// Failed - critical issues
    Red,
    /// Skipped - no checks defined
    White,
    /// Informational only
    Blue,
}

impl ValidationStatus {
    /// Returns true if validation passed (Green or Yellow)
    #[must_use]
    pub const fn passed(&self) -> bool {
        matches!(self, Self::Green | Self::Yellow)
    }

    /// Returns the Andon signal representation
    #[must_use]
    pub const fn signal(&self) -> AndonSignal {
        match self {
            Self::Green => AndonSignal::Green,
            Self::Yellow => AndonSignal::Yellow,
            Self::Red => AndonSignal::Red,
            Self::White => AndonSignal::White,
            Self::Blue => AndonSignal::Blue,
        }
    }

    /// Returns colored string for terminal output
    #[must_use]
    pub fn colored_string(&self) -> String {
        match self {
            Self::Green => "GREEN".green().to_string(),
            Self::Yellow => "YELLOW".yellow().to_string(),
            Self::Red => "RED".red().to_string(),
            Self::White => "WHITE".white().to_string(),
            Self::Blue => "BLUE".blue().to_string(),
        }
    }
}

impl fmt::Display for ValidationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.colored_string())
    }
}

/// Andon signal for visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AndonSignal {
    /// All clear
    Green,
    /// Warning - attention needed
    Yellow,
    /// Stop - problem detected
    Red,
    /// Skipped - not applicable
    White,
    /// Information
    Blue,
}

impl AndonSignal {
    /// Returns emoji representation
    #[must_use]
    pub const fn emoji(&self) -> &'static str {
        match self {
            Self::Green => "🟢",
            Self::Yellow => "🟡",
            Self::Red => "🔴",
            Self::White => "⚪",
            Self::Blue => "🔵",
        }
    }
}

/// Severity of a check
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckSeverity {
    /// Informational - doesn't affect pass/fail
    Info,
    /// Warning - can pass with warnings
    Warning,
    /// Critical - must pass
    Critical,
}

/// Result of a single validation check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Check name (identifier)
    pub name: String,
    /// Status (passed/failed)
    pub status: ValidationStatus,
    /// Human-readable message
    pub message: String,
    /// Severity level
    pub severity: CheckSeverity,
    /// Level this check belongs to
    pub level: ValidationLevel,
    /// Additional evidence/data
    #[serde(default)]
    pub evidence: HashMap<String, serde_json::Value>,
    /// Duration in milliseconds
    #[serde(default)]
    pub duration_ms: f64,
}

impl CheckResult {
    /// Creates a new passing check
    #[must_use]
    pub fn pass(name: &str, level: ValidationLevel) -> Self {
        Self {
            name: name.to_string(),
            status: ValidationStatus::Green,
            message: String::new(),
            severity: CheckSeverity::Critical,
            level,
            evidence: HashMap::new(),
            duration_ms: 0.0,
        }
    }

    /// Creates a new failing check
    #[must_use]
    pub fn fail(name: &str, level: ValidationLevel, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status: ValidationStatus::Red,
            message: message.to_string(),
            severity: CheckSeverity::Critical,
            level,
            evidence: HashMap::new(),
            duration_ms: 0.0,
        }
    }

    /// Creates a warning check
    #[must_use]
    pub fn warn(name: &str, level: ValidationLevel, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status: ValidationStatus::Yellow,
            message: message.to_string(),
            severity: CheckSeverity::Warning,
            level,
            evidence: HashMap::new(),
            duration_ms: 0.0,
        }
    }

    /// Sets the message
    #[must_use]
    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    /// Adds evidence
    #[must_use]
    pub fn with_evidence(mut self, key: &str, value: serde_json::Value) -> Self {
        self.evidence.insert(key.to_string(), value);
        self
    }

    /// Sets severity
    #[must_use]
    pub const fn with_severity(mut self, severity: CheckSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Returns true if check passed
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.status.passed()
    }
}

/// Result of validating a single level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelResult {
    /// The level
    pub level: ValidationLevel,
    /// Overall status for this level
    pub status: ValidationStatus,
    /// Individual check results
    pub checks: Vec<CheckResult>,
    /// Duration in milliseconds
    pub duration_ms: f64,
}

impl LevelResult {
    /// Creates a new level result
    #[must_use]
    pub fn new(level: ValidationLevel) -> Self {
        Self {
            level,
            status: ValidationStatus::White,
            checks: Vec::new(),
            duration_ms: 0.0,
        }
    }

    /// Returns true if level passed
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.status.passed()
    }

    /// Returns count of passed checks
    #[must_use]
    pub fn checks_passed(&self) -> usize {
        self.checks.iter().filter(|c| c.passed()).count()
    }

    /// Returns total check count
    #[must_use]
    pub fn checks_total(&self) -> usize {
        self.checks.len()
    }

    /// Calculates pass rate (0.0 - 1.0)
    #[must_use]
    pub fn pass_rate(&self) -> f64 {
        if self.checks.is_empty() {
            return 0.0;
        }
        self.checks_passed() as f64 / self.checks_total() as f64
    }

    /// Determines status from checks
    pub fn calculate_status(&mut self) {
        if self.checks.is_empty() {
            self.status = ValidationStatus::White;
            return;
        }

        let has_critical_failure = self
            .checks
            .iter()
            .any(|c| c.severity == CheckSeverity::Critical && c.status == ValidationStatus::Red);

        let has_warning = self
            .checks
            .iter()
            .any(|c| c.status == ValidationStatus::Yellow);

        self.status = if has_critical_failure {
            ValidationStatus::Red
        } else if has_warning {
            ValidationStatus::Yellow
        } else {
            ValidationStatus::Green
        };
    }
}

/// Complete validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Target that was validated
    pub target: String,
    /// Domain (e.g., "skill", "agent", "config")
    pub domain: String,
    /// Results per level
    pub levels: HashMap<ValidationLevel, LevelResult>,
    /// Highest level that passed
    pub highest_passed: Option<ValidationLevel>,
    /// Overall status
    pub overall_status: ValidationStatus,
    /// Overall score (0-100)
    pub score: f64,
    /// Total duration in milliseconds
    pub duration_ms: f64,
    /// ISO timestamp
    pub timestamp: String,
}

impl ValidationResult {
    /// Creates a new validation result
    #[must_use]
    pub fn new(target: &str, domain: &str) -> Self {
        Self {
            target: target.to_string(),
            domain: domain.to_string(),
            levels: HashMap::new(),
            highest_passed: None,
            overall_status: ValidationStatus::White,
            score: 0.0,
            duration_ms: 0.0,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Calculates overall status and score from level results
    pub fn finalize(&mut self) {
        // Find highest passed level
        self.highest_passed = None;
        for level in ValidationLevel::all() {
            if let Some(result) = self.levels.get(&level) {
                if result.passed() {
                    self.highest_passed = Some(level);
                } else {
                    break; // Stop at first failure (dependency rule)
                }
            }
        }

        // Calculate overall status
        let has_failure = self
            .levels
            .values()
            .any(|r| r.status == ValidationStatus::Red);
        let has_warning = self
            .levels
            .values()
            .any(|r| r.status == ValidationStatus::Yellow);
        let all_white = self
            .levels
            .values()
            .all(|r| r.status == ValidationStatus::White);

        self.overall_status = if all_white {
            ValidationStatus::White
        } else if has_failure {
            ValidationStatus::Red
        } else if has_warning {
            ValidationStatus::Yellow
        } else {
            ValidationStatus::Green
        };

        // Calculate weighted score
        let mut total_weight = 0u32;
        let mut weighted_score = 0.0;

        for level in ValidationLevel::all() {
            if let Some(result) = self.levels.get(&level) {
                if !result.checks.is_empty() {
                    let weight = u32::from(level.weight());
                    weighted_score += f64::from(weight) * result.pass_rate();
                    total_weight += weight;
                }
            }
        }

        self.score = if total_weight > 0 {
            (weighted_score / f64::from(total_weight)) * 100.0
        } else {
            0.0
        };
    }

    /// Converts to JSON
    ///
    /// # Errors
    /// Returns error if serialization fails
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Generates a text report
    #[must_use]
    pub fn report(&self) -> String {
        let mut r = String::new();
        r.push_str(&format!(
            "\n╔══════════════════════════════════════════════════════╗\n"
        ));
        r.push_str(&format!(
            "║  📋 Universal Validation: {:<27}║\n",
            truncate(&self.target, 27)
        ));
        r.push_str(&format!("║  Domain: {:<43}║\n", self.domain));
        r.push_str(&format!(
            "╠══════════════════════════════════════════════════════╣\n"
        ));

        for level in ValidationLevel::all() {
            if let Some(result) = self.levels.get(&level) {
                let signal = result.status.signal();
                r.push_str(&format!(
                    "║  {} L{} {:<12} {:>3}/{:<3} checks  {:>6.1}ms  ║\n",
                    signal.emoji(),
                    level.number(),
                    level.name(),
                    result.checks_passed(),
                    result.checks_total(),
                    result.duration_ms
                ));
            } else {
                r.push_str(&format!(
                    "║  ⚪ L{} {:<12} (not run)                    ║\n",
                    level.number(),
                    level.name()
                ));
            }
        }

        r.push_str(&format!(
            "╠══════════════════════════════════════════════════════╣\n"
        ));
        r.push_str(&format!(
            "║  Score: {:>5.1}%  Highest: {:<25}║\n",
            self.score,
            self.highest_passed
                .map(|l| format!("L{}", l.number()))
                .unwrap_or_else(|| "None".to_string())
        ));
        r.push_str(&format!(
            "║  Status: {:<44}║\n",
            format!("{:?}", self.overall_status)
        ));
        r.push_str(&format!(
            "╚══════════════════════════════════════════════════════╝\n"
        ));

        r
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_ordering() {
        assert!(ValidationLevel::L1Coherence < ValidationLevel::L2Structural);
        assert!(ValidationLevel::L4Operational < ValidationLevel::L5Impact);
    }

    #[test]
    fn test_level_weights_sum_to_100() {
        let total: u8 = ValidationLevel::all().iter().map(|l| l.weight()).sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn test_check_result_builders() {
        let pass = CheckResult::pass("test", ValidationLevel::L1Coherence);
        assert!(pass.passed());

        let fail = CheckResult::fail("test", ValidationLevel::L1Coherence, "error");
        assert!(!fail.passed());
    }

    #[test]
    fn test_level_result_status_calculation() {
        let mut result = LevelResult::new(ValidationLevel::L1Coherence);
        result
            .checks
            .push(CheckResult::pass("check1", ValidationLevel::L1Coherence));
        result
            .checks
            .push(CheckResult::pass("check2", ValidationLevel::L1Coherence));
        result.calculate_status();
        assert_eq!(result.status, ValidationStatus::Green);

        result.checks.push(CheckResult::fail(
            "check3",
            ValidationLevel::L1Coherence,
            "failed",
        ));
        result.calculate_status();
        assert_eq!(result.status, ValidationStatus::Red);
    }
}
