//! Domain validator trait and implementations

use super::types::{CheckResult, CheckSeverity, ValidationLevel, ValidationStatus};
use std::path::Path;

/// Trait for domain-specific validators
///
/// Domain validators implement checks for each validation level.
/// The [`crate::UniversalValidator`] orchestrates level iteration.
pub trait DomainValidator: Send + Sync {
    /// Returns the domain name (e.g., "skill", "agent", "config")
    fn domain(&self) -> &str;

    /// Returns L1 coherence checks
    fn get_l1_checks(&self, target: &Path) -> Vec<CheckResult>;

    /// Returns L2 structural checks
    fn get_l2_checks(&self, target: &Path) -> Vec<CheckResult>;

    /// Returns L3 functional checks
    fn get_l3_checks(&self, target: &Path) -> Vec<CheckResult>;

    /// Returns L4 operational checks (optional)
    fn get_l4_checks(&self, _target: &Path) -> Vec<CheckResult> {
        Vec::new()
    }

    /// Returns L5 impact checks (optional)
    fn get_l5_checks(&self, _target: &Path) -> Vec<CheckResult> {
        Vec::new()
    }

    /// Returns all checks for a level
    fn get_checks(&self, level: ValidationLevel, target: &Path) -> Vec<CheckResult> {
        match level {
            ValidationLevel::L1Coherence => self.get_l1_checks(target),
            ValidationLevel::L2Structural => self.get_l2_checks(target),
            ValidationLevel::L3Functional => self.get_l3_checks(target),
            ValidationLevel::L4Operational => self.get_l4_checks(target),
            ValidationLevel::L5Impact => self.get_l5_checks(target),
        }
    }

    /// Returns which levels this domain implements
    fn implemented_levels(&self) -> Vec<ValidationLevel> {
        vec![
            ValidationLevel::L1Coherence,
            ValidationLevel::L2Structural,
            ValidationLevel::L3Functional,
        ]
    }
}

/// Generic domain validator for basic file/directory validation
///
/// Provides common checks that work for any domain:
/// - L1: Target exists, basic file checks
/// - L2: Directory structure checks
/// - L3: No functional checks (domain-specific)
pub struct GenericDomainValidator {
    domain_name: String,
}

impl GenericDomainValidator {
    /// Creates a new generic validator for the given domain
    #[must_use]
    pub fn new(domain: &str) -> Self {
        Self {
            domain_name: domain.to_string(),
        }
    }
}

impl DomainValidator for GenericDomainValidator {
    fn domain(&self) -> &str {
        &self.domain_name
    }

    fn get_l1_checks(&self, target: &Path) -> Vec<CheckResult> {
        let mut checks = Vec::new();

        // Check: Target exists
        let exists = target.exists();
        checks.push(if exists {
            CheckResult::pass("target_exists", ValidationLevel::L1Coherence)
                .with_message(&format!("Target exists: {}", target.display()))
        } else {
            CheckResult::fail(
                "target_exists",
                ValidationLevel::L1Coherence,
                &format!("Target does not exist: {}", target.display()),
            )
        });

        // If it's a file, check readability
        if exists && target.is_file() {
            match std::fs::read_to_string(target) {
                Ok(content) => {
                    checks.push(
                        CheckResult::pass("file_readable", ValidationLevel::L1Coherence)
                            .with_message("File is readable")
                            .with_evidence("size_bytes", serde_json::json!(content.len())),
                    );

                    // Check for obvious syntax issues (UTF-8)
                    checks.push(
                        CheckResult::pass("valid_utf8", ValidationLevel::L1Coherence)
                            .with_message("File contains valid UTF-8"),
                    );
                }
                Err(e) => {
                    checks.push(CheckResult::fail(
                        "file_readable",
                        ValidationLevel::L1Coherence,
                        &format!("Cannot read file: {}", e),
                    ));
                }
            }
        }

        // If it's a directory, check accessibility
        if exists && target.is_dir() {
            match std::fs::read_dir(target) {
                Ok(_) => {
                    checks.push(
                        CheckResult::pass("dir_accessible", ValidationLevel::L1Coherence)
                            .with_message("Directory is accessible"),
                    );
                }
                Err(e) => {
                    checks.push(CheckResult::fail(
                        "dir_accessible",
                        ValidationLevel::L1Coherence,
                        &format!("Cannot access directory: {}", e),
                    ));
                }
            }
        }

        checks
    }

    fn get_l2_checks(&self, target: &Path) -> Vec<CheckResult> {
        let mut checks = Vec::new();

        if !target.exists() {
            return checks;
        }

        if target.is_dir() {
            // Check for common expected files/directories
            let common_files = [
                ("README.md", false), // (file, required)
                ("Cargo.toml", false),
                ("package.json", false),
            ];

            for (file, required) in common_files {
                let path = target.join(file);
                if path.exists() {
                    checks.push(
                        CheckResult::pass(
                            &format!("{}_present", file.replace('.', "_")),
                            ValidationLevel::L2Structural,
                        )
                        .with_message(&format!("{} found", file))
                        .with_severity(if required {
                            CheckSeverity::Critical
                        } else {
                            CheckSeverity::Info
                        }),
                    );
                } else if required {
                    checks.push(CheckResult::fail(
                        &format!("{}_present", file.replace('.', "_")),
                        ValidationLevel::L2Structural,
                        &format!("Required file missing: {}", file),
                    ));
                }
            }

            // Check for src/ directory (common in code projects)
            let src_dir = target.join("src");
            if src_dir.exists() && src_dir.is_dir() {
                checks.push(
                    CheckResult::pass("src_directory", ValidationLevel::L2Structural)
                        .with_message("src/ directory present")
                        .with_severity(CheckSeverity::Info),
                );
            }
        }

        // If no structural checks apply, add a "pass" placeholder
        if checks.is_empty() {
            checks.push(
                CheckResult::pass("structure_ok", ValidationLevel::L2Structural)
                    .with_message("No structural requirements for this target type")
                    .with_severity(CheckSeverity::Info),
            );
        }

        checks
    }

    fn get_l3_checks(&self, _target: &Path) -> Vec<CheckResult> {
        // Generic validator doesn't do functional checks
        // Domain-specific validators should override this
        vec![CheckResult {
            name: "no_functional_checks".to_string(),
            status: ValidationStatus::White,
            message: "No functional checks defined for generic domain".to_string(),
            severity: CheckSeverity::Info,
            level: ValidationLevel::L3Functional,
            evidence: std::collections::HashMap::new(),
            duration_ms: 0.0,
        }]
    }
}

/// Skill domain validator
///
/// Validates Claude Code skills against Diamond v2 compliance
pub struct SkillDomainValidator;

impl DomainValidator for SkillDomainValidator {
    fn domain(&self) -> &str {
        "skill"
    }

    fn get_l1_checks(&self, target: &Path) -> Vec<CheckResult> {
        let mut checks = Vec::new();

        // Check: SKILL.md exists
        let skill_md = target.join("SKILL.md");
        if skill_md.exists() {
            checks.push(
                CheckResult::pass("skill_md_exists", ValidationLevel::L1Coherence)
                    .with_message("SKILL.md found"),
            );

            // Check frontmatter
            if let Ok(content) = std::fs::read_to_string(&skill_md) {
                if content.starts_with("---") {
                    checks.push(
                        CheckResult::pass("frontmatter_present", ValidationLevel::L1Coherence)
                            .with_message("YAML frontmatter present"),
                    );
                } else {
                    checks.push(CheckResult::fail(
                        "frontmatter_present",
                        ValidationLevel::L1Coherence,
                        "Missing YAML frontmatter (should start with ---)",
                    ));
                }
            }
        } else {
            checks.push(CheckResult::fail(
                "skill_md_exists",
                ValidationLevel::L1Coherence,
                "SKILL.md not found",
            ));
        }

        checks
    }

    fn get_l2_checks(&self, target: &Path) -> Vec<CheckResult> {
        let mut checks = Vec::new();

        // Check: scripts/ directory
        let scripts_dir = target.join("scripts");
        if scripts_dir.exists() && scripts_dir.is_dir() {
            checks.push(
                CheckResult::pass("scripts_dir", ValidationLevel::L2Structural)
                    .with_message("scripts/ directory present"),
            );
        } else {
            checks.push(
                CheckResult::warn(
                    "scripts_dir",
                    ValidationLevel::L2Structural,
                    "scripts/ directory not found",
                )
                .with_severity(CheckSeverity::Warning),
            );
        }

        // Check: references/ directory (Gold+)
        let refs_dir = target.join("references");
        if refs_dir.exists() && refs_dir.is_dir() {
            checks.push(
                CheckResult::pass("references_dir", ValidationLevel::L2Structural)
                    .with_message("references/ directory present (Gold+)")
                    .with_severity(CheckSeverity::Info),
            );
        }

        // Check: templates/ directory (Gold+)
        let templates_dir = target.join("templates");
        if templates_dir.exists() && templates_dir.is_dir() {
            checks.push(
                CheckResult::pass("templates_dir", ValidationLevel::L2Structural)
                    .with_message("templates/ directory present (Gold+)")
                    .with_severity(CheckSeverity::Info),
            );
        }

        // Check: verify.py or verify.rs (Platinum+)
        let has_verify = target.join("scripts/verify.py").exists()
            || target.join("scripts/verify.rs").exists()
            || target.join("verify.py").exists();
        if has_verify {
            checks.push(
                CheckResult::pass("verify_script", ValidationLevel::L2Structural)
                    .with_message("Verification script present (Platinum+)")
                    .with_severity(CheckSeverity::Info),
            );
        }

        checks
    }

    fn get_l3_checks(&self, target: &Path) -> Vec<CheckResult> {
        let mut checks = Vec::new();

        // Check: Machine Specification section in SKILL.md
        let skill_md = target.join("SKILL.md");
        if let Ok(content) = std::fs::read_to_string(&skill_md) {
            // Check for SMST sections
            let smst_sections = [
                ("INPUTS", "### INPUTS"),
                ("OUTPUTS", "### OUTPUTS"),
                ("STATE", "### STATE"),
                ("OPERATOR MODE", "### OPERATOR MODE"),
                ("INVARIANTS", "### INVARIANTS"),
            ];

            for (name, marker) in smst_sections {
                if content.contains(marker) {
                    checks.push(
                        CheckResult::pass(
                            &format!("smst_{}", name.to_lowercase().replace(' ', "_")),
                            ValidationLevel::L3Functional,
                        )
                        .with_message(&format!("{} section present", name)),
                    );
                } else {
                    checks.push(
                        CheckResult::warn(
                            &format!("smst_{}", name.to_lowercase().replace(' ', "_")),
                            ValidationLevel::L3Functional,
                            &format!("{} section not found", name),
                        )
                        .with_severity(CheckSeverity::Warning),
                    );
                }
            }
        }

        checks
    }

    fn implemented_levels(&self) -> Vec<ValidationLevel> {
        vec![
            ValidationLevel::L1Coherence,
            ValidationLevel::L2Structural,
            ValidationLevel::L3Functional,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generic_validator_nonexistent() {
        let validator = GenericDomainValidator::new("test");
        let checks = validator.get_l1_checks(Path::new("/nonexistent/path"));
        assert!(!checks.is_empty());
        assert!(
            checks
                .iter()
                .any(|c| c.name == "target_exists" && !c.passed())
        );
    }

    #[test]
    fn test_generic_validator_existing_dir() {
        let dir = tempdir().unwrap();
        let validator = GenericDomainValidator::new("test");
        let checks = validator.get_l1_checks(dir.path());
        assert!(
            checks
                .iter()
                .any(|c| c.name == "target_exists" && c.passed())
        );
        assert!(
            checks
                .iter()
                .any(|c| c.name == "dir_accessible" && c.passed())
        );
    }

    #[test]
    fn test_skill_validator_missing_skill_md() {
        let dir = tempdir().unwrap();
        let validator = SkillDomainValidator;
        let checks = validator.get_l1_checks(dir.path());
        assert!(
            checks
                .iter()
                .any(|c| c.name == "skill_md_exists" && !c.passed())
        );
    }
}
