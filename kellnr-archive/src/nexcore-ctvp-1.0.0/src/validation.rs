//! Validation execution for CTVP phases.
//!
//! This module provides the core validation logic for each CTVP phase,
//! including the trait-based validator interface and default implementations.

use crate::capability::Capability;
use crate::error::{CtvpError, CtvpResult};
use crate::types::*;
use std::path::Path;
use std::time::Instant;

/// Trait for validating capabilities across CTVP phases.
pub trait Validatable {
    /// Returns the capability being validated
    fn get_capability(&self) -> &Capability;

    /// Phase 0: Preclinical - Mechanism validity
    fn validate_preclinical(&self) -> CtvpResult<ValidationResult>;

    /// Phase 1: Safety - Failure mode validation
    fn validate_phase1_safety(&self) -> CtvpResult<ValidationResult>;

    /// Phase 2: Efficacy - Capability achievement with real data
    fn validate_phase2_efficacy(&self) -> CtvpResult<ValidationResult>;

    /// Phase 3: Confirmation - Scale validation
    fn validate_phase3_confirmation(&self) -> CtvpResult<ValidationResult>;

    /// Phase 4: Surveillance - Ongoing correctness
    fn validate_phase4_surveillance(&self) -> CtvpResult<ValidationResult>;

    /// Validates all phases in order
    fn validate_all(&self) -> CtvpResult<Vec<ValidationResult>> {
        Ok(vec![
            self.validate_preclinical()?,
            self.validate_phase1_safety()?,
            self.validate_phase2_efficacy()?,
            self.validate_phase3_confirmation()?,
            self.validate_phase4_surveillance()?,
        ])
    }

    /// Validates up to and including the specified phase
    fn validate_through(&self, phase: ValidationPhase) -> CtvpResult<Vec<ValidationResult>> {
        let mut results = Vec::new();

        for p in ValidationPhase::get_all().into_iter() {
            results.push(match p {
                ValidationPhase::Preclinical => self.validate_preclinical()?,
                ValidationPhase::Phase1Safety => self.validate_phase1_safety()?,
                ValidationPhase::Phase2Efficacy => self.validate_phase2_efficacy()?,
                ValidationPhase::Phase3Confirmation => self.validate_phase3_confirmation()?,
                ValidationPhase::Phase4Surveillance => self.validate_phase4_surveillance()?,
            });

            if p == phase {
                break;
            }
        }

        Ok(results)
    }
}

/// Default capability validator implementation.
pub struct CapabilityValidator {
    capability: Capability,
    config: ValidatorConfig,
}

/// Configuration for the validator.
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Path to the deliverable being validated
    pub deliverable_path: Option<std::path::PathBuf>,

    /// Minimum coverage threshold for Phase 0
    pub min_coverage: f64,

    /// Minimum property test iterations for Phase 0
    pub min_property_iterations: u64,

    /// Fault injection coverage threshold for Phase 1
    pub fault_injection_coverage: f64,

    /// Shadow divergence threshold for Phase 3
    pub shadow_divergence_threshold: f64,

    /// Drift detection threshold for Phase 4
    pub drift_threshold: f64,

    /// Enable verbose output
    pub verbose: bool,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            deliverable_path: None,
            min_coverage: 0.80,
            min_property_iterations: 1000,
            fault_injection_coverage: 0.90,
            shadow_divergence_threshold: 0.001,
            drift_threshold: 0.10,
            verbose: false,
        }
    }
}

impl CapabilityValidator {
    /// Creates a new validator for a capability
    pub fn new(capability: Capability) -> Self {
        Self {
            capability,
            config: ValidatorConfig::default(),
        }
    }

    /// Creates a validator with custom configuration
    pub fn with_config(capability: Capability, config: ValidatorConfig) -> Self {
        Self { capability, config }
    }

    /// Sets the deliverable path
    pub fn for_deliverable(mut self, path: impl AsRef<Path>) -> Self {
        self.config.deliverable_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Extracts Phase 0 evidence from the deliverable
    fn extract_phase0_evidence(&self) -> CtvpResult<(Vec<Evidence>, EvidenceQuality)> {
        let path = self.get_deliverable_path()?;
        let mut evidence = Vec::new();
        let mut q_score = 0.0;
        let mut q_factors = 0;

        self.check_test_files(path, &mut evidence, &mut q_score, &mut q_factors);
        self.check_cargo_config(path, &mut evidence, &mut q_score, &mut q_factors);
        self.check_coverage_config(path, &mut evidence, &mut q_score, &mut q_factors);

        let quality = self.calculate_quality(q_score, q_factors);
        Ok((evidence, quality))
    }

    fn get_deliverable_path(&self) -> CtvpResult<&Path> {
        self.config
            .deliverable_path
            .as_deref()
            .ok_or(CtvpError::Config("No deliverable path configured".into()))
    }

    fn check_test_files(
        &self,
        path: &Path,
        ev: &mut Vec<Evidence>,
        score: &mut f64,
        factors: &mut usize,
    ) {
        let count = match count_test_files(path) {
            Ok(c) if c > 0 => c,
            _ => return,
        };

        let et = EvidenceType::TestResult {
            name: "Test file count".into(),
            passed: true,
            duration_ms: 0,
        };
        ev.push(Evidence::new(
            et,
            format!("Found {} test files", count),
            "file_scan",
        ));
        *score += if count >= 10 { 1.0 } else { 0.5 };
        *factors += 1;
    }

    fn check_cargo_config(
        &self,
        path: &Path,
        ev: &mut Vec<Evidence>,
        score: &mut f64,
        factors: &mut usize,
    ) {
        let content = match std::fs::read_to_string(path.join("Cargo.toml")) {
            Ok(c) => c,
            _ => return,
        };

        if content.contains("[dev-dependencies]") {
            self.add_cargo_evidence(
                ev,
                score,
                factors,
                "Dev dependencies configured",
                "has_dev_deps",
                0.5,
            );
        }
        if content.contains("proptest") || content.contains("quickcheck") {
            self.add_cargo_evidence(
                ev,
                score,
                factors,
                "Property testing configured",
                "has_property_tests",
                1.0,
            );
        }
    }

    fn add_cargo_evidence(
        &self,
        ev: &mut Vec<Evidence>,
        score: &mut f64,
        factors: &mut usize,
        desc: &str,
        field: &str,
        val: f64,
    ) {
        let et = EvidenceType::Snapshot {
            component: "Cargo.toml".into(),
            state: serde_json::json!({field: true}),
        };
        ev.push(Evidence::new(et, desc, "Cargo.toml"));
        *score += val;
        *factors += 1;
    }

    fn check_coverage_config(
        &self,
        path: &Path,
        ev: &mut Vec<Evidence>,
        score: &mut f64,
        factors: &mut usize,
    ) {
        if path.join("codecov.yml").exists() || path.join("tarpaulin.toml").exists() {
            ev.push(Evidence::new(
                EvidenceType::Coverage {
                    line_coverage: 0.0,
                    branch_coverage: None,
                    function_coverage: None,
                },
                "Coverage tooling configured",
                "coverage_config",
            ));
            *score += 0.5;
            *factors += 1;
        }
    }

    fn calculate_quality(&self, score: f64, factors: usize) -> EvidenceQuality {
        if factors == 0 {
            return EvidenceQuality::None;
        }
        let avg = score / factors as f64;
        if avg >= 0.8 {
            EvidenceQuality::Strong
        } else if avg >= 0.5 {
            EvidenceQuality::Moderate
        } else if avg > 0.0 {
            EvidenceQuality::Weak
        } else {
            EvidenceQuality::None
        }
    }

    /// Extracts Phase 1 evidence from the deliverable
    fn extract_phase1_evidence(&self) -> CtvpResult<(Vec<Evidence>, EvidenceQuality)> {
        let path = self.get_deliverable_path()?;
        let mut evidence = Vec::new();
        let mut q_score = 0.0;
        let mut q_factors = 0;

        self.check_chaos_patterns(path, &mut evidence, &mut q_score, &mut q_factors);
        self.check_timeout_patterns(path, &mut evidence, &mut q_score, &mut q_factors);

        Ok((evidence, self.calculate_quality(q_score, q_factors)))
    }

    fn check_chaos_patterns(
        &self,
        path: &Path,
        ev: &mut Vec<Evidence>,
        score: &mut f64,
        factors: &mut usize,
    ) {
        let p = [
            "toxiproxy",
            "chaos",
            "fault",
            "failure",
            "circuit_breaker",
            "resilience",
        ];
        if let Some(c) = self.find_patterns(path, &p) {
            let et = EvidenceType::Snapshot {
                component: "fault_injection".into(),
                state: serde_json::json!({"indicators": c}),
            };
            ev.push(Evidence::new(
                et,
                "Fault injection patterns detected",
                "code_scan",
            ));
            *score += 1.0;
            *factors += 1;
        }
    }

    fn check_timeout_patterns(
        &self,
        path: &Path,
        ev: &mut Vec<Evidence>,
        score: &mut f64,
        factors: &mut usize,
    ) {
        let p = ["timeout", "deadline", "retry", "backoff"];
        if let Some(c) = self.find_patterns(path, &p) {
            let et = EvidenceType::Snapshot {
                component: "timeout_config".into(),
                state: serde_json::json!({"patterns": c}),
            };
            ev.push(Evidence::new(
                et,
                "Timeout/retry patterns detected",
                "code_scan",
            ));
            *score += 0.5;
            *factors += 1;
        }
    }

    /// Extracts Phase 2 evidence from the deliverable
    fn extract_phase2_evidence(&self) -> CtvpResult<(Vec<Evidence>, EvidenceQuality)> {
        let path = self.get_deliverable_path()?;
        let mut evidence = Vec::new();
        let mut q_score = 0.0;
        let mut q_factors = 0;

        self.check_slo_patterns(path, &mut evidence, &mut q_score, &mut q_factors);
        self.check_integration_patterns(path, &mut evidence, &mut q_score, &mut q_factors);

        Ok((evidence, self.calculate_quality(q_score, q_factors)))
    }

    fn check_slo_patterns(
        &self,
        path: &Path,
        ev: &mut Vec<Evidence>,
        score: &mut f64,
        factors: &mut usize,
    ) {
        let p = ["slo", "sli", "sla", "latency_target", "error_budget"];
        if let Some(c) = self.find_patterns(path, &p) {
            let et = EvidenceType::Snapshot {
                component: "slo_config".into(),
                state: serde_json::json!({"patterns": c}),
            };
            ev.push(Evidence::new(
                et,
                "SLO/SLI definitions detected",
                "code_scan",
            ));
            *score += 1.0;
            *factors += 1;
        }
    }

    fn check_integration_patterns(
        &self,
        path: &Path,
        ev: &mut Vec<Evidence>,
        score: &mut f64,
        factors: &mut usize,
    ) {
        let p = ["integration", "e2e", "end_to_end"];
        if let Ok(c) = search_files_for_patterns(path, &p) {
            if !c.is_empty() {
                let et = EvidenceType::TestResult {
                    name: "Integration tests".into(),
                    passed: true,
                    duration_ms: 0,
                };
                ev.push(Evidence::new(
                    et,
                    "Integration/E2E tests detected",
                    "file_scan",
                ));
                *score += 0.75;
                *factors += 1;
            }
        }
    }

    /// Extracts Phase 3 evidence from the deliverable
    fn extract_phase3_evidence(&self) -> CtvpResult<(Vec<Evidence>, EvidenceQuality)> {
        let path = self.get_deliverable_path()?;
        let mut evidence = Vec::new();
        let mut q_score = 0.0;
        let mut q_factors = 0;

        self.check_deploy_patterns(path, &mut evidence, &mut q_score, &mut q_factors);

        Ok((evidence, self.calculate_quality(q_score, q_factors)))
    }

    fn check_deploy_patterns(
        &self,
        path: &Path,
        ev: &mut Vec<Evidence>,
        score: &mut f64,
        factors: &mut usize,
    ) {
        let p = [
            "canary",
            "shadow",
            "blue_green",
            "rollout",
            "feature_flag",
            "ab_test",
        ];
        if let Some(c) = self.find_patterns(path, &p) {
            let et = EvidenceType::Snapshot {
                component: "deployment_config".into(),
                state: serde_json::json!({"patterns": c}),
            };
            ev.push(Evidence::new(
                et,
                "Progressive deployment detected",
                "code_scan",
            ));
            *score += 1.0;
            *factors += 1;
        }
    }

    /// Extracts Phase 4 evidence from the deliverable
    fn extract_phase4_evidence(&self) -> CtvpResult<(Vec<Evidence>, EvidenceQuality)> {
        let path = self.get_deliverable_path()?;
        let mut evidence = Vec::new();
        let mut q_score = 0.0;
        let mut q_factors = 0;

        self.check_obs_patterns(path, &mut evidence, &mut q_score, &mut q_factors);
        self.check_drift_patterns(path, &mut evidence, &mut q_score, &mut q_factors);

        Ok((evidence, self.calculate_quality(q_score, q_factors)))
    }

    fn check_obs_patterns(
        &self,
        path: &Path,
        ev: &mut Vec<Evidence>,
        score: &mut f64,
        factors: &mut usize,
    ) {
        let p = [
            "metrics",
            "tracing",
            "prometheus",
            "opentelemetry",
            "datadog",
            "grafana",
            "alert",
        ];
        if let Some(c) = self.find_patterns(path, &p) {
            let et = EvidenceType::Snapshot {
                component: "observability".into(),
                state: serde_json::json!({"patterns": c}),
            };
            ev.push(Evidence::new(
                et,
                "Observability tooling detected",
                "code_scan",
            ));
            *score += 0.75;
            *factors += 1;
        }
    }

    fn check_drift_patterns(
        &self,
        path: &Path,
        ev: &mut Vec<Evidence>,
        score: &mut f64,
        factors: &mut usize,
    ) {
        let p = ["drift", "baseline", "anomaly", "psi", "kolmogorov"];
        if let Some(c) = self.find_patterns(path, &p) {
            let et = EvidenceType::Snapshot {
                component: "drift_detection".into(),
                state: serde_json::json!({"patterns": c}),
            };
            ev.push(Evidence::new(et, "Drift detection detected", "code_scan"));
            *score += 1.0;
            *factors += 1;
        }
    }

    fn find_patterns(&self, path: &Path, patterns: &[&str]) -> Option<Vec<String>> {
        let c = search_files_for_patterns(path, patterns).ok()?;
        if c.is_empty() { None } else { Some(c) }
    }
}

impl Validatable for CapabilityValidator {
    fn get_capability(&self) -> &Capability {
        &self.capability
    }

    fn validate_preclinical(&self) -> CtvpResult<ValidationResult> {
        self.run_validation(ValidationPhase::Preclinical, |s| {
            s.extract_phase0_evidence()
        })
    }

    fn validate_phase1_safety(&self) -> CtvpResult<ValidationResult> {
        self.run_validation(ValidationPhase::Phase1Safety, |s| {
            s.extract_phase1_evidence()
        })
    }

    fn validate_phase2_efficacy(&self) -> CtvpResult<ValidationResult> {
        self.run_validation(ValidationPhase::Phase2Efficacy, |s| {
            s.extract_phase2_evidence()
        })
    }

    fn validate_phase3_confirmation(&self) -> CtvpResult<ValidationResult> {
        self.run_validation(ValidationPhase::Phase3Confirmation, |s| {
            s.extract_phase3_evidence()
        })
    }

    fn validate_phase4_surveillance(&self) -> CtvpResult<ValidationResult> {
        self.run_validation(ValidationPhase::Phase4Surveillance, |s| {
            s.extract_phase4_evidence()
        })
    }
}

impl CapabilityValidator {
    fn run_validation<F>(
        &self,
        phase: ValidationPhase,
        extractor: F,
    ) -> CtvpResult<ValidationResult>
    where
        F: Fn(&Self) -> CtvpResult<(Vec<Evidence>, EvidenceQuality)>,
    {
        let start = Instant::now();
        let (evidence, quality) = extractor(self)?;
        let outcome = self.determine_outcome(phase, quality);

        let mut result = ValidationResult::new(&self.capability.id, phase, outcome, quality);
        for e in evidence {
            result = result.with_evidence(e);
        }
        Ok(result.with_duration(start.elapsed()))
    }

    fn determine_outcome(
        &self,
        phase: ValidationPhase,
        quality: EvidenceQuality,
    ) -> ValidationOutcome {
        match phase {
            ValidationPhase::Preclinical => {
                if quality >= EvidenceQuality::Weak {
                    ValidationOutcome::Validated
                } else {
                    ValidationOutcome::Failed {
                        reason: "No evidence".into(),
                    }
                }
            }
            _ => {
                if quality >= EvidenceQuality::Moderate {
                    ValidationOutcome::Validated
                } else if quality >= EvidenceQuality::Weak {
                    ValidationOutcome::Inconclusive {
                        reason: "Partial evidence".into(),
                    }
                } else {
                    ValidationOutcome::NotApplicable
                }
            }
        }
    }
}

// Helper functions

fn count_test_files(path: &Path) -> CtvpResult<usize> {
    use walkdir::WalkDir;

    let count = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            name.contains("test") || name.contains("spec")
        })
        .count();

    Ok(count)
}

fn search_files_for_patterns(path: &Path, patterns: &[&str]) -> CtvpResult<Vec<String>> {
    use walkdir::WalkDir;
    let mut found = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        process_pattern_file(entry.path(), patterns, &mut found);
    }
    Ok(found)
}

fn process_pattern_file(path: &Path, patterns: &[&str], found: &mut Vec<String>) {
    if should_skip_path(path) {
        return;
    }

    if let Ok(content) = std::fs::read_to_string(path) {
        let lower = content.to_lowercase();
        self::collect_found_patterns(path, patterns, &lower, found);
    }
}

fn collect_found_patterns(path: &Path, patterns: &[&str], content: &str, found: &mut Vec<String>) {
    for p in patterns {
        if content.contains(p) {
            found.push(format!("{}:{}", path.display(), p));
        }
    }
}

fn should_skip_path(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy();
        if !["rs", "toml", "yaml", "yml", "json", "md", "txt"].contains(&ext_str.as_ref()) {
            return true;
        }
    }
    path.components().any(|c| {
        let s = c.as_os_str();
        s == "target" || s == "node_modules" || s == ".git"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validator_creation() -> CtvpResult<()> {
        let cap = Capability::builder()
            .id("CAP-001")
            .name("Test")
            .desired_effect("Test effect")
            .measurement("test_metric")
            .threshold(Threshold::gte(0.99))
            .build()?;

        let validator = CapabilityValidator::new(cap);
        assert_eq!(validator.get_capability().id, "CAP-001");
        Ok(())
    }

    #[test]
    fn test_validator_with_temp_dir() -> CtvpResult<()> {
        let temp_dir = TempDir::new().map_err(|e| CtvpError::Config(e.to_string()))?;

        // Create a test file
        std::fs::write(temp_dir.path().join("test_example.rs"), "fn test() {}")
            .map_err(|e| CtvpError::Config(e.to_string()))?;

        let cap = Capability::builder()
            .id("CAP-001")
            .name("Test")
            .desired_effect("Test effect")
            .measurement("test_metric")
            .threshold(Threshold::gte(0.99))
            .build()?;

        let validator = CapabilityValidator::new(cap).for_deliverable(temp_dir.path());

        let result = validator.validate_preclinical()?;
        assert!(result.evidence_quality >= EvidenceQuality::Weak);
        Ok(())
    }
}
