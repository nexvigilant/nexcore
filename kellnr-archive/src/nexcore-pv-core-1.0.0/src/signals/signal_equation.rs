//! # ToV Signal Detection Equation (§20-§23)
//!
//! Type-safe implementation of the fundamental signal equation:
//!
//! **S = U × R × T**

use crate::grounded::{
    Bits, Measured, RecognitionR as GroundedR, SignalStrengthS as GroundedS,
    TemporalT as GroundedT, UniquenessU as GroundedU,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SignalComponentError {
    Negative(f64),
    NaN,
    Infinite,
    OutOfRange { value: f64, min: f64, max: f64 },
}

impl fmt::Display for SignalComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negative(v) => write!(f, "Negative: {v}"),
            Self::NaN => write!(f, "NaN"),
            Self::Infinite => write!(f, "Infinite"),
            Self::OutOfRange { value, .. } => write!(f, "Out of range: {value}"),
        }
    }
}

impl std::error::Error for SignalComponentError {}

pub type UnrepeatablePattern = UniquenessU;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct UniquenessU(pub Bits);

impl UniquenessU {
    pub const NON_RECURRENCE_THRESHOLD: f64 = 63.0;
    pub fn new(v: f64) -> Result<Self, SignalComponentError> {
        Ok(Self(Bits(v)))
    }
    pub fn from_ratio(o: f64, e: f64) -> Self {
        if e <= 0.0 {
            Self(Bits(0.0))
        } else {
            Self(Bits((o / e).log2()))
        }
    }
    pub fn from_ln1p_ratio(o: u64, e: f64) -> Self {
        if e <= 0.0 {
            Self(Bits(0.0))
        } else {
            Self(Bits((o as f64 / e).ln_1p() / 2.0f64.ln()))
        }
    }
    pub const fn value(&self) -> f64 {
        self.0.0
    }
    pub const ZERO: Self = Self(Bits(0.0));
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct RecognitionPresence(pub f64);

impl RecognitionPresence {
    pub fn from_dme_status(dme: bool, prev: bool) -> Self {
        let b = if dme { 2.0 } else { 1.0 };
        Self(if prev { b * 0.5 } else { b })
    }
    pub const fn value(&self) -> f64 {
        self.0
    }
    pub const FULL: Self = Self(1.0);
    pub const ZERO: Self = Self(0.0);
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct TemporalWindow(pub f64);

impl TemporalWindow {
    pub fn from_elapsed(d: u32, t: u32) -> Self {
        if t == 0 {
            Self(1.0)
        } else {
            Self((1.0 - (d as f64 / t as f64)).clamp(0.0, 1.0))
        }
    }
    pub const fn value(&self) -> f64 {
        self.0
    }
    pub const FULL: Self = Self(1.0);
    pub const ZERO: Self = Self(0.0);
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct SignalStrength(pub Measured<f64>);

impl SignalStrength {
    pub fn calculate(u: UniquenessU, r: RecognitionPresence, t: TemporalWindow) -> Self {
        Self(Measured {
            value: u.value() * r.value() * t.value(),
            confidence: 1.0,
        })
    }
    pub const fn value(&self) -> f64 {
        self.0.value
    }
    pub const fn confidence(&self) -> f64 {
        self.0.confidence
    }
    pub const ZERO: Self = Self(Measured {
        value: 0.0,
        confidence: 1.0,
    });
    pub fn from_value(v: f64) -> Self {
        Self(Measured::certain(v.max(0.0)))
    }
    pub fn is_signal(&self) -> bool {
        self.value() > 0.0
    }
    pub fn exceeds_non_recurrence(&self) -> bool {
        self.value() >= 63.0
    }
}

impl Default for SignalStrength {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<SignalStrength> for f64 {
    fn from(s: SignalStrength) -> f64 {
        s.value()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEquationResult {
    pub signal_strength: SignalStrength,
    pub unrepeatability: UniquenessU,
    pub recognition: RecognitionPresence,
    pub temporal_factor: TemporalWindow,
}

impl SignalEquationResult {
    pub fn evaluate(o: u64, e: f64, dme: bool, prev: bool, ds: u32, td: u32) -> Self {
        let u = UniquenessU::from_ln1p_ratio(o, e);
        let r = RecognitionPresence::from_dme_status(dme, prev);
        let t = TemporalWindow::from_elapsed(ds, td);
        Self {
            signal_strength: SignalStrength::calculate(u, r, t),
            unrepeatability: u,
            recognition: r,
            temporal_factor: t,
        }
    }
}

impl From<UniquenessU> for GroundedU {
    fn from(u: UniquenessU) -> Self {
        Self(u.0)
    }
}
impl From<RecognitionPresence> for GroundedR {
    fn from(r: RecognitionPresence) -> Self {
        Self(r.value())
    }
}
impl From<TemporalWindow> for GroundedT {
    fn from(t: TemporalWindow) -> Self {
        Self(t.value())
    }
}
impl From<SignalStrength> for GroundedS {
    fn from(s: SignalStrength) -> Self {
        Self(Bits(s.value()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniqueness_u_creation() {
        let u = UniquenessU::new(2.5).unwrap_or(UniquenessU::ZERO);
        assert!((u.value() - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_signal_strength_calculation() {
        let u = UniquenessU::new(2.0).unwrap_or(UniquenessU::ZERO);
        let r = RecognitionPresence::from_dme_status(false, false);
        let t = TemporalWindow::from_elapsed(20, 100);

        let s = SignalStrength::calculate(u, r, t);
        assert!((s.value() - (2.0 * 1.0 * 0.8)).abs() < f64::EPSILON);
    }
}
