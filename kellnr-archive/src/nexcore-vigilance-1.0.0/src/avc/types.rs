//! # AVC Type Taxonomy
//!
//! Autonomous Vigilance Company types grounded to T1 primitives.
//! Each type implements `GroundsTo` for Lex Primitiva grounding.
//!
//! ## Tier Hierarchy
//!
//! - **T1 Wrappers**: Seq, Cmp, Boundary, Audit, Qty, Freq
//! - **T2-P Newtypes**: ClientId, SignalId, Thresh, Baseline, Alert
//! - **T2-C Composites**: Detection, Feedback, ClientConfig
//! - **T3 Domain**: Avc (the autonomous system)

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::lex_primitiva::{GroundsTo, LexPrimitiva, PrimitiveComposition};

// ═══════════════════════════════════════════════════════════
// T1 WRAPPERS
// ═══════════════════════════════════════════════════════════

/// σ: Timestamped sequence of observations.
/// Tier: T1 (σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries<T> {
    entries: Vec<(SystemTime, T)>,
}

impl<T> Default for TimeSeries<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TimeSeries<T> {
    /// Creates an empty time series.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Pushes a new timestamped entry.
    pub fn push(&mut self, ts: SystemTime, value: T) {
        self.entries.push((ts, value));
    }

    /// Pushes a value with current timestamp.
    pub fn push_now(&mut self, value: T) {
        self.entries.push((SystemTime::now(), value));
    }

    /// Returns the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns entries within a recent window.
    #[must_use]
    pub fn recent(&self, window: Duration) -> Vec<&(SystemTime, T)> {
        let cutoff = SystemTime::now()
            .checked_sub(window)
            .unwrap_or(SystemTime::UNIX_EPOCH);
        self.entries
            .iter()
            .filter(|(ts, _)| *ts >= cutoff)
            .collect()
    }

    /// Drains all entries, returning them.
    pub fn drain(&mut self) -> Vec<(SystemTime, T)> {
        std::mem::take(&mut self.entries)
    }

    /// Returns a slice of all entries.
    #[must_use]
    pub fn entries(&self) -> &[(SystemTime, T)] {
        &self.entries
    }
}

impl GroundsTo for TimeSeries<f64> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence])
    }
}

/// ς: Discrete operational state.
/// Tier: T1 (ς)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationalState {
    /// System healthy, all metrics nominal.
    Nominal,
    /// Elevated awareness, some metrics out of band.
    Alert,
    /// Reduced capacity, self-healing active.
    Degraded,
    /// Immediate attention required, escalation active.
    Critical,
}

impl OperationalState {
    /// Returns true if the system is in a healthy state.
    #[must_use]
    pub fn is_healthy(self) -> bool {
        matches!(self, Self::Nominal | Self::Alert)
    }

    /// Returns true if human intervention is recommended.
    #[must_use]
    pub fn needs_intervention(self) -> bool {
        matches!(self, Self::Critical)
    }

    /// Severity level (0=nominal, 3=critical).
    #[must_use]
    pub fn severity(self) -> u8 {
        match self {
            Self::Nominal => 0,
            Self::Alert => 1,
            Self::Degraded => 2,
            Self::Critical => 3,
        }
    }
}

impl GroundsTo for OperationalState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State])
    }
}

/// ∂: Boundary with low and high thresholds.
/// Tier: T1 (∂)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Boundary {
    /// Lower threshold.
    pub lo: f64,
    /// Upper threshold.
    pub hi: f64,
}

impl Boundary {
    /// Creates a new boundary.
    ///
    /// # Errors
    /// Returns `Err` if `lo > hi`.
    pub fn new(lo: f64, hi: f64) -> Result<Self, AvcError> {
        if lo > hi {
            return Err(AvcError::InvalidBoundary { lo, hi });
        }
        Ok(Self { lo, hi })
    }

    /// Returns true if value exceeds the upper boundary.
    #[must_use]
    pub fn exceeded(&self, value: f64) -> bool {
        value > self.hi
    }

    /// Returns true if value is below the lower boundary.
    #[must_use]
    pub fn undershot(&self, value: f64) -> bool {
        value < self.lo
    }

    /// Returns true if value is within bounds.
    #[must_use]
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lo && value <= self.hi
    }

    /// Width of the boundary band.
    #[must_use]
    pub fn width(&self) -> f64 {
        self.hi - self.lo
    }

    /// Adjusts boundary up by a factor (tightens: fewer alerts).
    #[must_use]
    pub fn adjust_up(self, factor: f64) -> Self {
        Self {
            lo: self.lo * factor,
            hi: self.hi * factor,
        }
    }

    /// Adjusts boundary down by a factor (loosens: more alerts).
    #[must_use]
    pub fn adjust_down(self, factor: f64) -> Self {
        let factor_inv = if factor.abs() < f64::EPSILON {
            1.0
        } else {
            1.0 / factor
        };
        Self {
            lo: self.lo * factor_inv,
            hi: self.hi * factor_inv,
        }
    }
}

impl GroundsTo for Boundary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
    }
}

/// κ: Comparison result — observed vs expected.
/// Tier: T1 (κ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Comparison {
    /// Observed value.
    pub observed: f64,
    /// Expected (baseline) value.
    pub expected: f64,
    /// Absolute delta.
    pub delta: f64,
}

impl Comparison {
    /// Creates a new comparison computing delta automatically.
    #[must_use]
    pub fn new(observed: f64, expected: f64) -> Self {
        Self {
            observed,
            expected,
            delta: (observed - expected).abs(),
        }
    }

    /// Returns the ratio of observed/expected.
    /// Returns `None` if expected is zero.
    #[must_use]
    pub fn ratio(&self) -> Option<f64> {
        if self.expected.abs() < f64::EPSILON {
            None
        } else {
            Some(self.observed / self.expected)
        }
    }

    /// Returns the relative deviation.
    /// Returns `None` if expected is zero.
    #[must_use]
    pub fn relative_deviation(&self) -> Option<f64> {
        if self.expected.abs() < f64::EPSILON {
            None
        } else {
            Some(self.delta / self.expected.abs())
        }
    }
}

impl GroundsTo for Comparison {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
    }
}

/// π: Tamper-evident audit record.
/// Tier: T2-P (π + σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord<T: Serialize> {
    /// The audited data.
    pub data: T,
    /// Timestamp of record creation.
    pub timestamp: SystemTime,
    /// Simple hash for integrity (FNV-1a of serialized data).
    pub hash: u64,
}

impl<T: Serialize> AuditRecord<T> {
    /// Creates a new audit record with current timestamp.
    pub fn stamp(data: T) -> Result<Self, AvcError> {
        let serialized = serde_json::to_string(&data)
            .map_err(|e| AvcError::SerializationError(e.to_string()))?;
        let hash = fnv1a_hash(serialized.as_bytes());
        Ok(Self {
            data,
            timestamp: SystemTime::now(),
            hash,
        })
    }

    /// Verifies record integrity.
    pub fn verify(&self) -> Result<bool, AvcError> {
        let serialized = serde_json::to_string(&self.data)
            .map_err(|e| AvcError::SerializationError(e.to_string()))?;
        Ok(fnv1a_hash(serialized.as_bytes()) == self.hash)
    }
}

impl GroundsTo for AuditRecord<String> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Persistence, LexPrimitiva::Sequence])
    }
}

/// ν: Frequency — count within a time window.
/// Tier: T2-P (ν + N)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Frequency {
    /// Number of events.
    pub count: u64,
    /// Time window for the count.
    pub window: Duration,
}

impl Frequency {
    /// Creates a new frequency measurement.
    #[must_use]
    pub fn new(count: u64, window: Duration) -> Self {
        Self { count, window }
    }

    /// Events per second.
    #[must_use]
    pub fn rate_per_second(&self) -> f64 {
        let secs = self.window.as_secs_f64();
        if secs < f64::EPSILON {
            0.0
        } else {
            self.count as f64 / secs
        }
    }
}

impl GroundsTo for Frequency {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Frequency, LexPrimitiva::Quantity])
    }
}

// ═══════════════════════════════════════════════════════════
// T2-P DOMAIN NEWTYPES
// ═══════════════════════════════════════════════════════════

/// Client identifier.
/// Tier: T2-P (N)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(pub u64);

impl GroundsTo for ClientId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
    }
}

/// Signal identifier.
/// Tier: T2-P (N)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SignalId(pub u64);

impl GroundsTo for SignalId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
    }
}

/// Domain threshold (boundary applied to a domain).
/// Tier: T2-P (∂)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Threshold(pub Boundary);

impl Threshold {
    /// Creates a threshold from low/high bounds.
    ///
    /// # Errors
    /// Returns `Err` if lo > hi.
    pub fn new(lo: f64, hi: f64) -> Result<Self, AvcError> {
        Ok(Self(Boundary::new(lo, hi)?))
    }

    /// Returns true if value exceeds threshold.
    #[must_use]
    pub fn exceeded(&self, value: f64) -> bool {
        self.0.exceeded(value)
    }
}

impl GroundsTo for Threshold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
    }
}

/// Baseline: expected metric timeline.
/// Tier: T2-P (σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    values: TimeSeries<f64>,
    mean: f64,
    std_dev: f64,
}

impl Baseline {
    /// Creates a baseline from a series of values.
    #[must_use]
    pub fn from_values(values: &[f64]) -> Self {
        let n = values.len() as f64;
        let mean = if n > 0.0 {
            values.iter().sum::<f64>() / n
        } else {
            0.0
        };
        let variance = if n > 1.0 {
            values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0)
        } else {
            0.0
        };
        let std_dev = variance.sqrt();

        let mut ts = TimeSeries::new();
        for &v in values {
            ts.push_now(v);
        }

        Self {
            values: ts,
            mean,
            std_dev,
        }
    }

    /// Returns the expected (mean) value.
    #[must_use]
    pub fn expected(&self) -> f64 {
        self.mean
    }

    /// Returns the standard deviation.
    #[must_use]
    pub fn std_dev(&self) -> f64 {
        self.std_dev
    }

    /// Returns number of standard deviations from mean.
    #[must_use]
    pub fn z_score(&self, value: f64) -> f64 {
        if self.std_dev < f64::EPSILON {
            0.0
        } else {
            (value - self.mean) / self.std_dev
        }
    }
}

impl GroundsTo for Baseline {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence])
    }
}

/// Source identifier / location.
/// Tier: T2-P (λ)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Source(pub String);

impl GroundsTo for Source {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Location])
    }
}

/// Domain identifier for routing.
/// Tier: T2-P (μ)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Domain(pub String);

impl GroundsTo for Domain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping])
    }
}

// ═══════════════════════════════════════════════════════════
// T2-C COMPOSITES
// ═══════════════════════════════════════════════════════════

/// Detection result: κ + ∂ + ∃
/// The core comparison outcome: observed metric vs threshold.
/// Tier: T2-C (κ + ∂ + ∃)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Detection {
    /// Comparison result (observed vs expected).
    pub comparison: Comparison,
    /// Whether the signal is present (exceeds threshold).
    pub signal_present: bool,
    /// Confidence in this detection (0.0-1.0).
    pub confidence: f64,
    /// Normalized severity: delta / (hi + delta), range [0, 1).
    /// When severity >= CRITICAL_BOUNDARY, escalate to human.
    pub severity: f64,
}

impl Detection {
    /// Creates a detection from a comparison against a threshold.
    #[must_use]
    pub fn evaluate(comparison: Comparison, threshold: &Threshold) -> Self {
        let signal_present = threshold.exceeded(comparison.delta);

        // Confidence increases with distance from threshold
        let confidence = if signal_present {
            let excess = comparison.delta - threshold.0.hi;
            let range = threshold.0.width().max(f64::EPSILON);
            (0.5 + 0.5 * (excess / (range + excess))).clamp(0.5, 1.0)
        } else {
            let shortfall = threshold.0.hi - comparison.delta;
            let range = threshold.0.width().max(f64::EPSILON);
            (shortfall / range).clamp(0.0, 0.5)
        };

        // Severity: normalized to [0, 1) — approaches 1 when delta >> hi
        let hi = threshold.0.hi.abs().max(f64::EPSILON);
        let severity = comparison.delta / (hi + comparison.delta);

        Self {
            comparison,
            signal_present,
            confidence,
            severity,
        }
    }
}

impl GroundsTo for Detection {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// Learning feedback: κ + ρ + π
/// Captures predicted vs actual for model improvement.
/// Tier: T2-C (κ + ρ + π)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    /// What we predicted (the detection).
    pub predicted: Detection,
    /// What actually happened.
    pub actual_outcome: Outcome,
    /// Timestamp of feedback.
    pub timestamp: SystemTime,
}

impl Feedback {
    /// Creates feedback from a detection and its actual outcome.
    #[must_use]
    pub fn new(predicted: Detection, actual_outcome: Outcome) -> Self {
        Self {
            predicted,
            actual_outcome,
            timestamp: SystemTime::now(),
        }
    }

    /// Returns true if the prediction was correct.
    #[must_use]
    pub fn was_correct(&self) -> bool {
        match self.actual_outcome {
            Outcome::TruePositive | Outcome::TrueNegative => true,
            Outcome::FalsePositive | Outcome::FalseNegative => false,
        }
    }

    /// Returns true if this was a false positive.
    #[must_use]
    pub fn is_false_positive(&self) -> bool {
        self.actual_outcome == Outcome::FalsePositive
    }

    /// Returns true if this was a false negative.
    #[must_use]
    pub fn is_false_negative(&self) -> bool {
        self.actual_outcome == Outcome::FalseNegative
    }
}

impl GroundsTo for Feedback {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Recursion,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.90)
    }
}

/// Outcome of a detection prediction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// Signal predicted and confirmed.
    TruePositive,
    /// No signal predicted, correctly.
    TrueNegative,
    /// Signal predicted but not real.
    FalsePositive,
    /// Signal missed — should have detected.
    FalseNegative,
}

/// Client configuration: μ + ∂ + λ
/// Maps domains to thresholds for a specific client.
/// Tier: T2-C (μ + ∂ + λ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Client identifier.
    pub client_id: ClientId,
    /// Source location.
    pub source: Source,
    /// Domain-specific thresholds.
    pub thresholds: HashMap<String, Threshold>,
}

impl ClientConfig {
    /// Creates a new client config.
    #[must_use]
    pub fn new(client_id: ClientId, source: Source) -> Self {
        Self {
            client_id,
            source,
            thresholds: HashMap::new(),
        }
    }

    /// Adds a domain threshold.
    pub fn with_threshold(mut self, domain: &str, threshold: Threshold) -> Self {
        self.thresholds.insert(domain.to_string(), threshold);
        self
    }

    /// Gets the threshold for a domain.
    #[must_use]
    pub fn threshold_for(&self, domain: &str) -> Option<&Threshold> {
        self.thresholds.get(domain)
    }
}

impl GroundsTo for ClientConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

// ═══════════════════════════════════════════════════════════
// T3 ACTIONS AND DECISIONS
// ═══════════════════════════════════════════════════════════

/// Autonomous PV response action (AutoAlert, signal escalation).
/// Tier: T2-C (→ + ∂ + ∃)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PvAction {
    /// Autonomous alert — sent without human involvement.
    AutoAlert {
        signal_id: SignalId,
        detection: Detection,
        domain: String,
    },
    /// Escalation — routed to human operator.
    Escalate {
        signal_id: SignalId,
        detection: Detection,
        reason: String,
    },
    /// Log only — no active response needed.
    Log {
        detection: Detection,
        domain: String,
    },
}

/// Backward-compatible alias.
#[deprecated(note = "use PvAction — F2 equivocation fix")]
pub type Action = PvAction;

impl GroundsTo for PvAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.90)
    }
}

/// Decision record — auditable trace of what was decided and why.
/// Tier: T2-C (→ + π + κ + ς)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// The action taken.
    pub action: PvAction,
    /// System state at time of decision.
    pub system_state: OperationalState,
    /// Timestamp.
    pub timestamp: SystemTime,
}

impl Decision {
    /// Creates a new decision record.
    #[must_use]
    pub fn new(action: PvAction, state: OperationalState) -> Self {
        Self {
            action,
            system_state: state,
            timestamp: SystemTime::now(),
        }
    }
}

impl GroundsTo for Decision {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Persistence,
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// An incoming event to be analyzed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Domain this event belongs to.
    pub domain: String,
    /// The metric value.
    pub metric: f64,
    /// Source of the event.
    pub source: Source,
    /// Timestamp.
    pub timestamp: SystemTime,
}

impl Event {
    /// Creates a new event.
    #[must_use]
    pub fn new(domain: &str, metric: f64, source: Source) -> Self {
        Self {
            domain: domain.to_string(),
            metric,
            source,
            timestamp: SystemTime::now(),
        }
    }
}

// ═══════════════════════════════════════════════════════════
// TRUST BOUNDARIES (CONSTANTS)
// ═══════════════════════════════════════════════════════════

/// ∂ᶜʳⁱᵗ — delta above this escalates to human.
pub const CRITICAL_BOUNDARY: f64 = 0.95;

/// ∂ᶠᵖ — max tolerable false positive rate.
pub const MAX_FALSE_POSITIVE_RATE: f64 = 0.05;

/// ∂ᶠⁿ — max tolerable false negative rate.
pub const MAX_FALSE_NEGATIVE_RATE: f64 = 0.01;

/// N — minimum feedback batch size before model update.
pub const LEARNING_BATCH_SIZE: usize = 100;

/// Health threshold for Nominal state.
pub const HEALTH_NOMINAL: f64 = 0.95;

/// Health threshold for Alert state.
pub const HEALTH_ALERT: f64 = 0.80;

/// Health threshold for Degraded state.
pub const HEALTH_DEGRADED: f64 = 0.50;

/// Threshold adjustment factor.
pub const ADJUSTMENT_FACTOR: f64 = 1.05;

// ═══════════════════════════════════════════════════════════
// HUMAN OVERRIDE
// ═══════════════════════════════════════════════════════════

/// Human override commands — always available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HumanCommand {
    /// Force system to a specific state.
    ForceState(OperationalState),
    /// Adjust a domain threshold.
    AdjustThreshold {
        domain: String,
        threshold: Threshold,
    },
    /// Halt processing (graceful shutdown).
    Halt,
}

// ═══════════════════════════════════════════════════════════
// ERROR TYPE
// ═══════════════════════════════════════════════════════════

/// AVC error type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AvcError {
    /// Invalid boundary (lo > hi).
    InvalidBoundary { lo: f64, hi: f64 },
    /// Domain not configured.
    UnknownDomain(String),
    /// Client not registered.
    UnknownClient(ClientId),
    /// Serialization failure.
    SerializationError(String),
    /// System halted by human command.
    Halted,
}

impl std::fmt::Display for AvcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBoundary { lo, hi } => {
                write!(f, "invalid boundary: lo ({lo}) > hi ({hi})")
            }
            Self::UnknownDomain(d) => write!(f, "unknown domain: {d}"),
            Self::UnknownClient(id) => write!(f, "unknown client: {}", id.0),
            Self::SerializationError(e) => write!(f, "serialization error: {e}"),
            Self::Halted => write!(f, "system halted by human command"),
        }
    }
}

impl std::error::Error for AvcError {}

// ═══════════════════════════════════════════════════════════
// UTILITY
// ═══════════════════════════════════════════════════════════

/// FNV-1a hash for audit records (zero-dep, deterministic).
#[must_use]
pub fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in data {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex_primitiva::GroundingTier;

    #[test]
    fn test_t1_operational_state() {
        let comp = OperationalState::primitive_composition();
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T1Universal);
        assert!(comp.is_pure());
    }

    #[test]
    fn test_t1_boundary() {
        let comp = Boundary::primitive_composition();
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T1Universal);
    }

    #[test]
    fn test_t1_comparison() {
        let comp = Comparison::primitive_composition();
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T1Universal);
    }

    #[test]
    fn test_t2p_frequency() {
        let comp = Frequency::primitive_composition();
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_t2p_client_id() {
        let comp = ClientId::primitive_composition();
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T1Universal);
    }

    #[test]
    fn test_t2c_detection() {
        let comp = Detection::primitive_composition();
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn test_t2c_feedback() {
        let comp = Feedback::primitive_composition();
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
    }

    #[test]
    fn test_t2c_decision() {
        let comp = Decision::primitive_composition();
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
    }

    #[test]
    fn test_boundary_validation() {
        assert!(Boundary::new(0.0, 1.0).is_ok());
        assert!(Boundary::new(1.0, 0.0).is_err());
    }

    #[test]
    fn test_boundary_contains() {
        let bd = Boundary::new(0.2, 0.8).ok();
        assert!(bd.is_some());
        if let Some(bd) = bd {
            assert!(bd.contains(0.5));
            assert!(!bd.contains(0.9));
            assert!(!bd.contains(0.1));
        }
    }

    #[test]
    fn test_comparison_delta() {
        let cmp = Comparison::new(7.5, 5.0);
        assert!((cmp.delta - 2.5).abs() < f64::EPSILON);
        let ratio = cmp.ratio();
        assert!(ratio.is_some());
        if let Some(r) = ratio {
            assert!((r - 1.5).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_detection_evaluate() {
        let cmp = Comparison::new(0.9, 0.3);
        let thresh = Threshold::new(0.0, 0.5).ok();
        assert!(thresh.is_some());
        if let Some(t) = thresh {
            let det = Detection::evaluate(cmp, &t);
            assert!(det.signal_present); // delta=0.6 > hi=0.5
            assert!(det.confidence >= 0.5);
        }
    }

    #[test]
    fn test_feedback_correctness() {
        let det = Detection {
            comparison: Comparison::new(1.0, 0.0),
            signal_present: true,
            confidence: 0.9,
            severity: 0.5,
        };
        let fb_correct = Feedback::new(det, Outcome::TruePositive);
        assert!(fb_correct.was_correct());

        let fb_wrong = Feedback::new(det, Outcome::FalsePositive);
        assert!(!fb_wrong.was_correct());
        assert!(fb_wrong.is_false_positive());
    }

    #[test]
    fn test_time_series_recent() {
        let mut ts: TimeSeries<f64> = TimeSeries::new();
        ts.push_now(1.0);
        ts.push_now(2.0);
        ts.push_now(3.0);
        assert_eq!(ts.len(), 3);
        let recent = ts.recent(Duration::from_secs(60));
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_baseline_z_score() {
        let bl = Baseline::from_values(&[10.0, 10.0, 10.0, 10.0, 10.0]);
        assert!((bl.expected() - 10.0).abs() < f64::EPSILON);
        assert!(bl.std_dev() < f64::EPSILON);
        assert!((bl.z_score(15.0) - 0.0).abs() < f64::EPSILON); // std_dev=0 → returns 0
    }

    #[test]
    fn test_baseline_with_variance() {
        let bl = Baseline::from_values(&[8.0, 10.0, 12.0]);
        assert!((bl.expected() - 10.0).abs() < f64::EPSILON);
        assert!(bl.std_dev() > 0.0);
        let z = bl.z_score(14.0);
        assert!(z > 1.0); // 14 is above mean by more than 1 std dev
    }

    #[test]
    fn test_fnv1a_deterministic() {
        let h1 = fnv1a_hash(b"hello");
        let h2 = fnv1a_hash(b"hello");
        assert_eq!(h1, h2);
        let h3 = fnv1a_hash(b"world");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_operational_state_severity() {
        assert_eq!(OperationalState::Nominal.severity(), 0);
        assert_eq!(OperationalState::Critical.severity(), 3);
        assert!(OperationalState::Nominal.is_healthy());
        assert!(OperationalState::Critical.needs_intervention());
    }

    #[test]
    fn test_action_grounding() {
        let comp = PvAction::primitive_composition();
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
    }
}
