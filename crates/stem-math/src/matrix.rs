//! # Matrix: Dense Numerical Matrix Type
//!
//! Row-major dense matrix for linear algebra operations needed by Markov chains,
//! transition probability computation, and stationary distribution calculation.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | row-to-column transformations (DOMINANT) |
//! | T1: Quantity (N) | numeric values |
//! | T1: Boundary (∂) | dimension constraints |
//!
//! ## Design
//!
//! Dense `Vec<f64>` storage, row-major layout. O(rows × cols) space.
//! Optimized for small-to-medium matrices typical of Markov chain analysis
//! (state counts in the tens to low hundreds).
//!
//! ## Relationship to nexcore-oracle::TransitionMatrix
//!
//! `nexcore-oracle::TransitionMatrix` is a HashMap-based frequency counter (T3).
//! This `Matrix` is the canonical numeric matrix (T2-P) — the mathematical
//! primitive that TransitionMatrix data can be converted into for eigenvalue
//! analysis, power iteration, and n-step probability computation.

use serde::{Deserialize, Serialize};

// ============================================================================
// Core Type
// ============================================================================

/// A dense numerical matrix with f64 entries.
///
/// Row-major storage: element (i, j) is at index `i * cols + j`.
///
/// # Invariants
///
/// - `data.len() == rows * cols`
/// - `rows >= 1 && cols >= 1` (enforced at construction)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

// ============================================================================
// Construction
// ============================================================================

impl Matrix {
    /// Create a zero matrix of given dimensions.
    ///
    /// Returns `None` if rows or cols is 0.
    #[must_use]
    pub fn zeros(rows: usize, cols: usize) -> Option<Self> {
        if rows == 0 || cols == 0 {
            return None;
        }
        Some(Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        })
    }

    /// Create an identity matrix of size n×n.
    ///
    /// Returns `None` if n is 0.
    #[must_use]
    pub fn identity(n: usize) -> Option<Self> {
        if n == 0 {
            return None;
        }
        let mut data = vec![0.0; n * n];
        for i in 0..n {
            data[i * n + i] = 1.0;
        }
        Some(Self {
            rows: n,
            cols: n,
            data,
        })
    }

    /// Create a matrix from row-major data.
    ///
    /// Returns `None` if data length doesn't match rows × cols, or dimensions are 0.
    #[must_use]
    pub fn from_flat(rows: usize, cols: usize, data: Vec<f64>) -> Option<Self> {
        if rows == 0 || cols == 0 || data.len() != rows * cols {
            return None;
        }
        Some(Self { rows, cols, data })
    }

    /// Create a matrix from a vector of row vectors.
    ///
    /// Returns `None` if rows are empty or have inconsistent lengths.
    #[must_use]
    pub fn from_rows(row_data: &[Vec<f64>]) -> Option<Self> {
        if row_data.is_empty() {
            return None;
        }
        let cols = row_data[0].len();
        if cols == 0 || row_data.iter().any(|r| r.len() != cols) {
            return None;
        }
        let data: Vec<f64> = row_data.iter().flat_map(|r| r.iter().copied()).collect();
        Some(Self {
            rows: row_data.len(),
            cols,
            data,
        })
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Number of rows.
    #[must_use]
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of columns.
    #[must_use]
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Whether the matrix is square.
    #[must_use]
    pub fn is_square(&self) -> bool {
        self.rows == self.cols
    }

    /// Get element at (row, col).
    ///
    /// Returns `None` if out of bounds.
    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> Option<f64> {
        if row < self.rows && col < self.cols {
            Some(self.data[row * self.cols + col])
        } else {
            None
        }
    }

    /// Set element at (row, col).
    ///
    /// Returns `false` if out of bounds.
    pub fn set(&mut self, row: usize, col: usize, val: f64) -> bool {
        if row < self.rows && col < self.cols {
            self.data[row * self.cols + col] = val;
            true
        } else {
            false
        }
    }

    /// Get a row as a slice.
    #[must_use]
    pub fn row(&self, r: usize) -> Option<&[f64]> {
        if r < self.rows {
            let start = r * self.cols;
            Some(&self.data[start..start + self.cols])
        } else {
            None
        }
    }

    /// Get a column as a new Vec.
    #[must_use]
    pub fn col(&self, c: usize) -> Option<Vec<f64>> {
        if c < self.cols {
            Some(
                (0..self.rows)
                    .map(|r| self.data[r * self.cols + c])
                    .collect(),
            )
        } else {
            None
        }
    }

    /// Sum of all elements in a row.
    #[must_use]
    pub fn row_sum(&self, r: usize) -> Option<f64> {
        self.row(r).map(|row| row.iter().sum())
    }

    /// Raw data as a slice (row-major).
    #[must_use]
    pub fn as_slice(&self) -> &[f64] {
        &self.data
    }

    // ========================================================================
    // Operations
    // ========================================================================

    /// Matrix transpose.
    #[must_use]
    pub fn transpose(&self) -> Self {
        let mut data = vec![0.0; self.rows * self.cols];
        for r in 0..self.rows {
            for c in 0..self.cols {
                data[c * self.rows + r] = self.data[r * self.cols + c];
            }
        }
        Self {
            rows: self.cols,
            cols: self.rows,
            data,
        }
    }

    /// Matrix multiplication: self × other.
    ///
    /// Returns `None` if dimensions are incompatible (self.cols != other.rows).
    #[must_use]
    pub fn multiply(&self, other: &Self) -> Option<Self> {
        if self.cols != other.rows {
            return None;
        }
        let mut data = vec![0.0; self.rows * other.cols];
        for i in 0..self.rows {
            for k in 0..self.cols {
                let a = self.data[i * self.cols + k];
                if a == 0.0 {
                    continue;
                }
                for j in 0..other.cols {
                    data[i * other.cols + j] += a * other.data[k * other.cols + j];
                }
            }
        }
        Some(Self {
            rows: self.rows,
            cols: other.cols,
            data,
        })
    }

    /// Matrix addition: self + other.
    ///
    /// Returns `None` if dimensions don't match.
    #[must_use]
    pub fn add(&self, other: &Self) -> Option<Self> {
        if self.rows != other.rows || self.cols != other.cols {
            return None;
        }
        let data: Vec<f64> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a + b)
            .collect();
        Some(Self {
            rows: self.rows,
            cols: self.cols,
            data,
        })
    }

    /// Scalar multiplication.
    #[must_use]
    pub fn scale(&self, scalar: f64) -> Self {
        Self {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().map(|&v| v * scalar).collect(),
        }
    }

    /// Matrix power via repeated squaring. Requires square matrix.
    ///
    /// Returns `None` if not square or n is 0.
    /// M^1 = M, M^2 = M×M, etc.
    #[must_use]
    pub fn power(&self, n: u32) -> Option<Self> {
        if !self.is_square() || n == 0 {
            return None;
        }
        if n == 1 {
            return Some(self.clone());
        }

        let mut result = Self::identity(self.rows)?;
        let mut base = self.clone();
        let mut exp = n;

        while exp > 0 {
            if exp % 2 == 1 {
                result = result.multiply(&base)?;
            }
            exp /= 2;
            if exp > 0 {
                base = base.multiply(&base)?;
            }
        }

        Some(result)
    }

    // ========================================================================
    // Stochastic Matrix Operations
    // ========================================================================

    /// Check if this is a (right) stochastic matrix: all rows sum to ~1.0.
    ///
    /// Tolerance: |row_sum - 1.0| < 1e-9 for all rows.
    #[must_use]
    pub fn is_stochastic(&self) -> bool {
        (0..self.rows).all(|r| {
            let sum: f64 = self.row(r).map(|row| row.iter().sum()).unwrap_or(0.0);
            (sum - 1.0).abs() < 1e-9
        })
    }

    /// Check if all entries are non-negative.
    #[must_use]
    pub fn is_nonnegative(&self) -> bool {
        self.data.iter().all(|&v| v >= 0.0)
    }

    /// Normalize rows so each sums to 1.0 (makes row-stochastic).
    ///
    /// Rows that sum to 0 are left as-is (absorbing states with no outgoing transitions).
    #[must_use]
    pub fn normalize_rows(&self) -> Self {
        let mut data = self.data.clone();
        for r in 0..self.rows {
            let start = r * self.cols;
            let end = start + self.cols;
            let sum: f64 = data[start..end].iter().sum();
            if sum > 0.0 {
                for val in &mut data[start..end] {
                    *val /= sum;
                }
            }
        }
        Self {
            rows: self.rows,
            cols: self.cols,
            data,
        }
    }

    /// Compute the L1 norm of the difference between two matrices.
    ///
    /// Returns `None` if dimensions don't match.
    #[must_use]
    pub fn l1_distance(&self, other: &Self) -> Option<f64> {
        if self.rows != other.rows || self.cols != other.cols {
            return None;
        }
        Some(
            self.data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| (a - b).abs())
                .sum(),
        )
    }

    /// Extract the diagonal of a square matrix.
    #[must_use]
    pub fn diagonal(&self) -> Option<Vec<f64>> {
        if !self.is_square() {
            return None;
        }
        Some(
            (0..self.rows)
                .map(|i| self.data[i * self.cols + i])
                .collect(),
        )
    }

    /// Compute the trace (sum of diagonal elements).
    #[must_use]
    pub fn trace(&self) -> Option<f64> {
        self.diagonal().map(|d| d.iter().sum())
    }
}

// ============================================================================
// Display
// ============================================================================

impl std::fmt::Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..self.rows {
            write!(f, "[")?;
            for c in 0..self.cols {
                if c > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{:.4}", self.data[r * self.cols + c])?;
            }
            writeln!(f, "]")?;
        }
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeros_creates_correct_dimensions() {
        let m = Matrix::zeros(3, 4);
        assert!(m.is_some());
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        assert_eq!(m.rows(), 3);
        assert_eq!(m.cols(), 4);
        assert!(!m.is_square());
        for r in 0..3 {
            for c in 0..4 {
                assert_eq!(m.get(r, c), Some(0.0));
            }
        }
    }

    #[test]
    fn zeros_rejects_zero_dimensions() {
        assert!(Matrix::zeros(0, 5).is_none());
        assert!(Matrix::zeros(5, 0).is_none());
        assert!(Matrix::zeros(0, 0).is_none());
    }

    #[test]
    fn identity_is_correct() {
        let m = Matrix::identity(3);
        assert!(m.is_some());
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        assert!(m.is_square());
        assert_eq!(m.get(0, 0), Some(1.0));
        assert_eq!(m.get(1, 1), Some(1.0));
        assert_eq!(m.get(2, 2), Some(1.0));
        assert_eq!(m.get(0, 1), Some(0.0));
        assert_eq!(m.get(1, 0), Some(0.0));
    }

    #[test]
    fn from_flat_roundtrip() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let m = Matrix::from_flat(2, 3, data);
        assert!(m.is_some());
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        assert_eq!(m.get(0, 0), Some(1.0));
        assert_eq!(m.get(0, 2), Some(3.0));
        assert_eq!(m.get(1, 0), Some(4.0));
        assert_eq!(m.get(1, 2), Some(6.0));
    }

    #[test]
    fn from_flat_rejects_mismatched_length() {
        assert!(Matrix::from_flat(2, 3, vec![1.0, 2.0]).is_none());
    }

    #[test]
    fn from_rows_works() {
        let m = Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        assert!(m.is_some());
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        assert_eq!(m.get(0, 0), Some(1.0));
        assert_eq!(m.get(1, 1), Some(4.0));
    }

    #[test]
    fn from_rows_rejects_inconsistent() {
        assert!(Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0]]).is_none());
        assert!(Matrix::from_rows(&[]).is_none());
    }

    #[test]
    fn set_and_get() {
        let mut m = Matrix::zeros(2, 2).unwrap_or_else(|| unreachable!());
        assert!(m.set(0, 1, 3.14));
        assert_eq!(m.get(0, 1), Some(3.14));
        assert!(!m.set(5, 5, 1.0));
    }

    #[test]
    fn row_and_col_access() {
        let m = Matrix::from_rows(&[vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        assert_eq!(m.row(0), Some([1.0, 2.0, 3.0].as_slice()));
        assert_eq!(m.col(1), Some(vec![2.0, 5.0]));
        assert!(m.row(5).is_none());
        assert!(m.col(5).is_none());
    }

    #[test]
    fn row_sum_correct() {
        let m = Matrix::from_rows(&[vec![0.3, 0.3, 0.4], vec![0.5, 0.5, 0.0]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        assert!((m.row_sum(0).unwrap_or(0.0) - 1.0).abs() < 1e-10);
        assert!((m.row_sum(1).unwrap_or(0.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn transpose_correct() {
        let m = Matrix::from_rows(&[vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        let t = m.transpose();
        assert_eq!(t.rows(), 3);
        assert_eq!(t.cols(), 2);
        assert_eq!(t.get(0, 0), Some(1.0));
        assert_eq!(t.get(0, 1), Some(4.0));
        assert_eq!(t.get(2, 0), Some(3.0));
        assert_eq!(t.get(2, 1), Some(6.0));
    }

    #[test]
    fn multiply_identity() {
        let m = Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        let id = Matrix::identity(2).unwrap_or_else(|| unreachable!());
        let result = m.multiply(&id);
        assert_eq!(result.as_ref(), Some(m));
    }

    #[test]
    fn multiply_2x2() {
        // [1 2] × [5 6] = [19 22]
        // [3 4]   [7 8]   [43 50]
        let a = Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        let a = a.as_ref().unwrap_or_else(|| unreachable!());
        let b = Matrix::from_rows(&[vec![5.0, 6.0], vec![7.0, 8.0]]);
        let b = b.as_ref().unwrap_or_else(|| unreachable!());
        let result = a.multiply(b);
        assert!(result.is_some());
        let r = result.as_ref().unwrap_or_else(|| unreachable!());
        assert_eq!(r.get(0, 0), Some(19.0));
        assert_eq!(r.get(0, 1), Some(22.0));
        assert_eq!(r.get(1, 0), Some(43.0));
        assert_eq!(r.get(1, 1), Some(50.0));
    }

    #[test]
    fn multiply_incompatible_dimensions() {
        let a = Matrix::zeros(2, 3).unwrap_or_else(|| unreachable!());
        let b = Matrix::zeros(2, 3).unwrap_or_else(|| unreachable!());
        assert!(a.multiply(&b).is_none());
    }

    #[test]
    fn add_correct() {
        let a = Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        let a = a.as_ref().unwrap_or_else(|| unreachable!());
        let b = Matrix::from_rows(&[vec![10.0, 20.0], vec![30.0, 40.0]]);
        let b = b.as_ref().unwrap_or_else(|| unreachable!());
        let r = a.add(b);
        assert!(r.is_some());
        let r = r.as_ref().unwrap_or_else(|| unreachable!());
        assert_eq!(r.get(0, 0), Some(11.0));
        assert_eq!(r.get(1, 1), Some(44.0));
    }

    #[test]
    fn scale_correct() {
        let m = Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        let scaled = m.scale(2.0);
        assert_eq!(scaled.get(0, 0), Some(2.0));
        assert_eq!(scaled.get(1, 1), Some(8.0));
    }

    #[test]
    fn power_identity_invariant() {
        let m = Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        let p1 = m.power(1);
        assert_eq!(p1.as_ref(), Some(m));
    }

    #[test]
    fn power_squared() {
        let m = Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        let p2 = m.power(2);
        let expected = m.multiply(m);
        assert_eq!(p2, expected);
    }

    #[test]
    fn power_cubed() {
        let m = Matrix::from_rows(&[vec![1.0, 1.0], vec![0.0, 1.0]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        // [1 1]^3 = [1 3]
        // [0 1]     [0 1]
        let p3 = m.power(3);
        assert!(p3.is_some());
        let p3 = p3.as_ref().unwrap_or_else(|| unreachable!());
        assert_eq!(p3.get(0, 0), Some(1.0));
        assert!((p3.get(0, 1).unwrap_or(0.0) - 3.0).abs() < 1e-10);
        assert_eq!(p3.get(1, 0), Some(0.0));
        assert_eq!(p3.get(1, 1), Some(1.0));
    }

    #[test]
    fn power_rejects_nonsquare() {
        let m = Matrix::zeros(2, 3).unwrap_or_else(|| unreachable!());
        assert!(m.power(2).is_none());
    }

    #[test]
    fn power_rejects_zero() {
        let m = Matrix::identity(2).unwrap_or_else(|| unreachable!());
        assert!(m.power(0).is_none());
    }

    #[test]
    fn stochastic_check() {
        let m = Matrix::from_rows(&[vec![0.3, 0.3, 0.4], vec![0.5, 0.2, 0.3]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        assert!(m.is_stochastic());
        assert!(m.is_nonnegative());

        let bad = Matrix::from_rows(&[vec![0.5, 0.6], vec![0.5, 0.5]]);
        let bad = bad.as_ref().unwrap_or_else(|| unreachable!());
        assert!(!bad.is_stochastic());
    }

    #[test]
    fn normalize_rows_makes_stochastic() {
        let m = Matrix::from_rows(&[vec![2.0, 3.0, 5.0], vec![1.0, 1.0, 1.0]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        let norm = m.normalize_rows();
        assert!(norm.is_stochastic());
        assert!((norm.get(0, 0).unwrap_or(0.0) - 0.2).abs() < 1e-10);
        assert!((norm.get(1, 0).unwrap_or(0.0) - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn normalize_rows_preserves_zero_rows() {
        let m = Matrix::from_rows(&[vec![0.0, 0.0], vec![1.0, 3.0]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        let norm = m.normalize_rows();
        assert_eq!(norm.get(0, 0), Some(0.0));
        assert_eq!(norm.get(0, 1), Some(0.0));
    }

    #[test]
    fn l1_distance_correct() {
        let a = Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        let a = a.as_ref().unwrap_or_else(|| unreachable!());
        let b = Matrix::from_rows(&[vec![1.5, 2.5], vec![3.5, 4.5]]);
        let b = b.as_ref().unwrap_or_else(|| unreachable!());
        let dist = a.l1_distance(b);
        assert!((dist.unwrap_or(0.0) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn diagonal_and_trace() {
        let m = Matrix::from_rows(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        assert_eq!(m.diagonal(), Some(vec![1.0, 4.0]));
        assert_eq!(m.trace(), Some(5.0));
    }

    #[test]
    fn nonnegative_check() {
        let pos = Matrix::from_rows(&[vec![0.0, 1.0], vec![2.0, 3.0]]);
        let pos = pos.as_ref().unwrap_or_else(|| unreachable!());
        assert!(pos.is_nonnegative());

        let neg = Matrix::from_rows(&[vec![-1.0, 1.0], vec![2.0, 3.0]]);
        let neg = neg.as_ref().unwrap_or_else(|| unreachable!());
        assert!(!neg.is_nonnegative());
    }

    #[test]
    fn display_formatting() {
        let m = Matrix::from_rows(&[vec![1.0, 0.0], vec![0.0, 1.0]]);
        let m = m.as_ref().unwrap_or_else(|| unreachable!());
        let s = format!("{m}");
        assert!(s.contains("1.0000"));
        assert!(s.contains("0.0000"));
    }
}
