//! Betting Disproportionality Index (BDI) calculation.
//!
//! Adapted from Proportional Reporting Ratio (PRR) in pharmacovigilance.
//!
//! # References
//! - Evans SJW et al. (2001) Pharmacoepidemiol Drug Saf 10:483-6
//! - pv-toolkit signal detection module
//!
//! The BDI measures disproportionality between expected and observed
//! line movements given public betting percentages, analogous to how
//! PRR measures disproportionality between drug-event pairs.
//!
//! # Codex Compliance
//! - **Tier**: T2-C / T3 (Domain-Specific)
//! - **Grounding**: All newtypes ground to T1 primitives (f64, u32)
//! - **Quantification**: Qualitative strength maps to quantitative BDI values

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;

use super::thresholds::SignalStrength;

// =============================================================================
// CONSTANTS - T1 Primitives
// =============================================================================

/// Haldane-Anscombe correction value for zero cells.
const HALDANE_CORRECTION: f64 = 0.5;

/// Z-score for 95% confidence interval.
const Z_95: f64 = 1.96;

/// Small sample threshold for Yates correction.
const SMALL_SAMPLE_THRESHOLD: u32 = 40;

/// Default minimum BDI for signal detection (Evans criteria).
pub const DEFAULT_BDI_SIGNAL: f64 = 2.0;

/// Default minimum chi-square for statistical significance (p < 0.05, df=1).
pub const DEFAULT_CHI_SQUARE: f64 = 3.841;

/// Default minimum observations for signal validity.
pub const DEFAULT_MIN_OBSERVATIONS: u32 = 3;

/// Chi-square critical value for p < 0.01 (Strict).
pub const CHI_SQUARE_01: f64 = 6.635;

/// Chi-square critical value for p < 0.10 (Sensitive).
pub const CHI_SQUARE_10: f64 = 2.706;

// =============================================================================
// ERROR TYPES
// =============================================================================

/// Error for invalid metric values.
#[derive(Debug, Clone, PartialEq)]
pub enum BdiError {
    /// Value is negative (invalid for BDI).
    Negative(f64),
    /// Value is NaN.
    NaN,
    /// Value is infinite (unconstrained).
    Infinite,
    /// Contingency table has invalid values.
    InvalidTable(String),
}

impl fmt::Display for BdiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negative(v) => write!(f, "BDI value cannot be negative: {v}"),
            Self::NaN => write!(f, "BDI value cannot be NaN"),
            Self::Infinite => write!(f, "BDI value cannot be infinite"),
            Self::InvalidTable(msg) => write!(f, "Invalid contingency table: {msg}"),
        }
    }
}

impl std::error::Error for BdiError {}

// =============================================================================
// NEWTYPES - T2-P
// =============================================================================

/// Betting Disproportionality Index (BDI).
///
/// BDI = (a / (a+b)) / (c / (c+d))
///
/// Adapted from PRR (Proportional Reporting Ratio) in pharmacovigilance.
/// Signal threshold: >= 2.0 (Evans criteria adapted).
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy)]
pub struct Bdi(f64);

impl Bdi {
    /// Standard signal threshold (Evans criteria).
    pub const SIGNAL_THRESHOLD: f64 = DEFAULT_BDI_SIGNAL;

    /// Create a new BDI value.
    ///
    /// # Errors
    ///
    /// Returns `BdiError` if value is negative, NaN, or infinite.
    #[inline]
    pub fn new(value: f64) -> Result<Self, BdiError> {
        if value.is_nan() {
            Err(BdiError::NaN)
        } else if value.is_infinite() {
            Err(BdiError::Infinite)
        } else if value < 0.0 {
            Err(BdiError::Negative(value))
        } else {
            Ok(Self(value))
        }
    }

    /// Create a new BDI value, clamping to valid range.
    ///
    /// NaN becomes 0.0, negative becomes 0.0, infinite becomes `f64::MAX`.
    #[inline]
    #[must_use]
    pub fn new_clamped(value: f64) -> Self {
        if value.is_nan() || value < 0.0 {
            Self(0.0)
        } else if value.is_infinite() {
            Self(f64::MAX)
        } else {
            Self(value)
        }
    }

    /// Create from raw value without validation.
    #[inline]
    #[must_use]
    pub const fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    /// Get the raw f64 value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Check if this BDI exceeds the standard signal threshold.
    #[inline]
    #[must_use]
    pub fn exceeds_threshold(self) -> bool {
        self.0 >= Self::SIGNAL_THRESHOLD
    }

    /// Check if this BDI exceeds a custom threshold.
    #[inline]
    #[must_use]
    pub fn exceeds(self, threshold: f64) -> bool {
        self.0 >= threshold
    }

    /// Zero value (no signal).
    pub const ZERO: Self = Self(0.0);

    /// Unity value (no disproportionality).
    pub const ONE: Self = Self(1.0);
}

impl Default for Bdi {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Bdi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

impl PartialEq for Bdi {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON
    }
}

impl PartialOrd for Bdi {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Serialize for Bdi {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for Bdi {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BdiVisitor;

        impl Visitor<'_> for BdiVisitor {
            type Value = f64;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a non-negative finite number")
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<f64, E> {
                Ok(value)
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<f64, E> {
                Ok(value as f64)
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<f64, E> {
                Ok(value as f64)
            }
        }

        let value = deserializer.deserialize_f64(BdiVisitor)?;
        Bdi::new(value).map_err(de::Error::custom)
    }
}

impl From<Bdi> for f64 {
    fn from(bdi: Bdi) -> f64 {
        bdi.0
    }
}

/// Chi-square statistic for significance testing.
///
/// Used with BDI for Evans criteria (threshold: 3.841 for p < 0.05).
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy)]
pub struct ChiSquare(f64);

impl ChiSquare {
    /// Chi-square critical value for p < 0.05 with df = 1.
    pub const CRITICAL_05: f64 = DEFAULT_CHI_SQUARE;

    /// Chi-square critical value for p < 0.01 with df = 1.
    pub const CRITICAL_01: f64 = CHI_SQUARE_01;

    /// Chi-square critical value for p < 0.10 with df = 1.
    pub const CRITICAL_10: f64 = CHI_SQUARE_10;

    /// Create a new chi-square value.
    ///
    /// # Errors
    ///
    /// Returns `BdiError` if value is negative, NaN, or infinite.
    #[inline]
    pub fn new(value: f64) -> Result<Self, BdiError> {
        if value.is_nan() {
            Err(BdiError::NaN)
        } else if value.is_infinite() {
            Err(BdiError::Infinite)
        } else if value < 0.0 {
            Err(BdiError::Negative(value))
        } else {
            Ok(Self(value))
        }
    }

    /// Create from raw value without validation.
    #[inline]
    #[must_use]
    pub const fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    /// Get the raw f64 value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Check if statistically significant at p < 0.05.
    #[inline]
    #[must_use]
    pub fn is_significant(self) -> bool {
        self.0 >= Self::CRITICAL_05
    }

    /// Check if statistically significant at p < 0.01.
    #[inline]
    #[must_use]
    pub fn is_highly_significant(self) -> bool {
        self.0 >= Self::CRITICAL_01
    }

    /// Zero value.
    pub const ZERO: Self = Self(0.0);
}

impl Default for ChiSquare {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for ChiSquare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

impl PartialEq for ChiSquare {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON
    }
}

impl PartialOrd for ChiSquare {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Serialize for ChiSquare {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for ChiSquare {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f64::deserialize(deserializer)?;
        ChiSquare::new(value).map_err(de::Error::custom)
    }
}

impl From<ChiSquare> for f64 {
    fn from(chi: ChiSquare) -> f64 {
        chi.0
    }
}

/// P-value for statistical significance testing.
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy)]
pub struct PValue(f64);

impl PValue {
    /// Create a new p-value.
    ///
    /// # Errors
    ///
    /// Returns `BdiError` if value is outside [0, 1], NaN, or infinite.
    #[inline]
    pub fn new(value: f64) -> Result<Self, BdiError> {
        if value.is_nan() {
            Err(BdiError::NaN)
        } else if value.is_infinite() {
            Err(BdiError::Infinite)
        } else if !(0.0..=1.0).contains(&value) {
            Err(BdiError::InvalidTable(format!(
                "P-value must be in [0, 1], got {value}"
            )))
        } else {
            Ok(Self(value))
        }
    }

    /// Create from raw value, clamping to [0, 1].
    #[inline]
    #[must_use]
    pub fn new_clamped(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get the raw f64 value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Check if significant at alpha = 0.05.
    #[inline]
    #[must_use]
    pub fn is_significant(self) -> bool {
        self.0 < 0.05
    }

    /// Maximum p-value (no significance).
    pub const ONE: Self = Self(1.0);
}

impl Default for PValue {
    fn default() -> Self {
        Self::ONE
    }
}

impl fmt::Display for PValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

impl Serialize for PValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for PValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f64::deserialize(deserializer)?;
        PValue::new(value).map_err(de::Error::custom)
    }
}

/// Confidence interval bound.
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CiBound(f64);

impl CiBound {
    /// Create a new CI bound.
    #[inline]
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// Get the raw f64 value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Check if lower bound excludes null (> 1.0 for ratio metrics).
    #[inline]
    #[must_use]
    pub fn excludes_null(self) -> bool {
        self.0 > 1.0
    }
}

impl Default for CiBound {
    fn default() -> Self {
        Self(0.0)
    }
}

impl fmt::Display for CiBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

// =============================================================================
// ENUMS - T2-C
// =============================================================================

/// BDI-level signal type classification.
///
/// This is the low-level signal type determined directly from BDI calculation.
/// For the public-facing signal type used in combined classification, see
/// `classifier::SignalType`.
///
/// Mapped from PV signal types:
/// - `SharpBuy` -> New safety signal requiring action
/// - `ReverseLineMovement` -> Contradictory signal (observed != expected)
/// - `SteamMove` -> Acute/rapid signal emergence
/// - `PublicNoise` -> Background noise / no signal
///
/// # Tier: T2-C
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BdiSignalType {
    /// Sharp money action detected - significant edge potential.
    SharpBuy,
    /// Reverse Line Movement - line moves against public money.
    ReverseLineMovement,
    /// Steam Move - rapid cross-book movement indicating sharp action.
    SteamMove,
    /// Public aligned - movement matches public expectations.
    PublicAligned,
    /// Public Noise - no meaningful signal, background variance.
    PublicNoise,
    /// Neutral - insufficient data or borderline conditions.
    Neutral,
}

impl fmt::Display for BdiSignalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SharpBuy => write!(f, "Sharp Buy"),
            Self::ReverseLineMovement => write!(f, "Reverse Line Movement"),
            Self::SteamMove => write!(f, "Steam Move"),
            Self::PublicAligned => write!(f, "Public Aligned"),
            Self::PublicNoise => write!(f, "Public Noise"),
            Self::Neutral => write!(f, "Neutral"),
        }
    }
}

// =============================================================================
// CONTINGENCY TABLE - T2-C
// =============================================================================

/// 2x2 contingency table for BDI calculation.
///
/// Layout:
/// ```text
///                     | Line Moved Against Public | Line Moved With Public |
/// --------------------|---------------------------|------------------------|
/// Sharp Money Signal  |            a              |           b            |
/// No Sharp Signal     |            c              |           d            |
/// ```
///
/// # Tier: T2-C
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ContingencyTable {
    /// Sharp signal + Reverse movement (a).
    pub a: f64,
    /// Sharp signal + Expected movement (b).
    pub b: f64,
    /// No sharp signal + Reverse movement (c).
    pub c: f64,
    /// No sharp signal + Expected movement (d).
    pub d: f64,
}

impl ContingencyTable {
    /// Create new contingency table.
    #[must_use]
    pub fn new(a: f64, b: f64, c: f64, d: f64) -> Self {
        Self { a, b, c, d }
    }

    /// Create from integer counts.
    #[must_use]
    pub fn from_counts(a: u32, b: u32, c: u32, d: u32) -> Self {
        Self {
            a: f64::from(a),
            b: f64::from(b),
            c: f64::from(c),
            d: f64::from(d),
        }
    }

    /// Apply Haldane-Anscombe correction for zero cells.
    ///
    /// Zero cells cause mathematical issues (log(0), division by zero).
    /// This applies +0.5 correction only when a cell equals zero.
    #[must_use]
    pub fn with_haldane_correction(self) -> Self {
        if self.a == 0.0 || self.b == 0.0 || self.c == 0.0 || self.d == 0.0 {
            Self {
                a: self.a + HALDANE_CORRECTION,
                b: self.b + HALDANE_CORRECTION,
                c: self.c + HALDANE_CORRECTION,
                d: self.d + HALDANE_CORRECTION,
            }
        } else {
            self
        }
    }

    /// Total observations.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.a + self.b + self.c + self.d
    }

    /// Total observations as integer (truncated).
    #[must_use]
    pub fn n(&self) -> u32 {
        self.total() as u32
    }

    /// Row 1 total (sharp signals): a + b.
    #[must_use]
    pub fn row1_total(&self) -> f64 {
        self.a + self.b
    }

    /// Row 2 total (no sharp signals): c + d.
    #[must_use]
    pub fn row2_total(&self) -> f64 {
        self.c + self.d
    }

    /// Column 1 total (reverse movements): a + c.
    #[must_use]
    pub fn col1_total(&self) -> f64 {
        self.a + self.c
    }

    /// Column 2 total (expected movements): b + d.
    #[must_use]
    pub fn col2_total(&self) -> f64 {
        self.b + self.d
    }

    /// Row marginals as tuple (a+b, c+d).
    #[must_use]
    pub fn row_totals(&self) -> (f64, f64) {
        (self.row1_total(), self.row2_total())
    }

    /// Column marginals as tuple (a+c, b+d).
    #[must_use]
    pub fn col_totals(&self) -> (f64, f64) {
        (self.col1_total(), self.col2_total())
    }

    /// Check if table is valid (non-negative, non-empty).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.a >= 0.0 && self.b >= 0.0 && self.c >= 0.0 && self.d >= 0.0 && self.total() > 0.0
    }
}

impl Default for ContingencyTable {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

// =============================================================================
// THRESHOLDS - T2-C
// =============================================================================

/// Thresholds for BDI signal detection.
///
/// Adapted from:
/// - Evans criteria (PRR >= 2.0, chi-square >= 3.841, n >= 3)
/// - WHO-UMC criteria
/// - FDA-FAERS criteria
///
/// # Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdiThresholds {
    /// Minimum BDI for signal detection.
    pub bdi_signal: f64,
    /// BDI threshold for strong signal.
    pub bdi_strong: f64,
    /// BDI threshold for elite signal.
    pub bdi_elite: f64,
    /// Chi-square threshold for statistical significance.
    pub chi_square_threshold: f64,
    /// Minimum observations for signal validity.
    pub min_observations: u32,
    /// Lower CI must exceed this for ROR-style validation.
    pub ci_lower_must_exceed: f64,
    /// Threshold set name.
    pub name: String,
}

impl BdiThresholds {
    /// Standard Evans-adapted thresholds.
    #[must_use]
    pub fn evans() -> Self {
        Self {
            bdi_signal: 2.0,
            bdi_strong: 4.0,
            bdi_elite: 6.0,
            chi_square_threshold: DEFAULT_CHI_SQUARE,
            min_observations: DEFAULT_MIN_OBSERVATIONS,
            ci_lower_must_exceed: 1.0,
            name: "Evans-adapted".to_string(),
        }
    }

    /// Sensitive thresholds for early detection.
    #[must_use]
    pub fn sensitive() -> Self {
        Self {
            bdi_signal: 1.5,
            bdi_strong: 2.5,
            bdi_elite: 4.0,
            chi_square_threshold: CHI_SQUARE_10,
            min_observations: 2,
            ci_lower_must_exceed: 1.0,
            name: "Sensitive".to_string(),
        }
    }

    /// Conservative thresholds for high confidence.
    #[must_use]
    pub fn conservative() -> Self {
        Self {
            bdi_signal: 2.5,
            bdi_strong: 5.0,
            bdi_elite: 8.0,
            chi_square_threshold: CHI_SQUARE_01,
            min_observations: 5,
            ci_lower_must_exceed: 1.0,
            name: "Conservative".to_string(),
        }
    }

    /// Check if BDI meets Evans criteria with these thresholds.
    #[must_use]
    pub fn meets_criteria(&self, bdi: f64, chi_square: f64, n: u32) -> bool {
        bdi >= self.bdi_signal
            && chi_square >= self.chi_square_threshold
            && n >= self.min_observations
    }

    /// Determine signal strength from BDI value.
    #[must_use]
    pub fn classify_strength(&self, bdi: f64) -> SignalStrength {
        if bdi >= self.bdi_elite {
            SignalStrength::Elite
        } else if bdi >= self.bdi_strong {
            SignalStrength::Strong
        } else if bdi >= self.bdi_signal {
            SignalStrength::Moderate
        } else if bdi >= 1.0 {
            SignalStrength::Weak
        } else {
            SignalStrength::Avoid
        }
    }
}

impl Default for BdiThresholds {
    fn default() -> Self {
        Self::evans()
    }
}

// =============================================================================
// RESULT TYPES - T3
// =============================================================================

/// BDI calculation result.
///
/// Contains the BDI score, confidence interval, chi-square statistics,
/// and signal classification.
///
/// # Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdiResult {
    /// BDI value (PRR equivalent).
    pub bdi: f64,
    /// 95% confidence interval lower bound.
    pub ci_lower: f64,
    /// 95% confidence interval upper bound.
    pub ci_upper: f64,
    /// Chi-square statistic.
    pub chi_square: f64,
    /// P-value for chi-square test.
    pub p_value: f64,
    /// Total observations.
    pub n: u32,
    /// Original contingency table.
    pub contingency_table: ContingencyTable,
    /// Signal strength classification.
    pub signal_strength: SignalStrength,
    /// Signal type classification.
    pub signal_type: BdiSignalType,
    /// Whether signal meets Evans criteria.
    pub meets_criteria: bool,
    /// Factors contributing to signal confidence.
    pub confidence_factors: Vec<String>,
}

impl BdiResult {
    /// Check if this is an actionable signal.
    #[must_use]
    pub fn is_actionable(&self) -> bool {
        self.meets_criteria && self.signal_strength >= SignalStrength::Moderate
    }
}

// =============================================================================
// CORE CALCULATION FUNCTIONS
// =============================================================================

/// Calculate PRR-style ratio from contingency table cells.
///
/// PRR = (a / (a+b)) / (c / (c+d))
fn calculate_prr_ratio(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let row1 = a + b;
    let row2 = c + d;

    let p1 = if row1 > 0.0 { a / row1 } else { 0.0 };
    let p2 = if row2 > 0.0 { c / row2 } else { 0.0 };

    if p2 == 0.0 {
        if p1 > 0.0 {
            f64::MAX // Avoid infinity, use max representable
        } else {
            1.0
        }
    } else {
        p1 / p2
    }
}

/// Calculate log-transformed confidence interval for PRR/BDI.
///
/// CI = exp(ln(BDI) +/- z * SE)
/// where SE = sqrt(1/a - 1/(a+b) + 1/c - 1/(c+d))
fn calculate_confidence_interval(bdi: f64, a: f64, b: f64, c: f64, d: f64) -> (CiBound, CiBound) {
    if bdi <= 0.0 || bdi >= f64::MAX {
        return (CiBound::new(0.0), CiBound::new(f64::MAX));
    }

    let row1 = a + b;
    let row2 = c + d;

    // Calculate standard error on log scale
    let term1 = if a > 0.0 { 1.0 / a } else { 0.0 };
    let term2 = if row1 > 0.0 { 1.0 / row1 } else { 0.0 };
    let term3 = if c > 0.0 { 1.0 / c } else { 0.0 };
    let term4 = if row2 > 0.0 { 1.0 / row2 } else { 0.0 };

    let variance = term1 - term2 + term3 - term4;

    if variance < 0.0 {
        return (CiBound::new(0.0), CiBound::new(f64::MAX));
    }

    let se_log = variance.sqrt();
    let ln_bdi = bdi.ln();

    let lower = (ln_bdi - Z_95 * se_log).exp();
    let upper = (ln_bdi + Z_95 * se_log).exp();

    (CiBound::new(lower), CiBound::new(upper))
}

/// Calculate chi-square statistic from contingency table.
///
/// Applies Yates correction for small samples (N < 40).
fn calculate_chi_square_statistic(table: &ContingencyTable, use_yates: bool) -> ChiSquare {
    let n = table.total();
    if n == 0.0 {
        return ChiSquare::ZERO;
    }

    let row1 = table.row1_total();
    let row2 = table.row2_total();
    let col1 = table.col1_total();
    let col2 = table.col2_total();

    // Expected values
    let e_a = row1 * col1 / n;
    let e_b = row1 * col2 / n;
    let e_c = row2 * col1 / n;
    let e_d = row2 * col2 / n;

    // Check for zero expected values
    if e_a == 0.0 || e_b == 0.0 || e_c == 0.0 || e_d == 0.0 {
        return ChiSquare::ZERO;
    }

    // Yates correction
    let correction = if use_yates { 0.5 } else { 0.0 };

    let chi2_a = ((table.a - e_a).abs() - correction).powi(2) / e_a;
    let chi2_b = ((table.b - e_b).abs() - correction).powi(2) / e_b;
    let chi2_c = ((table.c - e_c).abs() - correction).powi(2) / e_c;
    let chi2_d = ((table.d - e_d).abs() - correction).powi(2) / e_d;

    let chi2 = chi2_a + chi2_b + chi2_c + chi2_d;

    // Clamp to valid range
    ChiSquare::new_unchecked(chi2.max(0.0))
}

/// Approximate p-value for chi-square distribution.
///
/// Uses Wilson-Hilferty approximation for chi-square CDF.
fn chi_square_to_p_value(chi_square: ChiSquare, df: u32) -> PValue {
    let chi2 = chi_square.value();
    let k = f64::from(df);

    if chi2 <= 0.0 || k <= 0.0 {
        return PValue::ONE;
    }

    // Wilson-Hilferty transformation
    let ratio = chi2 / k;
    let transformed = ratio.powf(1.0 / 3.0) - (1.0 - 2.0 / (9.0 * k));
    let z = transformed / (2.0 / (9.0 * k)).sqrt();

    // Standard normal CDF approximation
    let p = 1.0 - normal_cdf(z);

    PValue::new_clamped(p)
}

/// Standard normal CDF approximation (Abramowitz and Stegun).
fn normal_cdf(x: f64) -> f64 {
    const A1: f64 = 0.254_829_592;
    const A2: f64 = -0.284_496_736;
    const A3: f64 = 1.421_413_741;
    const A4: f64 = -1.453_152_027;
    const A5: f64 = 1.061_405_429;
    const P: f64 = 0.327_591_1;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs() / 2.0_f64.sqrt();

    let t = 1.0 / (1.0 + P * x);
    let y = 1.0 - (((((A5 * t + A4) * t) + A3) * t + A2) * t + A1) * t * (-x * x).exp();

    0.5 * (1.0 + sign * y)
}

/// Determine signal type based on BDI and context.
fn determine_signal_type(
    bdi: f64,
    is_reverse_movement: bool,
    is_steam_move: bool,
    thresholds: &BdiThresholds,
) -> BdiSignalType {
    if is_steam_move && bdi >= thresholds.bdi_signal {
        BdiSignalType::SteamMove
    } else if is_reverse_movement && bdi >= thresholds.bdi_signal {
        BdiSignalType::ReverseLineMovement
    } else if bdi >= thresholds.bdi_signal {
        BdiSignalType::SharpBuy
    } else if bdi < 1.0 {
        BdiSignalType::PublicNoise
    } else {
        BdiSignalType::Neutral
    }
}

/// Build confidence factors list explaining signal detection.
fn build_confidence_factors(
    bdi: f64,
    chi2: f64,
    p_value: f64,
    n: u32,
    ci_lower: f64,
    thresholds: &BdiThresholds,
) -> Vec<String> {
    let mut factors = Vec::new();

    // BDI threshold check
    if bdi >= thresholds.bdi_elite {
        factors.push(format!(
            "BDI {bdi:.2} exceeds elite threshold ({})",
            thresholds.bdi_elite
        ));
    } else if bdi >= thresholds.bdi_strong {
        factors.push(format!(
            "BDI {bdi:.2} exceeds strong threshold ({})",
            thresholds.bdi_strong
        ));
    } else if bdi >= thresholds.bdi_signal {
        factors.push(format!(
            "BDI {bdi:.2} meets signal threshold ({})",
            thresholds.bdi_signal
        ));
    } else {
        factors.push(format!(
            "BDI {bdi:.2} below signal threshold ({})",
            thresholds.bdi_signal
        ));
    }

    // Chi-square check (Evans criteria)
    if chi2 >= thresholds.chi_square_threshold {
        factors.push(format!("Chi-square {chi2:.2} significant (p={p_value:.4})"));
    } else {
        factors.push(format!(
            "Chi-square {chi2:.2} not significant - use caution"
        ));
    }

    // Sample size check
    if n >= thresholds.min_observations {
        factors.push(format!(
            "Sample size {n} meets minimum ({})",
            thresholds.min_observations
        ));
    } else {
        factors.push(format!("Sample size {n} below minimum - signal unreliable"));
    }

    // CI check (ROR-style)
    if ci_lower > thresholds.ci_lower_must_exceed {
        factors.push(format!("Lower 95% CI ({ci_lower:.2}) excludes null"));
    }

    factors
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Calculate Betting Disproportionality Index.
///
/// This is the primary frequentist signal detection method,
/// adapted from PRR (Proportional Reporting Ratio) in PV.
///
/// # Signal Criteria (Evans adapted)
/// 1. BDI >= 2.0
/// 2. Chi-square >= 3.841 (p < 0.05)
/// 3. N >= 3 observations
///
/// # Example
///
/// ```
/// use nexcore_vigilance::betting::bdi::{ContingencyTable, calculate_bdi};
///
/// let table = ContingencyTable::new(15.0, 5.0, 10.0, 70.0);
/// let result = calculate_bdi(table);
///
/// assert!(result.meets_criteria);
/// assert!(result.bdi >= 2.0);
/// ```
#[must_use]
pub fn calculate_bdi(table: ContingencyTable) -> BdiResult {
    calculate_bdi_with_options(table, false, false, &BdiThresholds::default())
}

/// Calculate BDI with custom options.
///
/// # Arguments
/// * `table` - 2x2 contingency table
/// * `is_reverse_movement` - Whether line moved against public betting
/// * `is_steam_move` - Whether rapid cross-book movement detected
/// * `thresholds` - BDI thresholds to use
#[must_use]
pub fn calculate_bdi_with_options(
    table: ContingencyTable,
    is_reverse_movement: bool,
    is_steam_move: bool,
    thresholds: &BdiThresholds,
) -> BdiResult {
    // Apply Haldane correction if needed
    let corrected = table.with_haldane_correction();
    let n = corrected.n();

    // Calculate BDI (PRR-style)
    let bdi_raw = calculate_prr_ratio(corrected.a, corrected.b, corrected.c, corrected.d);
    let bdi = if bdi_raw.is_finite() {
        bdi_raw
    } else {
        f64::MAX
    };

    // Calculate confidence interval
    let (ci_lower, ci_upper) =
        calculate_confidence_interval(bdi, corrected.a, corrected.b, corrected.c, corrected.d);

    // Calculate chi-square (with Yates correction for small samples)
    let use_yates = n < SMALL_SAMPLE_THRESHOLD;
    let chi_square = calculate_chi_square_statistic(&corrected, use_yates);
    let p_value = chi_square_to_p_value(chi_square, 1);

    // Determine if signal detected (Evans criteria)
    let meets_criteria = thresholds.meets_criteria(bdi, chi_square.value(), n);

    // Classify signal
    let signal_type = determine_signal_type(bdi, is_reverse_movement, is_steam_move, thresholds);
    let signal_strength = thresholds.classify_strength(bdi);

    // Build confidence factors
    let confidence_factors = build_confidence_factors(
        bdi,
        chi_square.value(),
        p_value.value(),
        n,
        ci_lower.value(),
        thresholds,
    );

    BdiResult {
        bdi,
        ci_lower: ci_lower.value(),
        ci_upper: ci_upper.value(),
        chi_square: chi_square.value(),
        p_value: p_value.value(),
        n,
        contingency_table: table,
        signal_strength,
        signal_type,
        meets_criteria,
        confidence_factors,
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Newtype Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_bdi_creation() {
        let bdi = Bdi::new(3.5);
        assert!(bdi.is_ok());
        assert!((bdi.map(Bdi::value).unwrap_or(0.0) - 3.5).abs() < f64::EPSILON);

        // Negative should fail
        assert!(Bdi::new(-1.0).is_err());

        // NaN should fail
        assert!(Bdi::new(f64::NAN).is_err());

        // Infinite should fail
        assert!(Bdi::new(f64::INFINITY).is_err());
    }

    #[test]
    fn test_bdi_clamped() {
        let bdi = Bdi::new_clamped(-1.0);
        assert!((bdi.value() - 0.0).abs() < f64::EPSILON);

        let bdi = Bdi::new_clamped(f64::NAN);
        assert!((bdi.value() - 0.0).abs() < f64::EPSILON);

        let bdi = Bdi::new_clamped(f64::INFINITY);
        assert_eq!(bdi.value(), f64::MAX);
    }

    #[test]
    fn test_bdi_threshold() {
        let below = Bdi::new_unchecked(1.5);
        let above = Bdi::new_unchecked(2.5);

        assert!(!below.exceeds_threshold());
        assert!(above.exceeds_threshold());
    }

    #[test]
    fn test_chi_square_significance() {
        let low = ChiSquare::new_unchecked(2.0);
        let high = ChiSquare::new_unchecked(5.0);
        let critical = ChiSquare::new_unchecked(3.841);

        assert!(!low.is_significant());
        assert!(high.is_significant());
        assert!(critical.is_significant());
    }

    #[test]
    fn test_p_value_creation() {
        assert!(PValue::new(0.05).is_ok());
        assert!(PValue::new(0.0).is_ok());
        assert!(PValue::new(1.0).is_ok());
        assert!(PValue::new(-0.1).is_err());
        assert!(PValue::new(1.1).is_err());
    }

    // -------------------------------------------------------------------------
    // Contingency Table Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_contingency_table_totals() {
        let table = ContingencyTable::new(15.0, 5.0, 10.0, 70.0);

        assert!((table.total() - 100.0).abs() < f64::EPSILON);
        assert!((table.row1_total() - 20.0).abs() < f64::EPSILON);
        assert!((table.row2_total() - 80.0).abs() < f64::EPSILON);
        assert!((table.col1_total() - 25.0).abs() < f64::EPSILON);
        assert!((table.col2_total() - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_haldane_correction() {
        let table = ContingencyTable::new(0.0, 10.0, 5.0, 85.0);
        let corrected = table.with_haldane_correction();

        assert!((corrected.a - 0.5).abs() < f64::EPSILON);
        assert!((corrected.b - 10.5).abs() < f64::EPSILON);
        assert!((corrected.c - 5.5).abs() < f64::EPSILON);
        assert!((corrected.d - 85.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_no_correction_when_not_needed() {
        let table = ContingencyTable::new(10.0, 10.0, 10.0, 70.0);
        let corrected = table.with_haldane_correction();

        // No correction applied when no zeros
        assert!((corrected.a - 10.0).abs() < f64::EPSILON);
        assert!((corrected.b - 10.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // BDI Calculation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_bdi_calculation_signal() {
        // Strong sharp signal with reverse movement
        let table = ContingencyTable::new(15.0, 5.0, 10.0, 70.0);
        let result = calculate_bdi(table);

        // BDI = (15/20) / (10/80) = 0.75 / 0.125 = 6.0
        assert!(result.bdi > 2.0, "BDI should indicate signal");
        assert!(
            result.chi_square > 3.841,
            "Should be statistically significant"
        );
        assert!(result.meets_criteria);
        assert!(result.is_actionable());
    }

    #[test]
    fn test_bdi_calculation_no_signal() {
        // No disproportionality
        let table = ContingencyTable::new(25.0, 25.0, 25.0, 25.0);
        let result = calculate_bdi(table);

        // BDI = (25/50) / (25/50) = 0.5 / 0.5 = 1.0
        assert!(
            (result.bdi - 1.0).abs() < 0.1,
            "BDI should be near 1.0, got {}",
            result.bdi
        );
        assert!(!result.meets_criteria);
    }

    #[test]
    fn test_bdi_with_zero_cells() {
        // Zero in cell a - Haldane correction should apply
        let table = ContingencyTable::new(0.0, 10.0, 5.0, 85.0);
        let result = calculate_bdi(table);

        // Should not panic, should have valid BDI
        assert!(result.bdi.is_finite());
        assert!(result.bdi >= 0.0);
    }

    #[test]
    fn test_bdi_signal_strength_classification() {
        let strong = ContingencyTable::new(20.0, 5.0, 5.0, 70.0);
        let result = calculate_bdi(strong);

        assert!(matches!(
            result.signal_strength,
            SignalStrength::Elite | SignalStrength::Strong
        ));
    }

    #[test]
    fn test_bdi_with_custom_thresholds() {
        let table = ContingencyTable::new(10.0, 10.0, 10.0, 70.0);
        let sensitive = BdiThresholds::sensitive();

        let result = calculate_bdi_with_options(table, false, false, &sensitive);

        // With sensitive thresholds (bdi_signal = 1.5), this should still be evaluated
        assert!(result.confidence_factors.len() >= 3);
    }

    // -------------------------------------------------------------------------
    // Signal Type Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_signal_type_steam_move() {
        let table = ContingencyTable::new(20.0, 5.0, 5.0, 70.0);
        let thresholds = BdiThresholds::default();

        let result = calculate_bdi_with_options(table, false, true, &thresholds);

        assert_eq!(result.signal_type, BdiSignalType::SteamMove);
    }

    #[test]
    fn test_signal_type_reverse_movement() {
        let table = ContingencyTable::new(20.0, 5.0, 5.0, 70.0);
        let thresholds = BdiThresholds::default();

        let result = calculate_bdi_with_options(table, true, false, &thresholds);

        assert_eq!(result.signal_type, BdiSignalType::ReverseLineMovement);
    }

    #[test]
    fn test_signal_type_sharp_buy() {
        let table = ContingencyTable::new(20.0, 5.0, 5.0, 70.0);
        let thresholds = BdiThresholds::default();

        let result = calculate_bdi_with_options(table, false, false, &thresholds);

        assert_eq!(result.signal_type, BdiSignalType::SharpBuy);
    }

    #[test]
    fn test_signal_type_public_noise() {
        // BDI < 1.0 scenario
        let table = ContingencyTable::new(5.0, 45.0, 20.0, 30.0);
        let result = calculate_bdi(table);

        assert_eq!(result.signal_type, BdiSignalType::PublicNoise);
    }

    // -------------------------------------------------------------------------
    // Confidence Factor Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_confidence_factors_generated() {
        let table = ContingencyTable::new(15.0, 5.0, 10.0, 70.0);
        let result = calculate_bdi(table);

        // Should have BDI, chi-square, sample size, and possibly CI factors
        assert!(
            result.confidence_factors.len() >= 3,
            "Should have at least 3 confidence factors"
        );

        // Check that BDI factor is present
        let has_bdi_factor = result.confidence_factors.iter().any(|f| f.contains("BDI"));
        assert!(has_bdi_factor, "Should have BDI confidence factor");
    }

    // -------------------------------------------------------------------------
    // Threshold Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_threshold_presets() {
        let evans = BdiThresholds::evans();
        assert!((evans.bdi_signal - 2.0).abs() < f64::EPSILON);
        assert!((evans.chi_square_threshold - 3.841).abs() < f64::EPSILON);

        let sensitive = BdiThresholds::sensitive();
        assert!((sensitive.bdi_signal - 1.5).abs() < f64::EPSILON);

        let conservative = BdiThresholds::conservative();
        assert!((conservative.bdi_signal - 2.5).abs() < f64::EPSILON);
        assert_eq!(conservative.min_observations, 5);
    }

    #[test]
    fn test_threshold_meets_criteria() {
        let thresholds = BdiThresholds::evans();

        assert!(thresholds.meets_criteria(2.5, 4.0, 5));
        assert!(!thresholds.meets_criteria(1.5, 4.0, 5)); // BDI too low
        assert!(!thresholds.meets_criteria(2.5, 3.0, 5)); // Chi-square too low
        assert!(!thresholds.meets_criteria(2.5, 4.0, 2)); // N too low
    }

    // -------------------------------------------------------------------------
    // Serialization Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_bdi_serialization() {
        let bdi = Bdi::new_unchecked(3.5);
        let json = serde_json::to_string(&bdi);
        assert!(json.is_ok());
        assert_eq!(json.unwrap_or_default(), "3.5");
    }

    #[test]
    fn test_result_serialization() {
        let table = ContingencyTable::new(15.0, 5.0, 10.0, 70.0);
        let result = calculate_bdi(table);

        let json = serde_json::to_string(&result);
        assert!(json.is_ok());

        let json_str = json.unwrap_or_default();
        assert!(json_str.contains("\"bdi\""));
        assert!(json_str.contains("\"chi_square\""));
        assert!(json_str.contains("\"meets_criteria\""));
    }

    // -------------------------------------------------------------------------
    // Edge Case Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_empty_table() {
        let table = ContingencyTable::new(0.0, 0.0, 0.0, 0.0);
        let result = calculate_bdi(table);

        // Should handle gracefully
        assert!(result.bdi.is_finite());
    }

    #[test]
    fn test_single_cell_nonzero() {
        let table = ContingencyTable::new(100.0, 0.0, 0.0, 0.0);
        let result = calculate_bdi(table);

        // Should handle gracefully with Haldane correction
        assert!(result.bdi.is_finite());
    }

    #[test]
    fn test_large_values() {
        let table = ContingencyTable::new(10000.0, 1000.0, 100.0, 100000.0);
        let result = calculate_bdi(table);

        assert!(result.bdi.is_finite());
        assert!(result.chi_square.is_finite());
        assert!(result.p_value >= 0.0 && result.p_value <= 1.0);
    }

    // -------------------------------------------------------------------------
    // Display Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_signal_type_display() {
        assert_eq!(format!("{}", BdiSignalType::SharpBuy), "Sharp Buy");
        assert_eq!(
            format!("{}", BdiSignalType::ReverseLineMovement),
            "Reverse Line Movement"
        );
        assert_eq!(format!("{}", BdiSignalType::SteamMove), "Steam Move");
        assert_eq!(format!("{}", BdiSignalType::PublicNoise), "Public Noise");
    }

    #[test]
    fn test_bdi_display() {
        let bdi = Bdi::new_unchecked(3.14159);
        assert_eq!(format!("{}", bdi), "3.1416");
    }
}
