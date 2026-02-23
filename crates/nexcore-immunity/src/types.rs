// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Core types for the antipattern immunity system.
//!
//! ## Primitive Grounding
//!
//! | Type | T1 Primitive | Symbol |
//! |------|--------------|--------|
//! | ThreatType | Sum | Σ |
//! | Severity | Comparison | κ |
//! | Antibody | Mapping | μ |
//! | ThreatMatch | State | ς |
//! | ResponseAction | Causality | → |

use serde::{Deserialize, Serialize};

/// Threat classification: external (PAMP) or internal (DAMP).
///
/// ## Tier: T1 (Σ - Sum type)
///
/// PAMPs = Pathogen-Associated Molecular Patterns (external threats)
/// DAMPs = Damage-Associated Molecular Patterns (internal damage)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ThreatType {
    /// External threat (from user input, templates, etc.)
    Pamp,
    /// Internal damage (compilation errors, test failures, etc.)
    Damp,
}

impl std::fmt::Display for ThreatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pamp => write!(f, "PAMP"),
            Self::Damp => write!(f, "DAMP"),
        }
    }
}

/// Ordinal escalation scale for real-time operational threat detection.
///
/// ## Tier: T2-P (κ + N - Comparison with magnitude)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreatLevel {
    /// Low threat - informational
    Low,
    /// Medium threat - should be addressed
    Medium,
    /// High threat - must be addressed
    High,
    /// Critical threat - blocks operation
    Critical,
}

/// Backward-compatible alias.
#[deprecated(note = "use ThreatLevel — F2 equivocation fix")]
pub type Severity = ThreatLevel;

impl ThreatLevel {
    /// Get numeric weight for threat level.
    #[must_use]
    pub const fn weight(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
}

impl std::fmt::Display for ThreatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Response strategy for detected threats.
///
/// ## Tier: T2-P (→ + ∂ - Causality with boundary effect)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStrategy {
    /// Block the operation entirely.
    Block,
    /// Warn but allow to proceed.
    Warn,
    /// Apply automatic fix.
    Fix,
    /// Filter out offending elements.
    Filter,
    /// Suggest alternatives.
    SuggestAlternative,
    /// Require manual audit.
    Audit,
    /// Extract binding to fix lifetime.
    ExtractBinding,
    /// Warn with suggestions.
    WarnWithSuggestions,
    /// Suggest safe alternative.
    SuggestSafeAlternative,
    /// Audit and isolate.
    AuditAndIsolate,
    /// Convert to Result type.
    ConvertToResult,
    /// Filter authoritative.
    FilterAuthoritative,
    /// Warn with suggestion.
    WarnWithSuggestion,
}

impl std::fmt::Display for ResponseStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block => write!(f, "block"),
            Self::Warn => write!(f, "warn"),
            Self::Fix => write!(f, "fix"),
            Self::Filter => write!(f, "filter"),
            Self::SuggestAlternative => write!(f, "suggest_alternative"),
            Self::Audit => write!(f, "audit"),
            Self::ExtractBinding => write!(f, "extract_binding"),
            Self::WarnWithSuggestions => write!(f, "warn_with_suggestions"),
            Self::SuggestSafeAlternative => write!(f, "suggest_safe_alternative"),
            Self::AuditAndIsolate => write!(f, "audit_and_isolate"),
            Self::ConvertToResult => write!(f, "convert_to_result"),
            Self::FilterAuthoritative => write!(f, "filter_authoritative"),
            Self::WarnWithSuggestion => write!(f, "warn_with_suggestion"),
        }
    }
}

/// Code pattern for detection.
///
/// ## Tier: T2-C (κ + σ + λ - Comparison over sequence at location)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    /// Regex pattern to match.
    pub pattern: String,
    /// Language this applies to.
    #[serde(default)]
    pub language: Option<String>,
    /// Description of what this matches.
    #[serde(default)]
    pub description: Option<String>,
}

/// Detection configuration for an antibody.
///
/// ## Tier: T2-C (κ + μ + σ)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Detection {
    /// Error message patterns to match.
    #[serde(default)]
    pub error_patterns: Vec<String>,
    /// File context globs.
    #[serde(default)]
    pub file_contexts: Vec<String>,
    /// Code patterns to detect.
    #[serde(default)]
    pub code_patterns: Vec<CodePattern>,
    /// Exceptions that prevent matching.
    #[serde(default)]
    pub exceptions: Vec<String>,
    /// Metric thresholds.
    #[serde(default)]
    pub metrics: Option<std::collections::HashMap<String, String>>,
}

/// Response configuration for an antibody.
///
/// ## Tier: T2-C (→ + μ + π)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Strategy to apply.
    pub strategy: ResponseStrategy,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Template for Rust fix.
    #[serde(default)]
    pub rust_template: Option<String>,
    /// Alternative suggestions.
    #[serde(default)]
    pub alternatives: Vec<String>,
}

/// An antibody definition.
///
/// ## Tier: T2-C (μ + π + κ + →)
///
/// An antibody maps a threat pattern to a response action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Antibody {
    /// Unique identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Type of threat this addresses.
    pub threat_type: ThreatType,
    /// Severity level.
    pub severity: ThreatLevel,
    /// Description of the threat.
    pub description: String,
    /// Detection configuration.
    pub detection: Detection,
    /// Response configuration.
    pub response: Response,
    /// Confidence score (0.0 - 1.0).
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    /// Number of successful applications.
    #[serde(default)]
    pub applications: u32,
    /// Number of false positives.
    #[serde(default)]
    pub false_positives: u32,
    /// Number of false negatives (missed threats).
    #[serde(default)]
    pub false_negatives: u32,
    /// Source of this antibody.
    #[serde(default)]
    pub learned_from: Option<String>,
    /// Reference documentation.
    #[serde(default)]
    pub reference: Option<String>,
    /// Promoted from proposal ID.
    #[serde(default)]
    pub promoted_from: Option<String>,
    /// Promotion timestamp.
    #[serde(default)]
    pub promoted_at: Option<String>,
    /// Validation configuration.
    #[serde(default)]
    pub validation: Option<Validation>,
    /// Examples for this antibody.
    #[serde(default)]
    pub examples: Option<Examples>,
}

fn default_confidence() -> f64 {
    0.7
}

/// Validation configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Validation {
    /// Test code.
    #[serde(default)]
    pub test: Option<String>,
}

/// Example pass/fail cases.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Examples {
    /// Passing examples.
    #[serde(default)]
    pub pass: Vec<ExampleCase>,
    /// Failing examples.
    #[serde(default)]
    pub fail: Vec<ExampleCase>,
}

/// An example case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleCase {
    /// File path if real.
    #[serde(default)]
    pub file: Option<String>,
    /// Hypothetical scenario.
    #[serde(default)]
    pub hypothetical: Option<String>,
    /// Why it passes/fails.
    #[serde(default)]
    pub reason: Option<String>,
    /// What it should be.
    #[serde(default)]
    pub should_be: Option<String>,
}

/// A detected threat match.
///
/// ## Tier: T2-C (ς + λ + κ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatMatch {
    /// The antibody that matched.
    pub antibody_id: String,
    /// Antibody name.
    pub antibody_name: String,
    /// Type of threat.
    pub threat_type: ThreatType,
    /// Severity level.
    pub severity: ThreatLevel,
    /// Location in file (line number).
    pub location: Option<usize>,
    /// The matched content.
    pub matched_content: String,
    /// Confidence of this match.
    pub confidence: f64,
    /// Suggested response.
    pub response: ResponseStrategy,
}

/// Scan result for a file or content.
///
/// ## Tier: T2-C (σ + κ + N)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanResult {
    /// All detected threats.
    pub threats: Vec<ThreatMatch>,
    /// Whether content is clean.
    pub clean: bool,
    /// Antibodies that were applied.
    pub antibodies_applied: Vec<String>,
    /// Scan metrics.
    pub metrics: ScanMetrics,
}

/// Metrics from a scan.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanMetrics {
    /// Total items scanned.
    pub total_scanned: u32,
    /// Threats detected.
    pub threats_detected: u32,
    /// Auto-fixed count.
    pub auto_fixed: u32,
    /// False positives.
    pub false_positives: u32,
}

fn default_sensitivity() -> f64 {
    0.5
}

/// Antibody registry loaded from YAML.
///
/// ## Tier: T2-C (σ + π + μ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntibodyRegistry {
    /// Version of the registry format.
    pub version: String,
    /// All registered antibodies.
    pub antibodies: Vec<Antibody>,
    /// Sensitivity threshold for minimax tuning (0.3–0.95).
    ///
    /// Controls the trade-off between false positive rate and false negative
    /// rate. Lower values increase sensitivity (catch more threats, more FP);
    /// higher values increase specificity (fewer FP, may miss threats).
    #[serde(default = "default_sensitivity")]
    pub sensitivity_threshold: f64,
}

impl AntibodyRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            antibodies: Vec::new(),
            sensitivity_threshold: default_sensitivity(),
        }
    }

    /// Get antibody count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.antibodies.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.antibodies.is_empty()
    }

    /// Find antibody by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Antibody> {
        self.antibodies.iter().find(|ab| ab.id == id)
    }

    /// Get all antibodies of a given type.
    #[must_use]
    pub fn by_type(&self, threat_type: ThreatType) -> Vec<&Antibody> {
        self.antibodies
            .iter()
            .filter(|ab| ab.threat_type == threat_type)
            .collect()
    }

    /// Get all antibodies at or above a severity.
    #[must_use]
    pub fn by_min_severity(&self, min_severity: ThreatLevel) -> Vec<&Antibody> {
        self.antibodies
            .iter()
            .filter(|ab| ab.severity >= min_severity)
            .collect()
    }

    /// Minimax sensitivity tuning: minimize max(FPR, FNR) given threat level.
    ///
    /// During high threat, lower threshold (more sensitive, more FP).
    /// During low threat, raise threshold (fewer FP, may miss).
    ///
    /// The threshold is nudged toward FPR==FNR equilibrium on each call,
    /// then adjusted by the threat-level bias, and finally clamped to [0.3, 0.95].
    pub fn tune_sensitivity(&mut self, threat_level: ThreatLevel) {
        let (false_positive_total, false_negative_total, total_apps) =
            self.antibodies.iter().fold(
                (0u32, 0u32, 0u32),
                |(fp, fn_, apps), ab| {
                    (
                        fp + ab.false_positives,
                        fn_ + ab.false_negatives,
                        apps + ab.applications,
                    )
                },
            );
        let total = f64::from(total_apps.max(1));
        let fpr = f64::from(false_positive_total) / total;
        let fnr = f64::from(false_negative_total) / total;

        // Threat-level adjustment: higher threat → lower threshold (more sensitive).
        let adjustment = match threat_level {
            ThreatLevel::Critical => -0.1,
            ThreatLevel::High => -0.05,
            ThreatLevel::Medium => 0.0,
            ThreatLevel::Low => 0.05,
        };

        // Minimax nudge: equalize FPR and FNR.
        if fpr > fnr {
            self.sensitivity_threshold += 0.02;
        } else {
            self.sensitivity_threshold -= 0.02;
        }

        self.sensitivity_threshold += adjustment;
        self.sensitivity_threshold = self.sensitivity_threshold.clamp(0.3, 0.95);
    }
}

impl Default for AntibodyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AUTOIMMUNE DETECTION (Biological Alignment v2.0 §8)
// ═══════════════════════════════════════════════════════════════════════════════

/// Autoimmune status of an antibody.
///
/// Like biological autoimmune conditions, an antibody that rejects legitimate
/// code (high false positive rate) is "autoimmune" — attacking self.
///
/// ## Thresholds (per biological T-cell tolerance)
/// - Healthy: false positive rate < 5%
/// - Suspicious: false positive rate 5%-15%
/// - Autoimmune: false positive rate > 15%
///
/// ## Tier: T2-P (Σ + κ)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoimmuneStatus {
    /// False positive rate below 5% — healthy immune response.
    Healthy,
    /// False positive rate 5%-15% — monitor closely.
    Suspicious,
    /// False positive rate above 15% — autoimmune, consider deactivation.
    Autoimmune,
}

impl core::fmt::Display for AutoimmuneStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Suspicious => write!(f, "suspicious"),
            Self::Autoimmune => write!(f, "autoimmune"),
        }
    }
}

/// Per-antibody autoimmune diagnosis.
///
/// ## Tier: T2-C (ς + κ + ν)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoimmuneDiagnosis {
    /// Antibody ID.
    pub antibody_id: String,
    /// Antibody name.
    pub antibody_name: String,
    /// Total activations (applications + false_positives).
    pub total_activations: u32,
    /// False positive count.
    pub false_positives: u32,
    /// False positive rate (0.0-1.0).
    pub false_positive_rate: f64,
    /// Autoimmune status.
    pub status: AutoimmuneStatus,
}

/// System-wide autoimmune report.
///
/// Like a clinical autoimmune panel, this reports on the health of each
/// antibody and identifies those attacking self (legitimate code).
///
/// ## Tier: T2-C (σ + κ + ν)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoimmuneReport {
    /// Per-antibody diagnoses.
    pub diagnoses: Vec<AutoimmuneDiagnosis>,
    /// Total antibodies assessed.
    pub total_antibodies: usize,
    /// Number with autoimmune status.
    pub autoimmune_count: usize,
    /// Number with suspicious status.
    pub suspicious_count: usize,
    /// System-wide false positive rate.
    pub system_false_positive_rate: f64,
    /// Whether the immune system overall is healthy.
    pub system_healthy: bool,
}

impl Antibody {
    /// Compute the false positive rate for this antibody.
    ///
    /// Rate = false_positives / (applications + false_positives).
    /// Returns 0.0 if no activations have occurred.
    #[must_use]
    pub fn false_positive_rate(&self) -> f64 {
        let total = self.applications + self.false_positives;
        if total == 0 {
            return 0.0;
        }
        f64::from(self.false_positives) / f64::from(total)
    }

    /// Compute the false negative rate for this antibody.
    ///
    /// Rate = false_negatives / (applications + false_negatives).
    /// Returns 0.0 if no activations have occurred.
    #[must_use]
    pub fn false_negative_rate(&self) -> f64 {
        let total = self.applications + self.false_negatives;
        if total == 0 {
            return 0.0;
        }
        f64::from(self.false_negatives) / f64::from(total)
    }

    /// Diagnose the autoimmune status of this antibody.
    ///
    /// Thresholds modeled after biological T-cell tolerance:
    /// - < 5%: Healthy (normal immune function)
    /// - 5%-15%: Suspicious (regulatory T-cell dysfunction)
    /// - > 15%: Autoimmune (self-attack, consider deactivation)
    #[must_use]
    pub fn autoimmune_status(&self) -> AutoimmuneStatus {
        let rate = self.false_positive_rate();
        if rate > 0.15 {
            AutoimmuneStatus::Autoimmune
        } else if rate >= 0.05 {
            AutoimmuneStatus::Suspicious
        } else {
            AutoimmuneStatus::Healthy
        }
    }

    /// Diagnose this antibody for autoimmune behavior.
    #[must_use]
    pub fn diagnose_autoimmune(&self) -> AutoimmuneDiagnosis {
        AutoimmuneDiagnosis {
            antibody_id: self.id.clone(),
            antibody_name: self.name.clone(),
            total_activations: self.applications + self.false_positives,
            false_positives: self.false_positives,
            false_positive_rate: self.false_positive_rate(),
            status: self.autoimmune_status(),
        }
    }
}

impl ScanMetrics {
    /// Compute the false positive rate for this scan.
    ///
    /// Rate = false_positives / total_scanned.
    /// Returns 0.0 if nothing was scanned.
    #[must_use]
    pub fn false_positive_rate(&self) -> f64 {
        if self.total_scanned == 0 {
            return 0.0;
        }
        f64::from(self.false_positives) / f64::from(self.total_scanned)
    }
}

impl AntibodyRegistry {
    /// Run autoimmune panel: diagnose all antibodies for self-attack behavior.
    ///
    /// Like a clinical autoimmune antibody panel, this checks each antibody's
    /// false positive rate and flags those attacking legitimate code.
    ///
    /// System is healthy when no antibodies are autoimmune AND system-wide
    /// false positive rate is below 5%.
    #[must_use]
    pub fn autoimmune_panel(&self) -> AutoimmuneReport {
        let mut diagnoses = Vec::new();
        let mut total_applications: u64 = 0;
        let mut total_false_positives: u64 = 0;
        let mut autoimmune_count = 0;
        let mut suspicious_count = 0;

        for antibody in &self.antibodies {
            let diag = antibody.diagnose_autoimmune();
            total_applications += u64::from(antibody.applications + antibody.false_positives);
            total_false_positives += u64::from(antibody.false_positives);

            match diag.status {
                AutoimmuneStatus::Autoimmune => autoimmune_count += 1,
                AutoimmuneStatus::Suspicious => suspicious_count += 1,
                AutoimmuneStatus::Healthy => {}
            }

            diagnoses.push(diag);
        }

        let system_false_positive_rate = if total_applications == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)] // u64 values won't exceed f64 mantissa in practice
            { total_false_positives as f64 / total_applications as f64 }
        };

        AutoimmuneReport {
            total_antibodies: self.antibodies.len(),
            autoimmune_count,
            suspicious_count,
            system_false_positive_rate,
            system_healthy: autoimmune_count == 0 && system_false_positive_rate < 0.05,
            diagnoses,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_type_display() {
        assert_eq!(ThreatType::Pamp.to_string(), "PAMP");
        assert_eq!(ThreatType::Damp.to_string(), "DAMP");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(ThreatLevel::Low < ThreatLevel::Medium);
        assert!(ThreatLevel::Medium < ThreatLevel::High);
        assert!(ThreatLevel::High < ThreatLevel::Critical);
    }

    #[test]
    fn test_severity_weight() {
        assert_eq!(ThreatLevel::Low.weight(), 1);
        assert_eq!(ThreatLevel::Critical.weight(), 4);
    }

    #[test]
    fn test_registry_new() {
        let registry = AntibodyRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_get() {
        let mut registry = AntibodyRegistry::new();
        registry.antibodies.push(Antibody {
            id: "TEST-001".to_string(),
            name: "test".to_string(),
            threat_type: ThreatType::Damp,
            severity: ThreatLevel::High,
            description: "Test antibody".to_string(),
            detection: Detection::default(),
            response: Response {
                strategy: ResponseStrategy::Warn,
                description: None,
                rust_template: None,
                alternatives: Vec::new(),
            },
            confidence: 0.9,
            applications: 0,
            false_positives: 0,
            false_negatives: 0,
            learned_from: None,
            reference: None,
            promoted_from: None,
            promoted_at: None,
            validation: None,
            examples: None,
        });

        assert!(registry.get("TEST-001").is_some());
        assert!(registry.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_registry_by_type() {
        let mut registry = AntibodyRegistry::new();
        registry.antibodies.push(Antibody {
            id: "PAMP-001".to_string(),
            name: "pamp".to_string(),
            threat_type: ThreatType::Pamp,
            severity: ThreatLevel::High,
            description: "PAMP antibody".to_string(),
            detection: Detection::default(),
            response: Response {
                strategy: ResponseStrategy::Block,
                description: None,
                rust_template: None,
                alternatives: Vec::new(),
            },
            confidence: 0.9,
            applications: 0,
            false_positives: 0,
            false_negatives: 0,
            learned_from: None,
            reference: None,
            promoted_from: None,
            promoted_at: None,
            validation: None,
            examples: None,
        });
        registry.antibodies.push(Antibody {
            id: "DAMP-001".to_string(),
            name: "damp".to_string(),
            threat_type: ThreatType::Damp,
            severity: ThreatLevel::Medium,
            description: "DAMP antibody".to_string(),
            detection: Detection::default(),
            response: Response {
                strategy: ResponseStrategy::Warn,
                description: None,
                rust_template: None,
                alternatives: Vec::new(),
            },
            confidence: 0.8,
            applications: 0,
            false_positives: 0,
            false_negatives: 0,
            learned_from: None,
            reference: None,
            promoted_from: None,
            promoted_at: None,
            validation: None,
            examples: None,
        });

        let pamps = registry.by_type(ThreatType::Pamp);
        assert_eq!(pamps.len(), 1);
        assert_eq!(pamps[0].id, "PAMP-001");

        let damps = registry.by_type(ThreatType::Damp);
        assert_eq!(damps.len(), 1);
        assert_eq!(damps[0].id, "DAMP-001");
    }

    #[test]
    fn test_registry_by_min_severity() {
        let mut registry = AntibodyRegistry::new();
        for (id, severity) in [
            ("LOW-001", ThreatLevel::Low),
            ("MED-001", ThreatLevel::Medium),
            ("HIGH-001", ThreatLevel::High),
            ("CRIT-001", ThreatLevel::Critical),
        ] {
            registry.antibodies.push(Antibody {
                id: id.to_string(),
                name: id.to_lowercase(),
                threat_type: ThreatType::Damp,
                severity,
                description: format!("{severity} antibody"),
                detection: Detection::default(),
                response: Response {
                    strategy: ResponseStrategy::Warn,
                    description: None,
                    rust_template: None,
                    alternatives: Vec::new(),
                },
                confidence: 0.8,
                applications: 0,
                false_positives: 0,
                false_negatives: 0,
                learned_from: None,
                reference: None,
                promoted_from: None,
                promoted_at: None,
                validation: None,
                examples: None,
            });
        }

        assert_eq!(registry.by_min_severity(ThreatLevel::Critical).len(), 1);
        assert_eq!(registry.by_min_severity(ThreatLevel::High).len(), 2);
        assert_eq!(registry.by_min_severity(ThreatLevel::Medium).len(), 3);
        assert_eq!(registry.by_min_severity(ThreatLevel::Low).len(), 4);
    }

    #[test]
    fn test_response_strategy_display() {
        assert_eq!(ResponseStrategy::Block.to_string(), "block");
        assert_eq!(
            ResponseStrategy::SuggestAlternative.to_string(),
            "suggest_alternative"
        );
    }

    // ── Autoimmune Detection Tests ─────────────────────────────────────

    fn make_antibody(id: &str, applications: u32, false_positives: u32) -> Antibody {
        Antibody {
            id: id.to_string(),
            name: id.to_lowercase(),
            threat_type: ThreatType::Damp,
            severity: ThreatLevel::High,
            description: format!("{id} test antibody"),
            detection: Detection::default(),
            response: Response {
                strategy: ResponseStrategy::Block,
                description: None,
                rust_template: None,
                alternatives: Vec::new(),
            },
            confidence: 0.9,
            applications,
            false_positives,
            false_negatives: 0,
            learned_from: None,
            reference: None,
            promoted_from: None,
            promoted_at: None,
            validation: None,
            examples: None,
        }
    }

    #[test]
    fn test_antibody_false_positive_rate_no_activations() {
        let ab = make_antibody("TEST-001", 0, 0);
        assert!((ab.false_positive_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_antibody_false_positive_rate_healthy() {
        let ab = make_antibody("TEST-001", 100, 2); // 2%
        assert!(ab.false_positive_rate() < 0.05);
        assert_eq!(ab.autoimmune_status(), AutoimmuneStatus::Healthy);
    }

    #[test]
    fn test_antibody_false_positive_rate_suspicious() {
        let ab = make_antibody("TEST-001", 90, 10); // 10%
        let rate = ab.false_positive_rate();
        assert!(rate >= 0.05 && rate <= 0.15);
        assert_eq!(ab.autoimmune_status(), AutoimmuneStatus::Suspicious);
    }

    #[test]
    fn test_antibody_false_positive_rate_autoimmune() {
        let ab = make_antibody("TEST-001", 70, 30); // 30%
        assert!(ab.false_positive_rate() > 0.15);
        assert_eq!(ab.autoimmune_status(), AutoimmuneStatus::Autoimmune);
    }

    #[test]
    fn test_antibody_diagnose_autoimmune() {
        let ab = make_antibody("PANIC-001", 50, 15); // 23%
        let diag = ab.diagnose_autoimmune();
        assert_eq!(diag.antibody_id, "PANIC-001");
        assert_eq!(diag.total_activations, 65);
        assert_eq!(diag.false_positives, 15);
        assert_eq!(diag.status, AutoimmuneStatus::Autoimmune);
    }

    #[test]
    fn test_scan_metrics_false_positive_rate() {
        let metrics = ScanMetrics {
            total_scanned: 100,
            threats_detected: 10,
            auto_fixed: 5,
            false_positives: 3,
        };
        assert!((metrics.false_positive_rate() - 0.03).abs() < f64::EPSILON);
    }

    #[test]
    fn test_scan_metrics_false_positive_rate_empty() {
        let metrics = ScanMetrics::default();
        assert!((metrics.false_positive_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_autoimmune_panel_all_healthy() {
        let mut registry = AntibodyRegistry::new();
        registry.antibodies.push(make_antibody("AB-001", 100, 2));
        registry.antibodies.push(make_antibody("AB-002", 80, 1));

        let report = registry.autoimmune_panel();
        assert_eq!(report.total_antibodies, 2);
        assert_eq!(report.autoimmune_count, 0);
        assert_eq!(report.suspicious_count, 0);
        assert!(report.system_healthy);
    }

    #[test]
    fn test_autoimmune_panel_detects_autoimmune() {
        let mut registry = AntibodyRegistry::new();
        registry.antibodies.push(make_antibody("GOOD-001", 100, 2)); // 2% - healthy
        registry.antibodies.push(make_antibody("BAD-001", 70, 30)); // 30% - autoimmune
        registry.antibodies.push(make_antibody("MEH-001", 90, 10)); // 10% - suspicious

        let report = registry.autoimmune_panel();
        assert_eq!(report.total_antibodies, 3);
        assert_eq!(report.autoimmune_count, 1);
        assert_eq!(report.suspicious_count, 1);
        assert!(!report.system_healthy);
    }

    #[test]
    fn test_autoimmune_panel_empty_registry() {
        let registry = AntibodyRegistry::new();
        let report = registry.autoimmune_panel();
        assert_eq!(report.total_antibodies, 0);
        assert_eq!(report.autoimmune_count, 0);
        assert!(report.system_healthy);
    }

    #[test]
    fn test_autoimmune_status_display() {
        assert_eq!(AutoimmuneStatus::Healthy.to_string(), "healthy");
        assert_eq!(AutoimmuneStatus::Suspicious.to_string(), "suspicious");
        assert_eq!(AutoimmuneStatus::Autoimmune.to_string(), "autoimmune");
    }

    #[test]
    fn test_autoimmune_threshold_boundary_5_percent() {
        // Exactly 5% should be Suspicious (>= 0.05)
        let ab = make_antibody("EDGE-001", 95, 5);
        assert_eq!(ab.autoimmune_status(), AutoimmuneStatus::Suspicious);
    }

    #[test]
    fn test_autoimmune_threshold_boundary_just_below_5() {
        // Just under 5% should be Healthy
        let ab = make_antibody("EDGE-002", 96, 4); // 4%
        assert_eq!(ab.autoimmune_status(), AutoimmuneStatus::Healthy);
    }

    // ── false_negative_rate tests ──────────────────────────────────────

    #[test]
    fn test_false_negative_rate_no_activations() {
        let ab = make_antibody("FNR-001", 0, 0);
        assert!((ab.false_negative_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_false_negative_rate_with_negatives() {
        let mut ab = make_antibody("FNR-002", 80, 0);
        ab.false_negatives = 20;
        // FNR = 20 / (80 + 20) = 0.2
        assert!((ab.false_negative_rate() - 0.2).abs() < f64::EPSILON);
    }

    // ── tune_sensitivity tests ─────────────────────────────────────────

    #[test]
    fn test_sensitivity_starts_at_default() {
        let registry = AntibodyRegistry::new();
        assert!((registry.sensitivity_threshold - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tune_sensitivity_critical_lowers_threshold() {
        let mut registry = AntibodyRegistry::new();
        let before = registry.sensitivity_threshold;
        registry.tune_sensitivity(ThreatLevel::Critical);
        // Critical applies -0.1 adjustment plus minimax nudge
        assert!(registry.sensitivity_threshold < before);
    }

    #[test]
    fn test_tune_sensitivity_low_raises_threshold() {
        let mut registry = AntibodyRegistry::new();
        let before = registry.sensitivity_threshold;
        registry.tune_sensitivity(ThreatLevel::Low);
        // Low applies +0.05 adjustment plus minimax nudge
        assert!(registry.sensitivity_threshold > before);
    }

    #[test]
    fn test_tune_sensitivity_clamps_to_bounds() {
        let mut registry = AntibodyRegistry::new();
        // Drive it to the floor
        for _ in 0..20 {
            registry.tune_sensitivity(ThreatLevel::Critical);
        }
        assert!(registry.sensitivity_threshold >= 0.3);
        // Drive it to the ceiling
        for _ in 0..20 {
            registry.tune_sensitivity(ThreatLevel::Low);
        }
        assert!(registry.sensitivity_threshold <= 0.95);
    }
}
