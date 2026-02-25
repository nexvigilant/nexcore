//! # Entropy and Information Loss Quantification
//!
//! **T1 Grounding:** ∝(Irreversibility) dominant, N(Quantity) for measurement,
//! κ(Comparison) for divergence.
//!
//! Entropy quantifies the irreducible uncertainty in a probability distribution.
//! It is rooted in ∝(Irreversibility) because information lost through coarsening
//! or aggregation cannot be recovered — the direction of entropy increase is one-way.
//! N(Quantity) grounds the raw count of uncertainty in bits. κ(Comparison) grounds
//! divergence measures that compare two distributions.
//!
//! ## Mathematical Foundation
//!
//! Shannon entropy:
//!
//! ```text
//! H(X) = -Σ p(x) · log₂(p(x))
//! ```
//!
//! KL Divergence (relative entropy):
//!
//! ```text
//! D_KL(P ∥ Q) = Σ p(x) · log₂(p(x) / q(x))
//! ```
//!
//! Mutual Information:
//!
//! ```text
//! I(X; Y) = H(X) + H(Y) - H(X, Y)
//! ```
//!
//! ## PV Transfer
//!
//! Signal entropy indicates reporting pattern stability across pharmacovigilance
//! datasets. Low entropy corresponds to highly predictable, concentrated reporting
//! (few drug–event pairs dominate). High entropy reflects chaotic, uniformly spread
//! reporting where no single signal dominates — a marker of noisy surveillance data.
//! Information loss between two temporal snapshots quantifies how much reporting
//! diversity has been compressed or gained by aggregation.
//!
//! ## Examples
//!
//! ```rust
//! use nexcore_primitives::entropy::{shannon_entropy, entropy_from_counts, kl_divergence};
//!
//! // Uniform distribution over 4 outcomes: H = 2.0 bits
//! let uniform = vec![0.25, 0.25, 0.25, 0.25];
//! let result = shannon_entropy(&uniform).unwrap();
//! assert!((result.bits - 2.0).abs() < 1e-10);
//! assert!((result.normalized - 1.0).abs() < 1e-10);
//!
//! // Entropy from raw counts
//! let counts = vec![10u64, 10];
//! let result = entropy_from_counts(&counts).unwrap();
//! assert!((result.bits - 1.0).abs() < 1e-10);
//!
//! // KL divergence: identical distributions → 0
//! let p = vec![0.5, 0.5];
//! let q = vec![0.5, 0.5];
//! let divergence = kl_divergence(&p, &q).unwrap();
//! assert!(divergence.abs() < 1e-10);
//! ```

use nexcore_error::Error;
use serde::{Deserialize, Serialize};

// ============================================================================
// Log Base
// ============================================================================

/// Logarithm base for entropy computation.
///
/// Controls the unit of information measurement. The canonical base is `Bits`
/// (log₂, Shannon's original formulation). Other bases are unit conversions:
/// 1 nat = 1/ln(2) bits ≈ 1.4427 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogBase {
    /// Log base 2 — Shannon entropy in bits (DEFAULT)
    Bits,
    /// Natural log — entropy in nats
    Nats,
    /// Log base 10 — entropy in hartleys/bans
    Hartleys,
}

impl LogBase {
    /// Compute the logarithm of `x` in this base.
    #[must_use]
    pub fn log(self, x: f64) -> f64 {
        match self {
            LogBase::Bits => x.log2(),
            LogBase::Nats => x.ln(),
            LogBase::Hartleys => x.log10(),
        }
    }

    /// Unit name for display purposes.
    #[must_use]
    pub const fn unit_name(self) -> &'static str {
        match self {
            LogBase::Bits => "bits",
            LogBase::Nats => "nats",
            LogBase::Hartleys => "hartleys",
        }
    }

    /// Conversion factor from bits to this base.
    /// Multiply a bits value by this factor to convert.
    #[must_use]
    pub fn from_bits_factor(self) -> f64 {
        match self {
            LogBase::Bits => 1.0,
            LogBase::Nats => std::f64::consts::LN_2,
            LogBase::Hartleys => 2.0_f64.log10(),
        }
    }
}

impl Default for LogBase {
    fn default() -> Self {
        LogBase::Bits
    }
}

// ============================================================================
// Error Type
// ============================================================================

/// Errors arising from entropy and information-theoretic computations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum EntropyError {
    /// The supplied probability distribution contains no elements.
    #[error("probability distribution is empty")]
    EmptyDistribution,

    /// The supplied probabilities do not sum to 1.0 within the required tolerance.
    #[error("probabilities must sum to 1.0 (got {0:.6})")]
    InvalidDistribution(f64),

    /// One or more probability values lie outside the valid range [0, 1].
    #[error("probability must be in [0, 1], got {0}")]
    InvalidProbability(f64),

    /// The two distributions supplied for comparison have different lengths.
    #[error("distributions must have equal length: {0} vs {1}")]
    LengthMismatch(usize, usize),

    /// KL divergence is undefined because Q assigns zero probability where P does not.
    #[error("KL divergence undefined: q[{index}] = 0 where p[{index}] > 0")]
    KlUndefined { index: usize },

    /// The counts array contains no elements or all counts are zero.
    #[error("counts array is empty")]
    EmptyCounts,

    /// The joint probability matrix is either empty or non-rectangular.
    #[error("joint distribution must be non-empty and rectangular")]
    InvalidJoint,
}

// ============================================================================
// Result Types
// ============================================================================

/// The result of a Shannon entropy computation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntropyResult {
    /// Entropy in bits (H).
    pub bits: f64,

    /// Normalized entropy H / log₂(n), always in [0, 1].
    ///
    /// Equals 1.0 for a uniform distribution and 0.0 for a deterministic one.
    /// When n = 1, this is defined as 0.0 (a single outcome carries no uncertainty).
    pub normalized: f64,

    /// Number of outcomes in the distribution (n).
    pub sample_count: usize,
}

/// The result of an information-loss computation between two distributions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InformationLoss {
    /// Entropy of the original distribution in bits.
    pub before_bits: f64,

    /// Entropy of the transformed distribution in bits.
    pub after_bits: f64,

    /// Bits lost: `before_bits - after_bits`.
    ///
    /// Negative when the transformation increases entropy (information was added
    /// or uncertainty grew).
    pub lost_bits: f64,

    /// Fractional loss: `lost_bits / before_bits`.
    ///
    /// Set to 0.0 when `before_bits` is zero (no information to lose).
    pub loss_fraction: f64,
}

// ============================================================================
// Validation Helpers
// ============================================================================

/// Tolerance used when checking that probabilities sum to 1.0.
const SUM_TOLERANCE: f64 = 1e-6;

/// Validate that every element of `probs` lies in [0, 1] and that the slice is
/// non-empty.  Does **not** check the sum — call [`validate_sum`] separately.
fn validate_probabilities(probs: &[f64]) -> Result<(), EntropyError> {
    if probs.is_empty() {
        return Err(EntropyError::EmptyDistribution);
    }
    for &p in probs {
        if !(0.0..=1.0).contains(&p) {
            return Err(EntropyError::InvalidProbability(p));
        }
    }
    Ok(())
}

/// Validate that `probs` sum to approximately 1.0.
fn validate_sum(probs: &[f64]) -> Result<(), EntropyError> {
    let total: f64 = probs.iter().sum();
    if (total - 1.0).abs() > SUM_TOLERANCE {
        return Err(EntropyError::InvalidDistribution(total));
    }
    Ok(())
}

/// Validate an entire probability distribution (non-empty, values in [0,1], sum ≈ 1).
fn validate_distribution(probs: &[f64]) -> Result<(), EntropyError> {
    validate_probabilities(probs)?;
    validate_sum(probs)
}

// ============================================================================
// Core Functions
// ============================================================================

/// Compute the Shannon entropy of a probability distribution.
///
/// H(X) = -Σ p(x) · log₂(p(x))
///
/// By convention, 0 · log₂(0) is taken as 0 (the limit as p → 0⁺ is 0).
///
/// # Errors
///
/// - [`EntropyError::EmptyDistribution`] — `probabilities` is empty.
/// - [`EntropyError::InvalidProbability`] — any element is outside [0, 1].
/// - [`EntropyError::InvalidDistribution`] — the values do not sum to 1.0 within
///   tolerance (1e-6).
///
/// # Examples
///
/// ```rust
/// use nexcore_primitives::entropy::shannon_entropy;
///
/// // Two equally likely outcomes: 1 bit of entropy
/// let result = shannon_entropy(&[0.5, 0.5]).unwrap();
/// assert!((result.bits - 1.0).abs() < 1e-10);
///
/// // Deterministic outcome: 0 bits
/// let result = shannon_entropy(&[1.0]).unwrap();
/// assert_eq!(result.bits, 0.0);
/// ```
pub fn shannon_entropy(probabilities: &[f64]) -> Result<EntropyResult, EntropyError> {
    validate_distribution(probabilities)?;

    let n = probabilities.len();

    let bits = probabilities
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.log2())
        .sum::<f64>();

    // Normalized entropy: H / log₂(n).
    // When n == 1, log₂(1) == 0 — define normalized as 0.0 (no uncertainty possible).
    let normalized = if n <= 1 {
        0.0
    } else {
        let max_entropy = (n as f64).log2();
        if max_entropy == 0.0 {
            0.0
        } else {
            bits / max_entropy
        }
    };

    Ok(EntropyResult {
        bits,
        normalized,
        sample_count: n,
    })
}

/// Compute Shannon entropy directly from raw event counts.
///
/// Internally converts counts to probabilities (`p_i = count_i / total`) and
/// delegates to [`shannon_entropy`].
///
/// # Errors
///
/// - [`EntropyError::EmptyCounts`] — `counts` is empty or all values are zero.
///
/// # Examples
///
/// ```rust
/// use nexcore_primitives::entropy::entropy_from_counts;
///
/// // Equal counts → uniform distribution → maximum entropy for 2 outcomes
/// let result = entropy_from_counts(&[10, 10]).unwrap();
/// assert!((result.bits - 1.0).abs() < 1e-10);
///
/// // All mass on one outcome → zero entropy
/// let result = entropy_from_counts(&[100, 0]).unwrap();
/// assert_eq!(result.bits, 0.0);
/// ```
pub fn entropy_from_counts(counts: &[u64]) -> Result<EntropyResult, EntropyError> {
    if counts.is_empty() {
        return Err(EntropyError::EmptyCounts);
    }

    let total: u64 = counts.iter().sum();
    if total == 0 {
        return Err(EntropyError::EmptyCounts);
    }

    let total_f = total as f64;
    let probabilities: Vec<f64> = counts.iter().map(|&c| c as f64 / total_f).collect();

    // probabilities are guaranteed valid by construction; call inner helper
    // that skips re-validation to avoid floating-point drift errors.
    shannon_entropy_unchecked(&probabilities)
}

/// Internal variant of [`shannon_entropy`] that skips validation.
///
/// Used when the caller has already ensured the distribution is valid (e.g.,
/// when constructing probabilities from integer counts, where rounding may
/// cause the sum to differ from 1.0 by more than `SUM_TOLERANCE`).
fn shannon_entropy_unchecked(probabilities: &[f64]) -> Result<EntropyResult, EntropyError> {
    if probabilities.is_empty() {
        return Err(EntropyError::EmptyDistribution);
    }

    let n = probabilities.len();

    let bits = probabilities
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.log2())
        .sum::<f64>();

    let normalized = if n <= 1 {
        0.0
    } else {
        let max_entropy = (n as f64).log2();
        if max_entropy == 0.0 {
            0.0
        } else {
            bits / max_entropy
        }
    };

    Ok(EntropyResult {
        bits,
        normalized,
        sample_count: n,
    })
}

/// Compute the Kullback-Leibler divergence D_KL(P ∥ Q).
///
/// ```text
/// D_KL(P ∥ Q) = Σ p(x) · log₂(p(x) / q(x))
/// ```
///
/// By convention, terms where p(x) = 0 contribute 0 regardless of q(x).
/// Terms where q(x) = 0 and p(x) > 0 are undefined and produce an error.
///
/// The result is always non-negative (Gibbs' inequality).  It equals 0 if and
/// only if P = Q almost everywhere.
///
/// # Errors
///
/// - [`EntropyError::EmptyDistribution`] — either slice is empty.
/// - [`EntropyError::InvalidProbability`] — any element is outside [0, 1].
/// - [`EntropyError::InvalidDistribution`] — either slice does not sum to 1.0.
/// - [`EntropyError::LengthMismatch`] — P and Q have different lengths.
/// - [`EntropyError::KlUndefined`] — q[i] = 0 where p[i] > 0.
///
/// # Examples
///
/// ```rust
/// use nexcore_primitives::entropy::kl_divergence;
///
/// // Identical distributions → 0
/// let d = kl_divergence(&[0.5, 0.5], &[0.5, 0.5]).unwrap();
/// assert!(d.abs() < 1e-10);
///
/// // KL is asymmetric
/// let p = vec![0.9, 0.1];
/// let q = vec![0.5, 0.5];
/// let dpq = kl_divergence(&p, &q).unwrap();
/// let dqp = kl_divergence(&q, &p).unwrap();
/// assert!((dpq - dqp).abs() > 1e-6);
/// ```
pub fn kl_divergence(p: &[f64], q: &[f64]) -> Result<f64, EntropyError> {
    if p.len() != q.len() {
        return Err(EntropyError::LengthMismatch(p.len(), q.len()));
    }

    validate_distribution(p)?;
    validate_distribution(q)?;

    let mut divergence = 0.0_f64;

    for (index, (&pi, &qi)) in p.iter().zip(q.iter()).enumerate() {
        if pi == 0.0 {
            // 0 · log(anything) = 0 by convention
            continue;
        }
        if qi == 0.0 {
            return Err(EntropyError::KlUndefined { index });
        }
        divergence += pi * (pi / qi).log2();
    }

    Ok(divergence)
}

/// Quantify the information lost when a distribution is transformed.
///
/// Computes Shannon entropy before and after, then reports the difference and
/// the fractional change.
///
/// # Errors
///
/// Returns any error that [`shannon_entropy`] would return for either input.
///
/// # Examples
///
/// ```rust
/// use nexcore_primitives::entropy::information_loss;
///
/// // Same distribution → no loss
/// let dist = vec![0.5, 0.5];
/// let loss = information_loss(&dist, &dist).unwrap();
/// assert!(loss.lost_bits.abs() < 1e-10);
/// assert_eq!(loss.loss_fraction, 0.0);
///
/// // Compressing to deterministic → 1 bit lost
/// let before = vec![0.5, 0.5];
/// let after = vec![1.0, 0.0];
/// let loss = information_loss(&before, &after).unwrap();
/// assert!((loss.lost_bits - 1.0).abs() < 1e-10);
/// ```
pub fn information_loss(before: &[f64], after: &[f64]) -> Result<InformationLoss, EntropyError> {
    let before_result = shannon_entropy(before)?;
    let after_result = shannon_entropy(after)?;

    let before_bits = before_result.bits;
    let after_bits = after_result.bits;
    let lost_bits = before_bits - after_bits;

    let loss_fraction = if before_bits == 0.0 {
        0.0
    } else {
        lost_bits / before_bits
    };

    Ok(InformationLoss {
        before_bits,
        after_bits,
        lost_bits,
        loss_fraction,
    })
}

/// Compute the joint entropy H(X, Y) from a 2-D joint probability matrix.
///
/// ```text
/// H(X, Y) = -Σ_ij p(x_i, y_j) · log₂(p(x_i, y_j))
/// ```
///
/// The input is a rectangular `rows × cols` matrix where each element is a
/// joint probability.  The matrix must be non-empty, rectangular, and sum to
/// approximately 1.0.
///
/// # Errors
///
/// - [`EntropyError::InvalidJoint`] — the matrix is empty or non-rectangular.
/// - [`EntropyError::InvalidProbability`] — any cell is outside [0, 1].
/// - [`EntropyError::InvalidDistribution`] — the cells do not sum to 1.0.
///
/// # Examples
///
/// ```rust
/// use nexcore_primitives::entropy::joint_entropy;
///
/// // 2×2 uniform joint distribution
/// let joint = vec![
///     vec![0.25, 0.25],
///     vec![0.25, 0.25],
/// ];
/// let h = joint_entropy(&joint).unwrap();
/// assert!((h - 2.0).abs() < 1e-10);
/// ```
pub fn joint_entropy(joint: &[Vec<f64>]) -> Result<f64, EntropyError> {
    if joint.is_empty() {
        return Err(EntropyError::InvalidJoint);
    }

    let ncols = joint[0].len();
    if ncols == 0 {
        return Err(EntropyError::InvalidJoint);
    }

    // Verify rectangularity
    if joint.iter().any(|row| row.len() != ncols) {
        return Err(EntropyError::InvalidJoint);
    }

    // Flatten and validate
    let flat: Vec<f64> = joint.iter().flat_map(|row| row.iter().copied()).collect();

    for &p in &flat {
        if !(0.0..=1.0).contains(&p) {
            return Err(EntropyError::InvalidProbability(p));
        }
    }

    let total: f64 = flat.iter().sum();
    if (total - 1.0).abs() > SUM_TOLERANCE {
        return Err(EntropyError::InvalidDistribution(total));
    }

    let h = flat
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.log2())
        .sum::<f64>();

    Ok(h)
}

/// Compute the mutual information I(X; Y) from a joint probability matrix.
///
/// ```text
/// I(X; Y) = H(X) + H(Y) - H(X, Y)
/// ```
///
/// Row marginals give the distribution of X; column marginals give the
/// distribution of Y.
///
/// For independent variables the result is 0 (up to floating-point precision).
/// For perfectly correlated variables I(X; Y) = H(X) = H(Y).
///
/// # Errors
///
/// Returns any error that [`joint_entropy`] would return.
pub fn mutual_information(joint: &[Vec<f64>]) -> Result<f64, EntropyError> {
    // Validate the joint distribution (rectangularity + sum check) via joint_entropy.
    let h_xy = joint_entropy(joint)?;

    let nrows = joint.len();
    let ncols = joint[0].len();

    // Row marginals → distribution of X
    let mut marginal_x: Vec<f64> = vec![0.0; nrows];
    for (i, row) in joint.iter().enumerate() {
        marginal_x[i] = row.iter().sum::<f64>();
    }

    // Column marginals → distribution of Y
    let mut marginal_y: Vec<f64> = vec![0.0; ncols];
    for row in joint {
        for (j, &p) in row.iter().enumerate() {
            marginal_y[j] += p;
        }
    }

    // Compute H(X) and H(Y) without re-validating sums (marginals sum correctly
    // since the joint already validated, but may have floating-point drift).
    let h_x = shannon_entropy_unchecked(&marginal_x)?.bits;
    let h_y = shannon_entropy_unchecked(&marginal_y)?.bits;

    // Clamp to zero to avoid returning tiny negative values from floating-point
    // arithmetic when the true MI is exactly 0.
    let mi = (h_x + h_y - h_xy).max(0.0);

    Ok(mi)
}

// ============================================================================
// Log-Base Parameterized Variants (B.1)
// ============================================================================

/// Shannon entropy with configurable logarithm base.
///
/// Equivalent to [`shannon_entropy`] when `base` is [`LogBase::Bits`].
/// The `normalized` field in the result is always computed using the same base.
pub fn shannon_entropy_with_base(
    probabilities: &[f64],
    base: LogBase,
) -> Result<EntropyResult, EntropyError> {
    validate_distribution(probabilities)?;
    entropy_core(probabilities, base)
}

/// KL divergence with configurable logarithm base.
///
/// Equivalent to [`kl_divergence`] when `base` is [`LogBase::Bits`].
pub fn kl_divergence_with_base(p: &[f64], q: &[f64], base: LogBase) -> Result<f64, EntropyError> {
    if p.len() != q.len() {
        return Err(EntropyError::LengthMismatch(p.len(), q.len()));
    }
    validate_distribution(p)?;
    validate_distribution(q)?;

    let mut divergence = 0.0_f64;
    for (index, (&pi, &qi)) in p.iter().zip(q.iter()).enumerate() {
        if pi == 0.0 {
            continue;
        }
        if qi == 0.0 {
            return Err(EntropyError::KlUndefined { index });
        }
        divergence += pi * base.log(pi / qi);
    }
    Ok(divergence)
}

/// Joint entropy with configurable logarithm base.
///
/// Equivalent to [`joint_entropy`] when `base` is [`LogBase::Bits`].
pub fn joint_entropy_with_base(joint: &[Vec<f64>], base: LogBase) -> Result<f64, EntropyError> {
    if joint.is_empty() {
        return Err(EntropyError::InvalidJoint);
    }
    let ncols = joint[0].len();
    if ncols == 0 {
        return Err(EntropyError::InvalidJoint);
    }
    if joint.iter().any(|row| row.len() != ncols) {
        return Err(EntropyError::InvalidJoint);
    }
    let flat: Vec<f64> = joint.iter().flat_map(|row| row.iter().copied()).collect();
    for &p in &flat {
        if !(0.0..=1.0).contains(&p) {
            return Err(EntropyError::InvalidProbability(p));
        }
    }
    let total: f64 = flat.iter().sum();
    if (total - 1.0).abs() > SUM_TOLERANCE {
        return Err(EntropyError::InvalidDistribution(total));
    }
    let h = flat
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * base.log(p))
        .sum::<f64>();
    Ok(h)
}

/// Mutual information with configurable logarithm base.
///
/// Equivalent to [`mutual_information`] when `base` is [`LogBase::Bits`].
pub fn mutual_information_with_base(
    joint: &[Vec<f64>],
    base: LogBase,
) -> Result<f64, EntropyError> {
    let h_xy = joint_entropy_with_base(joint, base)?;
    let nrows = joint.len();
    let ncols = joint[0].len();

    let mut marginal_x: Vec<f64> = vec![0.0; nrows];
    for (i, row) in joint.iter().enumerate() {
        marginal_x[i] = row.iter().sum::<f64>();
    }
    let mut marginal_y: Vec<f64> = vec![0.0; ncols];
    for row in joint {
        for (j, &p) in row.iter().enumerate() {
            marginal_y[j] += p;
        }
    }
    let h_x = entropy_core_unchecked(&marginal_x, base)?.bits;
    let h_y = entropy_core_unchecked(&marginal_y, base)?.bits;
    let mi = (h_x + h_y - h_xy).max(0.0);
    Ok(mi)
}

/// Entropy from counts with configurable logarithm base.
///
/// Equivalent to [`entropy_from_counts`] when `base` is [`LogBase::Bits`].
pub fn entropy_from_counts_with_base(
    counts: &[u64],
    base: LogBase,
) -> Result<EntropyResult, EntropyError> {
    if counts.is_empty() {
        return Err(EntropyError::EmptyCounts);
    }
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return Err(EntropyError::EmptyCounts);
    }
    let total_f = total as f64;
    let probabilities: Vec<f64> = counts.iter().map(|&c| c as f64 / total_f).collect();
    entropy_core_unchecked(&probabilities, base)
}

/// Internal entropy computation with configurable base.
fn entropy_core(probabilities: &[f64], base: LogBase) -> Result<EntropyResult, EntropyError> {
    if probabilities.is_empty() {
        return Err(EntropyError::EmptyDistribution);
    }
    let n = probabilities.len();
    let h = probabilities
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * base.log(p))
        .sum::<f64>();
    let normalized = if n <= 1 {
        0.0
    } else {
        let max_h = base.log(n as f64);
        if max_h == 0.0 { 0.0 } else { h / max_h }
    };
    Ok(EntropyResult {
        bits: h, // NOTE: field name is "bits" for backward compat, actual unit depends on base
        normalized,
        sample_count: n,
    })
}

/// Internal entropy computation (unchecked) with configurable base.
fn entropy_core_unchecked(
    probabilities: &[f64],
    base: LogBase,
) -> Result<EntropyResult, EntropyError> {
    if probabilities.is_empty() {
        return Err(EntropyError::EmptyDistribution);
    }
    let n = probabilities.len();
    let h = probabilities
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * base.log(p))
        .sum::<f64>();
    let normalized = if n <= 1 {
        0.0
    } else {
        let max_h = base.log(n as f64);
        if max_h == 0.0 { 0.0 } else { h / max_h }
    };
    Ok(EntropyResult {
        bits: h,
        normalized,
        sample_count: n,
    })
}

// ============================================================================
// Missing Entropy Functions (B.2)
// ============================================================================

/// Cross-entropy: H(P,Q) = -Σ pᵢ·log(qᵢ)
///
/// Measures the average number of bits (or nats/hartleys) needed to encode
/// samples from distribution P using a code optimized for distribution Q.
///
/// Cross-entropy is always >= H(P). The difference H(P,Q) - H(P) = D_KL(P||Q).
///
/// # Errors
///
/// - [`EntropyError::LengthMismatch`] — P and Q have different lengths.
/// - [`EntropyError::KlUndefined`] — q[i] = 0 where p[i] > 0.
/// - Standard distribution validation errors.
pub fn cross_entropy(p: &[f64], q: &[f64], base: LogBase) -> Result<f64, EntropyError> {
    if p.len() != q.len() {
        return Err(EntropyError::LengthMismatch(p.len(), q.len()));
    }
    validate_distribution(p)?;
    validate_distribution(q)?;

    let mut ce = 0.0_f64;
    for (index, (&pi, &qi)) in p.iter().zip(q.iter()).enumerate() {
        if pi == 0.0 {
            continue;
        }
        if qi == 0.0 {
            return Err(EntropyError::KlUndefined { index });
        }
        ce -= pi * base.log(qi);
    }
    Ok(ce)
}

/// Conditional entropy: H(Y|X) = H(X,Y) - H(X)
///
/// Measures the remaining uncertainty about Y given knowledge of X.
///
/// Takes a joint probability matrix (rows = X, columns = Y) and uses the row
/// marginals as the distribution of X.
///
/// # Errors
///
/// Returns any error that [`joint_entropy_with_base`] would return.
pub fn conditional_entropy(joint: &[Vec<f64>], base: LogBase) -> Result<f64, EntropyError> {
    let h_xy = joint_entropy_with_base(joint, base)?;

    // Row marginals -> distribution of X
    let marginal_x: Vec<f64> = joint.iter().map(|row| row.iter().sum::<f64>()).collect();
    let h_x = entropy_core_unchecked(&marginal_x, base)?.bits;

    // H(Y|X) = H(X,Y) - H(X), clamped to 0 for floating-point safety
    Ok((h_xy - h_x).max(0.0))
}

/// Normalized entropy: H(X) / H_max = H(X) / log(n)
///
/// Returns a value in [0, 1] where 0 = perfectly ordered (deterministic)
/// and 1 = maximum disorder (uniform distribution).
///
/// This is equivalent to `EntropyResult::normalized`, but provided as a
/// standalone function for convenience when only the normalized value is needed.
///
/// # Errors
///
/// Standard distribution validation errors.
pub fn normalized_entropy(distribution: &[f64], base: LogBase) -> Result<f64, EntropyError> {
    let result = shannon_entropy_with_base(distribution, base)?;
    Ok(result.normalized)
}

// ============================================================================
// Measured Entropy (B.4)
// ============================================================================

/// Shannon entropy with confidence based on sample size.
///
/// Takes raw counts and returns `Measured<EntropyResult>` where confidence
/// is calibrated by the ratio of total observations to category count:
/// confidence = clamp(1.0 - k/total, 0.05, 0.99) where k = number of categories.
///
/// Rationale: need at least k observations for each category to have one
/// expected count per bin. Small samples produce unreliable entropy estimates.
///
/// # Errors
///
/// Returns [`EntropyError::EmptyCounts`] if counts is empty or all zeros.
pub fn shannon_entropy_measured(
    counts: &[u64],
) -> Result<crate::Measured<EntropyResult>, EntropyError> {
    let result = entropy_from_counts(counts)?;
    let total: u64 = counts.iter().sum();
    let k = counts.len() as f64;
    let confidence = (1.0 - k / total as f64).clamp(0.05, 0.99);
    Ok(crate::Measured::new(
        result,
        crate::Confidence::new(confidence),
    ))
}

/// Shannon entropy measured with configurable base.
///
/// Combines [`entropy_from_counts_with_base`] with sample-size confidence.
pub fn shannon_entropy_measured_with_base(
    counts: &[u64],
    base: LogBase,
) -> Result<crate::Measured<EntropyResult>, EntropyError> {
    let result = entropy_from_counts_with_base(counts, base)?;
    let total: u64 = counts.iter().sum();
    let k = counts.len() as f64;
    let confidence = (1.0 - k / total as f64).clamp(0.05, 0.99);
    Ok(crate::Measured::new(
        result,
        crate::Confidence::new(confidence),
    ))
}

// ============================================================================
// PMI / Entropy Relationship Documentation (B.3)
// ============================================================================

// The Information Component (IC) used in BCPNN signal detection is
// mathematically equivalent to Pointwise Mutual Information (PMI) with
// Bayesian shrinkage (k=0.5 additive smoothing):
//
//   IC(drug, event) = log2((observed + 0.5) / (expected + 0.5))
//
// This is the pointwise form of mutual information. The aggregate MI computed
// by `mutual_information()` is the expected value of PMI across all pairs:
//
//   I(X; Y) = E[PMI(x, y)] = sum p(x,y) * log2(p(x,y) / (p(x)*p(y)))
//
// The IC implementation in nexcore-pv-core/src/signals/bayesian/ic.rs
// computes the per-pair pointwise form with signal detection-specific smoothing,
// while this module computes the aggregate form from full distributions.

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- shannon_entropy: uniform distributions ---

    #[test]
    fn test_uniform_2_outcomes() {
        let result = shannon_entropy(&[0.5, 0.5]).unwrap();
        assert!(
            (result.bits - 1.0).abs() < 1e-10,
            "H({:?}) should be 1.0",
            result.bits
        );
        assert_eq!(result.sample_count, 2);
    }

    #[test]
    fn test_uniform_4_outcomes() {
        let probs = vec![0.25; 4];
        let result = shannon_entropy(&probs).unwrap();
        assert!((result.bits - 2.0).abs() < 1e-10, "H = {}", result.bits);
    }

    #[test]
    fn test_uniform_8_outcomes() {
        let probs = vec![0.125; 8];
        let result = shannon_entropy(&probs).unwrap();
        assert!((result.bits - 3.0).abs() < 1e-10, "H = {}", result.bits);
    }

    // --- shannon_entropy: deterministic / boundary cases ---

    #[test]
    fn test_single_element_entropy() {
        let result = shannon_entropy(&[1.0]).unwrap();
        assert_eq!(result.bits, 0.0);
        assert_eq!(result.normalized, 0.0);
        assert_eq!(result.sample_count, 1);
    }

    #[test]
    fn test_deterministic_first() {
        // p = [1, 0, 0] — all mass on first outcome
        let probs = vec![1.0, 0.0, 0.0];
        let result = shannon_entropy(&probs).unwrap();
        assert_eq!(result.bits, 0.0);
        assert_eq!(result.normalized, 0.0);
    }

    #[test]
    fn test_deterministic_middle() {
        // p = [0, 1, 0]
        let probs = vec![0.0, 1.0, 0.0];
        let result = shannon_entropy(&probs).unwrap();
        assert_eq!(result.bits, 0.0);
    }

    #[test]
    fn test_binary_half_half() {
        let result = shannon_entropy(&[0.5, 0.5]).unwrap();
        assert!((result.bits - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_binary_skewed() {
        // H(0.9, 0.1) ≈ 0.4690 bits
        let result = shannon_entropy(&[0.9, 0.1]).unwrap();
        let expected = -(0.9_f64 * 0.9_f64.log2() + 0.1_f64 * 0.1_f64.log2());
        assert!(
            (result.bits - expected).abs() < 1e-10,
            "H = {}",
            result.bits
        );
    }

    // --- shannon_entropy: normalized entropy ---

    #[test]
    fn test_normalized_uniform_is_one() {
        let probs = vec![0.25; 4];
        let result = shannon_entropy(&probs).unwrap();
        assert!((result.normalized - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalized_deterministic_is_zero() {
        let probs = vec![1.0, 0.0, 0.0, 0.0];
        let result = shannon_entropy(&probs).unwrap();
        assert_eq!(result.normalized, 0.0);
    }

    #[test]
    fn test_normalized_single_element_is_zero() {
        let result = shannon_entropy(&[1.0]).unwrap();
        assert_eq!(result.normalized, 0.0);
    }

    // --- entropy_from_counts ---

    #[test]
    fn test_counts_equal_gives_one_bit() {
        let result = entropy_from_counts(&[10, 10]).unwrap();
        assert!((result.bits - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_counts_deterministic_is_zero() {
        let result = entropy_from_counts(&[100, 0]).unwrap();
        assert_eq!(result.bits, 0.0);
    }

    #[test]
    fn test_counts_all_zeros_is_error() {
        let err = entropy_from_counts(&[0, 0, 0]).unwrap_err();
        assert_eq!(err, EntropyError::EmptyCounts);
    }

    #[test]
    fn test_counts_empty_is_error() {
        let err = entropy_from_counts(&[]).unwrap_err();
        assert_eq!(err, EntropyError::EmptyCounts);
    }

    #[test]
    fn test_counts_uniform_4() {
        let result = entropy_from_counts(&[25, 25, 25, 25]).unwrap();
        assert!((result.bits - 2.0).abs() < 1e-10);
    }

    // --- kl_divergence ---

    #[test]
    fn test_kl_identical_is_zero() {
        let p = vec![0.5, 0.5];
        let q = vec![0.5, 0.5];
        let d = kl_divergence(&p, &q).unwrap();
        assert!(d.abs() < 1e-10);
    }

    #[test]
    fn test_kl_non_negative() {
        let p = vec![0.9, 0.1];
        let q = vec![0.5, 0.5];
        let d = kl_divergence(&p, &q).unwrap();
        assert!(d >= 0.0);
    }

    #[test]
    fn test_kl_asymmetry() {
        let p = vec![0.9, 0.1];
        let q = vec![0.5, 0.5];
        let dpq = kl_divergence(&p, &q).unwrap();
        let dqp = kl_divergence(&q, &p).unwrap();
        // D_KL(P||Q) != D_KL(Q||P) for non-identical distributions
        assert!((dpq - dqp).abs() > 1e-6, "dpq={dpq}, dqp={dqp}");
    }

    #[test]
    fn test_kl_undefined_when_q_zero() {
        let p = vec![0.5, 0.5];
        let q = vec![1.0, 0.0];
        let err = kl_divergence(&p, &q).unwrap_err();
        assert_eq!(err, EntropyError::KlUndefined { index: 1 });
    }

    #[test]
    fn test_kl_defined_when_p_zero_q_nonzero() {
        // p[1] = 0, q[1] = 1 — this is fine; the term contributes 0
        let p = vec![1.0, 0.0];
        let q = vec![0.5, 0.5];
        let d = kl_divergence(&p, &q).unwrap();
        // D_KL = 1.0 * log2(1.0 / 0.5) = 1.0
        assert!((d - 1.0).abs() < 1e-10, "d = {d}");
    }

    #[test]
    fn test_kl_length_mismatch() {
        let p = vec![0.5, 0.5];
        let q = vec![0.333, 0.333, 0.334];
        let err = kl_divergence(&p, &q).unwrap_err();
        assert_eq!(err, EntropyError::LengthMismatch(2, 3));
    }

    // --- information_loss ---

    #[test]
    fn test_information_loss_same_distribution() {
        let dist = vec![0.5, 0.5];
        let loss = information_loss(&dist, &dist).unwrap();
        assert!(loss.lost_bits.abs() < 1e-10);
        assert_eq!(loss.loss_fraction, 0.0);
    }

    #[test]
    fn test_information_loss_entropy_reduced() {
        // From uniform to deterministic: lose 1 bit
        let before = vec![0.5, 0.5];
        let after = vec![1.0, 0.0];
        let loss = information_loss(&before, &after).unwrap();
        assert!((loss.lost_bits - 1.0).abs() < 1e-10);
        assert!((loss.loss_fraction - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_information_loss_entropy_increased() {
        // From deterministic to uniform: lost_bits is negative (entropy grew)
        let before = vec![1.0, 0.0];
        let after = vec![0.5, 0.5];
        let loss = information_loss(&before, &after).unwrap();
        assert!(loss.lost_bits < 0.0);
        assert_eq!(loss.before_bits, 0.0);
        // before_bits == 0, so loss_fraction must be 0
        assert_eq!(loss.loss_fraction, 0.0);
    }

    #[test]
    fn test_information_loss_fields_consistent() {
        let before = vec![0.25, 0.25, 0.25, 0.25];
        let after = vec![0.5, 0.5, 0.0, 0.0];
        let loss = information_loss(&before, &after).unwrap();
        assert!((loss.before_bits - 2.0).abs() < 1e-10);
        assert!((loss.after_bits - 1.0).abs() < 1e-10);
        assert!((loss.lost_bits - 1.0).abs() < 1e-10);
        assert!((loss.loss_fraction - 0.5).abs() < 1e-10);
    }

    // --- joint_entropy ---

    #[test]
    fn test_joint_entropy_2x2_uniform() {
        let joint = vec![vec![0.25, 0.25], vec![0.25, 0.25]];
        let h = joint_entropy(&joint).unwrap();
        assert!((h - 2.0).abs() < 1e-10, "H(X,Y) = {h}");
    }

    #[test]
    fn test_joint_entropy_independent_variables() {
        // X: Bernoulli(0.5), Y: Bernoulli(0.5), independent
        // H(X,Y) = H(X) + H(Y) = 2.0
        let joint = vec![vec![0.25, 0.25], vec![0.25, 0.25]];
        let h = joint_entropy(&joint).unwrap();
        assert!((h - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_joint_entropy_invalid_empty() {
        let err = joint_entropy(&[]).unwrap_err();
        assert_eq!(err, EntropyError::InvalidJoint);
    }

    #[test]
    fn test_joint_entropy_non_rectangular() {
        let joint = vec![vec![0.5, 0.5], vec![1.0]];
        let err = joint_entropy(&joint).unwrap_err();
        assert_eq!(err, EntropyError::InvalidJoint);
    }

    #[test]
    fn test_joint_entropy_invalid_sum() {
        let joint = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        let err = joint_entropy(&joint).unwrap_err();
        // Sum is 2.0, not 1.0
        assert!(matches!(err, EntropyError::InvalidDistribution(_)));
    }

    // --- mutual_information ---

    #[test]
    fn test_mutual_information_independent_is_zero() {
        let joint = vec![vec![0.25, 0.25], vec![0.25, 0.25]];
        let mi = mutual_information(&joint).unwrap();
        assert!(mi.abs() < 1e-10, "MI for independent vars = {mi}");
    }

    #[test]
    fn test_mutual_information_perfectly_correlated() {
        // X determines Y: p(0,0) = 0.5, p(1,1) = 0.5, others = 0
        // I(X;Y) = H(X) = H(Y) = 1.0 bit
        let joint = vec![vec![0.5, 0.0], vec![0.0, 0.5]];
        let mi = mutual_information(&joint).unwrap();
        assert!((mi - 1.0).abs() < 1e-10, "MI for correlated vars = {mi}");
    }

    #[test]
    fn test_mutual_information_non_negative() {
        let joint = vec![vec![0.4, 0.1], vec![0.1, 0.4]];
        let mi = mutual_information(&joint).unwrap();
        assert!(mi >= 0.0);
    }

    // --- error cases ---

    #[test]
    fn test_empty_distribution_error() {
        let err = shannon_entropy(&[]).unwrap_err();
        assert_eq!(err, EntropyError::EmptyDistribution);
    }

    #[test]
    fn test_invalid_probability_negative() {
        let err = shannon_entropy(&[-0.1, 1.1]).unwrap_err();
        assert_eq!(err, EntropyError::InvalidProbability(-0.1));
    }

    #[test]
    fn test_invalid_probability_exceeds_one() {
        let err = shannon_entropy(&[1.5, -0.5]).unwrap_err();
        assert_eq!(err, EntropyError::InvalidProbability(1.5));
    }

    #[test]
    fn test_invalid_sum() {
        let err = shannon_entropy(&[0.3, 0.3]).unwrap_err();
        assert!(matches!(err, EntropyError::InvalidDistribution(_)));
    }

    // --- edge cases ---

    #[test]
    fn test_very_small_probabilities() {
        // 1000 outcomes where one dominates, rest have tiny probability
        let epsilon = 1e-10;
        let remaining = (1.0 - 999.0 * epsilon) / 1.0;
        let mut probs: Vec<f64> = vec![epsilon; 999];
        probs.push(remaining);

        // Should not panic or error; entropy should be very close to 0
        let result = shannon_entropy(&probs).unwrap();
        // The distribution is very concentrated so entropy is small
        assert!(result.bits >= 0.0);
        assert!(result.normalized >= 0.0 && result.normalized <= 1.0);
    }

    #[test]
    fn test_large_distribution_1000_elements() {
        // Uniform over 1000 outcomes: H = log₂(1000) ≈ 9.9658 bits
        let n = 1000usize;
        let p = 1.0 / n as f64;
        let probs = vec![p; n];
        let result = shannon_entropy(&probs).unwrap();
        let expected = (n as f64).log2();
        assert!((result.bits - expected).abs() < 1e-6, "H = {}", result.bits);
        // Normalized should be exactly 1.0 for a uniform distribution
        assert!((result.normalized - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_entropy_result_sample_count() {
        let probs = vec![0.1, 0.2, 0.3, 0.4];
        let result = shannon_entropy(&probs).unwrap();
        assert_eq!(result.sample_count, 4);
    }

    #[test]
    fn test_kl_divergence_known_value() {
        // D_KL([0.9, 0.1] || [0.5, 0.5]) = 0.9*log2(1.8) + 0.1*log2(0.2)
        let p = vec![0.9, 0.1];
        let q = vec![0.5, 0.5];
        let d = kl_divergence(&p, &q).unwrap();
        let expected = 0.9 * (0.9_f64 / 0.5).log2() + 0.1 * (0.1_f64 / 0.5).log2();
        assert!((d - expected).abs() < 1e-10, "d={d}, expected={expected}");
    }

    #[test]
    fn test_mutual_information_partial_correlation() {
        // Asymmetric joint: slightly more correlated than independent
        let joint = vec![vec![0.4, 0.1], vec![0.1, 0.4]];
        let mi = mutual_information(&joint).unwrap();
        // MI should be positive (correlation present) and less than H(X) = 1 bit
        assert!(mi > 0.0);
        assert!(mi < 1.0);
    }

    // ================================================================
    // B.1 — LogBase parameterization tests
    // ================================================================

    #[test]
    fn test_logbase_bits_matches_original() {
        let dist = [0.5, 0.5];
        let original = shannon_entropy(&dist).unwrap();
        let with_base = shannon_entropy_with_base(&dist, LogBase::Bits).unwrap();
        assert!((original.bits - with_base.bits).abs() < 1e-10);
        assert!((original.normalized - with_base.normalized).abs() < 1e-10);
    }

    #[test]
    fn test_logbase_nats_conversion() {
        // 1 bit = ln(2) nats ≈ 0.6931
        let dist = [0.5, 0.5];
        let bits = shannon_entropy_with_base(&dist, LogBase::Bits).unwrap();
        let nats = shannon_entropy_with_base(&dist, LogBase::Nats).unwrap();
        let expected_nats = bits.bits * std::f64::consts::LN_2;
        assert!(
            (nats.bits - expected_nats).abs() < 1e-10,
            "nats={}, expected={}",
            nats.bits,
            expected_nats
        );
    }

    #[test]
    fn test_logbase_hartleys_conversion() {
        // 1 bit = log10(2) hartleys ≈ 0.3010
        let dist = [0.5, 0.5];
        let bits = shannon_entropy_with_base(&dist, LogBase::Bits).unwrap();
        let hartleys = shannon_entropy_with_base(&dist, LogBase::Hartleys).unwrap();
        let expected_hartleys = bits.bits * 2.0_f64.log10();
        assert!(
            (hartleys.bits - expected_hartleys).abs() < 1e-10,
            "hartleys={}, expected={}",
            hartleys.bits,
            expected_hartleys
        );
    }

    #[test]
    fn test_logbase_normalized_invariant() {
        // Normalized entropy should be the same regardless of base
        let dist = [0.3, 0.7];
        let bits_n = shannon_entropy_with_base(&dist, LogBase::Bits)
            .unwrap()
            .normalized;
        let nats_n = shannon_entropy_with_base(&dist, LogBase::Nats)
            .unwrap()
            .normalized;
        let hart_n = shannon_entropy_with_base(&dist, LogBase::Hartleys)
            .unwrap()
            .normalized;
        assert!((bits_n - nats_n).abs() < 1e-10);
        assert!((bits_n - hart_n).abs() < 1e-10);
    }

    #[test]
    fn test_kl_divergence_with_base_nats() {
        let p = vec![0.9, 0.1];
        let q = vec![0.5, 0.5];
        let kl_bits = kl_divergence(&p, &q).unwrap();
        let kl_nats = kl_divergence_with_base(&p, &q, LogBase::Nats).unwrap();
        let expected_nats = kl_bits * std::f64::consts::LN_2;
        assert!((kl_nats - expected_nats).abs() < 1e-10);
    }

    #[test]
    fn test_entropy_from_counts_with_base() {
        let counts = [10u64, 10];
        let bits = entropy_from_counts(&counts).unwrap();
        let nats = entropy_from_counts_with_base(&counts, LogBase::Nats).unwrap();
        let expected_nats = bits.bits * std::f64::consts::LN_2;
        assert!((nats.bits - expected_nats).abs() < 1e-10);
    }

    #[test]
    fn test_logbase_default_is_bits() {
        assert_eq!(LogBase::default(), LogBase::Bits);
    }

    #[test]
    fn test_logbase_unit_names() {
        assert_eq!(LogBase::Bits.unit_name(), "bits");
        assert_eq!(LogBase::Nats.unit_name(), "nats");
        assert_eq!(LogBase::Hartleys.unit_name(), "hartleys");
    }

    #[test]
    fn test_logbase_from_bits_factor() {
        assert!((LogBase::Bits.from_bits_factor() - 1.0).abs() < 1e-10);
        assert!((LogBase::Nats.from_bits_factor() - std::f64::consts::LN_2).abs() < 1e-10);
        assert!((LogBase::Hartleys.from_bits_factor() - 2.0_f64.log10()).abs() < 1e-10);
    }

    // ================================================================
    // B.2 — Missing entropy function tests
    // ================================================================

    #[test]
    fn test_cross_entropy_identical_equals_shannon() {
        // H(P,P) = H(P) for identical distributions
        let p = vec![0.5, 0.5];
        let ce = cross_entropy(&p, &p, LogBase::Bits).unwrap();
        let h = shannon_entropy(&p).unwrap().bits;
        assert!((ce - h).abs() < 1e-10);
    }

    #[test]
    fn test_cross_entropy_ge_shannon() {
        // H(P,Q) >= H(P) always (Gibbs inequality)
        let p = vec![0.9, 0.1];
        let q = vec![0.5, 0.5];
        let ce = cross_entropy(&p, &q, LogBase::Bits).unwrap();
        let h = shannon_entropy(&p).unwrap().bits;
        assert!(ce >= h - 1e-10, "cross_entropy={ce} < shannon={h}");
    }

    #[test]
    fn test_cross_entropy_minus_shannon_equals_kl() {
        // H(P,Q) - H(P) = D_KL(P||Q)
        let p = vec![0.9, 0.1];
        let q = vec![0.5, 0.5];
        let ce = cross_entropy(&p, &q, LogBase::Bits).unwrap();
        let h = shannon_entropy(&p).unwrap().bits;
        let kl = kl_divergence(&p, &q).unwrap();
        assert!((ce - h - kl).abs() < 1e-10, "CE-H={}, KL={kl}", ce - h);
    }

    #[test]
    fn test_cross_entropy_with_nats() {
        let p = vec![0.5, 0.5];
        let ce_bits = cross_entropy(&p, &p, LogBase::Bits).unwrap();
        let ce_nats = cross_entropy(&p, &p, LogBase::Nats).unwrap();
        let expected_nats = ce_bits * std::f64::consts::LN_2;
        assert!((ce_nats - expected_nats).abs() < 1e-10);
    }

    #[test]
    fn test_cross_entropy_length_mismatch() {
        let p = vec![0.5, 0.5];
        let q = vec![0.33, 0.33, 0.34];
        let err = cross_entropy(&p, &q, LogBase::Bits).unwrap_err();
        assert_eq!(err, EntropyError::LengthMismatch(2, 3));
    }

    #[test]
    fn test_conditional_entropy_independent() {
        // Independent: H(Y|X) = H(Y)
        let joint = vec![vec![0.25, 0.25], vec![0.25, 0.25]];
        let h_yx = conditional_entropy(&joint, LogBase::Bits).unwrap();
        // H(Y) = 1.0 bit for Bernoulli(0.5)
        assert!((h_yx - 1.0).abs() < 1e-10, "H(Y|X) = {h_yx}");
    }

    #[test]
    fn test_conditional_entropy_perfectly_correlated() {
        // Perfectly correlated: H(Y|X) = 0
        let joint = vec![vec![0.5, 0.0], vec![0.0, 0.5]];
        let h_yx = conditional_entropy(&joint, LogBase::Bits).unwrap();
        assert!(h_yx.abs() < 1e-10, "H(Y|X) should be 0, got {h_yx}");
    }

    #[test]
    fn test_conditional_entropy_chain_rule() {
        // Chain rule: H(X,Y) = H(X) + H(Y|X)
        let joint = vec![vec![0.4, 0.1], vec![0.1, 0.4]];
        let h_xy = joint_entropy(&joint).unwrap();
        let marginal_x: Vec<f64> = joint.iter().map(|row| row.iter().sum::<f64>()).collect();
        let h_x = shannon_entropy_unchecked(&marginal_x).unwrap().bits;
        let h_yx = conditional_entropy(&joint, LogBase::Bits).unwrap();
        assert!((h_xy - h_x - h_yx).abs() < 1e-10);
    }

    #[test]
    fn test_normalized_entropy_uniform_is_one() {
        let uniform = vec![0.25, 0.25, 0.25, 0.25];
        let n = normalized_entropy(&uniform, LogBase::Bits).unwrap();
        assert!((n - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalized_entropy_deterministic_is_zero() {
        let det = vec![1.0, 0.0, 0.0, 0.0];
        let n = normalized_entropy(&det, LogBase::Bits).unwrap();
        assert_eq!(n, 0.0);
    }

    #[test]
    fn test_normalized_entropy_base_invariant() {
        let dist = vec![0.3, 0.7];
        let n_bits = normalized_entropy(&dist, LogBase::Bits).unwrap();
        let n_nats = normalized_entropy(&dist, LogBase::Nats).unwrap();
        assert!((n_bits - n_nats).abs() < 1e-10);
    }

    // ================================================================
    // B.4 — Measured entropy tests
    // ================================================================

    #[test]
    fn test_measured_entropy_value() {
        let counts = vec![50u64, 50];
        let m = shannon_entropy_measured(&counts).unwrap();
        assert!((m.value.bits - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_measured_entropy_high_confidence_many_samples() {
        // 1000 obs, 2 categories -> confidence = 1.0 - 2/1000 = 0.998 -> clamped to 0.99
        let counts = vec![500u64, 500];
        let m = shannon_entropy_measured(&counts).unwrap();
        assert!((m.confidence.value() - 0.99).abs() < 0.01);
    }

    #[test]
    fn test_measured_entropy_low_confidence_few_samples() {
        // 3 obs, 2 categories -> confidence = 1.0 - 2/3 = 0.333
        let counts = vec![2u64, 1];
        let m = shannon_entropy_measured(&counts).unwrap();
        assert!(
            (m.confidence.value() - (1.0 - 2.0 / 3.0)).abs() < 1e-10,
            "confidence = {}",
            m.confidence.value()
        );
    }

    #[test]
    fn test_measured_entropy_minimum_confidence() {
        // 1 obs, 1 category -> confidence = 1.0 - 1/1 = 0.0 -> clamped to 0.05
        let counts = vec![1u64];
        let m = shannon_entropy_measured(&counts).unwrap();
        assert!((m.confidence.value() - 0.05).abs() < 1e-10);
    }

    #[test]
    fn test_measured_entropy_with_base() {
        let counts = vec![50u64, 50];
        let m = shannon_entropy_measured_with_base(&counts, LogBase::Nats).unwrap();
        let expected_nats = std::f64::consts::LN_2; // 1 bit in nats
        assert!((m.value.bits - expected_nats).abs() < 1e-10);
    }

    #[test]
    fn test_measured_entropy_empty_counts_error() {
        let err = shannon_entropy_measured(&[]).unwrap_err();
        assert_eq!(err, EntropyError::EmptyCounts);
    }
}
