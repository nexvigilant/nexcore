// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Training dataset generation for predictive safety models.
//!
//! Converts retrocasting results into labeled training records suitable
//! for supervised machine learning (QSAR toxicity prediction models).
//!
//! ## Feature Engineering
//!
//! **Current (T7 pending):** SMILES character n-gram features (unigrams + bigrams)
//! normalized to a fixed-dimension feature vector.
//!
//! **Planned (T7 + T6 complete):** Replace with:
//! - Morgan/ECFP4 fingerprint bits (2048 dims) from `nexcore-molcore`
//! - Molecular descriptors: MW, LogP, TPSA, HBA, HBD (5 dims)
//! - Total: 2053-dimensional feature vector
//!
//! ## Label Assignment
//!
//! Positive label (true): drug-event pair with PRR ≥ 2.0 (significant signal).
//! Negative label (false): drug present in FAERS with no significant signals.
//!
//! ## Temporal Cohort Splitting
//!
//! Records split by approval year for temporal validation (avoids data leakage):
//! - Training cohort: drugs approved before cutoff year
//! - Test cohort: drugs approved at/after cutoff year
//!
//! ## Tier: T3 (Σ + π + σ + →)
//! Sum (Σ) of persisted (π) sequences (σ) causally mapped (→) to labels.

use crate::error::{RetrocastError, RetrocastResult};
use crate::types::{RetrocastResult as RetrocastAnalysis, TrainingDataset, TrainingRecord};
use std::collections::HashSet;

/// Feature vector dimension for SMILES n-gram features.
///
/// 128 ASCII characters as unigram presence (128 dims) +
/// 32 selected meaningful bigrams (32 dims) = 160 total.
pub const FEATURE_DIM: usize = 160;

/// Generate a labeled training dataset from retrocasting results.
///
/// Each `RetrocastResult` may produce multiple `TrainingRecord`s —
/// one per significant drug-event pair (positive) plus one negative
/// record per drug with no significant signals.
///
/// # Errors
/// Returns `RetrocastError::EmptyInput` if `retrocast_results` is empty.
/// Returns `RetrocastError::TrainingError` on feature extraction failure.
pub fn generate_training_data(
    retrocast_results: &[RetrocastAnalysis],
) -> RetrocastResult<TrainingDataset> {
    if retrocast_results.is_empty() {
        return Err(RetrocastError::EmptyInput(
            "No retrocasting results provided for training data generation".to_string(),
        ));
    }

    let mut records: Vec<TrainingRecord> = Vec::new();
    let mut cohort_years: HashSet<u32> = HashSet::new();

    for result in retrocast_results {
        let smiles_opt = result.smiles.as_deref();
        let features = extract_features(smiles_opt);

        // Significant signals → positive records
        let significant_signals: Vec<_> = result
            .signals
            .iter()
            .filter(|s| s.is_significant())
            .collect();

        if significant_signals.is_empty() {
            // No significant signals — one negative record for this drug
            records.push(TrainingRecord {
                drug_name: result.drug_name.clone(),
                event: "no_signal".to_string(),
                features: features.clone(),
                label: false,
                approval_year: None,
                prr: None,
                ror: None,
                case_count: None,
            });
        } else {
            for signal in significant_signals {
                let record = TrainingRecord {
                    drug_name: result.drug_name.clone(),
                    event: signal.event.clone(),
                    features: features.clone(),
                    label: true,
                    approval_year: None,
                    prr: Some(signal.prr),
                    ror: Some(signal.ror),
                    case_count: Some(signal.case_count),
                };
                records.push(record);
            }
        }
    }

    // Collect cohort years
    for record in &records {
        if let Some(year) = record.approval_year {
            cohort_years.insert(year);
        }
    }

    let positive_count = records.iter().filter(|r| r.label).count();
    let negative_count = records.iter().filter(|r| !r.label).count();

    let mut years: Vec<u32> = cohort_years.into_iter().collect();
    years.sort();

    Ok(TrainingDataset {
        records,
        feature_dim: FEATURE_DIM,
        positive_count,
        negative_count,
        cohort_years: years,
        generated_at: nexcore_chrono::DateTime::now(),
        version: "v1.0-ngram".to_string(),
    })
}

/// Generate training data with explicit approval year annotations.
///
/// Accepts a parallel `approval_years` slice where `approval_years[i]`
/// is the approval year for `retrocast_results[i]`.
///
/// # Errors
/// Returns `RetrocastError::TrainingError` if slice lengths don't match.
pub fn generate_training_data_with_years(
    retrocast_results: &[RetrocastAnalysis],
    approval_years: &[Option<u32>],
) -> RetrocastResult<TrainingDataset> {
    if retrocast_results.len() != approval_years.len() {
        return Err(RetrocastError::TrainingError(format!(
            "Length mismatch: {} results vs {} years",
            retrocast_results.len(),
            approval_years.len()
        )));
    }

    let mut dataset = generate_training_data(retrocast_results)?;

    // Annotate records with approval years
    let mut record_idx = 0;
    for (result_idx, result) in retrocast_results.iter().enumerate() {
        let year = approval_years[result_idx];
        let sig_count = result.signals.iter().filter(|s| s.is_significant()).count();
        let records_for_drug = if sig_count == 0 { 1 } else { sig_count };

        for i in 0..records_for_drug {
            let idx = record_idx + i;
            if idx < dataset.records.len() {
                dataset.records[idx].approval_year = year;
            }
        }
        record_idx += records_for_drug;

        if let Some(year) = year {
            if !dataset.cohort_years.contains(&year) {
                dataset.cohort_years.push(year);
            }
        }
    }

    dataset.cohort_years.sort();
    Ok(dataset)
}

/// Extract a fixed-dimension feature vector from a SMILES string.
///
/// ## Current implementation (T7 pending)
///
/// Uses character-level features:
/// - Dims 0–127: ASCII character presence flags (normalized 0.0 or 1.0)
/// - Dims 128–159: Selected SMILES bigram counts (normalized by SMILES length)
///
/// This is a valid structural proxy: molecules with similar substructures
/// share SMILES characters (e.g., `C(=O)` for carbonyl groups).
///
/// ## Planned (T7 complete)
///
/// Replace with `morgan_fingerprint(graph, radius=2, nbits=2048)` from
/// `nexcore_molcore::fingerprint`, yielding a 2048-bit vector plus
/// 5 molecular descriptors.
pub fn extract_features(smiles: Option<&str>) -> Vec<f64> {
    let mut features = vec![0.0f64; FEATURE_DIM];

    let smiles_str = match smiles {
        Some(s) if !s.is_empty() => s,
        _ => return features, // Zero vector for missing structures
    };

    let bytes = smiles_str.as_bytes();
    let len = bytes.len() as f64;

    // Dims 0–127: ASCII character presence (0.0 or 1.0)
    for &byte in bytes {
        if (byte as usize) < 128 {
            features[byte as usize] = 1.0;
        }
    }

    // Dims 128–159: Selected SMILES bigram normalized frequencies
    // These bigrams correspond to important chemical motifs
    let bigrams: [(&[u8], usize); 32] = [
        (b"C=", 128), // C=X double bond
        (b"=O", 129), // ketone/aldehyde
        (b"C(", 130), // branch start
        (b"(=", 131), // branch with double bond
        (b"O)", 132), // oxygen branch close
        (b"c1", 133), // aromatic ring open
        (b"cc", 134), // aromatic C-C
        (b"cn", 135), // aromatic C-N
        (b"c(", 136), // aromatic branch
        (b"N(", 137), // nitrogen branch
        (b"C#", 138), // triple bond
        (b"C@", 139), // stereo center
        (b"[N", 140), // bracket nitrogen
        (b"[C", 141), // bracket carbon
        (b"[O", 142), // bracket oxygen
        (b"+]", 143), // positive charge
        (b"-]", 144), // negative charge  (note: raw byte - not bond)
        (b"OC", 145), // ether/alcohol
        (b"NC", 146), // amine
        (b"SC", 147), // thioether
        (b"FC", 148), // fluoride
        (b"Cl", 149), // chloride
        (b"Br", 150), // bromide
        (b"c2", 151), // second aromatic ring
        (b"c3", 152), // third aromatic ring
        (b"n1", 153), // aromatic N ring
        (b"o1", 154), // aromatic O ring
        (b"s1", 155), // aromatic S ring
        (b"C1", 156), // aliphatic ring C
        (b"N1", 157), // aliphatic ring N
        (b"O1", 158), // aliphatic ring O
        (b"C2", 159), // second aliphatic ring
    ];

    for (bigram, dim) in &bigrams {
        let count = count_bigram(bytes, bigram) as f64;
        features[*dim] = if len > 0.0 { count / len } else { 0.0 };
    }

    features
}

/// Count occurrences of a byte bigram in a byte slice.
fn count_bigram(haystack: &[u8], bigram: &[u8]) -> usize {
    if bigram.len() != 2 || haystack.len() < 2 {
        return 0;
    }
    haystack
        .windows(2)
        .filter(|w| w[0] == bigram[0] && w[1] == bigram[1])
        .count()
}

/// Compute basic dataset statistics for logging and validation.
pub fn dataset_stats(dataset: &TrainingDataset) -> DatasetStats {
    let total = dataset.records.len();
    let mean_prr: f64 = if total == 0 {
        0.0
    } else {
        dataset.records.iter().filter_map(|r| r.prr).sum::<f64>() / total as f64
    };

    DatasetStats {
        total_records: total,
        positive_count: dataset.positive_count,
        negative_count: dataset.negative_count,
        class_balance: dataset.class_balance(),
        mean_prr_positive: mean_prr,
        feature_dim: dataset.feature_dim,
        cohort_years: dataset.cohort_years.clone(),
    }
}

/// Summary statistics for a training dataset.
#[derive(Debug, Clone)]
pub struct DatasetStats {
    pub total_records: usize,
    pub positive_count: usize,
    pub negative_count: usize,
    pub class_balance: f64,
    pub mean_prr_positive: f64,
    pub feature_dim: usize,
    pub cohort_years: Vec<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{RetrocastResult as RetrocastAnalysis, SignalRecord};

    fn make_result(
        drug: &str,
        smiles: Option<&str>,
        events: Vec<(&str, f64)>,
    ) -> RetrocastAnalysis {
        let signals = events
            .into_iter()
            .map(|(event, prr)| SignalRecord {
                drug: drug.to_string(),
                event: event.to_string(),
                prr,
                ror: prr * 0.9,
                case_count: 40,
                ror_lci: Some(prr * 0.7),
                prr_chi_sq: Some(if prr >= 2.0 { 5.0 } else { 1.0 }),
            })
            .collect();

        RetrocastAnalysis {
            drug_name: drug.to_string(),
            smiles: smiles.map(|s| s.to_string()),
            signals,
            structural_clusters: vec![],
            new_alert_candidates: vec![],
            confidence: 0.7,
            analyzed_at: nexcore_chrono::DateTime::now(),
        }
    }

    #[test]
    fn test_generate_empty_returns_error() {
        let result = generate_training_data(&[]);
        assert!(result.is_err());
        assert!(matches!(result, Err(RetrocastError::EmptyInput(_))));
    }

    #[test]
    fn test_generate_positive_records_for_significant_signals() {
        let results = vec![make_result(
            "aspirin",
            Some("CC(=O)Oc1ccccc1C(=O)O"),
            vec![("GI bleeding", 4.2), ("peptic ulcer", 3.1)],
        )];

        let dataset = generate_training_data(&results);
        assert!(dataset.is_ok());
        if let Ok(ds) = dataset {
            assert_eq!(ds.positive_count, 2);
            assert_eq!(ds.negative_count, 0);
            assert!(ds.records.iter().all(|r| r.label));
        }
    }

    #[test]
    fn test_generate_negative_record_for_no_signals() {
        let results = vec![make_result(
            "placebo_drug",
            Some("CC"),
            vec![("headache", 1.1)], // Below PRR threshold
        )];

        let dataset = generate_training_data(&results);
        assert!(dataset.is_ok());
        if let Ok(ds) = dataset {
            assert_eq!(ds.positive_count, 0);
            assert_eq!(ds.negative_count, 1);
            assert!(ds.records[0].event == "no_signal");
        }
    }

    #[test]
    fn test_feature_vector_dimension() {
        let features = extract_features(Some("CC(=O)Oc1ccccc1C(=O)O"));
        assert_eq!(features.len(), FEATURE_DIM);
    }

    #[test]
    fn test_feature_vector_zeros_for_no_smiles() {
        let features = extract_features(None);
        assert!(features.iter().all(|&f| f == 0.0));
    }

    #[test]
    fn test_feature_vector_zeros_for_empty_smiles() {
        let features = extract_features(Some(""));
        assert!(features.iter().all(|&f| f == 0.0));
    }

    #[test]
    fn test_aspirin_features_differ_from_caffeine() {
        let aspirin_f = extract_features(Some("CC(=O)Oc1ccccc1C(=O)O"));
        let caffeine_f = extract_features(Some("Cn1cnc2c1c(=O)n(c(=O)n2C)C"));

        let diff: f64 = aspirin_f
            .iter()
            .zip(caffeine_f.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();

        assert!(
            diff > 0.0,
            "Aspirin and caffeine should have different feature vectors"
        );
    }

    #[test]
    fn test_aspirin_ibuprofen_closer_than_aspirin_caffeine() {
        let aspirin_f = extract_features(Some("CC(=O)Oc1ccccc1C(=O)O"));
        let ibuprofen_f = extract_features(Some("CC(C)Cc1ccc(cc1)C(C)C(=O)O"));
        let caffeine_f = extract_features(Some("Cn1cnc2c1c(=O)n(c(=O)n2C)C"));

        let dist_nsaid: f64 = aspirin_f
            .iter()
            .zip(ibuprofen_f.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();

        let dist_unrelated: f64 = aspirin_f
            .iter()
            .zip(caffeine_f.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();

        assert!(
            dist_nsaid < dist_unrelated,
            "Aspirin-ibuprofen dist ({dist_nsaid:.3}) should be less than aspirin-caffeine ({dist_unrelated:.3})"
        );
    }

    #[test]
    fn test_temporal_split_on_generated_data() {
        let results = vec![make_result(
            "ibuprofen",
            Some("CC(C)Cc1ccc(cc1)C(C)C(=O)O"),
            vec![("GI bleed", 3.5)],
        )];
        let years = vec![Some(1990u32)];

        let dataset = generate_training_data_with_years(&results, &years);
        assert!(dataset.is_ok());
        if let Ok(ds) = dataset {
            let (train, test) = ds.temporal_split(2000);
            assert_eq!(train.len(), 1, "1990 drug should be in train pre-2000");
            assert_eq!(test.len(), 0);
        }
    }

    #[test]
    fn test_year_mismatch_returns_error() {
        let results = vec![make_result("drug1", None, vec![])];
        let years: Vec<Option<u32>> = vec![Some(2000), Some(2010)]; // Length mismatch
        let err = generate_training_data_with_years(&results, &years);
        assert!(err.is_err());
        assert!(matches!(err, Err(RetrocastError::TrainingError(_))));
    }

    #[test]
    fn test_dataset_stats_class_balance() {
        let results = vec![
            make_result("drug_a", Some("CC"), vec![("event_1", 3.0)]),
            make_result("drug_b", Some("CO"), vec![("headache", 1.2)]), // no signal
        ];

        let ds = generate_training_data(&results);
        assert!(ds.is_ok());
        if let Ok(dataset) = ds {
            let stats = dataset_stats(&dataset);
            assert_eq!(stats.positive_count, 1);
            assert_eq!(stats.negative_count, 1);
            assert!((stats.class_balance - 0.5).abs() < 0.01);
        }
    }
}
