//! Generic fold combinators — the Σ (Sum) primitive.
//!
//! Provides composable fold operations over sequences. Every aggregation
//! reduces to a fold: sum, product, mean, min, max, count, and custom.
//!
//! ## Tier: T1 (Σ)
//!
//! ## Lifecycle
//! - **begins**: Fold initialized with accumulator seed
//! - **exists**: Accumulator carries state through iteration
//! - **changes**: Each element transforms accumulator via step function
//! - **persists**: Final accumulator returned as result
//! - **ends**: Fold completes when sequence exhausted

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Core Fold Trait
// ---------------------------------------------------------------------------

/// A composable fold operation over a sequence of items.
///
/// ## Primitive Grounding
/// - Σ (Sum): The fundamental aggregation pattern
/// - σ (Sequence): Iterates over ordered elements
/// - ς (State): Accumulator carries mutable state
///
/// ```
/// use nexcore_aggregate::fold::{Fold, SumFold};
///
/// let data = vec![1.0, 2.0, 3.0, 4.0];
/// let sum_fold = SumFold;
/// let result = sum_fold.fold(&data);
/// assert!((result - 10.0).abs() < f64::EPSILON);
/// ```
pub trait Fold<T> {
    /// The type produced by the fold.
    type Output;

    /// The initial accumulator value.
    fn init(&self) -> Self::Output;

    /// Combine accumulator with the next element.
    fn step(&self, acc: Self::Output, item: &T) -> Self::Output;

    /// Run the full fold over a slice.
    fn fold(&self, items: &[T]) -> Self::Output {
        items
            .iter()
            .fold(self.init(), |acc, item| self.step(acc, item))
    }
}

// ---------------------------------------------------------------------------
// Built-in Folds
// ---------------------------------------------------------------------------

/// Sum fold: Σ(items) → total.
///
/// Tier: T1 (Σ + N)
pub struct SumFold;

impl Fold<f64> for SumFold {
    type Output = f64;
    fn init(&self) -> f64 {
        0.0
    }
    fn step(&self, acc: f64, item: &f64) -> f64 {
        acc + item
    }
}

impl Fold<i64> for SumFold {
    type Output = i64;
    fn init(&self) -> i64 {
        0
    }
    fn step(&self, acc: i64, item: &i64) -> i64 {
        acc + item
    }
}

impl Fold<usize> for SumFold {
    type Output = usize;
    fn init(&self) -> usize {
        0
    }
    fn step(&self, acc: usize, item: &usize) -> usize {
        acc + item
    }
}

/// Product fold: Π(items) → product.
///
/// Tier: T1 (Σ + N)
pub struct ProductFold;

impl Fold<f64> for ProductFold {
    type Output = f64;
    fn init(&self) -> f64 {
        1.0
    }
    fn step(&self, acc: f64, item: &f64) -> f64 {
        acc * item
    }
}

/// Count fold: |items| → count.
///
/// Tier: T1 (Σ + N)
pub struct CountFold;

impl<T> Fold<T> for CountFold {
    type Output = usize;
    fn init(&self) -> usize {
        0
    }
    fn step(&self, acc: usize, _item: &T) -> usize {
        acc + 1
    }
}

/// Min fold: finds minimum value.
///
/// Tier: T1 (Σ + κ)
pub struct MinFold;

impl Fold<f64> for MinFold {
    type Output = Option<f64>;
    fn init(&self) -> Option<f64> {
        None
    }
    fn step(&self, acc: Option<f64>, item: &f64) -> Option<f64> {
        Some(match acc {
            None => *item,
            Some(a) if *item < a => *item,
            Some(a) => a,
        })
    }
}

/// Max fold: finds maximum value.
///
/// Tier: T1 (Σ + κ)
pub struct MaxFold;

impl Fold<f64> for MaxFold {
    type Output = Option<f64>;
    fn init(&self) -> Option<f64> {
        None
    }
    fn step(&self, acc: Option<f64>, item: &f64) -> Option<f64> {
        Some(match acc {
            None => *item,
            Some(a) if *item > a => *item,
            Some(a) => a,
        })
    }
}

// ---------------------------------------------------------------------------
// Mean (compound fold: Σ/N)
// ---------------------------------------------------------------------------

/// Running mean accumulator — carries state through step() calls.
///
/// Call `.value()` to extract the final `Option<f64>` result.
///
/// Tier: T2-P (Σ + N + ς)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MeanAccumulator {
    /// Running sum.
    pub sum: f64,
    /// Element count.
    pub count: usize,
}

impl MeanAccumulator {
    /// Extract the arithmetic mean, or `None` if no elements were folded.
    pub fn value(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }
}

/// Mean fold: Σ(items) / N → arithmetic mean.
///
/// Uses a running accumulator — both `step()` and `fold()` work correctly.
/// Call `.value()` on the result to get `Option<f64>`.
///
/// Tier: T2-P (Σ + N + ∝)
pub struct MeanFold;

impl Fold<f64> for MeanFold {
    type Output = MeanAccumulator;

    fn init(&self) -> MeanAccumulator {
        MeanAccumulator { sum: 0.0, count: 0 }
    }

    fn step(&self, acc: MeanAccumulator, item: &f64) -> MeanAccumulator {
        MeanAccumulator {
            sum: acc.sum + item,
            count: acc.count + 1,
        }
    }
    // Default fold() works correctly via init() + step() iteration
}

// ---------------------------------------------------------------------------
// Variance (compound fold: Σ(x-μ)²/N via Welford's)
// ---------------------------------------------------------------------------

/// Running variance accumulator using Welford's online algorithm.
///
/// Call `.value()` to extract sample variance, or `.mean()` for running mean.
/// Numerically stable — avoids catastrophic cancellation.
///
/// Tier: T2-C (Σ + N + ∝ + κ + ς)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VarianceAccumulator {
    /// Element count.
    pub count: usize,
    /// Running mean (Welford).
    pub mean: f64,
    /// Running M2 (sum of squared deviations from current mean).
    pub m2: f64,
}

impl VarianceAccumulator {
    /// Extract the sample variance, or `None` if fewer than 2 elements.
    pub fn value(&self) -> Option<f64> {
        if self.count < 2 {
            None
        } else {
            Some(self.m2 / (self.count - 1) as f64)
        }
    }

    /// Extract the running mean, or `None` if no elements.
    pub fn mean_value(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.mean)
        }
    }
}

/// Variance fold using Welford's online algorithm.
///
/// Uses a running accumulator — both `step()` and `fold()` work correctly.
/// Call `.value()` on the result to get `Option<f64>`.
///
/// Tier: T2-P (Σ + N + ∝ + κ)
pub struct VarianceFold;

impl Fold<f64> for VarianceFold {
    type Output = VarianceAccumulator;

    fn init(&self) -> VarianceAccumulator {
        VarianceAccumulator {
            count: 0,
            mean: 0.0,
            m2: 0.0,
        }
    }

    fn step(&self, acc: VarianceAccumulator, item: &f64) -> VarianceAccumulator {
        let count = acc.count + 1;
        let delta = item - acc.mean;
        let mean = acc.mean + delta / count as f64;
        let delta2 = item - mean;
        let m2 = acc.m2 + delta * delta2;
        VarianceAccumulator { count, mean, m2 }
    }
    // Default fold() works correctly via init() + step() iteration
}

// ---------------------------------------------------------------------------
// FoldPipeline — compose multiple folds
// ---------------------------------------------------------------------------

/// Results of a multi-fold pipeline.
///
/// Tier: T2-C (Σ + σ + ς + N)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldResults {
    /// Sum of all values.
    pub sum: f64,
    /// Count of values.
    pub count: usize,
    /// Minimum value (None if empty).
    pub min: Option<f64>,
    /// Maximum value (None if empty).
    pub max: Option<f64>,
    /// Arithmetic mean (None if empty).
    pub mean: Option<f64>,
    /// Sample variance (None if < 2 elements).
    pub variance: Option<f64>,
}

/// Run all standard folds in a single pass.
///
/// More efficient than running each fold separately — O(n) instead of O(6n).
///
/// ## Primitive Grounding
/// - Σ (Sum): Core operation
/// - σ (Sequence): Single-pass iteration
/// - ς (State): Running accumulators
/// - κ (Comparison): Min/max tracking
/// - N (Quantity): Count
/// - ∝ (Proportion): Mean = sum/count
pub fn fold_all(items: &[f64]) -> FoldResults {
    if items.is_empty() {
        return FoldResults {
            sum: 0.0,
            count: 0,
            min: None,
            max: None,
            mean: None,
            variance: None,
        };
    }

    let mut sum = 0.0_f64;
    let mut count: usize = 0;
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    // Welford's for variance
    let mut w_mean = 0.0_f64;
    let mut m2 = 0.0_f64;

    for &x in items {
        sum += x;
        count += 1;
        if x < min {
            min = x;
        }
        if x > max {
            max = x;
        }
        let delta = x - w_mean;
        w_mean += delta / count as f64;
        let delta2 = x - w_mean;
        m2 += delta * delta2;
    }

    FoldResults {
        sum,
        count,
        min: Some(min),
        max: Some(max),
        mean: Some(sum / count as f64),
        variance: if count >= 2 {
            Some(m2 / (count - 1) as f64)
        } else {
            None
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_fold_f64() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((SumFold.fold(&data) - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sum_fold_empty() {
        let data: Vec<f64> = vec![];
        assert!((SumFold.fold(&data) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sum_fold_i64() {
        let data = vec![10_i64, 20, 30];
        assert_eq!(SumFold.fold(&data), 60);
    }

    #[test]
    fn test_product_fold() {
        let data = vec![2.0, 3.0, 4.0];
        assert!((ProductFold.fold(&data) - 24.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_product_fold_empty() {
        let data: Vec<f64> = vec![];
        assert!((ProductFold.fold(&data) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_count_fold() {
        let data = vec![1.0, 2.0, 3.0];
        assert_eq!(CountFold.fold(&data), 3);
    }

    #[test]
    fn test_min_fold() {
        let data = vec![3.0, 1.0, 4.0, 1.5, 9.0];
        assert_eq!(MinFold.fold(&data), Some(1.0));
    }

    #[test]
    fn test_min_fold_empty() {
        let data: Vec<f64> = vec![];
        assert_eq!(MinFold.fold(&data), None);
    }

    #[test]
    fn test_max_fold() {
        let data = vec![3.0, 1.0, 4.0, 1.5, 9.0];
        assert_eq!(MaxFold.fold(&data), Some(9.0));
    }

    #[test]
    fn test_mean_fold() {
        let data = vec![2.0, 4.0, 6.0];
        let result = MeanFold.fold(&data);
        let mean = result.value();
        assert!(mean.is_some());
        assert!((mean.unwrap_or(0.0) - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mean_fold_step_composable() {
        // Verify step() works correctly (previously vestigial)
        let data = vec![2.0, 4.0, 6.0];
        let mut acc = MeanFold.init();
        for item in &data {
            acc = MeanFold.step(acc, item);
        }
        let mean = acc.value();
        assert!(mean.is_some());
        assert!((mean.unwrap_or(0.0) - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mean_fold_empty() {
        let data: Vec<f64> = vec![];
        assert_eq!(MeanFold.fold(&data).value(), None);
    }

    #[test]
    fn test_variance_fold() {
        // Variance of [2, 4, 6] = 4.0 (sample variance)
        let data = vec![2.0, 4.0, 6.0];
        let result = VarianceFold.fold(&data);
        let var = result.value();
        assert!(var.is_some());
        assert!((var.unwrap_or(0.0) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_variance_fold_step_composable() {
        // Verify step() works correctly (previously vestigial)
        let data = vec![2.0, 4.0, 6.0];
        let mut acc = VarianceFold.init();
        for item in &data {
            acc = VarianceFold.step(acc, item);
        }
        let var = acc.value();
        assert!(var.is_some());
        assert!((var.unwrap_or(0.0) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_variance_fold_single() {
        let data = vec![5.0];
        assert_eq!(VarianceFold.fold(&data).value(), None);
    }

    #[test]
    fn test_fold_all() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = fold_all(&data);
        assert_eq!(r.count, 5);
        assert!((r.sum - 15.0).abs() < f64::EPSILON);
        assert_eq!(r.min, Some(1.0));
        assert_eq!(r.max, Some(5.0));
        assert!((r.mean.unwrap_or(0.0) - 3.0).abs() < f64::EPSILON);
        // Sample variance of [1,2,3,4,5] = 2.5
        assert!((r.variance.unwrap_or(0.0) - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_fold_all_empty() {
        let r = fold_all(&[]);
        assert_eq!(r.count, 0);
        assert_eq!(r.min, None);
        assert_eq!(r.mean, None);
    }
}
