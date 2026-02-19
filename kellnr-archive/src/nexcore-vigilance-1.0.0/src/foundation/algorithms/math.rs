//! # Mathematical Utilities
//!
//! Statistical calculations and metrics for skill analysis.

use serde::{Deserialize, Serialize};

/// Statistical summary of a dataset
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatisticalSummary {
    /// Number of data points
    pub count: usize,
    /// Arithmetic mean
    pub mean: f64,
    /// Population variance
    pub variance: f64,
    /// Population standard deviation
    pub std_dev: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
}

/// Calculate variance between actual and target values
#[must_use]
pub fn calculate_variance(actual: f64, target: f64) -> f64 {
    (actual - target).powi(2)
}

/// Calculate statistical summary of a dataset
#[must_use]
pub fn statistical_summary(data: &[f64]) -> Option<StatisticalSummary> {
    if data.is_empty() {
        return None;
    }

    let count = data.len();
    let sum: f64 = data.iter().sum();
    let mean = sum / count as f64;

    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / count as f64;

    let std_dev = variance.sqrt();

    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    Some(StatisticalSummary {
        count,
        mean,
        variance,
        std_dev,
        min,
        max,
    })
}

/// Calculate the coefficient of variation (CV)
#[must_use]
pub fn coefficient_of_variation(data: &[f64]) -> Option<f64> {
    let summary = statistical_summary(data)?;
    if summary.mean.abs() < f64::EPSILON {
        return None;
    }
    Some(summary.std_dev / summary.mean.abs())
}

/// Calculate percentile of a dataset
#[must_use]
pub fn percentile(data: &[f64], p: f64) -> Option<f64> {
    if data.is_empty() || !(0.0..=100.0).contains(&p) {
        return None;
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    Some(sorted[idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variance() {
        assert_eq!(calculate_variance(10.0, 8.0), 4.0);
        assert_eq!(calculate_variance(5.0, 5.0), 0.0);
    }

    #[test]
    fn test_statistical_summary() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let summary = statistical_summary(&data).unwrap();

        assert_eq!(summary.count, 8);
        assert_eq!(summary.mean, 5.0);
        assert_eq!(summary.min, 2.0);
        assert_eq!(summary.max, 9.0);
        assert!((summary.variance - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_percentile() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&data, 50.0), Some(3.0));
        assert_eq!(percentile(&data, 0.0), Some(1.0));
        assert_eq!(percentile(&data, 100.0), Some(5.0));
    }

    #[test]
    fn test_empty_data() {
        let data: Vec<f64> = vec![];
        assert!(statistical_summary(&data).is_none());
        assert!(percentile(&data, 50.0).is_none());
    }
}
