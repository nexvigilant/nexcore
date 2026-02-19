//! # FAERS Pipeline Configuration and Results
//!
//! Types for configuring and capturing results from FAERS signal detection pipelines.
//!
//! ## Design Note
//!
//! These types are pure data containers. Signal detection thresholds and evaluation
//! logic should use `crate::SignalCriteria` for threshold-based decisions.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_vigilance::pv::faers::pipeline::{FaersPipelineConfig, SignalStrength};
//!
//! // Use default configuration
//! let config = FaersPipelineConfig::default();
//!
//! // Or customize
//! let config = FaersPipelineConfig::builder()
//!     .min_cases(5)
//!     .suspect_drugs_only(true)
//!     .include_ebgm(true)
//!     .build();
//! ```

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

// =============================================================================
// Signal Strength Classification
// =============================================================================

/// Signal strength classification based on disproportionality metrics.
///
/// Used to categorize detected signals by their statistical strength.
/// Classification should be performed using signal criteria.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignalStrength {
    /// No signal detected (below all thresholds)
    #[default]
    None,
    /// Weak signal
    Weak,
    /// Moderate signal
    Moderate,
    /// Strong signal
    Strong,
    /// Very strong signal
    VeryStrong,
}

impl SignalStrength {
    /// Check if this represents a detected signal
    #[must_use]
    pub const fn is_signal(&self) -> bool {
        !matches!(self, Self::None)
    }
}

// =============================================================================
// Pipeline Configuration
// =============================================================================

/// Configuration for FAERS signal detection pipeline.
///
/// Controls behavior of the pipeline including filtering and
/// optional AI enrichment settings.
///
/// Note: Signal detection thresholds should be configured via `SignalCriteria`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaersPipelineConfig {
    /// Minimum cases required for signal detection
    pub min_cases: u32,

    /// Filter to only suspect drugs (PS/SS roles)
    pub suspect_drugs_only: bool,

    /// Include EBGM calculation (more compute intensive)
    pub include_ebgm: bool,

    /// Maximum cases to enrich with AI per batch
    pub max_ai_enrichment_batch: usize,

    /// Maximum concurrent AI enrichment calls
    pub ai_enrichment_concurrency: usize,

    /// Optional list of drugs to focus on (None = all drugs)
    pub drugs_of_interest: Option<Vec<String>>,

    /// Optional list of events to focus on (None = all events)
    pub events_of_interest: Option<Vec<String>>,
}

impl Default for FaersPipelineConfig {
    fn default() -> Self {
        Self {
            min_cases: 3,
            suspect_drugs_only: true,
            include_ebgm: true,
            max_ai_enrichment_batch: 100,
            ai_enrichment_concurrency: 10,
            drugs_of_interest: None,
            events_of_interest: None,
        }
    }
}

impl FaersPipelineConfig {
    /// Create a new builder for `FaersPipelineConfig`
    #[must_use]
    pub fn builder() -> FaersPipelineConfigBuilder {
        FaersPipelineConfigBuilder::default()
    }

    /// Check if a drug matches the interest filter
    #[must_use]
    pub fn matches_drug(&self, drug: &str) -> bool {
        match &self.drugs_of_interest {
            None => true,
            Some(drugs) => drugs.iter().any(|d| d.eq_ignore_ascii_case(drug)),
        }
    }

    /// Check if an event matches the interest filter
    #[must_use]
    pub fn matches_event(&self, event: &str) -> bool {
        match &self.events_of_interest {
            None => true,
            Some(events) => events.iter().any(|e| e.eq_ignore_ascii_case(event)),
        }
    }
}

/// Builder for `FaersPipelineConfig`
#[derive(Debug, Clone, Default)]
pub struct FaersPipelineConfigBuilder {
    min_cases: Option<u32>,
    suspect_drugs_only: Option<bool>,
    include_ebgm: Option<bool>,
    max_ai_enrichment_batch: Option<usize>,
    ai_enrichment_concurrency: Option<usize>,
    drugs_of_interest: Option<Vec<String>>,
    events_of_interest: Option<Vec<String>>,
}

impl FaersPipelineConfigBuilder {
    /// Set minimum case count for signal detection
    #[must_use]
    pub fn min_cases(mut self, n: u32) -> Self {
        self.min_cases = Some(n);
        self
    }

    /// Filter to suspect drugs only (PS/SS roles)
    #[must_use]
    pub fn suspect_drugs_only(mut self, yes: bool) -> Self {
        self.suspect_drugs_only = Some(yes);
        self
    }

    /// Include EBGM calculation
    #[must_use]
    pub fn include_ebgm(mut self, yes: bool) -> Self {
        self.include_ebgm = Some(yes);
        self
    }

    /// Set maximum AI enrichment batch size
    #[must_use]
    pub fn max_ai_enrichment_batch(mut self, n: usize) -> Self {
        self.max_ai_enrichment_batch = Some(n);
        self
    }

    /// Set AI enrichment concurrency limit
    #[must_use]
    pub fn ai_enrichment_concurrency(mut self, n: usize) -> Self {
        self.ai_enrichment_concurrency = Some(n);
        self
    }

    /// Focus on specific drugs
    #[must_use]
    pub fn drugs_of_interest(mut self, drugs: Vec<String>) -> Self {
        self.drugs_of_interest = Some(drugs);
        self
    }

    /// Focus on specific events
    #[must_use]
    pub fn events_of_interest(mut self, events: Vec<String>) -> Self {
        self.events_of_interest = Some(events);
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> FaersPipelineConfig {
        let default = FaersPipelineConfig::default();
        FaersPipelineConfig {
            min_cases: self.min_cases.unwrap_or(default.min_cases),
            suspect_drugs_only: self
                .suspect_drugs_only
                .unwrap_or(default.suspect_drugs_only),
            include_ebgm: self.include_ebgm.unwrap_or(default.include_ebgm),
            max_ai_enrichment_batch: self
                .max_ai_enrichment_batch
                .unwrap_or(default.max_ai_enrichment_batch),
            ai_enrichment_concurrency: self
                .ai_enrichment_concurrency
                .unwrap_or(default.ai_enrichment_concurrency),
            drugs_of_interest: self.drugs_of_interest,
            events_of_interest: self.events_of_interest,
        }
    }
}

// =============================================================================
// Disproportionality Metrics
// =============================================================================

/// Complete disproportionality analysis result.
///
/// Contains all standard pharmacovigilance signal detection metrics.
/// Values are computed by signal detection algorithms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisproportionalityResult {
    /// Proportional Reporting Ratio
    pub prr: f64,
    /// PRR 95% CI lower bound
    pub prr_lower_ci: f64,
    /// PRR 95% CI upper bound
    pub prr_upper_ci: f64,

    /// Reporting Odds Ratio
    pub ror: f64,
    /// ROR 95% CI lower bound
    pub ror_lower_ci: f64,
    /// ROR 95% CI upper bound
    pub ror_upper_ci: f64,

    /// Information Component
    pub ic: f64,
    /// IC 95% CI lower bound (IC025)
    pub ic_lower_ci: f64,
    /// IC 95% CI upper bound (IC975)
    pub ic_upper_ci: f64,

    /// Empirical Bayes Geometric Mean (optional)
    pub ebgm: Option<f64>,
    /// EBGM 5th percentile (EB05)
    pub eb05: Option<f64>,
    /// EBGM 95th percentile (EB95)
    pub eb95: Option<f64>,

    /// Chi-square statistic
    pub chi_square: f64,

    /// Whether signal is detected based on thresholds
    pub is_signal: bool,

    /// Classified signal strength
    pub signal_strength: SignalStrength,
}

impl DisproportionalityResult {
    /// Get a summary string of the main metrics
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "PRR={:.2} [{:.2}-{:.2}], ROR={:.2} [{:.2}-{:.2}], IC={:.2} [{:.2}-{:.2}], χ²={:.2}",
            self.prr,
            self.prr_lower_ci,
            self.prr_upper_ci,
            self.ror,
            self.ror_lower_ci,
            self.ror_upper_ci,
            self.ic,
            self.ic_lower_ci,
            self.ic_upper_ci,
            self.chi_square
        )
    }
}

// =============================================================================
// Signal Result
// =============================================================================

/// Result of signal detection for a drug-event pair.
///
/// Wraps the disproportionality metrics with drug/event identification
/// and case-level information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugEventSignal {
    /// Normalized drug name (uppercase)
    pub drug: String,

    /// MedDRA preferred term
    pub event: String,

    /// Disproportionality analysis result
    pub signal: DisproportionalityResult,

    /// Number of cases with this drug-event pair (cell 'a' in 2x2 table)
    pub case_count: u32,

    /// Set of case IDs (primaryid) contributing to this signal
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub case_ids: HashSet<u64>,
}

impl DrugEventSignal {
    /// Create a new drug-event signal result
    #[must_use]
    pub fn new(
        drug: String,
        event: String,
        signal: DisproportionalityResult,
        case_count: u32,
    ) -> Self {
        Self {
            drug,
            event,
            signal,
            case_count,
            case_ids: HashSet::new(),
        }
    }

    /// Create with case IDs
    #[must_use]
    pub fn with_case_ids(mut self, case_ids: HashSet<u64>) -> Self {
        self.case_ids = case_ids;
        self
    }

    /// Check if this meets signal detection criteria
    #[must_use]
    pub fn is_signal(&self) -> bool {
        self.signal.is_signal
    }

    /// Get signal strength classification
    #[must_use]
    pub fn strength(&self) -> SignalStrength {
        self.signal.signal_strength
    }
}

impl std::fmt::Display for DrugEventSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.is_signal() {
            "SIGNAL"
        } else {
            "no signal"
        };
        write!(
            f,
            "{} + {}: {} (PRR={:.2}, n={})",
            self.drug, self.event, status, self.signal.prr, self.case_count
        )
    }
}

// =============================================================================
// Pipeline Result
// =============================================================================

/// Complete result from FAERS pipeline execution.
///
/// Contains all signals detected, metadata about the dataset, and
/// processing statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaersPipelineResult {
    /// FAERS quarter identifier (e.g., "2024Q4")
    pub quarter: String,

    /// Total cases loaded from FAERS data
    pub total_cases: u64,

    /// Total drug records processed
    pub total_drug_records: u64,

    /// Total reaction records processed
    pub total_reaction_records: u64,

    /// Detected signals (meeting threshold criteria)
    pub signals: Vec<DrugEventSignal>,

    /// All drug-event pair results (including non-signals)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub all_results: Vec<DrugEventSignal>,

    /// Processing time in seconds
    pub processing_time_seconds: f64,
}

impl FaersPipelineResult {
    /// Number of detected signals
    #[must_use]
    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    /// Get all signals for a specific drug
    #[must_use]
    pub fn signals_by_drug(&self, drug: &str) -> Vec<&DrugEventSignal> {
        let drug_upper = drug.to_uppercase();
        self.signals
            .iter()
            .filter(|s| s.drug == drug_upper)
            .collect()
    }

    /// Get all signals for a specific event
    #[must_use]
    pub fn signals_by_event(&self, event: &str) -> Vec<&DrugEventSignal> {
        self.signals.iter().filter(|s| s.event == event).collect()
    }

    /// Get signals meeting minimum strength threshold
    #[must_use]
    pub fn signals_by_min_strength(&self, min_strength: SignalStrength) -> Vec<&DrugEventSignal> {
        self.signals
            .iter()
            .filter(|s| s.strength() >= min_strength)
            .collect()
    }

    /// Get top N signals sorted by PRR (descending)
    #[must_use]
    pub fn top_signals(&self, n: usize) -> Vec<&DrugEventSignal> {
        let mut sorted: Vec<_> = self.signals.iter().collect();
        sorted.sort_by(|a, b| {
            b.signal
                .prr
                .partial_cmp(&a.signal.prr)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(n).collect()
    }

    /// Summary string for display
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "FAERS {} | {} cases, {} drugs, {} reactions | {} signals detected in {:.2}s",
            self.quarter,
            self.total_cases,
            self.total_drug_records,
            self.total_reaction_records,
            self.signal_count(),
            self.processing_time_seconds
        )
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_strength_ordering() {
        assert!(SignalStrength::None < SignalStrength::Weak);
        assert!(SignalStrength::Weak < SignalStrength::Moderate);
        assert!(SignalStrength::Moderate < SignalStrength::Strong);
        assert!(SignalStrength::Strong < SignalStrength::VeryStrong);
    }

    #[test]
    fn test_signal_strength_is_signal() {
        assert!(!SignalStrength::None.is_signal());
        assert!(SignalStrength::Weak.is_signal());
        assert!(SignalStrength::Moderate.is_signal());
        assert!(SignalStrength::Strong.is_signal());
        assert!(SignalStrength::VeryStrong.is_signal());
    }

    #[test]
    fn test_config_default() {
        let config = FaersPipelineConfig::default();
        assert_eq!(config.min_cases, 3);
        assert!(config.suspect_drugs_only);
        assert!(config.include_ebgm);
    }

    #[test]
    fn test_config_builder() {
        let config = FaersPipelineConfig::builder()
            .min_cases(5)
            .suspect_drugs_only(false)
            .drugs_of_interest(vec!["ASPIRIN".to_string()])
            .build();

        assert_eq!(config.min_cases, 5);
        assert!(!config.suspect_drugs_only);
        assert!(config.drugs_of_interest.is_some());
    }

    #[test]
    fn test_config_matches_drug() {
        let config = FaersPipelineConfig::builder()
            .drugs_of_interest(vec!["ASPIRIN".to_string(), "IBUPROFEN".to_string()])
            .build();

        assert!(config.matches_drug("aspirin"));
        assert!(config.matches_drug("ASPIRIN"));
        assert!(config.matches_drug("Aspirin"));
        assert!(!config.matches_drug("TYLENOL"));

        // Without filter, everything matches
        let config_all = FaersPipelineConfig::default();
        assert!(config_all.matches_drug("ANYTHING"));
    }

    fn sample_signal(
        drug: &str,
        event: &str,
        prr: f64,
        strength: SignalStrength,
    ) -> DrugEventSignal {
        DrugEventSignal::new(
            drug.to_string(),
            event.to_string(),
            DisproportionalityResult {
                prr,
                prr_lower_ci: prr * 0.7,
                prr_upper_ci: prr * 1.4,
                ror: prr * 1.05,
                ror_lower_ci: prr * 0.75,
                ror_upper_ci: prr * 1.5,
                ic: prr.ln() / 2.0_f64.ln(),
                ic_lower_ci: (prr * 0.7).ln() / 2.0_f64.ln(),
                ic_upper_ci: (prr * 1.4).ln() / 2.0_f64.ln(),
                ebgm: None,
                eb05: None,
                eb95: None,
                chi_square: prr * 4.0,
                is_signal: strength.is_signal(),
                signal_strength: strength,
            },
            15,
        )
    }

    #[test]
    fn test_drug_event_signal_display() {
        let signal = sample_signal("ASPIRIN", "Headache", 2.5, SignalStrength::Weak);
        let display = signal.to_string();
        assert!(display.contains("ASPIRIN"));
        assert!(display.contains("Headache"));
        assert!(display.contains("SIGNAL"));
    }

    #[test]
    fn test_pipeline_result_queries() {
        let result = FaersPipelineResult {
            quarter: "2024Q4".to_string(),
            total_cases: 1000,
            total_drug_records: 5000,
            total_reaction_records: 3000,
            signals: vec![
                sample_signal("ASPIRIN", "Headache", 2.5, SignalStrength::Weak),
                sample_signal("ASPIRIN", "Nausea", 5.0, SignalStrength::Strong),
            ],
            all_results: vec![],
            processing_time_seconds: 1.5,
        };

        assert_eq!(result.signal_count(), 2);
        assert_eq!(result.signals_by_drug("aspirin").len(), 2);
        assert_eq!(result.signals_by_event("Headache").len(), 1);
        assert_eq!(
            result.signals_by_min_strength(SignalStrength::Strong).len(),
            1
        );
        assert_eq!(result.top_signals(1).len(), 1);
    }

    #[test]
    fn test_disproportionality_summary() {
        let signal = sample_signal("TEST", "EVENT", 3.0, SignalStrength::Moderate);
        let summary = signal.signal.summary();
        assert!(summary.contains("PRR="));
        assert!(summary.contains("ROR="));
        assert!(summary.contains("IC="));
    }
}
