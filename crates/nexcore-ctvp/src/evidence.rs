//! Evidence extraction and inventory for CTVP validation.
//!
//! This module provides utilities for extracting and cataloging evidence
//! from software deliverables across all CTVP validation phases.

use crate::error::{CtvpError, CtvpResult};
use crate::types::*;
use nexcore_fs::walk::WalkDir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Inventory of evidence extracted from a deliverable.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceInventory {
    /// Path to the deliverable
    pub deliverable_path: PathBuf,

    /// Evidence by phase
    pub by_phase: HashMap<ValidationPhase, Vec<Evidence>>,

    /// Files scanned
    pub files_scanned: usize,

    /// Extraction metadata
    pub metadata: HashMap<String, String>,
}

impl EvidenceInventory {
    /// Creates a new empty inventory
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            deliverable_path: path.as_ref().to_path_buf(),
            by_phase: HashMap::new(),
            files_scanned: 0,
            metadata: HashMap::new(),
        }
    }

    /// Adds evidence for a phase
    pub fn add(&mut self, phase: ValidationPhase, evidence: Evidence) {
        self.by_phase.entry(phase).or_default().push(evidence);
    }

    /// Returns evidence for a phase
    pub fn get_evidence(&self, phase: ValidationPhase) -> &[Evidence] {
        self.by_phase
            .get(&phase)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the evidence quality for a phase
    pub fn get_quality(&self, phase: ValidationPhase) -> EvidenceQuality {
        let evidence = self.get_evidence(phase);

        if evidence.is_empty() {
            return EvidenceQuality::None;
        }

        // Simple heuristic: more evidence = higher quality
        match evidence.len() {
            0 => EvidenceQuality::None,
            1..=2 => EvidenceQuality::Weak,
            3..=5 => EvidenceQuality::Moderate,
            _ => EvidenceQuality::Strong,
        }
    }

    /// Returns a summary of the inventory
    pub fn generate_summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "Evidence Inventory for: {}",
            self.deliverable_path.display()
        ));
        lines.push(format!("Files scanned: {}", self.files_scanned));
        lines.push(String::new());

        for phase in ValidationPhase::get_all().into_iter() {
            let count = self.get_evidence(phase).len();
            let quality = self.get_quality(phase);
            lines.push(format!("  {}: {} items ({:?})", phase, count, quality));
        }

        lines.join("\n")
    }
}

/// Extractor for pulling evidence from deliverables.
pub struct EvidenceExtractor {
    /// Configuration
    config: ExtractorConfig,
}

/// Configuration for the evidence extractor.
#[derive(Debug, Clone)]
pub struct ExtractorConfig {
    /// File extensions to scan
    pub extensions: Vec<String>,

    /// Directories to skip
    pub skip_dirs: Vec<String>,

    /// Maximum file size to read (bytes)
    pub max_file_size: usize,

    /// Enable verbose output
    pub verbose: bool,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            extensions: vec![
                "rs".into(),
                "toml".into(),
                "yaml".into(),
                "yml".into(),
                "json".into(),
                "md".into(),
            ],
            skip_dirs: vec![
                "target".into(),
                "node_modules".into(),
                ".git".into(),
                "vendor".into(),
            ],
            max_file_size: 1024 * 1024, // 1 MB
            verbose: false,
        }
    }
}

impl EvidenceExtractor {
    /// Creates a new extractor with default config
    pub fn new() -> Self {
        Self {
            config: ExtractorConfig::default(),
        }
    }

    /// Creates an extractor with custom config
    pub fn with_config(config: ExtractorConfig) -> Self {
        Self { config }
    }

    /// Extracts evidence from a deliverable path
    pub fn extract(&self, path: &Path) -> CtvpResult<EvidenceInventory> {
        let mut inventory = EvidenceInventory::new(path);

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            self.process_file(file_path, &mut inventory);
        }

        Ok(inventory)
    }

    fn process_file(&self, file_path: &Path, inventory: &mut EvidenceInventory) {
        if self.should_skip(file_path) {
            return;
        }

        // Read and analyze file
        if let Ok(content) = std::fs::read_to_string(file_path) {
            if content.len() <= self.config.max_file_size {
                self.extract_from_content(&content, file_path, inventory);
                inventory.files_scanned += 1;
            }
        }
    }

    /// Checks if a path should be skipped
    fn should_skip(&self, path: &Path) -> bool {
        // Check directory exclusions
        for component in path.components() {
            let name = component.as_os_str().to_string_lossy();
            if self.config.skip_dirs.iter().any(|d| d == name.as_ref()) {
                return true;
            }
        }

        // Check extension
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            if !self.config.extensions.iter().any(|e| e == ext_str.as_ref()) {
                return true;
            }
        } else {
            return true; // Skip files without extensions
        }

        false
    }

    /// Extracts evidence from file content
    fn extract_from_content(&self, content: &str, path: &Path, inventory: &mut EvidenceInventory) {
        let content_lower = content.to_lowercase();
        let source = path.display().to_string();

        // Phase 0: Test and coverage evidence
        self.extract_phase0_evidence(&content_lower, &source, inventory);

        // Phase 1: Fault injection evidence
        self.extract_phase1_evidence(&content_lower, &source, inventory);

        // Phase 2: Real data and SLO evidence
        self.extract_phase2_evidence(&content_lower, &source, inventory);

        // Phase 3: Deployment evidence
        self.extract_phase3_evidence(&content_lower, &source, inventory);

        // Phase 4: Observability evidence
        self.extract_phase4_evidence(&content_lower, &source, inventory);
    }

    fn extract_phase0_evidence(
        &self,
        content: &str,
        source: &str,
        inventory: &mut EvidenceInventory,
    ) {
        let patterns = [
            ("#[test]", "Unit test found"),
            ("#[cfg(test)]", "Test module found"),
            ("proptest!", "Property-based test found"),
            ("quickcheck", "QuickCheck test found"),
            ("assert!", "Assertion found"),
            ("assert_eq!", "Equality assertion found"),
            ("coverage", "Coverage configuration found"),
            ("tarpaulin", "Tarpaulin coverage tool found"),
            ("codecov", "Codecov integration found"),
        ];

        for (p, desc) in patterns {
            if !content.contains(p) {
                continue;
            }
            let et = EvidenceType::TestResult {
                name: p.to_string(),
                passed: true,
                duration_ms: 0,
            };
            inventory.add(
                ValidationPhase::Preclinical,
                Evidence::new(et, desc, source),
            );
        }
    }

    fn extract_phase1_evidence(
        &self,
        content: &str,
        source: &str,
        inventory: &mut EvidenceInventory,
    ) {
        let patterns = [
            ("toxiproxy", "Toxiproxy fault injection found"),
            ("chaos", "Chaos engineering pattern found"),
            ("circuit_breaker", "Circuit breaker found"),
            ("circuitbreaker", "Circuit breaker found"),
            ("retry", "Retry logic found"),
            ("backoff", "Backoff logic found"),
            ("timeout", "Timeout configuration found"),
            ("deadline", "Deadline configuration found"),
            ("failsafe", "Failsafe pattern found"),
            ("resilience", "Resilience pattern found"),
        ];

        for (p, desc) in patterns {
            if !content.contains(p) {
                continue;
            }
            let et = EvidenceType::Snapshot {
                component: "fault_handling".into(),
                state: serde_json::json!({"pattern": p}),
            };
            inventory.add(
                ValidationPhase::Phase1Safety,
                Evidence::new(et, desc, source),
            );
        }
    }

    fn extract_phase2_evidence(
        &self,
        content: &str,
        source: &str,
        inventory: &mut EvidenceInventory,
    ) {
        let patterns = [
            ("slo", "SLO definition found"),
            ("sli", "SLI definition found"),
            ("sla", "SLA definition found"),
            ("latency_target", "Latency target found"),
            ("error_budget", "Error budget found"),
            ("integration_test", "Integration test found"),
            ("e2e_test", "E2E test found"),
            ("end_to_end", "End-to-end test found"),
            ("real_data", "Real data reference found"),
            ("production_data", "Production data reference found"),
        ];

        for (p, desc) in patterns {
            if !content.contains(p) {
                continue;
            }
            let et = EvidenceType::Snapshot {
                component: "efficacy".into(),
                state: serde_json::json!({"pattern": p}),
            };
            inventory.add(
                ValidationPhase::Phase2Efficacy,
                Evidence::new(et, desc, source),
            );
        }
    }

    fn extract_phase3_evidence(
        &self,
        content: &str,
        source: &str,
        inventory: &mut EvidenceInventory,
    ) {
        let patterns = [
            ("canary", "Canary deployment found"),
            ("shadow", "Shadow deployment found"),
            ("blue_green", "Blue-green deployment found"),
            ("feature_flag", "Feature flag found"),
            ("ab_test", "A/B test found"),
            ("rollout", "Rollout configuration found"),
            ("gradual_release", "Gradual release found"),
        ];

        for (p, desc) in patterns {
            if !content.contains(p) {
                continue;
            }
            let et = EvidenceType::Snapshot {
                component: "deployment".into(),
                state: serde_json::json!({"pattern": p}),
            };
            inventory.add(
                ValidationPhase::Phase3Confirmation,
                Evidence::new(et, desc, source),
            );
        }
    }

    fn extract_phase4_evidence(
        &self,
        content: &str,
        source: &str,
        inventory: &mut EvidenceInventory,
    ) {
        let patterns = [
            ("prometheus", "Prometheus metrics found"),
            ("opentelemetry", "OpenTelemetry found"),
            ("datadog", "Datadog integration found"),
            ("grafana", "Grafana integration found"),
            ("tracing", "Tracing found"),
            ("metrics", "Metrics found"),
            ("alert", "Alert configuration found"),
            ("drift", "Drift detection found"),
            ("anomaly", "Anomaly detection found"),
            ("baseline", "Baseline reference found"),
        ];

        for (p, desc) in patterns {
            if !content.contains(p) {
                continue;
            }
            let et = EvidenceType::Snapshot {
                component: "observability".into(),
                state: serde_json::json!({"pattern": p}),
            };
            inventory.add(
                ValidationPhase::Phase4Surveillance,
                Evidence::new(et, desc, source),
            );
        }
    }
}

impl Default for EvidenceExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_evidence_inventory() {
        let mut inventory = EvidenceInventory::new("/test");
        let et = EvidenceType::TestResult {
            name: "test".into(),
            passed: true,
            duration_ms: 0,
        };
        inventory.add(
            ValidationPhase::Preclinical,
            Evidence::new(et, "Test evidence", "test.rs"),
        );

        assert_eq!(
            inventory.get_evidence(ValidationPhase::Preclinical).len(),
            1
        );
        assert_eq!(
            inventory.get_evidence(ValidationPhase::Phase1Safety).len(),
            0
        );
    }

    #[test]
    fn test_evidence_quality_calculation() {
        let mut inventory = EvidenceInventory::new("/test");

        assert_eq!(
            inventory.get_quality(ValidationPhase::Preclinical),
            EvidenceQuality::None
        );

        inventory.add(ValidationPhase::Preclinical, create_test_evidence("test1"));
        assert_eq!(
            inventory.get_quality(ValidationPhase::Preclinical),
            EvidenceQuality::Weak
        );

        for i in 2..=6 {
            inventory.add(
                ValidationPhase::Preclinical,
                create_test_evidence(&format!("test{}", i)),
            );
        }

        assert_eq!(
            inventory.get_quality(ValidationPhase::Preclinical),
            EvidenceQuality::Strong
        );
    }

    fn create_test_evidence(name: &str) -> Evidence {
        Evidence::new(
            EvidenceType::TestResult {
                name: name.into(),
                passed: true,
                duration_ms: 0,
            },
            format!("Test {}", name),
            "test.rs",
        )
    }

    #[test]
    fn test_extractor_with_rust_file() -> CtvpResult<()> {
        let temp_dir = TempDir::new().map_err(|e| CtvpError::Config(e.to_string()))?;
        let test_file = temp_dir.path().join("lib.rs");

        std::fs::write(
            &test_file,
            "#[test] fn test_example() { assert_eq!(1, 1); } fn with_timeout() {}",
        )
        .map_err(|e| CtvpError::Config(e.to_string()))?;

        let extractor = EvidenceExtractor::new();
        let inventory = extractor.extract(temp_dir.path())?;

        assert!(inventory.get_evidence(ValidationPhase::Preclinical).len() > 0);
        assert!(inventory.get_evidence(ValidationPhase::Phase1Safety).len() > 0);
        Ok(())
    }
}
