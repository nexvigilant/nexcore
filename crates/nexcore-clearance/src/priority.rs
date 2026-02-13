//! # Clearance Priority
//!
//! Where classification sits in the Guardian priority hierarchy.
//!
//! ## Primitive Grounding
//! - **Tier**: T1
//! - **Dominant**: κ Comparison (ordering/ranking)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Guardian priority hierarchy with classification at P2c.
///
/// Patient safety is supreme. Classification never overrides P0-P2a.
///
/// ## Tier: T1
/// ## Dominant: κ Comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ClearancePriority {
    /// P0: Patient Safety — supreme directive.
    PatientSafety = 0,
    /// P1: Signal Integrity — no signal lost or downgraded.
    SignalIntegrity = 1,
    /// P2: Regulatory Compliance — ICH E2A timelines.
    Regulatory = 2,
    /// P2b: Data Privacy — PII/PHI protection (Ghost).
    Privacy = 3,
    /// P2c: Security Classification — organizational secrecy.
    Classification = 4,
    /// P3: Data Quality — accuracy gates.
    DataQuality = 5,
    /// P4: Operational Efficiency — throughput/latency.
    Operational = 6,
    /// P5: Cost Optimization — resource allocation.
    Cost = 7,
}

impl ClearancePriority {
    /// Returns the numeric priority (lower = higher priority).
    #[must_use]
    pub fn ordinal(self) -> u8 {
        self as u8
    }

    /// Human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::PatientSafety => "P0: Patient Safety",
            Self::SignalIntegrity => "P1: Signal Integrity",
            Self::Regulatory => "P2: Regulatory",
            Self::Privacy => "P2b: Privacy",
            Self::Classification => "P2c: Classification",
            Self::DataQuality => "P3: Data Quality",
            Self::Operational => "P4: Operational",
            Self::Cost => "P5: Cost",
        }
    }

    /// Returns true if `self` outranks `other` (lower ordinal = higher priority).
    #[must_use]
    pub fn outranks(self, other: Self) -> bool {
        self.ordinal() < other.ordinal()
    }

    /// Patient safety always outranks classification.
    #[must_use]
    pub fn patient_safety_overrides_classification() -> bool {
        Self::PatientSafety.outranks(Self::Classification)
    }

    /// Privacy (Ghost) outranks classification.
    #[must_use]
    pub fn privacy_overrides_classification() -> bool {
        Self::Privacy.outranks(Self::Classification)
    }

    /// Classification outranks data quality and below.
    #[must_use]
    pub fn classification_outranks_quality() -> bool {
        Self::Classification.outranks(Self::DataQuality)
    }
}

impl PartialOrd for ClearancePriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ClearancePriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Lower ordinal = higher priority, so reverse
        self.ordinal().cmp(&other.ordinal()).reverse()
    }
}

impl fmt::Display for ClearancePriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patient_safety_supreme() {
        assert!(ClearancePriority::PatientSafety.outranks(ClearancePriority::Classification));
    }

    #[test]
    fn signal_integrity_outranks_classification() {
        assert!(ClearancePriority::SignalIntegrity.outranks(ClearancePriority::Classification));
    }

    #[test]
    fn regulatory_outranks_classification() {
        assert!(ClearancePriority::Regulatory.outranks(ClearancePriority::Classification));
    }

    #[test]
    fn privacy_outranks_classification() {
        assert!(ClearancePriority::privacy_overrides_classification());
    }

    #[test]
    fn classification_outranks_data_quality() {
        assert!(ClearancePriority::classification_outranks_quality());
    }

    #[test]
    fn classification_outranks_operational() {
        assert!(ClearancePriority::Classification.outranks(ClearancePriority::Operational));
    }

    #[test]
    fn classification_outranks_cost() {
        assert!(ClearancePriority::Classification.outranks(ClearancePriority::Cost));
    }

    #[test]
    fn ord_ordering() {
        // Higher priority sorts first (greater in Ord)
        assert!(ClearancePriority::PatientSafety > ClearancePriority::Classification);
        assert!(ClearancePriority::Classification > ClearancePriority::Cost);
    }
}
