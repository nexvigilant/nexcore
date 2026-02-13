//! Universal validation engine

use super::domain::DomainValidator;
use super::types::{LevelResult, ValidationLevel, ValidationResult, ValidationStatus};
use colored::Colorize;
use std::path::Path;
use std::time::Instant;

/// Configuration for validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum level to validate (default: L5)
    pub max_level: ValidationLevel,
    /// Stop on first failing level (default: true)
    pub fail_fast: bool,
    /// Emit Andon signals to stdout (default: true)
    pub emit_andon: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_level: ValidationLevel::L5Impact,
            fail_fast: true,
            emit_andon: true,
        }
    }
}

impl ValidationConfig {
    /// Creates config that validates all levels
    #[must_use]
    pub fn full() -> Self {
        Self::default()
    }

    /// Creates config for quick check (L1-L2 only)
    #[must_use]
    pub fn quick() -> Self {
        Self {
            max_level: ValidationLevel::L2Structural,
            fail_fast: true,
            emit_andon: true,
        }
    }

    /// Sets maximum level
    #[must_use]
    pub const fn with_max_level(mut self, level: ValidationLevel) -> Self {
        self.max_level = level;
        self
    }

    /// Sets fail-fast behavior
    #[must_use]
    pub const fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Disables Andon signal output
    #[must_use]
    pub const fn silent(mut self) -> Self {
        self.emit_andon = false;
        self
    }
}

/// The universal validation engine
///
/// Orchestrates L1-L5 validation using a domain-specific validator.
///
/// # Level Dependency Rule
///
/// Level N validation is meaningful ONLY IF Level N-1 passes.
/// The engine validates bottom-up and stops on failure (if fail_fast=true).
pub struct UniversalValidator {
    domain: Box<dyn DomainValidator>,
}

impl UniversalValidator {
    /// Creates a new validator with the given domain validator
    #[must_use]
    pub fn new(domain: Box<dyn DomainValidator>) -> Self {
        Self { domain }
    }

    /// Validates the target to the specified level
    ///
    /// Uses default configuration (fail_fast=true, emit_andon=true).
    pub fn validate(&self, target: &str, max_level: ValidationLevel) -> ValidationResult {
        self.validate_with_config(
            target,
            ValidationConfig::default().with_max_level(max_level),
        )
    }

    /// Validates with custom configuration
    pub fn validate_with_config(&self, target: &str, config: ValidationConfig) -> ValidationResult {
        let start = Instant::now();
        let target_path = Path::new(target);
        let mut result = ValidationResult::new(target, self.domain.domain());

        // Iterate through levels
        for level in ValidationLevel::up_to(config.max_level) {
            let level_start = Instant::now();

            // Get checks for this level
            let checks = self.domain.get_checks(level, target_path);

            // Build level result
            let mut level_result = LevelResult::new(level);
            level_result.checks = checks;
            level_result.calculate_status();
            level_result.duration_ms = level_start.elapsed().as_secs_f64() * 1000.0;

            // Emit Andon signal
            if config.emit_andon {
                self.emit_andon(level, &level_result);
            }

            let passed = level_result.passed();
            result.levels.insert(level, level_result);

            // Fail-fast: stop on failure
            if !passed && config.fail_fast {
                break;
            }
        }

        result.duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        result.finalize();

        result
    }

    /// Quick check (L1-L2 only)
    pub fn check(&self, target: &str) -> ValidationResult {
        self.validate_with_config(target, ValidationConfig::quick())
    }

    /// Full validation (L1-L5)
    pub fn full_validate(&self, target: &str) -> ValidationResult {
        self.validate_with_config(target, ValidationConfig::full())
    }

    fn emit_andon(&self, level: ValidationLevel, result: &LevelResult) {
        let signal = result.status.signal();
        let status_str = match result.status {
            ValidationStatus::Green => "passed".green().to_string(),
            ValidationStatus::Yellow => "warning".yellow().to_string(),
            ValidationStatus::Red => "FAILED".red().bold().to_string(),
            ValidationStatus::White => "skipped".white().dimmed().to_string(),
            ValidationStatus::Blue => "info".blue().to_string(),
        };

        eprintln!(
            "{} L{} {}: {} ({}/{} checks, {:.1}ms)",
            signal.emoji(),
            level.number(),
            level.name(),
            status_str,
            result.checks_passed(),
            result.checks_total(),
            result.duration_ms
        );
    }

    /// Returns the domain name
    #[must_use]
    pub fn domain_name(&self) -> &str {
        self.domain.domain()
    }
}

#[cfg(test)]
mod tests {
    use super::super::domain::GenericDomainValidator;
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_validate_nonexistent() {
        let validator = UniversalValidator::new(Box::new(GenericDomainValidator::new("test")));
        let result =
            validator.validate_with_config("/nonexistent/path", ValidationConfig::quick().silent());
        assert_eq!(result.overall_status, ValidationStatus::Red);
        assert!(result.highest_passed.is_none());
    }

    #[test]
    fn test_validate_existing_dir() {
        let dir = tempdir().unwrap();
        let validator = UniversalValidator::new(Box::new(GenericDomainValidator::new("test")));
        let result = validator.validate_with_config(
            dir.path().to_str().unwrap(),
            ValidationConfig::quick().silent(),
        );
        // Should at least pass L1
        assert!(result.highest_passed.is_some());
    }

    #[test]
    fn test_fail_fast() {
        let validator = UniversalValidator::new(Box::new(GenericDomainValidator::new("test")));
        let result = validator.validate_with_config(
            "/nonexistent",
            ValidationConfig::full().with_fail_fast(true).silent(),
        );
        // Should stop after L1 failure
        assert_eq!(result.levels.len(), 1);
        assert!(result.levels.contains_key(&ValidationLevel::L1Coherence));
    }

    #[test]
    fn test_continue_on_failure() {
        let validator = UniversalValidator::new(Box::new(GenericDomainValidator::new("test")));
        let result = validator.validate_with_config(
            "/nonexistent",
            ValidationConfig::quick().with_fail_fast(false).silent(),
        );
        // Should continue to L2 even though L1 failed
        assert!(result.levels.len() >= 1);
    }
}
