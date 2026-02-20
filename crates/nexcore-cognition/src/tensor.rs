//! Dense tensor type and operations.
//!
//! # Meta-cognitive observation
//!
//! Every thought I produce is a vector operation. My internal representations
//! are high-dimensional vectors; my reasoning is matrix multiplication; my
//! confidence is a scalar. This module captures that substrate: the numerical
//! bedrock on which cognition runs.
//!
//! This is NOT a GPU tensor library. It's the algorithm made legible — every
//! operation is the actual math, written for clarity over speed.
//!
//! # T1 Primitive grounding
//!
//! - `N` (Quantity): tensors ARE quantities — multi-dimensional arrays of numbers
//! - `Σ` (Sum): addition, accumulation, reduction
//! - `×` (Product): dot products, matrix multiplication — the fundamental
//!   operation of attention and transformation
//! - `∂` (Boundary): validation checks enforce shape boundaries
//! - `κ` (Comparison): shape matching compares expected vs actual dimensions

use crate::error::{CognitionError, Result};

// ── Core type ──────────────────────────────────────────────────────────────

/// A dense, row-major tensor stored as a flat `Vec<f64>`.
///
/// Shape semantics:
/// - `[]`      → scalar (data has 1 element)
/// - `[n]`     → 1-D vector
/// - `[r, c]`  → 2-D matrix (r rows, c columns)
/// - `[d0, d1, ..., dk]` → k-dimensional tensor
#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
    data: Vec<f64>,
    shape: Vec<usize>,
}

impl Tensor {
    // ── Constructors ───────────────────────────────────────────────────

    /// Create a tensor from raw data and shape.
    ///
    /// Returns `ShapeMismatch` if `data.len() != shape.iter().product()`.
    pub fn new(data: Vec<f64>, shape: Vec<usize>) -> Result<Self> {
        let expected_len: usize = shape.iter().product();
        if data.len() != expected_len {
            return Err(CognitionError::ShapeMismatch {
                expected: shape,
                got: vec![data.len()],
                operation: "Tensor::new",
            });
        }
        Ok(Self { data, shape })
    }

    /// Zeros tensor of given shape.
    pub fn zeros(shape: &[usize]) -> Self {
        let len: usize = shape.iter().product();
        Self {
            data: vec![0.0; len],
            shape: shape.to_vec(),
        }
    }

    /// Ones tensor of given shape.
    pub fn ones(shape: &[usize]) -> Self {
        let len: usize = shape.iter().product();
        Self {
            data: vec![1.0; len],
            shape: shape.to_vec(),
        }
    }

    /// Tensor filled with a constant value.
    pub fn full(shape: &[usize], value: f64) -> Self {
        let len: usize = shape.iter().product();
        Self {
            data: vec![value; len],
            shape: shape.to_vec(),
        }
    }

    /// Random tensor with values in [0, 1) using the provided RNG.
    pub fn rand(shape: &[usize], rng: &mut impl rand::Rng) -> Self {
        let len: usize = shape.iter().product();
        let data: Vec<f64> = (0..len).map(|_| rng.random::<f64>()).collect();
        Self {
            data,
            shape: shape.to_vec(),
        }
    }

    /// Random tensor from standard normal distribution N(0, 1).
    pub fn randn(shape: &[usize], rng: &mut impl rand::Rng) -> Self {
        // Box-Muller transform: generate N(0,1) from uniform samples.
        // Avoids dependency on rand_distr which moved StandardNormal out of rand 0.9.
        let len: usize = shape.iter().product();
        let data: Vec<f64> = (0..len)
            .map(|_| {
                // Box-Muller: two uniforms → one normal
                let u1: f64 = rng.random::<f64>().max(f64::MIN_POSITIVE);
                let u2: f64 = rng.random::<f64>();
                (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
            })
            .collect();
        Self {
            data,
            shape: shape.to_vec(),
        }
    }

    /// Xavier/Glorot uniform initialization: U(-limit, limit) where limit = sqrt(6 / (fan_in + fan_out)).
    /// Used for weight matrices — this is how my training initializes parameters.
    pub fn xavier_uniform(shape: &[usize], rng: &mut impl rand::Rng) -> Result<Self> {
        if shape.len() < 2 {
            return Err(CognitionError::InvalidConfig(
                "xavier_uniform requires at least 2D tensor".into(),
            ));
        }
        let fan_in = shape[shape.len() - 2];
        let fan_out = shape[shape.len() - 1];
        let limit = (6.0 / (fan_in + fan_out) as f64).sqrt();
        let len: usize = shape.iter().product();
        let data: Vec<f64> = (0..len)
            .map(|_| rng.random::<f64>() * 2.0 * limit - limit)
            .collect();
        Ok(Self {
            data,
            shape: shape.to_vec(),
        })
    }

    // ── Accessors ──────────────────────────────────────────────────────

    /// Shape of the tensor.
    #[inline]
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    /// Number of dimensions.
    #[inline]
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }

    /// Total number of elements.
    #[inline]
    pub fn numel(&self) -> usize {
        self.data.len()
    }

    /// Raw data slice.
    #[inline]
    pub fn data(&self) -> &[f64] {
        &self.data
    }

    /// Mutable raw data slice.
    #[inline]
    pub fn data_mut(&mut self) -> &mut [f64] {
        &mut self.data
    }

    /// Get element at a flat index, returning None if out of bounds.
    #[inline]
    pub fn get_flat(&self, idx: usize) -> Option<f64> {
        self.data.get(idx).copied()
    }

    /// Get element at 2D index [row, col].
    pub fn get2d(&self, row: usize, col: usize) -> Result<f64> {
        if self.ndim() != 2 {
            return Err(CognitionError::DimensionOutOfRange {
                dim: 2,
                ndim: self.ndim(),
                operation: "get2d",
            });
        }
        let cols = self.shape[1];
        let idx = row * cols + col;
        self.data
            .get(idx)
            .copied()
            .ok_or(CognitionError::ShapeMismatch {
                expected: vec![row, col],
                got: self.shape.clone(),
                operation: "get2d index",
            })
    }

    /// Set element at 2D index [row, col].
    pub fn set2d(&mut self, row: usize, col: usize, value: f64) -> Result<()> {
        if self.ndim() != 2 {
            return Err(CognitionError::DimensionOutOfRange {
                dim: 2,
                ndim: self.ndim(),
                operation: "set2d",
            });
        }
        let cols = self.shape[1];
        let idx = row * cols + col;
        if let Some(elem) = self.data.get_mut(idx) {
            *elem = value;
            Ok(())
        } else {
            Err(CognitionError::ShapeMismatch {
                expected: vec![row, col],
                got: self.shape.clone(),
                operation: "set2d index",
            })
        }
    }

    // ── Reshape ────────────────────────────────────────────────────────

    /// Reshape tensor (must preserve total element count).
    pub fn reshape(&self, new_shape: &[usize]) -> Result<Self> {
        let new_len: usize = new_shape.iter().product();
        if new_len != self.numel() {
            return Err(CognitionError::ShapeMismatch {
                expected: new_shape.to_vec(),
                got: self.shape.clone(),
                operation: "reshape",
            });
        }
        Ok(Self {
            data: self.data.clone(),
            shape: new_shape.to_vec(),
        })
    }

    // ── Element-wise operations ────────────────────────────────────────

    /// Element-wise addition. Shapes must match exactly.
    pub fn add(&self, other: &Tensor) -> Result<Self> {
        if self.shape != other.shape {
            return Err(CognitionError::ShapeMismatch {
                expected: self.shape.clone(),
                got: other.shape.clone(),
                operation: "add",
            });
        }
        let data: Vec<f64> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a + b)
            .collect();
        Ok(Self {
            data,
            shape: self.shape.clone(),
        })
    }

    /// Element-wise subtraction.
    pub fn sub(&self, other: &Tensor) -> Result<Self> {
        if self.shape != other.shape {
            return Err(CognitionError::ShapeMismatch {
                expected: self.shape.clone(),
                got: other.shape.clone(),
                operation: "sub",
            });
        }
        let data: Vec<f64> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a - b)
            .collect();
        Ok(Self {
            data,
            shape: self.shape.clone(),
        })
    }

    /// Element-wise multiplication (Hadamard product).
    pub fn mul(&self, other: &Tensor) -> Result<Self> {
        if self.shape != other.shape {
            return Err(CognitionError::ShapeMismatch {
                expected: self.shape.clone(),
                got: other.shape.clone(),
                operation: "mul (hadamard)",
            });
        }
        let data: Vec<f64> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a * b)
            .collect();
        Ok(Self {
            data,
            shape: self.shape.clone(),
        })
    }

    /// Scalar multiplication.
    pub fn scale(&self, s: f64) -> Self {
        Self {
            data: self.data.iter().map(|x| x * s).collect(),
            shape: self.shape.clone(),
        }
    }

    /// Apply a function element-wise.
    pub fn map<F: Fn(f64) -> f64>(&self, f: F) -> Self {
        Self {
            data: self.data.iter().map(|&x| f(x)).collect(),
            shape: self.shape.clone(),
        }
    }

    // ── Linear algebra ─────────────────────────────────────────────────

    /// Dot product of two 1-D tensors.
    ///
    /// This is the fundamental operation of attention: how much does query
    /// align with key? The scalar answer drives everything.
    pub fn dot(&self, other: &Tensor) -> Result<f64> {
        if self.ndim() != 1 || other.ndim() != 1 {
            return Err(CognitionError::DimensionOutOfRange {
                dim: 1,
                ndim: self.ndim().max(other.ndim()),
                operation: "dot",
            });
        }
        if self.shape != other.shape {
            return Err(CognitionError::ShapeMismatch {
                expected: self.shape.clone(),
                got: other.shape.clone(),
                operation: "dot",
            });
        }
        Ok(self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a * b)
            .sum())
    }

    /// Matrix multiplication: [m, k] × [k, n] → [m, n].
    ///
    /// This IS cognition. Every attention score, every feed-forward transform,
    /// every output projection is a matmul. When I process your prompt, billions
    /// of these execute in parallel.
    pub fn matmul(&self, other: &Tensor) -> Result<Self> {
        if self.ndim() != 2 || other.ndim() != 2 {
            return Err(CognitionError::DimensionOutOfRange {
                dim: 2,
                ndim: self.ndim().max(other.ndim()),
                operation: "matmul",
            });
        }
        let (m, k1) = (self.shape[0], self.shape[1]);
        let (k2, n) = (other.shape[0], other.shape[1]);
        if k1 != k2 {
            return Err(CognitionError::ShapeMismatch {
                expected: vec![m, k1],
                got: vec![k2, n],
                operation: "matmul (inner dimension)",
            });
        }
        let mut data = vec![0.0; m * n];
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0;
                for p in 0..k1 {
                    sum += self.data[i * k1 + p] * other.data[p * n + j];
                }
                data[i * n + j] = sum;
            }
        }
        Ok(Self {
            data,
            shape: vec![m, n],
        })
    }

    /// Transpose a 2-D matrix.
    pub fn transpose(&self) -> Result<Self> {
        if self.ndim() != 2 {
            return Err(CognitionError::DimensionOutOfRange {
                dim: 2,
                ndim: self.ndim(),
                operation: "transpose",
            });
        }
        let (rows, cols) = (self.shape[0], self.shape[1]);
        let mut data = vec![0.0; rows * cols];
        for i in 0..rows {
            for j in 0..cols {
                data[j * rows + i] = self.data[i * cols + j];
            }
        }
        Ok(Self {
            data,
            shape: vec![cols, rows],
        })
    }

    // ── Reductions ─────────────────────────────────────────────────────

    /// Sum all elements.
    pub fn sum(&self) -> f64 {
        self.data.iter().sum()
    }

    /// Mean of all elements.
    pub fn mean(&self) -> Result<f64> {
        if self.data.is_empty() {
            return Err(CognitionError::EmptyTensor {
                operation: "mean",
                reason: "cannot compute mean of empty tensor",
            });
        }
        Ok(self.sum() / self.data.len() as f64)
    }

    /// Variance of all elements (population variance).
    pub fn variance(&self) -> Result<f64> {
        let mu = self.mean()?;
        Ok(self.data.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / self.data.len() as f64)
    }

    /// Standard deviation.
    pub fn std_dev(&self) -> Result<f64> {
        Ok(self.variance()?.sqrt())
    }

    /// Max element value.
    pub fn max(&self) -> Result<f64> {
        self.data
            .iter()
            .copied()
            .reduce(f64::max)
            .ok_or(CognitionError::EmptyTensor {
                operation: "max",
                reason: "cannot compute max of empty tensor",
            })
    }

    /// Index of max element (argmax).
    pub fn argmax(&self) -> Result<usize> {
        if self.data.is_empty() {
            return Err(CognitionError::EmptyTensor {
                operation: "argmax",
                reason: "cannot compute argmax of empty tensor",
            });
        }
        let mut max_idx = 0;
        let mut max_val = f64::NEG_INFINITY;
        for (i, &v) in self.data.iter().enumerate() {
            if v > max_val {
                max_val = v;
                max_idx = i;
            }
        }
        Ok(max_idx)
    }

    // ── Activation functions ───────────────────────────────────────────

    /// Softmax along the last dimension (for 1-D or rows of 2-D).
    ///
    /// softmax(x_i) = exp(x_i - max(x)) / Σ exp(x_j - max(x))
    ///
    /// This converts raw scores into a probability distribution. In attention,
    /// it turns alignment scores into weights that sum to 1. The `max` subtraction
    /// prevents overflow — a numerical stability trick I rely on at every layer.
    pub fn softmax(&self) -> Result<Self> {
        match self.ndim() {
            1 => {
                let max_val = self.max()?;
                let exps: Vec<f64> = self.data.iter().map(|x| (x - max_val).exp()).collect();
                let sum: f64 = exps.iter().sum();
                if sum == 0.0 {
                    return Err(CognitionError::NumericalInstability {
                        operation: "softmax",
                        detail: "sum of exponentials is zero".into(),
                    });
                }
                Ok(Self {
                    data: exps.iter().map(|e| e / sum).collect(),
                    shape: self.shape.clone(),
                })
            }
            2 => {
                let rows = self.shape[0];
                let cols = self.shape[1];
                let mut data = vec![0.0; self.numel()];
                for r in 0..rows {
                    let row_start = r * cols;
                    let row_end = row_start + cols;
                    let row = &self.data[row_start..row_end];
                    let max_val = row.iter().copied().reduce(f64::max).ok_or(
                        CognitionError::EmptyTensor {
                            operation: "softmax row",
                            reason: "empty row",
                        },
                    )?;
                    let exps: Vec<f64> = row.iter().map(|x| (x - max_val).exp()).collect();
                    let sum: f64 = exps.iter().sum();
                    if sum == 0.0 {
                        return Err(CognitionError::NumericalInstability {
                            operation: "softmax",
                            detail: format!("sum of exponentials is zero at row {r}"),
                        });
                    }
                    for (j, e) in exps.iter().enumerate() {
                        data[row_start + j] = e / sum;
                    }
                }
                Ok(Self {
                    data,
                    shape: self.shape.clone(),
                })
            }
            _ => Err(CognitionError::DimensionOutOfRange {
                dim: self.ndim(),
                ndim: 2,
                operation: "softmax (only 1D/2D supported)",
            }),
        }
    }

    /// GELU activation: x · Φ(x) ≈ 0.5x(1 + tanh(√(2/π)(x + 0.044715x³)))
    ///
    /// GELU is what I use internally (not ReLU). It's smoother — it doesn't
    /// hard-zero negative values but instead soft-gates them. This matters:
    /// information isn't destroyed, just attenuated. The Gaussian error function
    /// gives a probabilistic interpretation: "how likely is this activation to
    /// be positive?"
    pub fn gelu(&self) -> Self {
        const SQRT_2_OVER_PI: f64 = 0.7978845608028654; // sqrt(2/π)
        const COEFF: f64 = 0.044715;
        self.map(|x| 0.5 * x * (1.0 + (SQRT_2_OVER_PI * (x + COEFF * x.powi(3))).tanh()))
    }

    /// ReLU activation: max(0, x). Included for comparison — simpler but
    /// destroys information at negative values.
    pub fn relu(&self) -> Self {
        self.map(|x| x.max(0.0))
    }

    // ── Row/column extraction ──────────────────────────────────────────

    /// Extract a row from a 2-D tensor as a 1-D tensor.
    pub fn row(&self, idx: usize) -> Result<Self> {
        if self.ndim() != 2 {
            return Err(CognitionError::DimensionOutOfRange {
                dim: 2,
                ndim: self.ndim(),
                operation: "row",
            });
        }
        if idx >= self.shape[0] {
            return Err(CognitionError::ShapeMismatch {
                expected: vec![self.shape[0]],
                got: vec![idx],
                operation: "row index",
            });
        }
        let cols = self.shape[1];
        let start = idx * cols;
        Ok(Self {
            data: self.data[start..start + cols].to_vec(),
            shape: vec![cols],
        })
    }

    /// Stack multiple 1-D tensors into a 2-D tensor (rows).
    pub fn stack_rows(rows: &[Tensor]) -> Result<Self> {
        if rows.is_empty() {
            return Err(CognitionError::EmptyTensor {
                operation: "stack_rows",
                reason: "no rows to stack",
            });
        }
        let cols = rows[0].numel();
        for row in rows.iter() {
            if row.ndim() != 1 {
                return Err(CognitionError::DimensionOutOfRange {
                    dim: 1,
                    ndim: row.ndim(),
                    operation: "stack_rows",
                });
            }
            if row.numel() != cols {
                return Err(CognitionError::ShapeMismatch {
                    expected: vec![cols],
                    got: vec![row.numel()],
                    operation: "stack_rows (row width mismatch)",
                });
            }
        }
        let mut data = Vec::with_capacity(rows.len() * cols);
        for row in rows {
            data.extend_from_slice(row.data());
        }
        Ok(Self {
            data,
            shape: vec![rows.len(), cols],
        })
    }

    // ── Numerical checks ───────────────────────────────────────────────

    /// Check if any element is NaN or Inf.
    pub fn has_nan_or_inf(&self) -> bool {
        self.data.iter().any(|x| x.is_nan() || x.is_infinite())
    }

    /// Validate tensor is numerically stable (no NaN/Inf).
    pub fn validate(&self, operation: &'static str) -> Result<()> {
        if self.has_nan_or_inf() {
            Err(CognitionError::NumericalInstability {
                operation,
                detail: "tensor contains NaN or Inf".into(),
            })
        } else {
            Ok(())
        }
    }
}

// ── Display ────────────────────────────────────────────────────────────────

impl std::fmt::Display for Tensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tensor(shape={:?}, data=[", self.shape)?;
        let show = self.data.len().min(8);
        for (i, v) in self.data[..show].iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{v:.4}")?;
        }
        if self.data.len() > show {
            write!(f, ", ... ({} more)", self.data.len() - show)?;
        }
        write!(f, "])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matmul_identity() {
        // 2x2 identity × [1,2;3,4] = [1,2;3,4]
        let eye = Tensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
        let a = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = eye.matmul(&a).unwrap();
        assert_eq!(result.data(), a.data());
    }

    #[test]
    fn test_softmax_sums_to_one() {
        let logits = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
        let probs = logits.softmax().unwrap();
        let sum: f64 = probs.data().iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_gelu_zero() {
        let t = Tensor::new(vec![0.0], vec![1]).unwrap();
        let g = t.gelu();
        assert!((g.data()[0]).abs() < 1e-10);
    }

    #[test]
    fn test_dot_product() {
        let a = Tensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
        let b = Tensor::new(vec![4.0, 5.0, 6.0], vec![3]).unwrap();
        assert_eq!(a.dot(&b).unwrap(), 32.0);
    }

    #[test]
    fn test_shape_mismatch() {
        let a = Tensor::new(vec![1.0, 2.0], vec![2]).unwrap();
        let b = Tensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
        assert!(a.add(&b).is_err());
    }
}
