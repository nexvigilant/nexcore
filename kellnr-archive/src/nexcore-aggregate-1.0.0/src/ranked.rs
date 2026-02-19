//! Ranked aggregation — the κ (Comparison) primitive.
//!
//! Provides comparison-based operations: TopN, percentile, ranking,
//! and ordered aggregation over named values.
//!
//! ## Tier: T2-P (κ + Σ + N)
//!
//! ## Lifecycle
//! - **begins**: Collection of (name, value) pairs provided
//! - **exists**: Internal sorted representation maintained
//! - **changes**: Ranking computed via comparison operations
//! - **persists**: Ranked results returned as ordered sequence
//! - **ends**: Consumer receives ranked view

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Ranked Entry
// ---------------------------------------------------------------------------

/// A named value that can be ranked by comparison.
///
/// ## Primitive Grounding
/// - κ (Comparison): Implements Ord for total ordering
/// - N (Quantity): Carries numeric value
/// - λ (Location): Named identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ranked {
    /// Name/identifier of this entry.
    pub name: String,
    /// The numeric value used for ranking.
    pub value: f64,
    /// Rank position (1-based, assigned after sorting).
    pub rank: usize,
}

impl Ranked {
    /// Create a new ranked entry (rank assigned as 0 until sorted).
    pub fn new(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
            rank: 0,
        }
    }
}

impl PartialEq for Ranked {
    fn eq(&self, other: &Self) -> bool {
        self.value.total_cmp(&other.value) == Ordering::Equal
    }
}

impl Eq for Ranked {}

impl PartialOrd for Ranked {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ranked {
    fn cmp(&self, other: &Self) -> Ordering {
        // Descending order: higher values rank first
        other.value.total_cmp(&self.value)
    }
}

// ---------------------------------------------------------------------------
// Ranking Operations
// ---------------------------------------------------------------------------

/// Assign ranks to a collection of (name, value) pairs.
///
/// Returns entries sorted descending by value with 1-based rank.
///
/// ## Primitive Grounding
/// - κ (Comparison): Sort-based ranking
/// - σ (Sequence): Produces ordered sequence
/// - N (Quantity): Rank numbers
pub fn rank(items: &[(impl AsRef<str>, f64)]) -> Vec<Ranked> {
    let mut ranked: Vec<Ranked> = items
        .iter()
        .map(|(name, value)| Ranked::new(name.as_ref(), *value))
        .collect();
    ranked.sort(); // Uses Ord impl (descending)
    for (i, entry) in ranked.iter_mut().enumerate() {
        entry.rank = i + 1;
    }
    ranked
}

/// Get the top N entries by value.
///
/// Tier: T2-P (κ + Σ + N)
pub fn top_n(items: &[(impl AsRef<str>, f64)], n: usize) -> Vec<Ranked> {
    let ranked = rank(items);
    ranked.into_iter().take(n).collect()
}

/// Get the bottom N entries by value.
///
/// Tier: T2-P (κ + Σ + N)
pub fn bottom_n(items: &[(impl AsRef<str>, f64)], n: usize) -> Vec<Ranked> {
    let ranked = rank(items);
    let len = ranked.len();
    if n >= len {
        return ranked;
    }
    ranked.into_iter().skip(len - n).collect()
}

/// Compute the value at a given percentile (0.0 to 1.0).
///
/// Uses linear interpolation between nearest ranks.
///
/// Tier: T2-P (κ + N + ∝)
pub fn percentile(values: &[f64], p: f64) -> Option<f64> {
    if values.is_empty() || !(0.0..=1.0).contains(&p) {
        return None;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let n = sorted.len();

    if n == 1 {
        return Some(sorted[0]);
    }

    let rank_f = p * (n - 1) as f64;
    let lower = rank_f.floor() as usize;
    let upper = rank_f.ceil() as usize;
    let frac = rank_f - lower as f64;

    if lower == upper {
        Some(sorted[lower])
    } else {
        Some(sorted[lower] * (1.0 - frac) + sorted[upper] * frac)
    }
}

/// Compute quartile boundaries (Q1, Q2/median, Q3).
///
/// Tier: T2-C (κ + N + ∝ + Σ)
pub fn quartiles(values: &[f64]) -> Option<(f64, f64, f64)> {
    let q1 = percentile(values, 0.25)?;
    let q2 = percentile(values, 0.50)?;
    let q3 = percentile(values, 0.75)?;
    Some((q1, q2, q3))
}

/// Interquartile range: Q3 - Q1.
///
/// Tier: T2-C (κ + N + ∝)
pub fn iqr(values: &[f64]) -> Option<f64> {
    let (q1, _, q3) = quartiles(values)?;
    Some(q3 - q1)
}

/// Detect outliers using the IQR method (values outside 1.5×IQR from quartiles).
///
/// Returns (outlier_name, outlier_value, direction) tuples.
///
/// Tier: T2-C (κ + ∂ + N + ∝)
pub fn detect_outliers(items: &[(impl AsRef<str>, f64)]) -> Vec<(String, f64, OutlierDirection)> {
    let values: Vec<f64> = items.iter().map(|(_, v)| *v).collect();
    let (q1, _, q3) = match quartiles(&values) {
        Some(q) => q,
        None => return Vec::new(),
    };

    let iqr_val = q3 - q1;
    let lower_fence = q1 - 1.5 * iqr_val;
    let upper_fence = q3 + 1.5 * iqr_val;

    items
        .iter()
        .filter_map(|(name, value)| {
            if *value < lower_fence {
                Some((name.as_ref().to_string(), *value, OutlierDirection::Low))
            } else if *value > upper_fence {
                Some((name.as_ref().to_string(), *value, OutlierDirection::High))
            } else {
                None
            }
        })
        .collect()
}

/// Direction of an outlier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutlierDirection {
    /// Below lower fence (Q1 - 1.5×IQR).
    Low,
    /// Above upper fence (Q3 + 1.5×IQR).
    High,
}

// ---------------------------------------------------------------------------
// Normalized Ranking
// ---------------------------------------------------------------------------

/// Normalize values to [0.0, 1.0] range (min-max normalization).
///
/// Tier: T2-P (κ + ∝ + N)
pub fn normalize(items: &[(impl AsRef<str>, f64)]) -> Vec<Ranked> {
    if items.is_empty() {
        return Vec::new();
    }

    let min = items.iter().map(|(_, v)| *v).fold(f64::INFINITY, f64::min);
    let max = items
        .iter()
        .map(|(_, v)| *v)
        .fold(f64::NEG_INFINITY, f64::max);

    let range = max - min;
    if range.abs() < f64::EPSILON {
        // All same value → all get 1.0
        return items
            .iter()
            .enumerate()
            .map(|(i, (name, _))| Ranked {
                name: name.as_ref().to_string(),
                value: 1.0,
                rank: i + 1,
            })
            .collect();
    }

    let mut ranked: Vec<Ranked> = items
        .iter()
        .map(|(name, value)| Ranked::new(name.as_ref(), (value - min) / range))
        .collect();
    ranked.sort();
    for (i, entry) in ranked.iter_mut().enumerate() {
        entry.rank = i + 1;
    }
    ranked
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_basic() {
        let items = vec![
            ("alpha", 3.0),
            ("beta", 1.0),
            ("gamma", 5.0),
            ("delta", 2.0),
        ];
        let ranked = rank(&items);
        assert_eq!(ranked[0].name, "gamma"); // highest
        assert_eq!(ranked[0].rank, 1);
        assert_eq!(ranked[3].name, "beta"); // lowest
        assert_eq!(ranked[3].rank, 4);
    }

    #[test]
    fn test_top_n() {
        let items = vec![("a", 10.0), ("b", 30.0), ("c", 20.0), ("d", 40.0)];
        let top = top_n(&items, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].name, "d");
        assert_eq!(top[1].name, "b");
    }

    #[test]
    fn test_bottom_n() {
        let items = vec![("a", 10.0), ("b", 30.0), ("c", 20.0), ("d", 40.0)];
        let bottom = bottom_n(&items, 2);
        assert_eq!(bottom.len(), 2);
        assert_eq!(bottom[0].name, "c");
        assert_eq!(bottom[1].name, "a");
    }

    #[test]
    fn test_percentile_median() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let p50 = percentile(&values, 0.5);
        assert_eq!(p50, Some(3.0));
    }

    #[test]
    fn test_percentile_extremes() {
        let values = vec![10.0, 20.0, 30.0];
        assert_eq!(percentile(&values, 0.0), Some(10.0));
        assert_eq!(percentile(&values, 1.0), Some(30.0));
    }

    #[test]
    fn test_percentile_empty() {
        let values: Vec<f64> = vec![];
        assert_eq!(percentile(&values, 0.5), None);
    }

    #[test]
    fn test_percentile_invalid() {
        let values = vec![1.0, 2.0];
        assert_eq!(percentile(&values, 1.5), None);
        assert_eq!(percentile(&values, -0.1), None);
    }

    #[test]
    fn test_quartiles() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let (q1, q2, q3) = quartiles(&values).unwrap_or((0.0, 0.0, 0.0));
        assert!((q2 - 4.0).abs() < f64::EPSILON); // median
        assert!((q1 - 2.5).abs() < f64::EPSILON);
        assert!((q3 - 5.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_iqr() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let result = iqr(&values);
        assert!(result.is_some());
        assert!((result.unwrap_or(0.0) - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_detect_outliers() {
        let items = vec![
            ("normal1", 5.0),
            ("normal2", 6.0),
            ("normal3", 7.0),
            ("normal4", 5.5),
            ("outlier_high", 100.0),
            ("outlier_low", -50.0),
        ];
        let outliers = detect_outliers(&items);
        assert_eq!(outliers.len(), 2);
        let high: Vec<_> = outliers
            .iter()
            .filter(|o| o.2 == OutlierDirection::High)
            .collect();
        let low: Vec<_> = outliers
            .iter()
            .filter(|o| o.2 == OutlierDirection::Low)
            .collect();
        assert_eq!(high.len(), 1);
        assert_eq!(low.len(), 1);
        assert_eq!(high[0].0, "outlier_high");
        assert_eq!(low[0].0, "outlier_low");
    }

    #[test]
    fn test_normalize() {
        let items = vec![("a", 0.0), ("b", 50.0), ("c", 100.0)];
        let normed = normalize(&items);
        assert_eq!(normed.len(), 3);
        // c=100 → 1.0, rank 1
        assert_eq!(normed[0].name, "c");
        assert!((normed[0].value - 1.0).abs() < f64::EPSILON);
        // a=0 → 0.0, rank 3
        assert_eq!(normed[2].name, "a");
        assert!((normed[2].value - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_normalize_equal_values() {
        let items = vec![("a", 5.0), ("b", 5.0), ("c", 5.0)];
        let normed = normalize(&items);
        // All equal → all get 1.0
        for entry in &normed {
            assert!((entry.value - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_ranked_ord() {
        let a = Ranked::new("a", 10.0);
        let b = Ranked::new("b", 20.0);
        // b should come before a (descending)
        assert!(b < a);
    }
}
