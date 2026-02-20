// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Core types for the retrocasting engine.
//!
//! ## Primitive Grounding
//!
//! | Type | Tier | Primitives | Meaning |
//! |------|------|-----------|---------|
//! | `RetrocastResult` | T3 | ν + κ + → + ∃ | Frequency-keyed comparison of causal signal existence |
//! | `SignalRecord` | T2-P | N + κ + ν | Quantitative comparison of frequency ratios |
//! | `StructuralCluster` | T2-C | Σ + κ + ∂ | Sum of members within a similarity boundary |
//! | `AlertCandidate` | T2-C | ∂ + κ + ∃ | Boundary-defined comparison asserting existence |
//! | `StructuredSignal` | T2-C | → + μ + ∃ | Causal mapping to existing structure |

use serde::{Deserialize, Serialize};

/// Complete result from a retrocasting analysis run.
///
/// Aggregates all signals, structural clusters, and candidate alert fragments
/// discovered for a given drug.
///
/// ## Tier: T3 (ν + κ + → + ∃)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrocastResult {
    /// Drug name used as the analysis subject.
    pub drug_name: String,
    /// Canonical SMILES string (if resolved).
    pub smiles: Option<String>,
    /// All FAERS signals associated with this drug.
    pub signals: Vec<SignalRecord>,
    /// Structural clusters identified from similar drugs sharing signals.
    pub structural_clusters: Vec<StructuralCluster>,
    /// New structural alert candidates discovered via retrocasting.
    pub new_alert_candidates: Vec<AlertCandidate>,
    /// Overall confidence in the retrocasting analysis (0.0–1.0).
    pub confidence: f64,
    /// UTC timestamp when analysis was completed.
    pub analyzed_at: chrono::DateTime<chrono::Utc>,
}

/// A single pharmacovigilance signal from FAERS.
///
/// Represents a drug-event pair with associated disproportionality statistics.
///
/// ## Tier: T2-P (N + κ + ν)
/// Quantitative (N) comparison (κ) of frequency (ν) ratios.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRecord {
    /// Drug name (INN or trade name from FAERS).
    pub drug: String,
    /// Adverse event term (MedDRA PT).
    pub event: String,
    /// Proportional Reporting Ratio.
    pub prr: f64,
    /// Reporting Odds Ratio.
    pub ror: f64,
    /// Number of cases in FAERS database.
    pub case_count: u64,
    /// ROR lower 95% confidence interval (optional).
    pub ror_lci: Option<f64>,
    /// PRR chi-squared statistic (optional).
    pub prr_chi_sq: Option<f64>,
}

impl SignalRecord {
    /// Returns `true` if this signal meets standard detection thresholds.
    ///
    /// Criteria (OR logic — any threshold met qualifies):
    /// - PRR ≥ 2.0 AND chi-sq ≥ 3.841
    /// - ROR lower CI > 1.0
    pub fn is_significant(&self) -> bool {
        let prr_sig = self.prr >= 2.0 && self.prr_chi_sq.is_some_and(|chi| chi >= 3.841);
        let ror_sig = self.ror_lci.is_some_and(|lci| lci > 1.0);
        prr_sig || ror_sig || self.prr >= 2.0
    }
}

/// A cluster of structurally similar drugs sharing adverse event patterns.
///
/// ## Tier: T2-C (Σ + κ + ∂)
/// Sum (Σ) of members within a similarity boundary (∂) defined by comparison (κ).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralCluster {
    /// Sequential cluster identifier within the analysis.
    pub cluster_id: usize,
    /// Drug names belonging to this cluster.
    pub members: Vec<String>,
    /// SMILES strings of common structural fragments across members.
    pub common_fragments: Vec<String>,
    /// Tanimoto similarity threshold used to form this cluster.
    pub similarity_threshold: f64,
    /// Adverse events shared by ≥ 50% of cluster members.
    pub shared_events: Vec<String>,
}

/// A candidate structural alert identified by retrocasting.
///
/// Represents a molecular fragment consistently associated with a specific
/// adverse event pattern across a structural cluster.
///
/// ## Tier: T2-C (∂ + κ + ∃)
/// Boundary (∂) fragment comparison (κ) asserting existence (∃) of an alert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCandidate {
    /// SMILES string of the candidate structural fragment.
    pub fragment_smiles: String,
    /// Adverse events associated with this fragment.
    pub associated_events: Vec<String>,
    /// Confidence score (0.0–1.0) for this alert association.
    pub confidence: f64,
    /// Drug names that support this alert-fragment association.
    pub supporting_drugs: Vec<String>,
    /// Mean PRR across supporting drugs for the primary event.
    pub mean_prr: f64,
    /// Number of independent signal observations supporting this alert.
    pub support_count: usize,
}

/// A FAERS signal linked to its resolved molecular structure.
///
/// Produced by the linker pipeline after resolving drug names to
/// `CompoundRecord`s from the compound registry.
///
/// ## Tier: T2-C (→ + μ + ∃)
/// Causal (→) mapping (μ) to existing (∃) structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredSignal {
    /// The underlying pharmacovigilance signal.
    pub signal: SignalRecord,
    /// Resolved compound record (None if resolution failed).
    pub compound: Option<nexcore_compound_registry::types::CompoundRecord>,
    /// Whether the SMILES was available for structural analysis.
    pub has_structure: bool,
}

impl StructuredSignal {
    /// Returns the SMILES string if both compound and SMILES are present.
    pub fn smiles(&self) -> Option<&str> {
        self.compound.as_ref().and_then(|c| c.smiles.as_deref())
    }
}

/// A labeled training record for predictive safety models.
///
/// Feature vector + binary label for supervised learning.
///
/// ## Tier: T2-C (N + σ + →)
/// Quantity (N) sequence (σ) causally mapped (→) to outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRecord {
    /// Drug name this record represents.
    pub drug_name: String,
    /// Adverse event term.
    pub event: String,
    /// Feature vector: placeholder for Morgan FP bits + descriptors.
    /// Populated with SMILES character n-gram features until T7 (fingerprints).
    pub features: Vec<f64>,
    /// Binary label: true if a significant signal was detected post-market.
    pub label: bool,
    /// Optional year of drug approval (for temporal cohort splitting).
    pub approval_year: Option<u32>,
    /// PRR value if signal is detected.
    pub prr: Option<f64>,
    /// ROR value if signal is detected.
    pub ror: Option<f64>,
    /// Case count from FAERS.
    pub case_count: Option<u64>,
}

/// A complete labeled dataset for predictive safety model training.
///
/// ## Tier: T3 (Σ + π + σ)
/// Sum (Σ) of persisted (π) sequences (σ) of labeled records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingDataset {
    /// All training records.
    pub records: Vec<TrainingRecord>,
    /// Feature dimension (number of features per record).
    pub feature_dim: usize,
    /// Number of positive labels (signal detected).
    pub positive_count: usize,
    /// Number of negative labels (no signal detected).
    pub negative_count: usize,
    /// Cohort years present in this dataset (for temporal splits).
    pub cohort_years: Vec<u32>,
    /// UTC timestamp when this dataset was generated.
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Version tag for reproducibility.
    pub version: String,
}

impl TrainingDataset {
    /// Split records by approval year into train/test cohorts.
    ///
    /// Returns `(train_records, test_records)` where test records are
    /// all records with `approval_year >= cutoff_year`.
    pub fn temporal_split(&self, cutoff_year: u32) -> (Vec<&TrainingRecord>, Vec<&TrainingRecord>) {
        let mut train = Vec::new();
        let mut test = Vec::new();
        for record in &self.records {
            match record.approval_year {
                Some(year) if year >= cutoff_year => test.push(record),
                _ => train.push(record),
            }
        }
        (train, test)
    }

    /// Returns the class balance ratio (positives / total).
    pub fn class_balance(&self) -> f64 {
        let total = self.positive_count + self.negative_count;
        if total == 0 {
            0.0
        } else {
            self.positive_count as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signal(drug: &str, event: &str, prr: f64, ror: f64) -> SignalRecord {
        SignalRecord {
            drug: drug.to_string(),
            event: event.to_string(),
            prr,
            ror,
            case_count: 50,
            ror_lci: Some(ror / 1.5),
            prr_chi_sq: Some(if prr >= 2.0 { 5.0 } else { 1.0 }),
        }
    }

    #[test]
    fn test_signal_significance_prr_threshold() {
        let sig = make_signal("aspirin", "bleeding", 3.0, 2.5);
        assert!(sig.is_significant());
    }

    #[test]
    fn test_signal_insignificant_below_threshold() {
        let sig = SignalRecord {
            drug: "placebo".to_string(),
            event: "headache".to_string(),
            prr: 1.1,
            ror: 1.2,
            case_count: 5,
            ror_lci: Some(0.8),
            prr_chi_sq: Some(0.5),
        };
        // prr < 2.0 and ror_lci < 1.0 — not significant
        assert!(!sig.is_significant());
    }

    #[test]
    fn test_retrocast_result_serialization() {
        let result = RetrocastResult {
            drug_name: "ibuprofen".to_string(),
            smiles: Some("CC(C)Cc1ccc(cc1)C(C)C(=O)O".to_string()),
            signals: vec![make_signal("ibuprofen", "GI bleeding", 4.2, 3.8)],
            structural_clusters: vec![],
            new_alert_candidates: vec![],
            confidence: 0.75,
            analyzed_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&result);
        assert!(json.is_ok());
    }

    #[test]
    fn test_training_dataset_temporal_split() {
        let records = vec![
            TrainingRecord {
                drug_name: "old_drug".to_string(),
                event: "headache".to_string(),
                features: vec![0.1, 0.2],
                label: false,
                approval_year: Some(1995),
                prr: None,
                ror: None,
                case_count: None,
            },
            TrainingRecord {
                drug_name: "new_drug".to_string(),
                event: "nausea".to_string(),
                features: vec![0.3, 0.4],
                label: true,
                approval_year: Some(2015),
                prr: Some(3.0),
                ror: Some(2.8),
                case_count: Some(42),
            },
        ];
        let dataset = TrainingDataset {
            records,
            feature_dim: 2,
            positive_count: 1,
            negative_count: 1,
            cohort_years: vec![1995, 2015],
            generated_at: chrono::Utc::now(),
            version: "v1.0".to_string(),
        };

        let (train, test) = dataset.temporal_split(2010);
        assert_eq!(train.len(), 1);
        assert_eq!(test.len(), 1);
        assert_eq!(train[0].drug_name, "old_drug");
        assert_eq!(test[0].drug_name, "new_drug");
    }

    #[test]
    fn test_class_balance_zero_total() {
        let dataset = TrainingDataset {
            records: vec![],
            feature_dim: 0,
            positive_count: 0,
            negative_count: 0,
            cohort_years: vec![],
            generated_at: chrono::Utc::now(),
            version: "v1.0".to_string(),
        };
        assert_eq!(dataset.class_balance(), 0.0);
    }

    #[test]
    fn test_class_balance_equal_split() {
        let dataset = TrainingDataset {
            records: vec![],
            feature_dim: 0,
            positive_count: 5,
            negative_count: 5,
            cohort_years: vec![],
            generated_at: chrono::Utc::now(),
            version: "v1.0".to_string(),
        };
        assert!((dataset.class_balance() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_structured_signal_no_smiles_when_compound_absent() {
        let sig = make_signal("unknown_drug", "rash", 2.5, 2.1);
        let structured = StructuredSignal {
            signal: sig,
            compound: None,
            has_structure: false,
        };
        assert!(structured.smiles().is_none());
        assert!(!structured.has_structure);
    }

    #[test]
    fn test_alert_candidate_fields() {
        let candidate = AlertCandidate {
            fragment_smiles: "C(=O)O".to_string(),
            associated_events: vec!["GI bleeding".to_string()],
            confidence: 0.82,
            supporting_drugs: vec!["aspirin".to_string(), "ibuprofen".to_string()],
            mean_prr: 3.4,
            support_count: 2,
        };
        assert_eq!(candidate.supporting_drugs.len(), 2);
        assert!(candidate.confidence > 0.8);
    }
}
