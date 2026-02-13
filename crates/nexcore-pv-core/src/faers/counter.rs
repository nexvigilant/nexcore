//! FAERS counting operations for signal detection.
//!
//! Stratified counting, age-adjusted counts, and parallel aggregation.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stratified count key
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct StratumKey {
    /// Drug name
    pub drug: String,
    /// Event (PT)
    pub event: String,
    /// Stratum value
    pub stratum: String,
}

/// Stratified drug-event counts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StratifiedCounts {
    /// Counts by stratum
    pub counts: HashMap<StratumKey, u32>,
    /// Stratum totals
    pub stratum_totals: HashMap<String, u32>,
    /// Drug totals
    pub drug_totals: HashMap<String, u32>,
    /// Event totals
    pub event_totals: HashMap<String, u32>,
}

impl StratifiedCounts {
    /// Create new stratified counts
    #[must_use]
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            stratum_totals: HashMap::new(),
            drug_totals: HashMap::new(),
            event_totals: HashMap::new(),
        }
    }

    /// Add a count
    pub fn add(&mut self, drug: &str, event: &str, stratum: &str) {
        let key = StratumKey {
            drug: drug.into(),
            event: event.into(),
            stratum: stratum.into(),
        };
        *self.counts.entry(key).or_insert(0) += 1;
        *self.stratum_totals.entry(stratum.into()).or_insert(0) += 1;
        *self.drug_totals.entry(drug.into()).or_insert(0) += 1;
        *self.event_totals.entry(event.into()).or_insert(0) += 1;
    }

    /// Get count for a key
    #[must_use]
    pub fn get(&self, drug: &str, event: &str, stratum: &str) -> u32 {
        let key = StratumKey {
            drug: drug.into(),
            event: event.into(),
            stratum: stratum.into(),
        };
        *self.counts.get(&key).unwrap_or(&0)
    }
}

impl Default for StratifiedCounts {
    fn default() -> Self {
        Self::new()
    }
}

/// Age-adjusted count result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeAdjustedCount {
    /// Drug
    pub drug: String,
    /// Event
    pub event: String,
    /// Raw count
    pub raw_count: u32,
    /// Age-adjusted count
    pub adjusted_count: f64,
    /// Standard population weights applied
    pub weights_used: Vec<f64>,
}

/// Age group weights (US 2000 standard)
pub const AGE_WEIGHTS: &[((&str, &str), f64)] = &[
    (("0", "4"), 0.069),
    (("5", "14"), 0.145),
    (("15", "24"), 0.139),
    (("25", "34"), 0.135),
    (("35", "44"), 0.162),
    (("45", "54"), 0.134),
    (("55", "64"), 0.087),
    (("65", "74"), 0.066),
    (("75", "84"), 0.044),
    (("85", "+"), 0.019),
];

/// Calculate age-adjusted count
#[must_use]
pub fn age_adjust(strat: &StratifiedCounts, drug: &str, event: &str) -> AgeAdjustedCount {
    let groups = [
        "0-4", "5-14", "15-24", "25-34", "35-44", "45-54", "55-64", "65-74", "75-84", "85+",
    ];
    let weights = [
        0.069, 0.145, 0.139, 0.135, 0.162, 0.134, 0.087, 0.066, 0.044, 0.019,
    ];
    let mut raw = 0u32;
    let mut adjusted = 0.0;
    let mut used_weights = vec![];
    for (g, w) in groups.iter().zip(weights.iter()) {
        let count = strat.get(drug, event, g);
        let pop = strat.stratum_totals.get(*g).copied().unwrap_or(1).max(1);
        let rate = count as f64 / pop as f64;
        raw += count;
        adjusted += rate * w;
        used_weights.push(*w);
    }
    AgeAdjustedCount {
        drug: drug.into(),
        event: event.into(),
        raw_count: raw,
        adjusted_count: adjusted,
        weights_used: used_weights,
    }
}

/// Count pairs in parallel
#[must_use]
pub fn count_pairs_parallel(data: &[(String, String)]) -> HashMap<(String, String), u32> {
    data.par_iter()
        .fold(HashMap::new, |mut acc, (d, e)| {
            *acc.entry((d.clone(), e.clone())).or_insert(0) += 1;
            acc
        })
        .reduce(HashMap::new, |mut a, b| {
            for (k, v) in b {
                *a.entry(k).or_insert(0) += v;
            }
            a
        })
}

/// Batch counting result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCountResult {
    /// Drug totals
    pub drugs: HashMap<String, u32>,
    /// Event totals
    pub events: HashMap<String, u32>,
    /// Pair counts
    pub pairs: HashMap<(String, String), u32>,
    /// Total records
    pub total: u32,
}

/// Count drugs and events in batch
#[must_use]
pub fn batch_count(data: &[(String, String)]) -> BatchCountResult {
    let mut drugs = HashMap::new();
    let mut events = HashMap::new();
    let mut pairs = HashMap::new();
    for (d, e) in data {
        *drugs.entry(d.clone()).or_insert(0) += 1;
        *events.entry(e.clone()).or_insert(0) += 1;
        *pairs.entry((d.clone(), e.clone())).or_insert(0) += 1;
    }
    BatchCountResult {
        drugs,
        events,
        pairs,
        total: data.len() as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stratified() {
        let mut s = StratifiedCounts::new();
        s.add("Aspirin", "Headache", "25-34");
        s.add("Aspirin", "Headache", "25-34");
        s.add("Aspirin", "Headache", "35-44");
        assert_eq!(s.get("Aspirin", "Headache", "25-34"), 2);
        assert_eq!(s.get("Aspirin", "Headache", "35-44"), 1);
    }

    #[test]
    fn test_batch_count() {
        let data = vec![
            ("A".into(), "X".into()),
            ("A".into(), "X".into()),
            ("B".into(), "Y".into()),
        ];
        let r = batch_count(&data);
        assert_eq!(r.total, 3);
        assert_eq!(r.drugs.get("A"), Some(&2));
    }

    #[test]
    fn test_parallel() {
        let data: Vec<(String, String)> = (0..100)
            .map(|i| (format!("D{}", i % 10), format!("E{}", i % 5)))
            .collect();
        let r = count_pairs_parallel(&data);
        assert!(!r.is_empty());
    }
}
