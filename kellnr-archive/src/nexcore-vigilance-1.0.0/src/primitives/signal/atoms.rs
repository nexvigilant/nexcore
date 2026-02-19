//! # T2-P Signal Atoms
//!
//! Primitive building blocks for signal detection, each grounding to T1.
//!
//! ## Tier Classification
//!
//! All types in this module are **T2-P (Cross-Domain Primitive)**:
//! - Simple newtypes over T1 primitives
//! - No domain-specific logic
//! - Usable across any signal detection domain

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

// ============================================================================
// T1: Count (Quantity Primitive - N)
// ============================================================================

/// A count of occurrences (T1: Quantity).
///
/// Grounds to: `u64` (non-negative integer).
///
/// # Lex Primitiva
/// - Symbol: **N**
/// - Name: Quantity
/// - Definition: Numerical magnitude
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Count(u64);

impl Count {
    /// Create a new count.
    #[inline]
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Get the raw value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Zero count.
    pub const ZERO: Self = Self(0);

    /// Check if zero.
    #[inline]
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Convert to f64 for calculations.
    #[inline]
    #[must_use]
    pub fn as_f64(self) -> f64 {
        self.0 as f64
    }
}

impl Default for Count {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Count {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for Count {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Count {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl From<u64> for Count {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<u32> for Count {
    fn from(value: u32) -> Self {
        Self(u64::from(value))
    }
}

impl From<Count> for u64 {
    fn from(count: Count) -> Self {
        count.0
    }
}

// ============================================================================
// T2-P: Frequency (Rate Primitive - f)
// ============================================================================

/// A frequency/rate of occurrence (T2-P: Rate).
///
/// Grounds to: `f64` (proportion in [0.0, 1.0] or rate > 0).
///
/// # Lex Primitiva
/// - Symbol: **f**
/// - Name: Frequency
/// - Definition: Rate of occurrence
///
/// # Invariants
/// - Value is finite and non-negative
/// - For proportions: value in [0.0, 1.0]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Frequency(f64);

impl Frequency {
    /// Create a new frequency.
    ///
    /// # Errors
    /// Returns `None` if value is negative, NaN, or infinite.
    #[inline]
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        if value.is_finite() && value >= 0.0 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Create from unchecked value (caller guarantees validity).
    ///
    /// # Safety (Logical)
    /// Caller must ensure value is non-negative and finite.
    #[inline]
    #[must_use]
    pub const fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    /// Compute frequency from count and total.
    ///
    /// Returns `None` if total is zero.
    #[inline]
    #[must_use]
    pub fn from_count(numerator: Count, denominator: Count) -> Option<Self> {
        if denominator.is_zero() {
            None
        } else {
            Some(Self(numerator.as_f64() / denominator.as_f64()))
        }
    }

    /// Get the raw value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Zero frequency.
    pub const ZERO: Self = Self(0.0);

    /// Check if approximately zero.
    #[inline]
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0.abs() < f64::EPSILON
    }
}

impl Default for Frequency {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6}", self.0)
    }
}

impl PartialEq for Frequency {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON
    }
}

impl PartialOrd for Frequency {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl From<Frequency> for f64 {
    fn from(freq: Frequency) -> Self {
        freq.0
    }
}

// ============================================================================
// T2-P: Ratio (Comparison Primitive - kappa)
// ============================================================================

/// A ratio comparing two quantities (T2-P: Comparison).
///
/// Grounds to: `f64` (quotient of two values).
///
/// # Lex Primitiva
/// - Symbol: **kappa**
/// - Name: Comparison
/// - Definition: Predicate matching
///
/// # Semantics
/// - Ratio = 1.0: No difference
/// - Ratio > 1.0: Numerator exceeds denominator
/// - Ratio < 1.0: Denominator exceeds numerator
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ratio(f64);

impl Ratio {
    /// Create a new ratio.
    ///
    /// # Errors
    /// Returns `None` if value is NaN or infinite.
    #[inline]
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Create from unchecked value.
    #[inline]
    #[must_use]
    pub const fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    /// Compute ratio from two frequencies.
    ///
    /// Returns `None` if denominator is zero.
    #[inline]
    #[must_use]
    pub fn from_frequencies(numerator: Frequency, denominator: Frequency) -> Option<Self> {
        if denominator.is_zero() {
            None
        } else {
            Self::new(numerator.value() / denominator.value())
        }
    }

    /// Get the raw value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Unity ratio (no difference).
    pub const UNITY: Self = Self(1.0);

    /// Check if ratio indicates elevation (> 1.0).
    #[inline]
    #[must_use]
    pub fn is_elevated(self) -> bool {
        self.0 > 1.0
    }

    /// Check if ratio is approximately unity.
    #[inline]
    #[must_use]
    pub fn is_unity(self) -> bool {
        (self.0 - 1.0).abs() < f64::EPSILON
    }
}

impl Default for Ratio {
    fn default() -> Self {
        Self::UNITY
    }
}

impl fmt::Display for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

impl PartialEq for Ratio {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON
    }
}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl From<Ratio> for f64 {
    fn from(ratio: Ratio) -> Self {
        ratio.0
    }
}

impl Mul for Ratio {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl Div for Ratio {
    type Output = Option<Self>;
    fn div(self, rhs: Self) -> Option<Self> {
        if rhs.0.abs() < f64::EPSILON {
            None
        } else {
            Self::new(self.0 / rhs.0)
        }
    }
}

// ============================================================================
// T2-P: Threshold (Boundary Primitive - partial)
// ============================================================================

/// A decision threshold (T2-P: Boundary).
///
/// Grounds to: `f64` (boundary value for comparison).
///
/// # Lex Primitiva
/// - Symbol: **partial**
/// - Name: Boundary
/// - Definition: Delimiters and limits
///
/// # Usage
/// Thresholds define the boundary between signal and noise.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Threshold(f64);

impl Threshold {
    /// Create a new threshold.
    #[inline]
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Get the raw value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Standard signal threshold (2.0 for PRR/ROR).
    pub const STANDARD: Self = Self(2.0);

    /// Strict threshold (3.0).
    pub const STRICT: Self = Self(3.0);

    /// Sensitive threshold (1.5).
    pub const SENSITIVE: Self = Self(1.5);

    /// Chi-square critical value (p < 0.05, df=1).
    pub const CHI_SQUARE_CRITICAL: Self = Self(3.841);
}

impl Default for Threshold {
    fn default() -> Self {
        Self::STANDARD
    }
}

impl fmt::Display for Threshold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}", self.0)
    }
}

impl PartialEq for Threshold {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON
    }
}

impl PartialOrd for Threshold {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl From<Threshold> for f64 {
    fn from(threshold: Threshold) -> Self {
        threshold.0
    }
}

// ============================================================================
// T1: Detected (Existence Primitive - exists)
// ============================================================================

/// Signal detection result (T1: Existence).
///
/// Grounds to: `bool`.
///
/// # Lex Primitiva
/// - Symbol: **exists**
/// - Name: Existence
/// - Definition: Instantiation of being
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Detected(bool);

impl Detected {
    /// Signal detected.
    pub const YES: Self = Self(true);

    /// No signal.
    pub const NO: Self = Self(false);

    /// Create from boolean.
    #[inline]
    #[must_use]
    pub const fn new(detected: bool) -> Self {
        Self(detected)
    }

    /// Check if signal was detected.
    #[inline]
    #[must_use]
    pub const fn is_signal(self) -> bool {
        self.0
    }

    /// Check if no signal.
    #[inline]
    #[must_use]
    pub const fn is_noise(self) -> bool {
        !self.0
    }
}

impl Default for Detected {
    fn default() -> Self {
        Self::NO
    }
}

impl fmt::Display for Detected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if self.0 { "SIGNAL" } else { "NOISE" })
    }
}

impl From<bool> for Detected {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl From<Detected> for bool {
    fn from(detected: Detected) -> Self {
        detected.0
    }
}

// ============================================================================
// T2-P: Source (Location Primitive - lambda)
// ============================================================================

/// Source identifier for data origin (T2-P: Location).
///
/// Grounds to: enumerated variants.
///
/// # Lex Primitiva
/// - Symbol: **lambda**
/// - Name: Location
/// - Definition: Positional context
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Source {
    /// Known, named source.
    Known(String),
    /// Unknown source (preserves original label).
    Unknown(String),
}

impl Source {
    /// Create a known source.
    #[inline]
    #[must_use]
    pub fn known(name: impl Into<String>) -> Self {
        Self::Known(name.into())
    }

    /// Create an unknown source.
    #[inline]
    #[must_use]
    pub fn unknown(label: impl Into<String>) -> Self {
        Self::Unknown(label.into())
    }

    /// Get the source name/label.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Known(name) | Self::Unknown(name) => name,
        }
    }

    /// Check if source is known.
    #[must_use]
    pub const fn is_known(&self) -> bool {
        matches!(self, Self::Known(_))
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Known(name) => write!(f, "{name}"),
            Self::Unknown(label) => write!(f, "Unknown({label})"),
        }
    }
}

impl Default for Source {
    fn default() -> Self {
        Self::Unknown("unspecified".to_string())
    }
}

// ============================================================================
// Primitive Operations
// ============================================================================

/// Compute ratio from two frequencies (kappa primitive).
///
/// # Returns
/// - `Some(Ratio)` if denominator is non-zero
/// - `None` if denominator is zero
#[inline]
#[must_use]
pub fn compute_ratio(numerator: Frequency, denominator: Frequency) -> Option<Ratio> {
    Ratio::from_frequencies(numerator, denominator)
}

/// Check if ratio exceeds threshold (kappa >= partial).
///
/// # Lex Primitiva
/// Combines Comparison (kappa) and Boundary (partial) to produce Existence (exists).
#[inline]
#[must_use]
pub fn exceeds_threshold(ratio: Ratio, threshold: Threshold) -> Detected {
    Detected::new(ratio.value() >= threshold.value())
}

/// Compute frequency from raw counts (f = N_observed / N_total).
///
/// # Returns
/// - `Some(Frequency)` if total is non-zero
/// - `None` if total is zero
#[inline]
#[must_use]
pub fn compute_frequency(observed: Count, total: Count) -> Option<Frequency> {
    Frequency::from_count(observed, total)
}

// ============================================================================
// T2-P: Timestamp (Sequence Primitive - sigma)
// ============================================================================

/// A point in time (T2-P: Sequence).
///
/// Grounds to: `i64` (Unix milliseconds).
///
/// # Lex Primitiva
/// - Symbol: **σ** (sigma)
/// - Name: Sequence
/// - Definition: Temporal ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Create a new timestamp from Unix milliseconds.
    #[inline]
    #[must_use]
    pub const fn new(millis: i64) -> Self {
        Self(millis)
    }

    /// Get the Unix milliseconds value.
    #[inline]
    #[must_use]
    pub const fn millis(self) -> i64 {
        self.0
    }

    /// Create from Unix seconds.
    #[inline]
    #[must_use]
    pub const fn from_secs(secs: i64) -> Self {
        Self(secs * 1000)
    }

    /// Get Unix seconds.
    #[inline]
    #[must_use]
    pub const fn secs(self) -> i64 {
        self.0 / 1000
    }

    /// Unix epoch (1970-01-01 00:00:00 UTC).
    pub const EPOCH: Self = Self(0);

    /// Check if this timestamp is before another.
    #[inline]
    #[must_use]
    pub const fn before(self, other: Self) -> bool {
        self.0 < other.0
    }

    /// Check if this timestamp is after another.
    #[inline]
    #[must_use]
    pub const fn after(self, other: Self) -> bool {
        self.0 > other.0
    }

    /// Duration between two timestamps in milliseconds.
    #[inline]
    #[must_use]
    pub const fn duration_to(self, other: Self) -> i64 {
        other.0 - self.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::EPOCH
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ms", self.0)
    }
}

impl From<i64> for Timestamp {
    fn from(millis: i64) -> Self {
        Self(millis)
    }
}

impl From<Timestamp> for i64 {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

// ============================================================================
// T2-P: Association (Causality Primitive - arrow)
// ============================================================================

/// An exposure-outcome association (T2-P: Causality).
///
/// Grounds to: `(String, String)` pair.
///
/// # Lex Primitiva
/// - Symbol: **→** (arrow)
/// - Name: Causality
/// - Definition: Cause-effect relationship
///
/// # Cross-Domain Instantiation
///
/// | Domain | Exposure | Outcome |
/// |--------|----------|---------|
/// | Pharmacovigilance | Drug | Adverse Event |
/// | Epidemiology | Risk Factor | Disease |
/// | Finance | Market Condition | Price Movement |
/// | Cybersecurity | Attack Vector | Breach |
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Association {
    /// The exposure/cause.
    exposure: String,
    /// The outcome/effect.
    outcome: String,
}

impl Association {
    /// Create a new association.
    #[must_use]
    pub fn new(exposure: impl Into<String>, outcome: impl Into<String>) -> Self {
        Self {
            exposure: exposure.into(),
            outcome: outcome.into(),
        }
    }

    /// Get the exposure.
    #[must_use]
    pub fn exposure(&self) -> &str {
        &self.exposure
    }

    /// Get the outcome.
    #[must_use]
    pub fn outcome(&self) -> &str {
        &self.outcome
    }

    /// Create a reversed association (outcome -> exposure).
    #[must_use]
    pub fn reverse(&self) -> Self {
        Self {
            exposure: self.outcome.clone(),
            outcome: self.exposure.clone(),
        }
    }
}

impl fmt::Display for Association {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} → {}", self.exposure, self.outcome)
    }
}

// ============================================================================
// T2-P: Method (Sum Primitive - Sigma)
// ============================================================================

/// Signal detection algorithm selection (T2-P: Sum/Coproduct).
///
/// Grounds to: Enumerated variants.
///
/// # Lex Primitiva
/// - Symbol: **Σ** (Sigma)
/// - Name: Sum
/// - Definition: Mutually exclusive alternatives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Method {
    /// Proportional Reporting Ratio.
    PRR,
    /// Reporting Odds Ratio.
    ROR,
    /// Information Component (Bayesian).
    IC,
    /// Empirical Bayes Geometric Mean (Multi-item Gamma Poisson Shrinker).
    EBGM,
    /// Chi-square test.
    ChiSquare,
    /// All methods combined.
    Complete,
}

impl Method {
    /// Get the threshold for this method.
    #[must_use]
    pub const fn default_threshold(self) -> Threshold {
        match self {
            Self::PRR | Self::ROR | Self::EBGM => Threshold::STANDARD,
            Self::IC => Threshold::new(0.0), // IC > 0 means signal
            Self::ChiSquare => Threshold::CHI_SQUARE_CRITICAL,
            Self::Complete => Threshold::STANDARD,
        }
    }

    /// Get a human-readable name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::PRR => "Proportional Reporting Ratio",
            Self::ROR => "Reporting Odds Ratio",
            Self::IC => "Information Component",
            Self::EBGM => "Empirical Bayes Geometric Mean",
            Self::ChiSquare => "Chi-Square Test",
            Self::Complete => "Complete Analysis",
        }
    }

    /// Get short abbreviation.
    #[must_use]
    pub const fn abbrev(self) -> &'static str {
        match self {
            Self::PRR => "PRR",
            Self::ROR => "ROR",
            Self::IC => "IC",
            Self::EBGM => "EBGM",
            Self::ChiSquare => "χ²",
            Self::Complete => "ALL",
        }
    }
}

impl Default for Method {
    fn default() -> Self {
        Self::PRR
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbrev())
    }
}
